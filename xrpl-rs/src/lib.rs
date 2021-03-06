//! A client that exposes methods for interacting with the XRP Ledger.
//!
//! # Example Usage
//! ```
//! use std::convert::TryInto;
//! use xrpl_rs::{XRPL, transports::HTTP, types::account::AccountInfoRequest, types::CurrencyAmount};
//! use tokio_test::block_on;
//!
//! // Create a new XRPL client with the HTTP transport.
//! let xrpl = XRPL::new(
//!     HTTP::builder()
//!         .with_endpoint("http://s1.ripple.com:51234/")
//!         .unwrap()
//!         .build()
//!         .unwrap());
//!
//! // Create a request
//! let mut req = AccountInfoRequest::default();
//! req.account = "rG1QQv2nh2gr7RCZ1P8YYcBUKCCN633jCn".to_owned();
//!
//! // Fetch the account info for an address.
//! let account_info = block_on(async {
//!     xrpl
//!         .account_info(req)
//!         .await
//!         .unwrap()
//! });
//!
//! assert_eq!(account_info.account_data.balance, CurrencyAmount::xrp(9977));
//! ```

use std::pin::Pin;

use futures::stream::Stream;
use serde::de::DeserializeOwned;
use transports::{DuplexTransport, Transport, TransportError};
use types::{
    account::{
        AccountChannelsRequest, AccountChannelsResponse, AccountCurrenciesRequest,
        AccountCurrenciesResponse, AccountInfoRequest, AccountInfoResponse, AccountLinesRequest,
        AccountLinesResponse, AccountOfferRequest, AccountOfferResponse,
    },
    channels::{ChannelVerifyRequest, ChannelVerifyResponse},
    fee::{FeeRequest, FeeResponse},
    ledger::{LedgerRequest, LedgerResponse},
    submit::{SignAndSubmitRequest, SubmitRequest, SubmitResponse},
    subscribe::{SubscribeRequest, SubscriptionEvent},
    tx::{TxRequest, TxResponse},
    TransactionEntryRequest, TransactionEntryResponse,
};

pub mod transaction;
pub mod transports;
pub mod types;
pub mod utils;
pub mod wallet;

/// An enum providing error types that can be returned when calling XRPL methods.
#[derive(Debug)]
pub enum Error {
    TransportError(TransportError),
}

impl From<TransportError> for Error {
    fn from(e: TransportError) -> Self {
        Self::TransportError(e)
    }
}

/// A client that exposes methods for interacting with the XRP Ledger.
///
/// # Examples
/// ```
/// use std::convert::TryInto;
/// use xrpl_rs::{XRPL, transports::HTTP, types::account::AccountInfoRequest, types::CurrencyAmount};
/// use tokio_test::block_on;
///
/// // Create a new XRPL client with the HTTP transport.
/// let xrpl = XRPL::new(
///     HTTP::builder()
///         .with_endpoint("http://s1.ripple.com:51234/")
///         .unwrap()
///         .build()
///         .unwrap());
///
/// // Create a request
/// let mut req = AccountInfoRequest::default();
/// req.account = "rG1QQv2nh2gr7RCZ1P8YYcBUKCCN633jCn".to_owned();
///
/// // Fetch the account info for an address.
/// let account_info = block_on(async {
///     xrpl
///         .account_info(req)
///         .await
///         .unwrap()
/// });
///
/// assert_eq!(account_info.account_data.balance, CurrencyAmount::xrp(9977));
/// ```
pub struct XRPL<T: Transport> {
    transport: T,
}

macro_rules! impl_rpc_method {
    ($(#[$attr:meta])* $name: ident, $method: expr, $request: ident, $response: ident) => {
        $(#[$attr])*
        pub async fn $name(&self, params: $request) -> Result<$response, Error> {
            Ok(self
                .transport
                .send_request::<$request, $response>($method, params)
                .await?)
        }
    };
}

impl<T: Transport> XRPL<T> {
    pub fn new(transport: T) -> Self {
        Self { transport }
    }
    impl_rpc_method!(
        /// The account_channels method returns information about an account's Payment Channels. This includes only channels where the specified account is the channel's source, not the destination. (A channel's "source" and "owner" are the same.) All information retrieved is relative to a particular version of the ledger.
        account_channels,
        "account_channels",
        AccountChannelsRequest,
        AccountChannelsResponse
    );
    impl_rpc_method!(
        /// The account_currencies command retrieves a list of currencies that an account can send or receive, based on its trust lines. (This is not a thoroughly confirmed list, but it can be used to populate user interfaces.)
        account_currencies,
        "account_currencies",
        AccountCurrenciesRequest,
        AccountCurrenciesResponse
    );
    impl_rpc_method!(
        /// The account_info command retrieves information about an account, its activity, and its XRP balance. All information retrieved is relative to a particular version of the ledger.
        account_info,
        "account_info",
        AccountInfoRequest,
        AccountInfoResponse
    );
    impl_rpc_method!(
        /// The account_lines method returns information about an account's trust lines, including balances in all non-XRP currencies and assets. All information retrieved is relative to a particular version of the ledger.
        account_lines,
        "account_lines",
        AccountLinesRequest,
        AccountLinesResponse
    );
    impl_rpc_method!(
        /// The account_offers method retrieves a list of offers made by a given account that are outstanding as of a particular ledger version.
        account_offers,
        "account_offers",
        AccountOfferRequest,
        AccountOfferResponse
    );
    impl_rpc_method!(
        /// The transaction_entry method retrieves information on a single transaction from a specific ledger version. (The tx method, by contrast, searches all ledgers for the specified transaction. We recommend using that method instead.)
        transaction_entry,
        "transaction_entry",
        TransactionEntryRequest,
        TransactionEntryResponse
    );
    impl_rpc_method!(
        /// The submit method applies a transaction and sends it to the network to be confirmed and included in future ledgers.
        submit,
        "submit",
        SubmitRequest,
        SubmitResponse
    );
    impl_rpc_method!(
        /// The sign_and_submit method applies a transaction and sends it to the network to be confirmed and included in future ledgers.
        sign_and_submit,
        "submit",
        SignAndSubmitRequest,
        SubmitResponse
    );
    impl_rpc_method!(
        /// The fee command reports the current state of the open-ledger requirements for the transaction cost. This requires the FeeEscalation amendment to be enabled. New in: rippled 0.31.0.
        fee,
        "fee",
        FeeRequest,
        FeeResponse
    );
    impl_rpc_method!(
        /// Retrieve information about the public ledger.
        ledger,
        "ledger",
        LedgerRequest,
        LedgerResponse
    );
    impl_rpc_method!(
        /// The channel_verify method checks the validity of a signature that can be used to redeem a specific amount of XRP from a payment channel.
        channel_verify,
        "channel_verify",
        ChannelVerifyRequest,
        ChannelVerifyResponse
    );
    impl_rpc_method!(
        /// The tx method retrieves information on a single transaction, by its identifying hash.
        tx,
        "tx",
        TxRequest,
        TxResponse
    );
}

impl<T: DuplexTransport> XRPL<T> {
    pub async fn subscribe(
        &self,
        request: SubscribeRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<SubscriptionEvent, TransportError>>>>, TransportError> {
        self.transport.subscribe(request).await
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{BigInt, CurrencyAmount};

    use super::{transports::HTTPBuilder, types, XRPL};
    #[test]
    fn create_client() {
        let _ = XRPL::new(
            HTTPBuilder::default()
                .with_endpoint("http://s1.ripple.com:51234/")
                .unwrap()
                .build()
                .unwrap(),
        );
    }
    #[tokio::test]
    async fn account_info() {
        let c = XRPL::new(
            HTTPBuilder::default()
                .with_endpoint("http://s1.ripple.com:51234/")
                .unwrap()
                .build()
                .unwrap(),
        );
        let res = c
            .account_info(types::account::AccountInfoRequest {
                account: "rG1QQv2nh2gr7RCZ1P8YYcBUKCCN633jCn".to_owned(),
                strict: None,
                queue: None,
                ledger_info: types::LedgerInfo::default(),
                signer_lists: None,
            })
            .await;
        match res {
            Err(e) => {
                eprintln!("test failed: {:?}", e);
            }
            Ok(res) => {
                assert_eq!(res.account_data.balance, CurrencyAmount::XRP(BigInt(9977)),);
            }
        }
    }
}
