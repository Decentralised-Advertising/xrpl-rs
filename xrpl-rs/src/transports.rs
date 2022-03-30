use super::types::{
    subscribe::{SubscribeRequest, SubscriptionEvent},
    Error as APIError, RequestId, Response, Result as APIResult,
};
use super::Error;
use async_trait::async_trait;
use futures::{
    channel::{mpsc, mpsc::UnboundedReceiver, oneshot},
    stream::Map,
    task::Context,
    SinkExt, Stream, StreamExt,
};
use reqwest::{header::CONTENT_TYPE, Client};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt::Debug;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Error as WSError, Message, Result},
};
use url::{ParseError, Url};

#[async_trait]
pub trait Transport {
    async fn send_request<Params: Serialize + Send, Res: DeserializeOwned + Debug + Send>(
        &self,
        method: &str,
        params: Params,
    ) -> Result<Res, TransportError>;
}

#[async_trait]
pub trait DuplexTransport: Transport {
    async fn subscribe<T: DeserializeOwned>(
        &self,
        request: SubscribeRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<T, TransportError>>>>, TransportError>;
    async fn unsubscribe(&self, request: SubscribeRequest) -> Result<(), TransportError>;
}

#[derive(Debug)]
pub enum TransportError {
    NoEndpoint,
    Error(&'static str),
    InvalidEndpoint(ParseError),
    ReqwestError(reqwest::Error),
    JSONError(serde_json::Error),
    WSError(WSError),
    ErrorResponse(String),
    APIError(Value),
}

impl From<reqwest::Error> for TransportError {
    fn from(e: reqwest::Error) -> Self {
        Self::ReqwestError(e)
    }
}

impl From<WSError> for TransportError {
    fn from(e: WSError) -> Self {
        Self::WSError(e)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsonRPCRequest<T: Serialize + Send> {
    pub id: RequestId,
    pub method: String,
    pub params: T,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WebSocketRPCRequest<T: Serialize + Send> {
    pub id: RequestId,
    pub command: String,
    #[serde(flatten)]
    pub params: T,
}

unsafe impl<T: Serialize + Send> Send for JsonRPCRequest<T> {}

pub struct HTTP {
    counter: AtomicU64,
    inner: Client,
    base_url: Url,
}

impl HTTP {
    pub fn builder() -> HTTPBuilder {
        HTTPBuilder::default()
    }
}

#[async_trait]
impl Transport for HTTP {
    async fn send_request<Params: Serialize + Send, Res: DeserializeOwned + Debug + Send>(
        &self,
        method: &str,
        params: Params,
    ) -> Result<Res, TransportError> {
        let json_str = serde_json::to_string(&JsonRPCRequest {
            id: self.counter.fetch_add(1u64, Ordering::SeqCst),
            method: method.to_owned(),
            params: vec![params],
        })
        .map_err(|e| TransportError::JSONError(e))?;
        let client = self.inner.clone();
        let res = client
            .post(self.base_url.clone())
            .header(CONTENT_TYPE, "application/json")
            .body(json_str)
            .send()
            .await?;
        let json = res.json::<Response<Res>>().await;
        match json.map_err(|e| TransportError::ReqwestError(e))?.result {
            APIResult::Ok(result) => Ok(result),
            APIResult::Error(e) => Err(TransportError::APIError(e)),
        }
    }
}

#[derive(Default)]
pub struct HTTPBuilder {
    pub endpoint: Option<Url>,
}

impl HTTPBuilder {
    pub fn with_endpoint<'b>(&'b mut self, endpoint: &str) -> Result<&'b mut Self, TransportError> {
        let u = Url::parse(endpoint).map_err(|e| TransportError::InvalidEndpoint(e))?;
        self.endpoint = Some(u);
        Ok(self)
    }

    pub fn build(&self) -> Result<HTTP, TransportError> {
        Ok(HTTP {
            counter: AtomicU64::new(0u64),
            base_url: self.endpoint.clone().ok_or(TransportError::NoEndpoint)?,
            inner: Client::new(),
        })
    }
}

#[derive(Debug, Clone)]
pub enum PendingRequest {
    Call {
        id: RequestId,
        request: WebSocketRPCRequest<Value>,
        response: mpsc::Sender<Response<Value>>,
    },
    Subscription {
        id: RequestId,
        request: WebSocketRPCRequest<Value>,
        channel: mpsc::UnboundedSender<Response<Value>>,
    },
}

pub struct WebSocket {
    counter: Arc<AtomicU64>,
    sender: mpsc::UnboundedSender<PendingRequest>,
    pending_requests: Arc<Mutex<HashMap<u64, PendingRequest>>>,
}

impl WebSocket {
    pub fn new(sender: mpsc::UnboundedSender<PendingRequest>) -> Self {
        Self {
            counter: Arc::new(AtomicU64::new(1u64)),
            sender,
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    pub fn builder() -> WebSocketBuilder {
        WebSocketBuilder::default()
    }
}

#[async_trait]
impl Transport for WebSocket {
    async fn send_request<Params: Serialize + Send, Res: DeserializeOwned + Debug + Send>(
        &self,
        method: &str,
        params: Params,
    ) -> Result<Res, TransportError> {
        let mut sender = self.sender.clone();
        let id = self.counter.fetch_add(1u64, Ordering::Relaxed);
        let (s, r) = mpsc::channel(1);
        let request = PendingRequest::Call {
            id,
            request: WebSocketRPCRequest {
                id,
                command: method.to_owned(),
                params: json!(params),
            },
            response: s.clone(),
        };
        if let Ok(mut pending_requests) = self.pending_requests.lock() {
            pending_requests.insert(id, request.clone());
        }
        sender
            .send(request)
            .await
            .map_err(|e| TransportError::ErrorResponse(format!("sending: {:?}", e)))?; //TODO: Add error type for websocket send error
        let response: Response<Value> = r
            .take(1)
            .collect::<Vec<Response<Value>>>()
            .await
            .first()
            .unwrap()
            .clone();
        match response.result {
            APIResult::Ok(result) => Ok(serde_json::from_value(result).unwrap()),
            APIResult::Error(e) => Err(TransportError::APIError(e)),
        }
    }
}

#[async_trait]
impl DuplexTransport for WebSocket {
    async fn subscribe<T: DeserializeOwned>(
        &self,
        request: SubscribeRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<T, TransportError>>>>, TransportError> {
        let mut sender = self.sender.clone();
        let id = self.counter.fetch_add(1u64, Ordering::Relaxed);
        let (s, r) = mpsc::unbounded();
        let req = PendingRequest::Subscription {
            id,
            request: WebSocketRPCRequest {
                id,
                command: "subscribe".to_owned(),
                params: json!(request),
            },
            channel: s.clone(),
        };
        if let Ok(mut pending_requests) = self.pending_requests.lock() {
            pending_requests.insert(id, req.clone());
        }
        sender
            .send(req)
            .await
            .map_err(|e| TransportError::ErrorResponse(format!("sending: {:?}", e)))?; //TODO: Add error type for websocket send error
        let stream = r.map(|response| match response.result {
            APIResult::Ok(result) => Ok(serde_json::from_value(result).unwrap()),
            APIResult::Error(e) => Err(TransportError::APIError(e)),
        });
        Ok(Box::pin(stream))
    }
    async fn unsubscribe(&self, _request: SubscribeRequest) -> Result<(), TransportError> {
        Err(TransportError::Error("test"))
    }
}

#[derive(Default)]
pub struct WebSocketBuilder {
    pub endpoint: Option<Url>,
}

impl WebSocketBuilder {
    pub fn with_endpoint<'b>(&'b mut self, endpoint: &str) -> Result<&'b mut Self, TransportError> {
        let u = Url::parse(endpoint).map_err(|e| TransportError::InvalidEndpoint(e))?;
        self.endpoint = Some(u);
        Ok(self)
    }

    pub async fn build(&self) -> Result<WebSocket, TransportError> {
        let (ws_stream, _) = connect_async(self.endpoint.clone().unwrap()).await?;
        let (sender, mut receiver) = mpsc::unbounded::<PendingRequest>();
        let (write, read) = ws_stream.split();
        let ws = WebSocket::new(sender);
        let pending_requests = ws.pending_requests.clone();
        tokio::spawn(async move {
            read.for_each(|message| async {
                let data = message.unwrap().into_data();
                let s = String::from_utf8_lossy(&data);
                let res: Option<Response<Value>> = serde_json::from_slice(&data).ok();
                match res {
                    Some(res) => {
                        let pr = pending_requests
                            .lock()
                            .map(|p| p.get(&res.id.unwrap()).unwrap().clone())
                            .unwrap();
                        match pr {
                            PendingRequest::Call { response, .. } => {
                                let mut r = response.clone();
                                r.send(res).await.unwrap();
                            }
                            _ => {}
                        }
                    }
                    None => {
                        if let Ok(event) = serde_json::from_slice::<SubscriptionEvent>(&data) {
                            println!("{:?}", event);
                        };
                    }
                }
            })
            .await;
        });
        tokio::spawn(async move {
            receiver
                .map(|req| match req {
                    PendingRequest::Call { request, .. } => {
                        Message::Text(serde_json::to_string(&request).unwrap())
                    }
                    PendingRequest::Subscription { request, .. } => {
                        Message::Text(serde_json::to_string(&request).unwrap())
                    }
                })
                .map(Ok)
                .forward(write)
                .await
                .unwrap();
        });
        Ok(ws)
    }
}
