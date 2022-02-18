use std::convert::TryInto;

use hex_literal::hex;
use rust_decimal::Decimal;
use secp256k1::{
    rand::rngs::OsRng, All, Error as Secp256k1Error, KeyPair as Secp256k1KeyPair, Message,
    PublicKey as Secp256k1PublicKey, Secp256k1, SecretKey as Secp256k1SecretKey,
};
use serde::Serialize;
use serde_json::json;
use serde_xrpl::types::Hash256;

use crate::transaction::types::{PaymentChannelClaim, Transaction};
use crate::types::account::AccountInfoRequest;
use crate::types::fee::FeeRequest;
use crate::types::ledger::LedgerRequest;
use crate::types::{BigInt, CurrencyAmount};
use crate::{Error as XRPLError, Transport, XRPL};
use lazy_static::lazy_static;
use ripemd::{Digest, Ripemd160};
use sha2::{Sha256, Sha512};

lazy_static! {
    static ref DEFAULT_MAX_FEE: BigInt = BigInt(100);
    static ref DEFAULT_LEDGER_OFFSET: u32 = 20; // Approx 1 minute.
}

#[derive(Debug)]
pub enum Error {
    InvalidSecret(bs58::decode::Error),
    XRPLError(XRPLError),
    SequenceRequired,
    FeeRequired,
    FeeAboveMax,
    InvalidDrops,
    Secp256k1Error(Secp256k1Error),
    LastLedgerSequenceRequired,
}

impl From<XRPLError> for Error {
    fn from(e: XRPLError) -> Self {
        Self::XRPLError(e)
    }
}

pub enum Signer {
    Secp256k1(Secp256k1<All>),
}

pub enum KeyPair {
    Secp256k1(Secp256k1KeyPair),
}

pub struct Wallet {
    keypair: KeyPair,
    sequence: Option<u32>,
    fee: Option<BigInt>,
    max_fee: BigInt,
    ledger_offset: u32,
}

impl Wallet {
    pub fn new_random() -> Self {
        let secp = Secp256k1::new();
        let mut rng = OsRng::new().expect("OsRng");
        let keypair = Secp256k1KeyPair::new(&secp, &mut rng);
        Self {
            keypair: KeyPair::Secp256k1(keypair),
            sequence: Some(0),
            fee: None,
            max_fee: DEFAULT_MAX_FEE.to_owned(),
            ledger_offset: DEFAULT_LEDGER_OFFSET.to_owned(),
        }
    }
    pub fn address(&self) -> String {
        let sha = sha256(match &self.keypair {
            KeyPair::Secp256k1(keypair) => {
                hex::decode(&Secp256k1PublicKey::from_keypair(&keypair).to_string()).unwrap()
            }
        });
        let rip = ripemd160(&sha);
        let prefixed = [vec![0x00], rip].concat();
        let chk = double_sha256(&prefixed)[0..4].to_vec();
        bs58::encode([prefixed, chk].concat())
            .with_alphabet(bs58::Alphabet::RIPPLE)
            .into_string()
    }
    pub fn from_secret(secret: &str) -> Result<Self, Error> {
        let keypair = keypair_from_secret(secret)?;
        Ok(Self {
            keypair,
            sequence: None,
            fee: None,
            max_fee: DEFAULT_MAX_FEE.to_owned(),
            ledger_offset: DEFAULT_LEDGER_OFFSET.to_owned(),
        })
    }
    pub fn set_sequence(&mut self, sequence: u32) {
        self.sequence = Some(sequence);
    }
    pub fn set_fee(&mut self, drops: BigInt) {
        self.fee = Some(drops);
    }
    pub fn set_max_fee(&mut self, drops: BigInt) {
        self.max_fee = drops;
    }
    pub fn set_ledger_offset<T: TryInto<BigInt>>(
        &mut self,
        ledger_offset: u32,
    ) -> Result<(), Error> {
        self.ledger_offset = ledger_offset;
        Ok(())
    }
    pub async fn fill_and_sign<T: Transport>(
        &mut self,
        tx: &mut Transaction,
        xrpl: &XRPL<T>,
    ) -> Result<String, Error> {
        self.auto_fill_fields(tx, xrpl).await?;
        self.sign(tx)
    }
    pub async fn auto_fill_fields<T: Transport>(
        &mut self,
        tx: &mut Transaction,
        xrpl: &XRPL<T>,
    ) -> Result<(), Error> {
        if tx.flags.is_none() {
            // tfFullyCanonicalSig is flags is not otherwise specified.
            tx.flags = Some(2147483648u32);
        }
        // Set the address of sender.
        tx.account = self.address();
        // If there is no sequence specified, then fetch from the ledger.
        if self.sequence.is_none() {
            let mut req = AccountInfoRequest::default();
            req.account = self.address();
            let account_info = xrpl.account_info(req).await?;
            self.sequence = Some(account_info.account_data.sequence);
        }
        // Set the sequence and increment.
        if let Some(sequence) = &mut self.sequence {
            tx.sequence = *sequence;
            *sequence += 1;
        } else {
            return Err(Error::SequenceRequired);
        }
        // If there is no fee available then fetch from the ledger.
        if self.fee.is_none() {
            let req = FeeRequest::default();
            let fee = xrpl.fee(req).await?;
            if let CurrencyAmount::XRP(drops) = fee.drops.open_ledger_fee {
                self.fee = Some(drops);
            }
        }
        // TODO calculate appropriate fee, see: https://github.com/XRPLF/xrpl.js/blob/07f36e127f76b72df57e8101979f014d9d221353/packages/xrpl/src/sugar/autofill.ts#L154
        tx.fee = self.fee.as_ref().ok_or(Error::FeeRequired)?.clone();
        // Check that the fee does not exceed the max fee.
        if tx.fee > self.max_fee {
            return Err(Error::FeeAboveMax);
        }
        // Assign the last ledger sequence to prevent the transaction from becoming stuck.
        let ledger_req = LedgerRequest::default();
        let ledger = xrpl.ledger(ledger_req).await?;
        tx.last_ledger_sequence = ledger
            .ledger
            .ledger_info
            .ledger_index
            .ok_or(Error::LastLedgerSequenceRequired)?
            + self.ledger_offset;
        Ok(())
    }
    // Signs the provided transaction updating the corresponding transaction fields and returns
    // the hex encoded serialized transaction.
    pub fn sign(&self, tx: &mut Transaction) -> Result<String, Error> {
        match &self.keypair {
            KeyPair::Secp256k1(keypair) => {
                let secp = Secp256k1::new();
                tx.signing_pub_key = Secp256k1PublicKey::from_keypair(keypair).to_string();
                let tx_blob_for_signing =
                    serde_xrpl::ser::to_bytes_for_signing(&serde_json::to_value(&tx).unwrap())
                        .unwrap();
                let mut mh = Sha512::new();
                mh.update(&tx_blob_for_signing);
                let mhh = mh.finalize()[..32].to_vec();
                let message = Message::from_slice(&mhh).unwrap();
                let sig = secp.sign_ecdsa(&message, &Secp256k1SecretKey::from_keypair(keypair));
                tx.txn_signature = Some(sig.to_string().to_uppercase());
            }
        }
        let tx_blob = serde_xrpl::ser::to_bytes(&serde_json::to_value(&tx).unwrap()).unwrap();
        let mut th = Sha512::new();
        th.update(&[hex!("54584e00").to_vec(), tx_blob.to_vec()].concat());
        let transaction_hash = th.finalize()[..32].to_vec();
        tx.hash = Some(hex::encode(transaction_hash).to_uppercase());
        Ok(hex::encode(tx_blob).to_uppercase())
    }
    pub fn public_key(&self) -> String {
        match &self.keypair {
            KeyPair::Secp256k1(keypair) => {
                return Secp256k1PublicKey::from_keypair(keypair).to_string();
            }
        }
    }
    pub fn private_key(&self) -> String {
        match &self.keypair {
            KeyPair::Secp256k1(keypair) => return keypair.display_secret().to_string(),
        }
    }
    pub fn sign_message<T: Serialize>(&self, message: T) -> Result<String, Error> {
        match &self.keypair {
            KeyPair::Secp256k1(keypair) => {
                let secp = Secp256k1::new();
                let message_blob_for_signing =
                    serde_xrpl::ser::to_bytes_for_claim(&serde_json::to_value(&message).unwrap())
                        .unwrap();
                let mut mh = Sha512::new();
                mh.update(&message_blob_for_signing);
                let mhh = mh.finalize()[..32].to_vec();
                let message = Message::from_slice(&mhh).unwrap();
                let sig = secp.sign_ecdsa(&message, &Secp256k1SecretKey::from_keypair(keypair));
                Ok(sig.to_string().to_uppercase())
            }
        }
    }
    pub fn sign_payment_channel_claim(
        &self,
        channel: String,
        amount: BigInt,
    ) -> Result<String, Error> {
        match &self.keypair {
            KeyPair::Secp256k1(keypair) => {
                let secp = Secp256k1::new();
                let mut mh = Sha512::new();
                let prefix = hex!("434c4d00").to_vec();
                let channel_bytes = Hash256(channel).to_bytes();
                let amount_bytes = amount.0.to_be_bytes().to_vec();
                mh.update([prefix, channel_bytes, amount_bytes].concat());
                let mhh = mh.finalize()[..32].to_vec();
                let message = Message::from_slice(&mhh).unwrap();
                let sig = secp.sign_ecdsa(&message, &Secp256k1SecretKey::from_keypair(keypair));
                Ok(sig.to_string().to_uppercase())
            }
        }
    }
}

fn decode_secret(secret: &str) -> Result<Vec<u8>, Error> {
    Ok(bs58::decode(secret.as_bytes())
        .with_alphabet(bs58::alphabet::Alphabet::RIPPLE)
        .with_check(None)
        .into_vec()
        .map_err(|e| Error::InvalidSecret(e))?[1..]
        .to_vec())
}

fn keypair_from_secret(secret: &str) -> Result<KeyPair, Error> {
    let decoded_secret = bs58::decode(secret.as_bytes())
        .with_alphabet(bs58::alphabet::Alphabet::RIPPLE)
        .with_check(None)
        .into_vec()
        .unwrap()[1..]
        .to_vec();
    let secp = Secp256k1::new();
    let mut sh = Sha512::new();
    sh.update([decoded_secret.to_vec(), 0u32.to_be_bytes().to_vec()].concat());
    let secret = sh.finalize();
    let root_secret_key =
        Secp256k1SecretKey::from_slice(&secret[..32]).map_err(|e| Error::Secp256k1Error(e))?;
    let mut intermediate_hash = Sha512::new();
    intermediate_hash.update(
        [
            Secp256k1PublicKey::from_secret_key(&secp, &root_secret_key)
                .serialize()
                .to_vec(),
            0u32.to_be_bytes().to_vec(),
            0u32.to_be_bytes().to_vec(),
        ]
        .concat(),
    );
    let mut account_secret_key =
        Secp256k1SecretKey::from_slice(&intermediate_hash.finalize()[..32])
            .map_err(|e| Error::Secp256k1Error(e))?;
    account_secret_key
        .add_assign(&root_secret_key.serialize_secret())
        .map_err(|e| Error::Secp256k1Error(e))?;
    let account_keypair = Secp256k1KeyPair::from_secret_key(&secp, account_secret_key);
    Ok(KeyPair::Secp256k1(account_keypair))
}

fn sha256(i: impl AsRef<[u8]>) -> Vec<u8> {
    let mut h = Sha256::new();
    h.update(i);
    h.finalize().to_vec()
}

fn double_sha256(i: impl AsRef<[u8]>) -> Vec<u8> {
    sha256(&sha256(i))
}

fn ripemd160(i: impl AsRef<[u8]>) -> Vec<u8> {
    let mut r = Ripemd160::new();
    r.update(&i);
    r.finalize().to_vec()
}
