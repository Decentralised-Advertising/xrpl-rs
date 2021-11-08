use super::transports::{JsonRPCRequest, JsonRPCResponse, RequestId};
use futures::channel::mpsc;
use futures::{
    task::{Context, Poll},
    Future, Sink, Stream, StreamExt,
};
use log::debug;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::pin::Pin;

struct Subscription {
    /// Subscription id received when subscribing.
    id: RequestId,
    /// Where to send messages to.
    channel: mpsc::UnboundedSender<JsonRPCResponse<Value>>,
}

impl Subscription {
    fn new(id: RequestId, channel: mpsc::UnboundedSender<JsonRPCResponse<Value>>) -> Self {
        Subscription { id, channel }
    }
}

enum PendingRequest {
    Call(mpsc::UnboundedSender<JsonRPCResponse<Value>>),
    Subscription(Subscription),
}

/// The Duplex handles sending and receiving asynchronous
/// messages through an underlying transport.
pub struct Duplex<TSink, TStream> {
    /// Channel from the client.
    channel: Option<
        mpsc::UnboundedReceiver<(
            JsonRPCRequest<Value>,
            mpsc::UnboundedSender<JsonRPCResponse<Value>>,
        )>,
    >,
    /// Requests that haven't received a response yet.
    pending_requests: HashMap<RequestId, PendingRequest>,
    /// A map from the subscription name to the subscription.
    subscriptions: HashMap<(RequestId, String), Subscription>,
    /// Incoming messages from the underlying transport.
    stream: Pin<Box<TStream>>,
    /// Unprocessed incoming messages.
    incoming: VecDeque<(
        RequestId,
        JsonRPCResponse<Value>,
        Option<String>,
        Option<RequestId>,
    )>,
    /// Unprocessed outgoing messages.
    outgoing: VecDeque<String>,
    /// Outgoing messages from the underlying transport.
    sink: Pin<Box<TSink>>,
}

impl<TSink, TStream> Duplex<TSink, TStream> {
    /// Creates a new `Duplex`.
    fn new(
        sink: Pin<Box<TSink>>,
        stream: Pin<Box<TStream>>,
        channel: mpsc::UnboundedReceiver<(
            JsonRPCRequest<Value>,
            mpsc::UnboundedSender<JsonRPCResponse<Value>>,
        )>,
    ) -> Self {
        log::debug!("open");
        Duplex {
            channel: Some(channel),
            pending_requests: Default::default(),
            subscriptions: Default::default(),
            stream,
            incoming: Default::default(),
            outgoing: Default::default(),
            sink,
        }
    }
}

/// Creates a new `Duplex`, along with a channel to communicate
pub fn duplex<TSink, TStream>(
    sink: Pin<Box<TSink>>,
    stream: Pin<Box<TStream>>,
) -> (
    Duplex<TSink, TStream>,
    mpsc::UnboundedSender<(
        JsonRPCRequest<Value>,
        mpsc::UnboundedSender<JsonRPCResponse<Value>>,
    )>,
)
where
    TSink: Sink<String>,
    TStream: Stream<Item = websocket::OwnedMessage>,
{
    let (sender, receiver) = mpsc::unbounded();
    let client = Duplex::new(sink, stream, receiver);
    (client, sender.into())
}

impl<TSink, TStream> Future for Duplex<TSink, TStream>
where
    TSink: Sink<String>,
    TStream: Stream<Item = String>,
{
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        // Handle requests from the client.
        log::debug!("handle requests from client");
        loop {
            // Check that the client channel is open
            let channel = match self.channel.as_mut() {
                Some(channel) => channel,
                None => break,
            };
            let next = match channel.poll_next_unpin(cx) {
                Poll::Ready(Some(next)) => next,
                Poll::Ready(None) => {
                    // When the channel is dropped we still need to finish
                    // outstanding requests.
                    self.channel.take();
                    break;
                }
                Poll::Pending => break,
            };
            let msg = next.0;
            let is_sub = msg.method == "subscribe";
            let id = msg.id.clone();
            if let Ok(msg_json) = serde_json::to_string(&msg) {
                match is_sub {
                    true => {
                        log::debug!("creating subscription with request id: {:?}", &id);
                        let sub = Subscription::new(id, next.1);
                        if self
                            .pending_requests
                            .insert(id.clone(), PendingRequest::Subscription(sub))
                            .is_some()
                        {
                            log::warn!("reuse of request id {:?}", id);
                        }
                    }
                    false => {
                        log::debug!("making rpc call with request id: {:?}", &id);
                        if self
                            .pending_requests
                            .insert(id.clone(), PendingRequest::Call(next.1))
                            .is_some()
                        {
                            log::warn!("reuse of request id {:?}", id);
                        }
                    }
                }
            }
        }

        // Handle stream.
        // Reads from stream and resolve pending requests.
        log::debug!("handle stream");
        loop {
            let response_str = match self.stream.as_mut().poll_next(cx) {
                Poll::Ready(Some(response_str)) => response_str,
                Poll::Ready(None) => {
                    // The websocket connection was closed so the client
                    // can be shutdown. Reopening closed connections must
                    // be handled by the transport.
                    debug!("connection closed");
                    return Poll::Ready(());
                }
                Poll::Pending => break,
            };
            log::debug!("incoming: {}", response_str);

            if let Ok(res) = serde_json::from_str::<JsonRPCResponse<Value>>(&response_str) {
                if let Some(id) = res.id {
                    if let Some(pending_request) = self.pending_requests.get(&id) {
                        match pending_request {
                            PendingRequest::Call(c) => {
                                if c.unbounded_send(res.clone()).is_err() {
                                    log::warn!("received request id {:?}, but the reply channel has closed.", &id);
                                };
                                self.pending_requests.remove(&id);
                            }
                            PendingRequest::Subscription(sub) => {
                                if sub.channel.unbounded_send(res.clone()).is_err() {
                                    log::warn!("received request id {:?}, but the reply channel has closed.", &id);
                                };
                            }
                        }
                    } else {
                        log::warn!("received response for nonexistent request id: {:?}", id);
                    }
                }
            }
        }

        // Handle outgoing queue.
        // Writes queued messages to sink.
        log::debug!("handle outgoing");
        loop {
            let err = || Err(RpcError::Client("closing".into()));
            match self.sink.as_mut().poll_ready(cx) {
                Poll::Ready(Ok(())) => {}
                Poll::Ready(Err(_)) => return err().into(),
                _ => break,
            }
            match self.outgoing.pop_front() {
                Some(request) => {
                    if let Err(_) = self.sink.as_mut().start_send(request) {
                        // the channel is disconnected.
                        return err().into();
                    }
                }
                None => break,
            }
        }
        log::debug!("handle sink");
        let sink_empty = match self.sink.as_mut().poll_flush(cx) {
            Poll::Ready(Ok(())) => true,
            Poll::Ready(Err(_)) => true,
            Poll::Pending => false,
        };

        log::debug!("{:?}", self);
        // Return ready when the future is complete
        if self.channel.is_none()
            && self.outgoing.is_empty()
            && self.incoming.is_empty()
            && self.pending_requests.is_empty()
            && self.subscriptions.is_empty()
            && sink_empty
        {
            log::debug!("close");
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        }
    }
}
