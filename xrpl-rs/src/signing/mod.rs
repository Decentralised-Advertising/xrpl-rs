use std::convert::TryInto;

use secp256k1::{
    All, Error as Secp256k1Error, KeyPair as Secp256k1KeyPair, Message,
    PublicKey as Secp256k1PublicKey, Secp256k1, SecretKey as Secp256k1SecretKey,
};

use crate::transaction::types::Transaction;
use crate::types::account::AccountInfoRequest;
use crate::types::fee::FeeRequest;
use crate::types::{CurrencyAmount, Drops};
use crate::{Error as XRPLError, Transport, XRPL};
use lazy_static::lazy_static;
use sha2::{Digest, Sha512};

lazy_static! {
    static ref DEFAULT_MAX_FEE: Drops = "100"
        .to_owned()
        .try_into()
        .expect("invalid number of drops");
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
    address: String,
    sequence: Option<u32>,
    fee: Option<Drops>,
    max_fee: Drops,
}

impl Wallet {
    pub fn from_secret(secret: &str, address: &str) -> Result<Self, Error> {
        let keypair = keypair_from_secret(secret)?;
        Ok(Self {
            keypair,
            address: address.to_owned(),
            sequence: None,
            fee: None,
            max_fee: DEFAULT_MAX_FEE.clone(),
        })
    }
    pub fn set_sequence(&mut self, sequence: u32) {
        self.sequence = Some(sequence);
    }
    pub fn set_fee<T: TryInto<Drops>>(&mut self, drops: T) -> Result<(), Error> {
        self.fee = Some(drops.try_into().map_err(|e| Error::InvalidDrops)?);
        Ok(())
    }
    pub fn set_max_fee<T: TryInto<Drops>>(&mut self, drops: T) -> Result<(), Error> {
        self.max_fee = drops.try_into().map_err(|e| Error::InvalidDrops)?;
        Ok(())
    }
    pub async fn sign<T: Transport>(
        &mut self,
        tx: &mut Transaction,
        xrpl: &XRPL<T>,
    ) -> Result<Vec<u8>, Error> {
        self.auto_fill_fields(tx, xrpl).await?;
        self.sign_and_serialize(tx)
    }
    pub async fn auto_fill_fields<T: Transport>(
        &mut self,
        tx: &mut Transaction,
        xrpl: &XRPL<T>,
    ) -> Result<(), Error> {
        // Set the address of sender.
        tx.account = self.address.clone();
        // If there is no sequence specified, then fetch from the ledger.
        if self.sequence.is_none() {
            let mut req = AccountInfoRequest::default();
            req.account = self.address.to_owned();
            let account_info = xrpl.account_info(req).await?;
            self.sequence = Some(account_info.account_data.sequence);
        }
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
        tx.fee = self.fee.as_ref().ok_or(Error::FeeRequired)?.clone();
        // Check that the fee does not exceed the max fee.
        if tx.fee > self.max_fee {
            return Err(Error::FeeAboveMax);
        }
        Ok(())
    }
    pub fn sign_and_serialize(&self, tx: &mut Transaction) -> Result<Vec<u8>, Error> {
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
        Ok(tx_blob)
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
