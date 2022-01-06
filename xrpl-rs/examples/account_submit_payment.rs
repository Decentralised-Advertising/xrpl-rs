use std::convert::TryInto;

use secp256k1::{Message, PublicKey, Secp256k1, SecretKey};
use serde::Serialize;
use sha2::{Digest, Sha512};
use xrpl_rs::{
    signing::{Signer, Wallet},
    transaction::types::{Payment, Transaction, TransactionType},
    transports::HTTP,
    types::{account::AccountInfoRequest, submit::SubmitRequest, CurrencyAmount},
    utils::testnet,
    XRPL,
};

#[tokio::main]
async fn main() {
    // Generate testnet credentials.
    let creds_one = testnet::get_testnet_credentials()
        .await
        .expect("error generating testnet credentials");
    println!("Created account: {:?}", creds_one);

    // Create a new XRPL client with the HTTP transport pointed at ripple testnet.
    let xrpl = XRPL::new(
        HTTP::builder()
            .with_endpoint("https://s.altnet.rippletest.net:51234/")
            .unwrap()
            .build()
            .unwrap(),
    );

    // Create wallet from secret
    let mut wallet =
        Wallet::from_secret(&creds_one.account.secret, &creds_one.account.address).unwrap();

    // Create a payment transaction.
    let mut payment = Payment::default();
    payment.amount = CurrencyAmount::XRP("100000000".try_into().unwrap()); // 100 XRP
    payment.destination = "rp7pmm4rzTGmtZDuvrG1z9Xrm3KwHRipDw".to_owned(); // Set the destination to the second account.

    // Convert the payment into a transaction.
    let mut tx = payment.into_transaction();

    let tx_blob = wallet.sign(&mut tx, &xrpl).await.unwrap();

    // Create a sign_and_submit request.
    let mut submit_req = SubmitRequest::default();
    submit_req.tx_blob = hex::encode(&tx_blob).to_uppercase();

    // Fetch the account info for an address.
    let submit_res = xrpl
        .submit(submit_req)
        .await
        .expect("failed to make submit request");
    println!("Got response to submit request: {:?}", submit_res);

    // Create an account info request to see the balance of account two.
    let mut req = AccountInfoRequest::default();
    // Set the account to the second set of testnet credentials.
    req.account = "rp7pmm4rzTGmtZDuvrG1z9Xrm3KwHRipDw".to_owned();
    // Fetch the account info for an address.
    let account_info = xrpl
        .account_info(req)
        .await
        .expect("failed to make account_info request");
    // Print the account and balance
    println!(
        "Address {} has balance of {:?}",
        account_info.account_data.account, account_info.account_data.balance
    );
}
