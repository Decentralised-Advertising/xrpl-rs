use futures::StreamExt;
use serde_json::Value;
use xrpl_rs::{
    transports::WebSocket,
    types::subscribe::SubscribeRequest,
    XRPL,
};

#[tokio::main]
async fn main() {
    // Create a new XRPL client with the HTTP transport pointed at ripple testnet.
    let xrpl = XRPL::new(
        WebSocket::builder()
            .with_endpoint("wss://xrplcluster.com/")
            .unwrap()
            .build()
            .await
            .unwrap(),
    );
    // Subscribe to ledger events.
    let ledgers = xrpl
        .subscribe::<Value>(SubscribeRequest::Streams(vec!["ledger".to_owned()]))
        .await
        .unwrap();
    // Print each ledger event as it comes through. 
    ledgers
        .for_each(|event| async {
            println!("{:?}", event.unwrap());
        })
        .await;
}
