# xrpl-rs

[![latest]][crates.io]
[![docs_badge]][docs]
[![deps_status]][deps]
![Downloads](https://img.shields.io/crates/d/xrpl-rs)

[latest]: https://img.shields.io/crates/v/xrpl-rs.svg
[crates.io]: https://crates.io/crates/xrpl-rs

[docs_badge]: https://docs.rs/xrpl-rs/badge.svg
[docs]: https://docs.rs/xrpl-rs

[deps_status]: https://deps.rs/repo/github/Decentralised-Advertising/xrpl-rs/status.svg
[deps]: https://deps.rs/repo/github/Decentralised-Advertising/xrpl-rs

A client implementation in Rust for interacting with the [XRPL](https://xrpl.org/).

## Example Usage

    use std::convert::TryInto;
    use xrpl_rs::{XRPL, transports::HTTP, types::account::AccountInfoRequest, types::CurrencyAmount};
    use tokio_test::block_on;

    // Create a new XRPL client with the HTTP transport.
    let xrpl = XRPL::new(
        HTTP::builder()
            .with_endpoint("http://s1.ripple.com:51234/")
            .unwrap()
            .build()
            .unwrap());

    // Create an account info request
    let mut req = AccountInfoRequest::default();
    req.account = "rG1QQv2nh2gr7RCZ1P8YYcBUKCCN633jCn".to_owned();

    // Fetch the account info for an address.
    let account_info = block_on(async {
        xrpl
            .account_info(req)
            .await
            .unwrap()
    });
    assert_eq!(account_info.account_data.balance, CurrencyAmount::XRP("9977".try_into().unwrap()));
