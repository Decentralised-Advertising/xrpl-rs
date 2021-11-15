use xrpl_rs::{
    transaction::types::{Payment, Transaction},
    transports::HTTP,
    types::{account::AccountInfoRequest, submit::SignAndSubmitRequest, CurrencyAmount},
    utils::testnet,
    XRPL,
};

#[tokio::main]
async fn main() {
    // Generate testnet credentials.
    let creds_one = testnet::get_testnet_credentials()
        .await
        .expect("error generating testnet credentials");
    println!(
        "Created account with address: {}",
        creds_one.account.address
    );
    let creds_two = testnet::get_testnet_credentials()
        .await
        .expect("error generating testnet credentials");
    println!(
        "Created account with address: {}",
        creds_one.account.address
    );
    // Create a new XRPL client with the HTTP transport pointed at ripple testnet.
    let xrpl = XRPL::new(
        HTTP::builder()
            .with_endpoint("http://127.0.0.1:5005/")
            .unwrap()
            .build()
            .unwrap(),
    );

    // Create a payment transaction.
    let mut payment = Payment::default();
    payment.amount = CurrencyAmount::xrp_from_str("10000000"); // 10 XRP
    payment.destination = creds_two.account.address.to_owned(); // Set the destination to the second account.

    // Create a sign_and_submit request.
    let mut sign_and_submit_req: SignAndSubmitRequest = SignAndSubmitRequest::default();
    sign_and_submit_req.tx_json = Transaction::Payment(payment);
    sign_and_submit_req.secret = Some(creds_one.account.secret);

    // Fetch the account info for an address.
    let sign_and_submit_res = xrpl
        .sign_and_submit(sign_and_submit_req)
        .await
        .expect("failed to make sign_and_submit request");
    println!(
        "Got response to sign_and_submit request: {:?}",
        sign_and_submit_res
    );

    // Create an account info request to see the balance of account two.
    let mut req = AccountInfoRequest::default();
    // Set the account to the second set of testnet credentials.
    req.account = creds_two.account.address.to_owned();
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
