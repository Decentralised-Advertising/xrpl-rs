use serde::{Serialize, Deserialize};
use serde_json::Value;

#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct SubmitRequest {
    /// Hex representation of the signed transaction to submit. This can be a multi-signed transaction.
    pub tx_blob: String,
    /// (Optional, defaults to false) If true, and the transaction fails locally, do not retry or relay the transaction to other servers.
    pub fail_hard: Option<bool>,
}

#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct SignAndSubmitRequest<T: Serialize> {
    /// Transaction definition in JSON format, optionally omitting any auto-fillable fields.
    pub tx_json: T,
    /// (Optional) Secret key of the account supplying the transaction, used to sign it. Do not send your secret to untrusted servers or through unsecured network connections. Cannot be used with key_type, seed, seed_hex, or passphrase.
    pub secret: Option<String>,
    /// (Optional) Secret key of the account supplying the transaction, used to sign it. Must be in the XRP Ledger's base58 format. If provided, you must also specify the key_type. Cannot be used with secret, seed_hex, or passphrase.
    pub seed: Option<String>,
    /// (Optional) Secret key of the account supplying the transaction, used to sign it. Must be in hexadecimal format. If provided, you must also specify the key_type. Cannot be used with secret, seed, or passphrase.
    pub seed_hex: Option<String>,
    /// (Optional) Type of cryptographic key provided in this request. Valid types are secp256k1 or ed25519. Defaults to secp256k1. Cannot be used with secret. Caution: Ed25519 support is experimental.
    pub key_type: Option<KeyType>,
    /// (Optional, defaults to false) If true, and the transaction fails locally, do not retry or relay the transaction to other servers.
    pub fail_hard: Option<bool>,
    /// (Optional, defaults to false) If true, when constructing the transaction, do not try to automatically fill in or validate values.
    pub offline: Option<bool>,
    /// (Optional) If this field is provided, the server auto-fills the Paths field of a Payment transaction before signing. You must omit this field if the transaction is a direct XRP payment or if it is not a Payment-type transaction. Caution: The server looks for the presence or absence of this field, not its value. This behavior may change. (Issue #3272 )
    pub build_path: Option<bool>,
    /// (Optional) Sign-and-submit fails with the error rpcHIGH_FEE if the auto-filled Fee value would be greater than the reference transaction cost × fee_mult_max ÷ fee_div_max. This field has no effect if you explicitly specify the Fee field of the transaction. The default is 10.
    pub fee_mult_max: Option<i64>,
    /// (Optional) Sign-and-submit fails with the error rpcHIGH_FEE if the auto-filled Fee value would be greater than the reference transaction cost × fee_mult_max ÷ fee_div_max. This field has no effect if you explicitly specify the Fee field of the transaction. The default is 1. New in: rippled 0.30.1 
    pub fee_div_max: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum KeyType {
    #[serde(rename = "secp256k1")]
    SECP256K1,
    #[serde(rename = "ed25519")]
    ED25519,
}

#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct SubmitResponse {}

