use super::{Address, CurrencyAmount, LedgerInfo, PaginationInfo, SignerList, AccountRoot, LedgerEntry, BigInt};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct ChannelVerifyRequest {
    pub amount: BigInt,
    pub channel_id: String,
    pub public_key: String,
    pub signature: String,
}

#[skip_serializing_none]
#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct ChannelVerifyResponse {
    pub signature_verified: bool,
}
