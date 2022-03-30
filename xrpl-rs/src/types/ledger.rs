use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::LedgerInfo;

#[skip_serializing_none]
#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct LedgerRequest {
    /// (Optional) A 20-byte hex string for the ledger version to use. (See Specifying Ledgers)
    pub ledger_hash: Option<String>,
    /// (Optional) The ledger index of the ledger to use, or a shortcut string to choose a ledger automatically. (See Specifying Ledgers)
    pub ledger_index: LedgerRequestIndex,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(untagged)]
pub enum LedgerRequestIndex {
    #[serde(rename="validated")]
    Validated,
    Index(u32),
}

impl Default for LedgerRequestIndex {
    fn default() -> Self {
        Self::Validated
    }
}

#[skip_serializing_none]
#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct LedgerResponse {
    /// The complete header data of this ledger.
    pub ledger: Ledger,
}

#[skip_serializing_none]
#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Ledger {
    #[serde(flatten)]
    pub ledger_info: LedgerInfo,
}