use crate::error::{Error, Result};
use crate::utils::{decode_base58, encode_issued_currency_amount, encode_variable_length};
use std::collections::HashMap;

pub enum Field {}

pub trait ToTypeCode {
    fn to_type_code() -> u16;
}

macro_rules! type_code {
    ($name: ident, $type_code: expr) => {
        impl ToTypeCode for $name {
            fn to_type_code() -> u16 {
                $type_code
            }
        }
    };
}

// type_code(Validation, 10003);
// type_code(Done, -1);
type_code!(Hash128, 4);
type_code!(Blob, 7);
type_code!(AccountID, 8);
type_code!(Amount, 6);
type_code!(Hash256, 5);
type_code!(u8, 16);
type_code!(Vector256, 19);
type_code!(STObject, 14);
// type_code!(Unknown, -2);
// type_code!(Transaction, 10001);
type_code!(Hash160, 17);
// type_code!(PathSet, 18);
// type_code!(LedgerEntry, 10002);
type_code!(u16, 1);
// type_code!(NotPresent, 0);
type_code!(u64, 3);
type_code!(u32, 2);
type_code!(STArray, 15);

#[derive(Debug, Clone)]
pub enum Value {
    Hash128(String),
    Blob(Blob),
    AccountID(String),
    Amount(Amount),
    Hash256(Hash256),
    UInt8(u8),
    Vector256(Vector256),
    STObject(STObject),
    Unknown,
    Transaction(u16),
    Hash160(Hash160),
    PathSet,
    LedgerEntry,
    UInt16(u16),
    NotPresent,
    UInt64(u64),
    UInt32(u32),
    STArray(Vec<Value>),
}

impl Value {
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        match self {
            Self::AccountID(account_id) => {
                let address = decode_base58(account_id, &[0x0])?;
                let length = encode_variable_length(address.len());
                Ok([length, address].concat())
            }
            Self::Amount(amount) => amount.to_bytes(),
            Self::UInt8(u) => Ok(u.to_be_bytes().to_vec()),
            Self::UInt16(u) => Ok(u.to_be_bytes().to_vec()),
            Self::UInt32(u) => Ok(u.to_be_bytes().to_vec()),
            Self::UInt64(u) => Ok(u.to_be_bytes().to_vec()),
            Self::Blob(blob) => {
                let data = hex::decode(&blob.0).unwrap();
                let length = encode_variable_length(data.len());
                Ok([length, data].concat())
            }
            Self::Transaction(tx) => Ok(tx.to_be_bytes().to_vec()),
            Self::Hash256(hash) => Ok(hex::decode(&hash.0).unwrap().to_vec()),
            Self::Vector256(v) => {
                let data: Vec<u8> =
                    v.0.iter()
                        .map(|h| hex::decode(&h.0).unwrap().to_vec())
                        .flatten()
                        .collect();
                let length = encode_variable_length(data.len());
                Ok([length, data].concat())
            }
            _ => {
                unimplemented!()
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Hash160(pub(crate) String);

#[derive(Debug, Clone)]
pub struct Hash128(pub(crate) String);

#[derive(Debug, Clone)]
pub struct Hash256(pub(crate) String);

#[derive(Debug, Clone)]
pub struct AccountID([u8; 20]);

impl AccountID {
    pub fn from_str(s: &str) -> Result<AccountID> {
        decode_base58(s, &[0x00]).map(|bytes| {
            if bytes.len() == 20 {
                return Ok(Self(bytes.try_into().map_err(|_| Error::InvalidAddress)?));
            }
            Err(Error::InvalidAddress)
        })?
    }
}

#[derive(Debug, Clone)]
pub enum Amount {
    XRP(u64),
    IssuedCurrency {
        value: String,
        currency: String,
        issuer: String,
    },
}

impl Amount {
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        match self {
            Self::XRP(amount) => Ok((amount | 0x4000000000000000u64).to_be_bytes().to_vec()),
            Self::IssuedCurrency {
                value,
                currency,
                issuer,
            } => encode_issued_currency_amount(&value, &currency, &issuer),
        }
    }
}

#[derive(Debug, Clone)]
pub struct STObject(HashMap<String, Value>);

#[derive(Debug, Clone)]
pub struct Blob(pub(crate) String);

#[derive(Debug, Clone)]
pub struct STArray(Vec<Value>);

#[derive(Debug, Clone)]
pub struct Vector256(pub(crate) Vec<Hash256>);
