use super::types::{
    subscribe::{SubscribeRequest, SubscriptionEvent},
    ErrorResponse, JsonRPCResponse, JsonRPCResponseResult, RequestId, WebsocketResponse,
};
use async_trait::async_trait;
use futures::{channel::mpsc, SinkExt, Stream, StreamExt};
use reqwest::{header::CONTENT_TYPE, Client};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
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
    async fn subscribe(
        &self,
        request: SubscribeRequest,
    ) -> Result<
        Pin<Box<dyn Stream<Item = Result<SubscriptionEvent, TransportError>>>>,
        TransportError,
    >;
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
    APIError(ErrorResponse),
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
        let json = res.json::<JsonRPCResponse<Res>>().await;
        match json.map_err(|e| TransportError::ReqwestError(e))?.result {
            JsonRPCResponseResult::Success(success) => Ok(success.result),
            JsonRPCResponseResult::Error(e) => Err(TransportError::APIError(e)),
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

pub enum Outbound {
    PendingRequest(PendingRequest),
    Subscription(Subscription),
}

#[derive(Debug, Clone)]
pub struct PendingRequest {
    id: RequestId,
    request: WebSocketRPCRequest<Value>,
    response: mpsc::Sender<WebsocketResponse<Value>>,
}

#[derive(Debug, Clone)]
pub struct Subscription {
    request: WebSocketRPCRequest<Value>,
    channel: mpsc::UnboundedSender<Result<SubscriptionEvent, TransportError>>,
}

pub struct WebSocket {
    counter: Arc<AtomicU64>,
    sender: mpsc::UnboundedSender<Outbound>,
    pending_requests: Arc<Mutex<HashMap<u64, PendingRequest>>>,
    subscriptions: Arc<Mutex<Vec<Subscription>>>,
}

impl WebSocket {
    pub fn new(sender: mpsc::UnboundedSender<Outbound>) -> Self {
        Self {
            counter: Arc::new(AtomicU64::new(1u64)),
            sender,
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
            subscriptions: Arc::new(Mutex::new(Vec::new())),
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
        let request = PendingRequest {
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
            .send(Outbound::PendingRequest(request))
            .await
            .map_err(|e| TransportError::ErrorResponse(format!("sending: {:?}", e)))?; //TODO: Add error type for websocket send error
        let response: WebsocketResponse<Value> = r
            .take(1)
            .collect::<Vec<WebsocketResponse<Value>>>()
            .await
            .first()
            .unwrap()
            .clone();
        match response {
            WebsocketResponse::Success(success) => {
                Ok(serde_json::from_value(success.result).unwrap())
            }
            WebsocketResponse::Error(e) => Err(TransportError::APIError(e)),
        }
    }
}

#[async_trait]
impl DuplexTransport for WebSocket {
    async fn subscribe(
        &self,
        request: SubscribeRequest,
    ) -> Result<
        Pin<Box<dyn Stream<Item = Result<SubscriptionEvent, TransportError>>>>,
        TransportError,
    > {
        let mut sender = self.sender.clone();
        let id = self.counter.fetch_add(1u64, Ordering::Relaxed);
        let (s, r) = mpsc::unbounded();
        let req = Subscription {
            request: WebSocketRPCRequest {
                id,
                command: "subscribe".to_owned(),
                params: json!(request),
            },
            channel: s.clone(),
        };
        if let Ok(mut subs) = self.subscriptions.lock() {
            subs.push(req.clone());
        }
        sender
            .send(Outbound::Subscription(req))
            .await
            .map_err(|e| TransportError::ErrorResponse(format!("sending: {:?}", e)))?; //TODO: Add error type for websocket send error
        Ok(Box::pin(r))
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
        let (sender, receiver) = mpsc::unbounded::<Outbound>();
        let (write, read) = ws_stream.split();
        let ws = WebSocket::new(sender);
        let pending_requests = ws.pending_requests.clone();
        let subscriptions = ws.subscriptions.clone();
        tokio::spawn(async move {
            read.for_each(|message| async {
                let data = message.unwrap().into_data();
                if data.len() == 0 {
                    return;
                }
                let res: Option<WebsocketResponse<Value>> = serde_json::from_slice(&data).ok();
                match res {
                    Some(res) => {
                        let pr = pending_requests
                            .lock()
                            .map(|p| p.get(&res.get_id().unwrap()).and_then(|p|Some(p.clone()))).unwrap();
                        if let Some(pending_request) = pr {
                            let mut r = pending_request.response.clone();
                            r.send(res).await.unwrap();
                        }
                    }
                    None => {
                        let subs = subscriptions.lock().unwrap().clone();
                        for sub in &subs {
                            let event = serde_json::from_slice::<SubscriptionEvent>(&data)
                                .map_err(|e| TransportError::JSONError(e));
                            let mut ch = sub.channel.clone();
                            ch.send(event).await.unwrap();
                        }
                    }
                }
            })
            .await;
        });
        tokio::spawn(async move {
            receiver
                .map(|req| match req {
                    Outbound::PendingRequest(req) => {
                        Message::Text(serde_json::to_string(&req.request).unwrap())
                    }
                    Outbound::Subscription(req) => {
                        Message::Text(serde_json::to_string(&req.request).unwrap())
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
