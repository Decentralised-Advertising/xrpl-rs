use crate::types::{Address, CurrencyAmount};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(tag = "TransactionType", rename_all = "PascalCase")]
pub enum Transaction {
    Unknown,
    Payment(Payment),
}

impl Default for Transaction {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct EscrowCreate {}

#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Payment {
    /// The amount of currency to deliver. For non-XRP amounts, the nested field names MUST be lower-case. If the tfPartialPayment flag is set, deliver up to this amount instead.
    pub amount: CurrencyAmount,
    /// The unique address of the account receiving the payment.
    pub destination: Address,
}
