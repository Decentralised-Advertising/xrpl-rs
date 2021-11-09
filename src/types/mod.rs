pub mod account;
pub mod submit;

use serde;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// An address used to identify an account.
type Address = String;

/// A Marker can be used to paginate the server response. It's content is intentionally undefined. Each server can define a marker as desired.
type Marker = Value;

/// Unique request id.
///
/// NOTE Assigning same id to different requests will cause the previous request to be unsubscribed.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RequestId {
    /// A numerical ID, represented by a `u64`.
    Number(u64),
    /// A non-numerical ID, for example a hash.
    String(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct LedgerInfo {
    /// (Optional) A 20-byte hex string for the ledger version to use. (See Specifying Ledgers)
    pub ledger_hash: Option<String>,
    /// (Optional) The ledger index of the ledger to use, or a shortcut string to choose a ledger automatically. (See Specifying Ledgers)
    pub ledger_index: Option<i64>,    
    /// (Omitted if ledger_index is provided instead) The ledger index of the current in-progress ledger, which was used when retrieving this information.
    pub ledger_current_index: Option<i64>,    
    /// (May be omitted) If true, the information in this response comes from a validated ledger version. Otherwise, the information is subject to change. New in: rippled 0.90.0 
    pub validated: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PaginationInfo {
    /// (Optional) Limit the number of transactions to retrieve. Cannot be less than 10 or more than 400. The default is 200.
    pub limit: Option<i64>,
    /// (Optional) Value from a previous paginated response. Resume retrieving data where that response left off. Updated in: rippled 1.5.0.
    pub marker: Option<Marker>,    
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Response<T> {
    /// (WebSocket only) ID provided in the request that prompted this response
    pub id: Option<RequestId>,
    /// (WebSocket only) The value success indicates the request was successfully received and understood by the server. Some client libraries omit this field on success.
    pub status: Option<String>,
    /// (WebSocket only) The value response indicates a direct response to an API request. Asynchronous notifications use a different value such as ledgerClosed or transaction.
    pub r#type: Option<String>,
    /// The result of the query; contents vary depending on the command.
    pub result: T,
    /// (May be omitted) If this field is provided, the value is the string load. This means the client is approaching the rate limiting threshold where the server will disconnect this client.
    pub warning: Option<String>,
    /// (May be omitted) If this field is provided, it contains one or more Warnings Objects with important warnings. For details, see API Warnings. New in: rippled 1.5.0
    /// TODO: Add Warnings Object.
    pub warnings: Option<Vec<Value>>,
    /// (May be omitted) If true, this request and response have been forwarded from a Reporting Mode server to a P2P Mode server (and back) because the request requires data that is not available in Reporting Mode. The default is false.
    pub forwarded: Option<bool>,
}

#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct AccountOfferRequest {
    pub account: Address,
    #[serde(flatten)]
    pub ledger_info: LedgerInfo,
    pub limit: Option<i64>,
    pub strict: Option<bool>,
}

#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct AccountOfferResponse {
    pub account: Address,
    pub offers: Vec<AccountOffer>,
}

#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct SignerList {
    #[serde(rename = "SignerEntries")]
    pub signer_entries: Vec<SignerEntry>,
    #[serde(rename = "SignerQuorum")]
    pub signer_quorum: u32,
}

#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct SignerEntry {
    #[serde(rename = "Account")]
    pub account: String,
    #[serde(rename = "SignerWeight")]
    pub signer_weight: u16,
}

#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct AccountOffer {
    pub flags: u64,
    pub seq: u64,
    pub taker_gets: CurrencyAmount,
    pub taker_pays: CurrencyAmount,
    pub quality: String,
    pub expiration: u64,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(untagged)]
pub enum CurrencyAmount {
    XRP(String),
    IssuedCurrency(IssuedCurrencyAmount),
}

impl Default for CurrencyAmount {
    fn default() -> Self {
        return Self::XRP("0".to_owned());
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct IssuedCurrencyAmount {
    pub value: String,
    pub currency: String,
    pub issuer: Address,
}

#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct TransactionEntryRequest {
    pub tx_hash: Option<String>,
    pub ledger_index: Option<u64>,
    pub ledger_hash: Option<String>,
}

#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct TransactionEntryResponse {
    pub tx_json: Option<Value>,
    pub ledger_index: Option<u64>,
    pub ledger_hash: Option<String>,
}
