use reqwest::{Client, Error};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TestNetCredentials {
    pub account: TestNetAccountDetails,
    pub amount: usize,
    pub balance: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestNetAccountDetails {
    pub address: String,
    #[serde(rename = "classicAddress")]
    pub classic_address: String,
    pub secret: String,
    #[serde(rename = "xAddress")]
    pub x_address: String,
}

/// Generates a set of testnet credentials using the ripple testnet faucet.
pub async fn get_testnet_credentials() -> Result<TestNetCredentials, Error> {
    let client = reqwest::Client::new();
    Ok(client
        .post("https://faucet.altnet.rippletest.net/accounts")
        .send()
        .await?
        .json()
        .await?)
}
