use crate::types::{Address, BigInt, CurrencyAmount, H256};
use rust_decimal::Decimal;
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
    pub fee: BigInt,
    pub sequence: u32,
    pub last_ledger_sequence: u32,
    pub signing_pub_key: String,
    pub txn_signature: Option<String>,
    pub flags: Option<TFFlag>,
    #[serde(flatten)]
    pub tx: Option<TransactionType>,
    pub hash: Option<String>,
}

type TFFlag = u32;

pub const TF_SETF_AUTH: TFFlag = 65536;
pub const TF_SET_NO_RIPPLE: TFFlag = 131072;
pub const TF_CLEAR_NO_RIPPLE: TFFlag = 262144;
pub const TF_SET_FREEZE: TFFlag = 1048576;
pub const TF_CLEAR_FREEZE: TFFlag = 2097152;
pub const TF_RENEW: TFFlag = 65536;
pub const TF_CLOSE: TFFlag = 131072;

pub const TF_BURNABLE: TFFlag = 0x00000001;
pub const TF_ONLY_XRP: TFFlag = 0x00000002;
pub const TF_TRUSTLINE: TFFlag = 0x00000004;
pub const TF_TRANSFERABLE: TFFlag = 0x00000008;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(tag = "TransactionType", rename_all = "PascalCase")]
pub enum TransactionType {
    Payment(Payment),
    AccountSet(AccountSet),
    TrustSet(TrustSet),
    PaymentChannelClaim(PaymentChannelClaim),
    PaymentChannelCreate(PaymentChannelCreate),
    PaymentChannelFund(PaymentChannelFund),
    NFTokenMint(NFTokenMint),
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

#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct AccountSet {
    /// (Optional) Unique identifier of a flag to disable for this account.
    pub clear_flag: Option<u32>,
    /// (Optional) The domain that owns this account, as a string of hex representing the ASCII for the domain in lowercase. Cannot be more than 256 bytes in length.
    pub domain: Option<String>,
    /// (Optional) Hash of an email address to be used for generating an avatar image. Conventionally, clients use Gravatar  to display this image.
    pub email_hash: Option<String>,
    /// (Optional) Public key for sending encrypted messages to this account. To set the key, it must be exactly 33 bytes, with the first byte indicating the key type: 0x02 or 0x03 for secp256k1 keys, 0xED for Ed25519 keys. To remove the key, use an empty value.
    pub message_key: Option<String>,
    /// (Optional) Integer flag to enable for this account.
    pub set_flag: Option<AccountSetFlag>,
    /// (Optional) The fee to charge when users transfer this account's issued currencies, represented as billionths of a unit. Cannot be more than 2000000000 or less than 1000000000, except for the special case 0 meaning no fee.
    pub transfer_rate: Option<u32>,
    /// (Optional) Tick size to use for offers involving a currency issued by this address. The exchange rates of those offers is rounded to this many significant digits. Valid values are 3 to 15 inclusive, or 0 to disable. (Added by the TickSize amendment.)
    pub tick_size: Option<u8>,
}

type AccountSetFlag = u32;

pub const ASF_ACCOUNT_TXN_ID: AccountSetFlag = 5;
pub const ASF_DEFAULT_RIPPLE: AccountSetFlag = 8;
pub const ASF_DEPOSIT_AUTH: AccountSetFlag = 9;
pub const ASF_DISABLE_MASTER: AccountSetFlag = 4;
pub const ASF_DISALLOW_XRP: AccountSetFlag = 3;
pub const ASF_GLOBAL_FREEZE: AccountSetFlag = 7;
pub const ASF_NO_FREEZE: AccountSetFlag = 6;
pub const ASF_REQUIRE_AUTH: AccountSetFlag = 2;
pub const ASF_REQUIRE_DEST: AccountSetFlag = 1;

into_transaction!(AccountSet);

#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct TrustSet {
    /// Object defining the trust line to create or modify, in the format of a Currency Amount.
    pub limit_amount: TrustSetLimitAmount,
    /// (Optional) Value incoming balances on this trust line at the ratio of this number per 1,000,000,000 units. A value of 0 is shorthand for treating balances at face value.
    pub quality_in: Option<u32>,
    /// (Optional) Value outgoing balances on this trust line at the ratio of this number per 1,000,000,000 units. A value of 0 is shorthand for treating balances at face value.
    pub quality_out: Option<u32>,
}

#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct TrustSetLimitAmount {
    /// The currency to this trust line applies to, as a three-letter ISO 4217 Currency Code  or a 160-bit hex value according to currency format. "XRP" is invalid.
    pub currency: String,
    /// Quoted decimal representation of the limit to set on this trust line.
    pub issuer: String,
    /// The address of the account to extend trust to.
    pub value: Decimal,
}

into_transaction!(TrustSet);

#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct PaymentChannelClaim {
    /// The unique ID of the channel, as a 64-character hexadecimal string.
    pub channel: H256,
    /// (Optional) Total amount of XRP, in drops, delivered by this channel after processing this claim. Required to deliver XRP. Must be more than the total amount delivered by the channel so far, but not greater than the Amount of the signed claim. Must be provided except when closing the channel.
    pub balance: Option<BigInt>,
    /// (Optional) The amount of XRP, in drops, authorized by the Signature. This must match the amount in the signed message. This is the cumulative amount of XRP that can be dispensed by the channel, including XRP previously redeemed.
    pub amount: Option<BigInt>,
    /// (Optional) The signature of this claim, as hexadecimal. The signed message contains the channel ID and the amount of the claim. Required unless the sender of the transaction is the source address of the channel.
    pub signature: Option<String>,
    /// (Optional) The public key used for the signature, as hexadecimal. This must match the PublicKey stored in the ledger for the channel. Required unless the sender of the transaction is the source address of the channel and the Signature field is omitted. (The transaction includes the public key so that rippled can check the validity of the signature before trying to apply the transaction to the ledger.)
    pub public_key: Option<String>,
}

into_transaction!(PaymentChannelClaim);

#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct PaymentChannelCreate {
    /// Amount of XRP, in drops, to deduct from the sender's balance and set aside in this channel. While the channel is open, the XRP can only go to the Destination address. When the channel closes, any unclaimed XRP is returned to the source address's balance.
    pub amount: BigInt,
    /// Address to receive XRP claims against this channel. This is also known as the "destination address" for the channel. Cannot be the same as the sender (Account).
    pub destination: Address,
    /// Amount of time the source address must wait before closing the channel if it has unclaimed XRP.
    pub settle_delay: u32,
    /// The public key of the key pair the source will use to sign claims against this channel, in hexadecimal. This can be any secp256k1 or Ed25519 public key.
    pub public_key: String,
    /// (Optional) The time, in seconds since the Ripple Epoch, when this channel expires. Any transaction that would modify the channel after this time closes the channel without otherwise affecting it. This value is immutable; the channel can be closed earlier than this time but cannot remain open after this time.
    pub cancel_after: Option<u32>,
    /// (Optional) Arbitrary tag to further specify the destination for this payment channel, such as a hosted recipient at the destination address.
    pub destination_tag: Option<u32>,
}

into_transaction!(PaymentChannelCreate);

#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct PaymentChannelFund {}

into_transaction!(PaymentChannelFund);

#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct NFTokenMint {
    /// Indicates the account that issues the token. This value is optional and should only be specified if the account executing the transaction is not the Issuer of the NFToken object. If it is present, the MintAccount field in the AccountRoot of the Issuer field must match the Account. Otherwise, the transaction fails.
    pub issuer: Option<Address>,
    /// The taxon associated with the token. The taxon is generally a value chosen by the minter of the token. A given taxon can be used for multiple tokens. This implementation reserves all taxon identifiers greater than or equal to 0x80000000; attempts to use mint tokens with such taxons should fail and a fee should be claimed.
    pub token_taxon: u32,
    /// The value specifies the fee charged by the issuer for secondary sales of the Token, if such sales are allowed. Valid values for this field are between 0 and 9999 inclusive, allowing transfer rates of between 0.00% and 99.99% in increments of 0.01. The field MUST NOT be present if tfTransferable is not set. If it is, the transaction should fail and the server should claim a fee.
    pub transfer_fee: u16,
    /// A URI that points to the data or metadata associated with the NFT. This field need not be an HTTP or HTTPS URL; it could be an IPFS URI, a magnet link, immediate data encoded as an RFC2379 "data" URL , or even an opaque issuer-specific encoding. The URI is NOT checked for validity, but the field is limited to a maximum length of 256 bytes.
    pub uri: String,
}

into_transaction!(NFTokenMint);

// #[test]
// pub fn test_serialize() {
//     let j = serde_json::json!(Payment{
//         amount: CurrencyAmount::XRP("14000".to_owned()),
//         destination: "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B".to_owned(),
//     });
//     assert_eq!(hex::encode(serde_xrpl::ser::to_bytes(&j).unwrap()), "");
// }
