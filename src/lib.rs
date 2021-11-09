use transports::{Transport, TransportError};
use types::{
    account::{
        AccountChannelsRequest, AccountChannelsResponse, AccountCurrenciesRequest,
        AccountCurrenciesResponse, AccountInfoRequest, AccountInfoResponse, AccountLinesRequest,
        AccountLinesResponse,
    },
    submit::{SubmitRequest, SignAndSubmitRequest, SubmitResponse},
    AccountOfferRequest, AccountOfferResponse,
    TransactionEntryRequest, TransactionEntryResponse,
};
use serde::Serialize;

pub mod transaction;
pub mod transports;
pub mod types;
pub mod utils;
pub mod codec;

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
/// assert_eq!(account_info.account_data.balance, CurrencyAmount::XRP("9977".to_owned()));
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
}

#[cfg(test)]
mod tests {
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
            Ok(mut res) => {
                res.ledger_info = types::LedgerInfo::default();
                let mut account_root = types::account::AccountRoot::default();
                account_root.account = "rG1QQv2nh2gr7RCZ1P8YYcBUKCCN633jCn".to_owned();
                account_root.balance = types::CurrencyAmount::XRP("9977".to_owned());
                assert_eq!(
                    res,
                    types::account::AccountInfoResponse {
                        account_data: account_root,
                        queue_data: None,
                        signer_lists: None,
                        ledger_info: types::LedgerInfo::default(),
                    }
                );
            }
        }
    }
}
