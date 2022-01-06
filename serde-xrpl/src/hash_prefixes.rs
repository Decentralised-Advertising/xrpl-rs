use hex_literal::hex;

pub static TRANSACTION_ID: [u8; 4] = hex!("54584e00");
  // transaction plus metadata
pub static TRANSACTION: [u8; 4] = hex!("534e4400");
// account state
pub static ACCOUNT_STATE_ENTRY: [u8; 4] = hex!("4d4c4e00");
// inner node in tree
pub static INNER_NODE: [u8; 4] = hex!("4d494e00");
// ledger master data for signing
pub static LEDGER_HEADER: [u8; 4] = hex!("4c575200");
// inner transaction to sign
pub static TRANSACTION_SIG: [u8; 4] = hex!("53545800");
// inner transaction to sign
pub static TRANSACTION_MULTI_SIG: [u8; 4] = hex!("534d5400");
// validation for signing
pub static VALIDATION: [u8; 4] = hex!("56414c00");
// proposal for signing
pub static PROPOSAL: [u8; 4] = hex!("50525000");
// payment channel claim
pub static PAYMENT_CHANNEL_CLAIM: [u8; 4] = hex!("434c4d00");
