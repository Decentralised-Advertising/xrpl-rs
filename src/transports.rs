use std::fmt::Debug;
use super::Error;
use super::types::{Response,Result as APIResult, Error as APIError, RequestId};
use async_trait::async_trait;
use futures::{
    channel::{mpsc, oneshot},
    task::Context,
};
use reqwest::{header::CONTENT_TYPE, Client};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use url::{ParseError, Url};
use websocket::{
    futures::{sink::Sink, Async, AsyncSink, Future, Stream},
    r#async::Client as WSClient,
    ClientBuilder, OwnedMessage, WebSocketError,
};

#[async_trait(?Send)]
pub trait Transport {
    async fn send_request<Params: Serialize, Res: DeserializeOwned + Debug>(
        &self,
        method: &str,
        params: Params,
    ) -> Result<Res, TransportError>;
}

#[async_trait(?Send)]
pub trait DuplexTransport: Transport {
    fn subscribe<T: DeserializeOwned, S: Stream<Item = T>>(&self) -> Result<S, ()>;
    fn unsubscribe(&self) -> Result<(), ()>;
}

#[derive(Debug)]
pub enum TransportError {
    NoEndpoint,
    Error(&'static str),
    InvalidEndpoint(ParseError),
    ReqwestError(reqwest::Error),
    WebSocketError(WebSocketError),
    ErrorResponse(String),
    APIError(APIError),
}

impl From<reqwest::Error> for TransportError {
    fn from(e: reqwest::Error) -> Self {
        Self::ReqwestError(e)
    }
}

impl From<WebSocketError> for TransportError {
    fn from(e: WebSocketError) -> Self {
        Self::WebSocketError(e)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsonRPCRequest<T: Serialize> {
    pub id: RequestId,
    pub method: String,
    pub params: T,
}

unsafe impl<T: Serialize> Send for JsonRPCRequest<T> {}

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

#[async_trait(?Send)]
impl Transport for HTTP {
    async fn send_request<Params: Serialize, Res: DeserializeOwned + Debug>(
        &self,
        method: &str,
        params: Params,
    ) -> Result<Res, TransportError> {
        match self
            .inner
            .post(self.base_url.clone())
            .header(CONTENT_TYPE, "application/json")
            .json(&JsonRPCRequest {
                id: RequestId::Number(self.counter.fetch_add(1u64, Ordering::SeqCst)),
                method: method.to_owned(),
                params: vec![params],
            })
            .send()
            .await?
            .json::<Response<Res>>()
            .await
            .map_err(|e| TransportError::ReqwestError(e))
            .and_then(|r| {
                if r.status != Some("success".to_owned()) {
                    return Err(TransportError::ErrorResponse(format!("{:?}", r)));
                }
                Ok(r)
            })?
            .result {
                APIResult::Ok(result) => Ok(result),
                APIResult::Error(e) => Err(TransportError::APIError(e))
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
        request: JsonRPCRequest<Value>,
        response: Arc<oneshot::Sender<Response<Value>>>,
    },
    Subscription {
        id: RequestId,
        request: JsonRPCRequest<Value>,
        channel: mpsc::UnboundedSender<Response<Value>>,
    },
}

pub struct WebSocket {
    counter: Arc<AtomicU64>,
    sender: mpsc::UnboundedSender<PendingRequest>,
}

impl WebSocket {
    pub fn new(sender: mpsc::UnboundedSender<PendingRequest>) -> Self {
        Self {
            counter: Arc::new(AtomicU64::new(1u64)),
            sender,
        }
    }
}

#[async_trait(?Send)]
impl Transport for WebSocket {
    async fn send_request<Params: Serialize, Res: DeserializeOwned + Debug>(
        &self,
        method: &str,
        params: Params,
    ) -> Result<Res, TransportError> {
        Err(TransportError::NoEndpoint)
    }
}

#[async_trait]
impl DuplexTransport for WebSocket {
    fn subscribe<T: DeserializeOwned, St: Stream<Item = T>>(&self) -> Result<St, ()> {
        Err(())
    }
    fn unsubscribe(&self) -> Result<(), ()> {
        Err(())
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
        ClientBuilder::new(self.endpoint.clone().unwrap().as_str())
            .unwrap()
            .async_connect(None)
            .map(|(client, _)| {
                let (mut sink, mut stream) = client.split();
                let (sender, mut receiver) = mpsc::unbounded::<PendingRequest>();
                let mut pending_requests: HashMap<RequestId, PendingRequest> = HashMap::new();
                let ws = WebSocket::new(sender);
                // Replace with tokio::spawn and future instead of dumb infinite loop...
                std::thread::spawn(move || {
                    loop {
                        // Handle outgoing requests.
                        loop {
                            // 1. Receive from reciever channel
                            // 2. Create and store pending request (call or sub).
                            // 3. Write to sink.
                            if let Some(pending_request) = receiver.try_next().ok().flatten() {
                                // Get the id from the pending request.
                                let id = match pending_request {
                                    PendingRequest::Call { ref id, .. } => id.clone(),
                                    PendingRequest::Subscription { ref id, .. } => id.clone(),
                                };
                                if pending_requests.contains_key(&id) {
                                    log::warn!("request already exists with id: {:?}", &id);
                                    break;
                                }
                                // Get the rpc request from the pending request.
                                let request = match pending_request {
                                    PendingRequest::Call { ref request, .. } => request.clone(),
                                    PendingRequest::Subscription { ref request, .. } => request.clone(),
                                };
                                if let Ok(req_json) = serde_json::to_string(&request) {
                                    // Add to pending requests.
                                    pending_requests.insert(id, pending_request);
                                    // Poll sink send to until the send has completed.
                                    loop {
                                        match sink.start_send(OwnedMessage::Text(req_json.clone())) {
                                            Ok(AsyncSink::Ready) => {
                                                break;
                                            }
                                            Ok(AsyncSink::NotReady(_)) => {
                                                continue;
                                            }
                                            Err(e) => {
                                                log::warn!("error sending request: {:?}", e);
                                            }
                                        }
                                    }
                                }
                            }
                            break;
                        }
        
                        // Handle incoming requests.
                        loop {
                            // 1. Receive from stream
                            // 2. Lookup id in pending requests.
                            // 3. Send received value to pending request channel (call or sub).WebSocket
                            // 4. Remove pending request (call only)
                            match stream.poll() {
                                Ok(Async::Ready(rec)) => {
                                    if let Some(OwnedMessage::Text(txt)) = rec {
                                        match serde_json::from_str::<Response<Value>>(&txt) {
                                            Ok(res) => {
                                                log::debug!("received message: {:?}", res);
                                                if let Some(pending_request) =
                                                    pending_requests.remove(&res.id.as_ref().unwrap())
                                                {
                                                    match pending_request {
                                                        PendingRequest::Call { response, .. } => {
                                                            let sender = Arc::try_unwrap(response).unwrap();
                                                            sender.send(res.clone()).unwrap();
                                                        }
                                                        PendingRequest::Subscription {
                                                            mut channel,
                                                            ..
                                                        } => {
                                                            // Poll channel send to until the send has succeeded.
                                                            loop {
                                                                match channel.start_send(res.clone()) {
                                                                    Ok(()) => {
                                                                        break;
                                                                    }
                                                                    Err(e) => {
                                                                        log::warn!(
                                                                            "error sending response: {:?}",
                                                                            e
                                                                        );
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                log::error!("received invalid message: {:?}", e);
                                            }
                                        }
                                    }
                                }
                                Ok(Async::NotReady) => {
                                    break;
                                }
                                Err(e) => {
                                    log::warn!("error receiving response: {:?}", e);
                                }
                            }
                            break;
                        }
                    }
                });
                ws
            })
            .wait()
            .map_err(|e| TransportError::WebSocketError(e))
    }
}

// impl<TSink, TStream, TError> Sink<TItem> for WebSocket<TSink, TStream>
// where
// 	TSink: Sink<OwnedMessage, Error = TError>,
// 	TStream: Stream<Item = OwnedMessage>,
// 	TError: Into<TransportError>,
// {
// 	type SinkItem = String;
// 	type SinkError = TransportError;

// 	fn start_send(&mut self, request: Self::SinkItem) -> Result<AsyncSink<Self::SinkItem>, Self::SinkError> {
// 		self.queue.push_back(OwnedMessage::Text(request));
// 		Ok(AsyncSink::Ready)
// 	}

// 	fn poll_complete(&mut self) -> Result<Async<()>, Self::SinkError> {
// 		loop {
// 			match self.queue.pop_front() {
// 				Some(request) => match self.sink.start_send(request) {
// 					Ok(AsyncSink::Ready) => continue,
// 					Ok(AsyncSink::NotReady(request)) => {
// 						self.queue.push_front(request);
// 						break;
// 					}
// 					Err(error) => return Err(RpcError::Other(error.into())),
// 				},
// 				None => break,
// 			}
// 		}
// 		self.sink.poll_complete().map_err(|error| RpcError::Other(error.into()))
// 	}
// }

// impl<TSink, TStream, TItem, TError> Stream for WebSocket<TSink, TStream>
// where
// 	TSink: Sink<TItem, Error = TError>,
// 	TStream: Stream<Item = OwnedMessage>,
// 	TError: Into<TransportError>,
// {
// 	type Item = TItem;

// 	fn poll_next(&mut self) -> core::task::Poll<Option<Self::Item>> {
// 		loop {
// 			match self.stream.poll_next() {
// 				Ok(Async::Ready(Some(message))) => match message {
// 					OwnedMessage::Text(data) => return Ok(Async::Ready(Some(data))),
// 					OwnedMessage::Binary(data) => info!("server sent binary data {:?}", data),
// 					OwnedMessage::Ping(p) => self.queue.push_front(OwnedMessage::Pong(p)),
// 					OwnedMessage::Pong(_) => {}
// 					OwnedMessage::Close(c) => self.queue.push_front(OwnedMessage::Close(c)),
// 				},
// 				Ok(Async::Ready(None)) => {
// 					// TODO try to reconnect (#411).
// 					return Ok(Async::Ready(None));
// 				}
// 				Ok(Async::NotReady) => return Ok(Async::NotReady),
// 				Err(error) => return Err(RpcError::Other(error.into())),
// 			}
// 		}
// 	}
// }
