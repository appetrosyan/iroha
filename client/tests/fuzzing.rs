#![allow(clippy::restriction)]

use eyre::Result;
use iroha_client::client;
use iroha_data_model::prelude::*;
use test_network::{Peer as TestPeer, *};

#[test_fuzz::test_fuzz]
fn fuzz_account_id_create(name: String, domain_name: String) -> Result<()> {
    let (_rt, _peer, mut test_client) = <TestPeer>::start_test_with_runtime();
    wait_for_genesis_committed(vec![test_client.clone()], 0);

    let normal_account_id = AccountId::new(&name, &domain_name)?;
    // Can we really assume that these two are congruent?
    let new_domain = Domain::new(DomainId::new(&domain_name)?);
    let create_account = RegisterBox::new(IdentifiableBox::from(NewAccount::new(
        normal_account_id.clone(),
    )));
    let create_domain = RegisterBox::new(IdentifiableBox::from(new_domain));
    test_client.submit_all_blocking(vec![create_account.into(), create_domain.into()])?;
    Ok(())
}

#[test_fuzz::test_fuzz]
fn fuzz_mint_amount_quantity(amount: u32) {
    let (_rt, _peer, mut test_client) = <TestPeer>::start_test_with_runtime();
    wait_for_genesis_committed(vec![test_client.clone()], 0);

    let account_id = AccountId::test("alice", "wonderland");
    let asset_definition_id = AssetDefinitionId::test("rose", "wonderland");
    let mint = MintBox::new(
        Value::U32(amount),
        IdBox::AssetId(AssetId::new(
            asset_definition_id.clone(),
            account_id.clone(),
        )),
    );
    test_client
        .submit_blocking(mint)
        .expect("This shouldn't fail");
}

#[test_fuzz::test_fuzz]
#[ignore = "This doesn't test anything really new"]
fn fuzz_mint_amount_big_quantity(amount: u128) {
    let (_rt, _peer, mut test_client) = <TestPeer>::start_test_with_runtime();
    wait_for_genesis_committed(vec![test_client.clone()], 0);

    let account_id = AccountId::test("alice", "wonderland");
    let asset_definition_id = AssetDefinitionId::test("rose", "wonderland");
    let mint = MintBox::new(
        Value::U128(amount),
        IdBox::AssetId(AssetId::new(
            asset_definition_id.clone(),
            account_id.clone(),
        )),
    );
    test_client
        .submit_blocking(mint)
        .expect("This shouldn't fail");
}

#[test_fuzz::test_fuzz]
fn fuzz_mint_amount_fixed(amount: f64) -> Result<()> {
    let (_rt, _peer, mut test_client) = <TestPeer>::start_test_with_runtime();

    // Given
    let account_id = AccountId::test("alice", "wonderland");
    let asset_definition_id = AssetDefinitionId::test("xor", "wonderland");
    let identifiable_box =
        IdentifiableBox::from(AssetDefinition::with_precision(asset_definition_id.clone()));
    let create_asset = RegisterBox::new(identifiable_box);
    let metadata = iroha_data_model::metadata::UnlimitedMetadata::default();

    //When
    let quantity: Fixed = if amount > 0_f64 {
        Fixed::try_from(amount).unwrap()
    } else {
        Fixed::try_from(-amount).unwrap()
    };
    let mint = MintBox::new(
        Value::Fixed(quantity),
        IdBox::AssetId(AssetId::new(
            asset_definition_id.clone(),
            account_id.clone(),
        )),
    );
    let tx = test_client
        .build_transaction(
            Executable::Instructions(vec![create_asset.into(), mint.into()]),
            metadata,
        )
        .unwrap();
    test_client.submit_transaction(tx).unwrap();
    test_client.poll_request(client::asset::by_account_id(account_id.clone()), |result| {
        result.iter().any(|asset| {
            asset.id.definition_id == asset_definition_id
                && asset.value == AssetValue::Fixed(quantity)
        })
    });
    Ok(())
}
