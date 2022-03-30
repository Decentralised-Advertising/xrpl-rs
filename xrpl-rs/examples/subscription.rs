use futures::StreamExt;
use xrpl_rs::{
    transports::WebSocket,
    types::subscribe::{SubscribeRequest, SubscriptionEvent},
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
        .subscribe(SubscribeRequest::Streams(vec!["ledger".to_owned()]))
        .await
        .unwrap();
    // Print each ledger event as it comes through.
    ledgers
        .for_each(|event| async move {
            match event {
                Ok(SubscriptionEvent::LedgerClosed(ledger_closed)) => {
                    println!("{}", ledger_closed.ledger_hash);
                }
                Err(e) => {
                    println!("error: {:?}", e);
                }
            }
        })
        .await;
}
