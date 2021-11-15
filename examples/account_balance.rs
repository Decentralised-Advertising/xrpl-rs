use xrpl_rs::{transports::HTTP, types::account::AccountInfoRequest, utils::testnet, XRPL};

#[tokio::main]
async fn main() {
    // Generate testnet credentials.
    let creds = testnet::get_testnet_credentials()
        .await
        .expect("error generating testnet credentials");
    // Create a new XRPL client with the HTTP transport pointed at ripple testnet.
    let xrpl = XRPL::new(
        HTTP::builder()
            .with_endpoint("https://s.altnet.rippletest.net:51234/")
            .unwrap()
            .build()
            .unwrap(),
    );
    // Create an account info request.
    let mut req = AccountInfoRequest::default();
    // Set the account to the testnet credentials.
    req.account = creds.account.address.to_owned();
    // Fetch the account info for an address.
    let account_info = xrpl.account_info(req).await.unwrap();
    // Print the account and balance
    println!(
        "Address {} has balance of {:?}",
        account_info.account_data.account, account_info.account_data.balance
    );
}