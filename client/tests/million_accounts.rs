#![allow(missing_docs, clippy::pedantic, clippy::restriction)]

use iroha_core::{
    genesis::{GenesisNetwork, GenesisNetworkTrait, GenesisTransaction, RawGenesisBlock},
    prelude::*,
    samples::get_config,
};
use test_network::{get_key_pair, Peer as TestPeer, TestRuntime};
use tokio::runtime::Runtime;

fn generate_accounts(num: u32) -> Vec<GenesisTransaction> {
    let mut ret = Vec::with_capacity(usize::try_from(num).expect("panic"));
    for _i in 0..num {
        ret.push(
            GenesisTransaction::new(
                &format!("Alice-{}", num),
                &format!("wonderland-{}", num),
                &PublicKey::default(),
            )
            .expect("Failed to create Genesis"),
        );
    }
    ret
}

fn generate_genesis(num: u32) -> RawGenesisBlock {
    let transactions = generate_accounts(num);
    RawGenesisBlock { transactions }
}

/// Generate genesis with a given number of accounts and try to submit
/// it. This will not check if the accounts are created, and is only
/// needed for you to be able to monitor the RAM usage using an
/// external program like `htop`.
#[test]
#[ignore = "Very slow. run with `cargo test --release` to significantly improve performance."]
fn create_million_accounts() {
    let mut peer = <TestPeer>::new().expect("Failed to create peer");
    let configuration = get_config(
        std::iter::once(peer.id.clone()).collect(),
        Some(get_key_pair()),
    );
    let rt = Runtime::test();
    let genesis = GenesisNetwork::from_configuration(
        true,
        generate_genesis(1000000),
        &configuration.genesis,
        configuration.sumeragi.max_instruction_number,
    )
    .expect("genesis creation failed");

    // This only submits the genesis. It doesn't check if the accounts
    // are created, because that check is 1) not needed for what the
    // test is actually for, 2) incredibly slow, making this sort of
    // test impractical, 3) very likely to overflow memory on systems
    // with less than 16GiB of free memory.
    rt.block_on(peer.start_with_config(genesis, configuration));
}
