use super::{Address, CurrencyAmount, LedgerInfo, PaginationInfo, SignerList, AccountRoot, LedgerEntry};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Used to make account_channels requests.
#[skip_serializing_none]
#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct FeeRequest {}

#[skip_serializing_none]
#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct FeeResponse {
    /// Various information about the transaction cost (the Fee field of a transaction), in drops of XRP.
    pub drops: FeeResponseDrops,
}

#[skip_serializing_none]
#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct FeeResponseDrops {
    /// The minimum transaction cost that a reference transaction must pay to be included in the current open ledger, represented in drops of XRP.
    pub open_ledger_fee: CurrencyAmount,
}