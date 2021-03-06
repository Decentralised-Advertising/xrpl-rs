use xrpl_rs::{transports::WebSocket, types::account::AccountInfoRequest, XRPL};

#[tokio::main]
async fn main() {
    // // Generate testnet credentials.
    // let creds = testnet::get_testnet_credentials()
    //     .await
    //     .expect("error generating testnet credentials");
    // // Print the account and balance
    // println!("Credentials: {:?}", creds,);
    // Create a new XRPL client with the HTTP transport pointed at ripple testnet.
    let xrpl = XRPL::new(
        WebSocket::builder()
            .with_endpoint("wss://xrplcluster.com/")
            .unwrap()
            .build()
            .await
            .unwrap(),
    );
    // // Create wallet from secret
    // let mut wallet = Wallet::from_secret(&creds.account.secret).unwrap();
    // println!("{}", wallet.address());

    // Create an account info request.
    let mut req = AccountInfoRequest::default();
    // Set the account to the testnet credentials.
    req.account = "rpD1ocF4rs3crXBjgdco84KhGQGep589YR".to_owned();
    // Fetch the account info for an address.
    let account_info = xrpl.account_info(req).await.unwrap();
    // Print the account and balance
    println!(
        "Address {}, Info: {:?}",
        account_info.account_data.account, account_info
    );
}
