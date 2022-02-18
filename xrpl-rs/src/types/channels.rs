use super::{Address, CurrencyAmount, LedgerInfo, PaginationInfo, SignerList, AccountRoot, LedgerEntry, BigInt};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct ChannelVerifyRequest {
    pub amount: BigInt,
    pub channel_id: String,
    pub public_key: String,
    pub signature: String,
}

#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct ChannelVerifyResponse {
    pub signature_verified: bool,
}
