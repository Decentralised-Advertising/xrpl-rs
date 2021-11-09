use serde::{Deserialize, Serialize};
use serde_json;
use lazy_static::lazy_static;

const DEFINITIONS_JSON: &str = include_str!("definitions.json");

lazy_static! {
    pub static ref DEFINITIONS: Definitions = serde_json::from_str(&DEFINITIONS_JSON).unwrap();
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub struct Definitions {
    pub types: Types,
    pub fields: Vec<Field>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Types {
    pub validation: i16,
    pub done: i16,
    pub hash_128: i16,
    pub blob: i16,
    #[serde(rename = "AccountID")]
    pub account_id: i16,
    pub amount: i16,
    pub hash_256: i16,
    pub u_int_8: i16,
    pub vector_256: i16,
    #[serde(rename = "STObject")]
    pub st_object: i16,
    pub unknown: i16,
    pub transaction: i16,
    pub hash_160: i16,
    pub path_set: i16,
    pub ledger_entry: i16,
    pub u_int_16: i16,
    pub not_present: i16,
    pub u_int_64: i16,
    pub u_int_32: i16,
    #[serde(rename = "STArray")]
    pub st_array: i16,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Field(pub String, pub FieldInfo);

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FieldInfo {
    /// The field code -- sort order position for
    /// fields of the same type.
    pub nth: i16,
    /// Whether the serialized length of this
    /// field varies.
    #[serde(rename = "isVLEncoded")]
    pub is_vl_encoded: bool,
    /// If the field is presented in binary
    /// serialized representation.
    pub is_serialized: bool,
    /// If the field should be included in signed
    /// transactions.
    pub is_signing_field: bool,
    /// The name of this field's serialization type,
    /// e.g. UInt32, AccountID, etc.
    pub r#type: String,
}
