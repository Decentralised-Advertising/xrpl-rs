use super::Address;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SubscribeRequest {
    #[serde(rename = "accounts")]
    Accounts(Vec<Address>),
    #[serde(rename = "streams")]
    Streams(Vec<String>),
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum SubscriptionEvent {
    #[serde(rename = "ledgerClosed")]
    LedgerClosed(LedgerClosed),
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LedgerClosed {
    /// The identifying hash of the ledger version that was closed.
    pub ledger_hash: String,
}
