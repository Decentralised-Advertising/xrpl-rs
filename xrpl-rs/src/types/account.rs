use super::{Address, CurrencyAmount, LedgerInfo, PaginationInfo, SignerList, AccountRoot, LedgerEntry};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Used to make account_channels requests.
#[skip_serializing_none]
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

#[skip_serializing_none]
#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct AccountChannelsResponse {
    /// The address of the source/owner of the payment channels. This corresponds to the account field of the request.
    pub account: Address,
    /// Payment channels owned by this account. Updated in: rippled 1.5.0
    pub channels: Vec<AccountChannel>,
    #[serde(flatten)]
    pub ledger_info: LedgerInfo,
    #[serde(flatten)]
    pub pagination: PaginationInfo,
}

#[skip_serializing_none]
#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct AccountChannel {
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
    pub expiration: Option<usize>,
    /// (May be omitted) Time, in seconds since the Ripple Epoch, of this channel's immutable expiration, if one was specified at channel creation. If this is before the close time of the most recent validated ledger, the channel is expired.
    pub cancel_after: Option<usize>,
    /// (May be omitted) A 32-bit unsigned integer to use as a source tag for payments through this payment channel, if one was specified at channel creation. This indicates the payment channel's originator or other purpose at the source account. Conventionally, if you bounce payments from this channel, you should specify this value in the DestinationTag of the return payment.
    pub source_tag: Option<usize>,
    /// (May be omitted) A 32-bit unsigned integer to use as a destination tag for payments through this channel, if one was specified at channel creation. This indicates the payment channel's beneficiary or other purpose at the destination account.
    pub destination_tag: Option<usize>,
}

/// Used to make account_currencies requests.
#[skip_serializing_none]
#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct AccountCurrenciesRequest {
    /// A unique identifier for the account, most commonly the account's Address.
    pub account: Address,
    #[serde(flatten)]
    pub ledger_info: LedgerInfo,
    #[serde(flatten)]
    pub pagination: PaginationInfo,
    /// (Optional) If true, then the account field only accepts a public key or XRP Ledger address. Otherwise, account can be a secret or passphrase (not recommended). The default is false.
    pub strict: Option<bool>,
}

/// The response type for an account_currencies request.
#[skip_serializing_none]
#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct AccountCurrenciesResponse {
    #[serde(flatten)]
    pub ledger_info: LedgerInfo,
    /// Array of Currency Codes for currencies that this account can receive.
    pub receive_currencies: Option<Vec<String>>,
    /// Array of Currency Codes for currencies that this account can send.
    pub send_currencies: Option<Vec<String>>,
}

/// Used to make account_info requests.
#[skip_serializing_none]
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

#[skip_serializing_none]
#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct AccountInfoResponse {
    /// The AccountRoot ledger object with this account's information, as stored in the ledger.
    pub account_data: AccountRoot,
    /// (Omitted unless the request specified signer_lists and at least one SignerList is associated with the account.) Array of SignerList ledger objects associated with this account for Multi-Signing. Since an account can own at most one SignerList, this array must have exactly one member if it is present.
    pub signer_lists: Option<Vec<SignerList>>,
    #[serde(flatten)]
    pub ledger_info: LedgerInfo,
    /// (Omitted unless queue specified as true and querying the current open ledger.) Information about queued transactions sent by this account. This information describes the state of the local rippled server, which may be different from other servers in the peer-to-peer XRP Ledger network. Some fields may be omitted because the values are calculated "lazily" by the queuing mechanism.
    pub queue_data: Option<AccountQueueData>,
}

#[skip_serializing_none]
#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct AccountQueueData {
    /// Number of queued transactions from this address.
    pub txn_count: Option<i64>,
    /// (May be omitted) Whether a transaction in the queue changes this address's ways of authorizing transactions. If true, this address can queue no further transactions until that transaction has been executed or dropped from the queue.
    pub auth_change_queued: Option<bool>,
    /// (May be omitted) The lowest Sequence Number among transactions queued by this address.
    pub lowest_sequence: Option<i64>,
    /// (May be omitted) The highest Sequence Number among transactions queued by this address.
    pub highest_sequence: Option<i64>,
    /// (May be omitted) Integer amount of drops of XRP that could be debited from this address if every transaction in the queue consumes the maximum amount of XRP possible.
    pub max_spend_drops_total: Option<CurrencyAmount>,
    /// (May be omitted) Information about each queued transaction from this address.
    pub transactions: Option<Vec<AccountQueuedTransaction>>,
}

#[skip_serializing_none]
#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct AccountQueuedTransaction {
    /// Whether this transaction changes this address's ways of authorizing transactions.
    pub auth_change_queued: Option<bool>,
    /// The Transaction Cost of this transaction, in drops of XRP.
    pub fee: Option<CurrencyAmount>,
    /// The transaction cost of this transaction, relative to the minimum cost for this type of transaction, in fee levels.
    pub fee_level: Option<CurrencyAmount>,
    /// The maximum amount of XRP, in drops, this transaction could send or destroy.
    pub max_spend_drops: Option<CurrencyAmount>,
    /// The Sequence Number of this transaction.
    pub seq: Option<i64>,
}

/// Used to make account_line requests.
#[skip_serializing_none]
#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct AccountLinesRequest {
    /// A unique identifier for the account, most commonly the account's Address.
    pub account: Address,
    #[serde(flatten)]
    pub ledger_info: LedgerInfo,
    /// (Optional) The Address of a second account. If provided, show only lines of trust connecting the two accounts.
    pub peer: Option<Address>,
    #[serde(flatten)]
    pub pagination: Option<PaginationInfo>,
}

#[skip_serializing_none]
#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct AccountLinesResponse {
    /// A unique identifier for the account, most commonly the account's Address.
    pub account: Address,
    /// Array of trust line objects, as described below. If the number of trust lines is large, only returns up to the limit at a time.
    pub lines: Option<Vec<AccountTrustLine>>,
    #[serde(flatten)]
    pub ledger_info: LedgerInfo,
    #[serde(flatten)]
    pub pagination: Option<PaginationInfo>,
}

#[skip_serializing_none]
#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct AccountTrustLine {
    /// The unique Address of the counterparty to this trust line.
    pub account: Address,
    /// Representation of the numeric balance currently held against this line. A positive balance means that the perspective account holds value; a negative balance means that the perspective account owes value.
    pub balance: String,
    /// A Currency Code identifying what currency this trust line can hold.
    pub currency: String,
    /// The maximum amount of the given currency that this account is willing to owe the peer account
    pub limit: String,
    /// The maximum amount of currency that the counterparty account is willing to owe the perspective account
    pub limit_peer: String,
    /// Rate at which the account values incoming balances on this trust line, as a ratio of this value per 1 billion units. (For example, a value of 500 million represents a 0.5:1 ratio.) As a special case, 0 is treated as a 1:1 ratio.
    pub quality_in: usize,
    /// Rate at which the account values outgoing balances on this trust line, as a ratio of this value per 1 billion units. (For example, a value of 500 million represents a 0.5:1 ratio.) As a special case, 0 is treated as a 1:1 ratio.
    pub quality_out: usize,
    /// (May be omitted) If true, this account has enabled the No Ripple flag for this trust line. If present and false, this account has disabled the No Ripple flag, but, because the account also has the Default Ripple flag enabled, that is not considered the default state. If omitted, the account has the No Ripple flag disabled for this trust line and Default Ripple disabled. Updated in: rippled 1.7.0
    pub no_ripple: Option<bool>,
    /// (May be omitted) If true, the peer account has enabled the No Ripple flag for this trust line. If present and false, this account has disabled the No Ripple flag, but, because the account also has the Default Ripple flag enabled, that is not considered the default state. If omitted, the account has the No Ripple flag disabled for this trust line and Default Ripple disabled. Updated in: rippled 1.7.0
    pub no_ripple_peer: Option<bool>,
    /// (May be omitted) If true, this account has authorized this trust line. The default is false.
    pub authorized: Option<bool>,
    /// (May be omitted) If true, the peer account has authorized this trust line. The default is false.
    pub peer_authorized: Option<bool>,
    /// (May be omitted) If true, this account has frozen this trust line. The default is false.
    pub freeze: Option<bool>,
    /// (May be omitted) If true, the peer account has frozen this trust line. The default is false.
    pub freeze_peer: Option<bool>,
}

#[skip_serializing_none]
#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct AccountOfferRequest {
    pub account: Address,
    #[serde(flatten)]
    pub ledger_info: LedgerInfo,
    pub limit: Option<i64>,
    pub strict: Option<bool>,
}

#[skip_serializing_none]
#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct AccountOfferResponse {
    pub account: Address,
    pub offers: Vec<AccountOffer>,
}

#[skip_serializing_none]
#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct AccountOffer {
    pub flags: u64,
    pub seq: u64,
    pub taker_gets: CurrencyAmount,
    pub taker_pays: CurrencyAmount,
    pub quality: String,
    pub expiration: u64,
}

/// Used to make account_objects requests.
#[skip_serializing_none]
#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct AccountObjectsRequest {
    /// A unique identifier for the account, most commonly the account's address.
    pub account: Address,
    /// (Optional) If included, filter results to include only this type of ledger object. The valid types are: check , deposit_preauth, escrow, offer, payment_channel, signer_list, ticket , and state (trust line).
    pub r#type: Option<AccountObjectType>,
    /// (Optional) If true, the response only includes objects that would block this account from being deleted. The default is false. New in: rippled 1.4.0
    pub deletion_blockers_only: Option<bool>,
    #[serde(flatten)]
    pub ledger_info: LedgerInfo,
    #[serde(flatten)]
    pub pagination: PaginationInfo,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AccountObjectType {
    Unknown,
    Check,
    DepositPreauth,
    Escrow,
    Offer,
    PaymentChannel,
    SignerList,
    Ticket,
    State,
}

impl Default for AccountObjectType {
    fn default() -> Self {
        Self::Unknown
    }
}

#[skip_serializing_none]
#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct AccountObjectsResponse {
    /// Unique Address of the account this request corresponds to.
    pub account: Address,
    /// Array of objects owned by this account. Each object is in its raw ledger format.
    pub account_objects: Option<Vec<LedgerEntry>>,
}


/// Used to make account_tx requests.
#[skip_serializing_none]
#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct AccountTXRequest {}

#[skip_serializing_none]
#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct AccountTXResponse {}
