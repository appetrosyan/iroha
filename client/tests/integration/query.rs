#![allow(clippy::restriction)]

use eyre::Result;
use iroha_client::client;
// use iroha_core::config::Configuration;
use iroha_data_model::prelude::*;
use parity_scale_codec::Encode;
use test_network::{Peer as TestPeer, *};

#[test]
fn find_all_domains_regression_test() -> Result<()> {
    let (_rt, _peer, mut test_client) = <TestPeer>::start_test_with_runtime();
    wait_for_genesis_committed(vec![test_client.clone()], 0);
    // let pipeline_time = Configuration::pipeline_time();

    // Given

    let normal_domain_name = "sora";
    let create_domain = RegisterBox::new(IdentifiableBox::from(Domain::test(normal_domain_name)));
    test_client.submit(create_domain)?;

    let reply = test_client.request(FindDomainById::new(
        DomainId::new("genesis").expect("Valid"),
    ))?;
    println!("{:#?}", reply);
    let reply = test_client.request(FindAccountsByDomainId::new(
        DomainId::new("genesis").expect("Valid"),
    ))?;
    println!("{:#?}", reply);
    println!("{}", String::from_utf8_lossy(&reply.encode()));
    let reply = test_client.request(FindAllAccounts::new())?;
    println!("{:#?}", reply);
    Ok(())
}
