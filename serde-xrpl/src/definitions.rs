use super::error::{Error, Result};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::convert::TryInto;

const DEFINITIONS_JSON: &str = include_str!("definitions.json");

lazy_static! {
    pub static ref DEFINITIONS: Definitions = serde_json::from_str(&DEFINITIONS_JSON).unwrap();
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub struct Definitions {
    pub types: HashMap<String, i16>,
    pub fields: Vec<Field>,
    pub transaction_types: HashMap<String, i16>,
    pub ledger_entry_types: HashMap<String, i16>,
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

/// Returns a boolean indicating whether the field is a signing field, i.e. should be included
/// in the binary representation for when signing.
pub fn is_signing_field(field_name: &str) -> Option<bool> {
    for field in &DEFINITIONS.fields {
        if field.0 == field_name {
            return Some(field.1.is_signing_field);
        }
    }
    None
}

pub fn is_serialized_field(field_name: &str) -> Option<bool> {
    for field in &DEFINITIONS.fields {
        if field.0 == field_name {
            return Some(field.1.is_serialized);
        }
    }
    None
}

pub fn get_field_code_and_type_code(field_name: &str) -> Result<(u8, u8)> {
    for field in &DEFINITIONS.fields {
        if field.0 != field_name {
            continue;
        }
        let field_type = DEFINITIONS
            .types
            .get(&field.1.r#type)
            .ok_or(Error::UnknownFieldType(field.1.r#type.to_owned()))?;
        return Ok((
            field
                .1
                .nth
                .try_into()
                .map_err(|e| Error::Message(format!("{:?}", e)))?,
            field_type
                .clone()
                .try_into()
                .map_err(|e| Error::Message(format!("{:?}", e)))?,
        ));
    }
    Err(Error::UnknownFieldName(field_name.to_owned()))
}

pub fn get_transaction_type(transaction_type_name: &str) -> Result<i16> {
    if let Some(transaction_type) = DEFINITIONS.transaction_types.get(transaction_type_name) {
        return Ok(*transaction_type);
    }
    if let Some(transaction_type) = DEFINITIONS.ledger_entry_types.get(transaction_type_name) {
        return Ok(*transaction_type);
    }
    Err(Error::InvalidTransactionType(
        transaction_type_name.to_owned(),
    ))
}
