use crate::types::{Address, CurrencyAmount, Drops};
use serde::{Deserialize, Serialize};

macro_rules! into_transaction {
    ($name: ident) => {
        impl $name {
            pub fn into_transaction(self) -> Transaction {
                let mut tx = Transaction::default();
                tx.tx = Some(TransactionType::$name(self));
                tx
            }
        }
    };
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Default)]
#[serde(rename_all = "PascalCase")]
pub struct Transaction {
    pub account: Address,
    pub fee: Drops,
    pub sequence: u32,
    pub signing_pub_key: String,
    pub txn_signature: Option<String>,
    pub flags: u32,
    #[serde(flatten)]
    pub tx: Option<TransactionType>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(tag = "TransactionType", rename_all = "PascalCase")]
pub enum TransactionType {
    Payment(Payment),
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

into_transaction!(Payment);

// #[test]
// pub fn test_serialize() {
//     let j = serde_json::json!(Payment{
//         amount: CurrencyAmount::XRP("14000".to_owned()),
//         destination: "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B".to_owned(),
//     });
//     assert_eq!(hex::encode(serde_xrpl::ser::to_bytes(&j).unwrap()), "");
// }