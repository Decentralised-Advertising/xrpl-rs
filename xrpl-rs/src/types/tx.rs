use crate::transaction::types::Transaction;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct TxRequest {
    /// The 256-bit hash of the transaction, as hex.
    pub transaction: String,
    /// (Optional) If true, return transaction data and metadata as binary serialized to hexadecimal strings. If false, return transaction data and metadata as JSON. The default is false.
    pub binary: Option<bool>,
}

#[skip_serializing_none]
#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct TxResponse {
    /// The SHA-512 hash of the transaction
    pub hash: String,
    /// Transaction metadata, which describes the results of the transaction.
    pub meta: Option<Value>,
}