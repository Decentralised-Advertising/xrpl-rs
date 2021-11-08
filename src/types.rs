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

/// Used to make account_channels requests.
#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct AccountChannelsRequest {
    /// A unique identifier for the account, most commonly the account's Address.
    pub account: Address,
    /// (Optional) The unique identifier of an account, typically the account's Address. If provided, filter results to payment channels whose destination is this account.
    pub destination_account: Option<Address>,
    #[serde(flatten)]
    pub ledger_info: LedgerInfo,
    #[serde(flatten)]
    pub pagination: PaginationInfo,
}

#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct AccountChannelsResponse {
    /// The address of the source/owner of the payment channels. This corresponds to the account field of the request.
    pub account: Address,
    /// Payment channels owned by this account. Updated in: rippled 1.5.0 
    pub channels: Vec<Channel>,
    #[serde(flatten)]
    pub ledger_info: LedgerInfo,
    #[serde(flatten)]
    pub pagination: PaginationInfo,
}

#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Channel {
    /// The owner of the channel, as an Address.
    pub account: Address,
    /// The total amount of XRP, in drops allocated to this channel.
    pub amount: CurrencyAmount,
    /// The total amount of XRP, in drops, paid out from this channel, as of the ledger version used. (You can calculate the amount of XRP left in the channel by subtracting balance from amount.)
    pub balance: CurrencyAmount,
    /// A unique ID for this channel, as a 64-character hexadecimal string. This is also the ID of the channel object in the ledger's state data.
    pub channel_id: String,
    /// The destination account of the channel, as an Address. Only this account can receive the XRP in the channel while it is open.
    pub destination_account: Address,
    /// The number of seconds the payment channel must stay open after the owner of the channel requests to close it.
    pub settle_delay: usize,
    /// (May be omitted) The public key for the payment channel in the XRP Ledger's base58 format. Signed claims against this channel must be redeemed with the matching key pair.
    pub public_key: Option<String>,
    /// (May be omitted) The public key for the payment channel in hexadecimal format, if one was specified at channel creation. Signed claims against this channel must be redeemed with the matching key pair.
    pub public_key_hex: Option<String>,
    /// (May be omitted) Time, in seconds since the Ripple Epoch, when this channel is set to expire. This expiration date is mutable. If this is before the close time of the most recent validated ledger, the channel is expired.
    pub expiration: usize,
    /// (May be omitted) Time, in seconds since the Ripple Epoch, of this channel's immutable expiration, if one was specified at channel creation. If this is before the close time of the most recent validated ledger, the channel is expired.
    pub cancel_after: usize,
    /// (May be omitted) A 32-bit unsigned integer to use as a source tag for payments through this payment channel, if one was specified at channel creation. This indicates the payment channel's originator or other purpose at the source account. Conventionally, if you bounce payments from this channel, you should specify this value in the DestinationTag of the return payment.
    pub source_tag: usize,
    /// (May be omitted) A 32-bit unsigned integer to use as a destination tag for payments through this channel, if one was specified at channel creation. This indicates the payment channel's beneficiary or other purpose at the destination account.
    pub destination_tag: usize,
}

/// Used to make account_info requests.
#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct AccountInfoRequest {
    /// A unique identifier for the account, most commonly the account's Address.
    pub account: Address,
    /// (Optional) If true, then the account field only accepts a public key or XRP Ledger address. Otherwise, account can be a secret or passphrase (not recommended). The default is false.
    pub strict: Option<bool>,
    #[serde(flatten)]
    pub ledger_info: LedgerInfo,
    /// (Optional) If true, and the FeeEscalation amendment is enabled, also returns stats about queued transactions associated with this account. Can only be used when querying for the data from the current open ledger. New in: rippled 0.33.0  Not available from servers in Reporting Mode.
    pub queue: Option<bool>,
    /// (Optional) If true, and the MultiSign amendment is enabled, also returns any SignerList objects associated with this account. New in: rippled 0.31.0
    pub signer_lists: Option<bool>,
}

#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct AccountInfoResponse {
    /// The AccountRoot ledger object with this account's information, as stored in the ledger.
    pub account_data: AccountRoot,
    /// (Omitted unless the request specified signer_lists and at least one SignerList is associated with the account.) Array of SignerList ledger objects associated with this account for Multi-Signing. Since an account can own at most one SignerList, this array must have exactly one member if it is present.
    pub signer_lists: Option<Vec<SignerList>>,
    #[serde(flatten)]
    pub ledger_info: LedgerInfo,
    /// (Omitted unless queue specified as true and querying the current open ledger.) Information about queued transactions sent by this account. This information describes the state of the local rippled server, which may be different from other servers in the peer-to-peer XRP Ledger network. Some fields may be omitted because the values are calculated "lazily" by the queuing mechanism.
    pub queue_data: Option<QueueData>,
}

#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct QueueData {}

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
pub struct AccountRoot {
    #[serde(rename = "Account")]
    pub account: Address,
    #[serde(rename = "Balance")]
    pub balance: String,
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

#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct SubmitRequest {}

#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct SubmitResponse {}

#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct EscrowCreateRequest {}

#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct EscrowCreateResponse {}
