use super::Address;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SubscribeRequest {
    #[serde(rename = "accounts")]
    Accounts(Vec<Address>),
    #[serde(rename = "streams")]
    Streams(Vec<String>),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum SubscriptionEvent {
    #[serde(rename = "ledgerClosed")]
    LedgerClosed(LedgerClosed),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LedgerClosed {
    /// The identifying hash of the ledger version that was closed.
    pub ledger_hash: String,
}
