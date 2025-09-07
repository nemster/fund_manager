mod common;
use common::*;
use scrypto_test::prelude::*;

#[test]
// Test unlock + unstake + investment in defi + fund units distribution 
fn test_unstake_and_distribution() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    let a_address = common.a_bucket.resource_address(&mut common.env)?;
    let b_address = common.b_bucket.resource_address(&mut common.env)?;
    common.create_and_add_defi_protocol(
        "A/B".to_string(),
        a_address,
        Some(b_address),
        100u8,
        true
    )?;

    let xrd_to_unstake = dec!(100);

    common.env.disable_auth_module();

    common.fund_manager.start_unlock_owner_stake_units(
        xrd_to_unstake,
        &mut common.env
    )?;

    let (_, claim_nft_id) = common.fund_manager.start_unstake(&mut common.env)?;

    let (
        xrd_to_buyback_fund,
        xrd_to_protocol,
        protocol_name,
        fund_unit_amount_to_distribute
    ) = common.fund_manager.finish_unstake(
        claim_nft_id,
        HashMap::new(),
        &mut common.env
    )?;

    let mut distribution = IndexMap::new();
    distribution.insert(
        common.account,
        dec!(1)
    );
    common.fund_manager.fund_units_distribution(
        distribution,
        false,
        &mut common.env
    )?;

    common.env.enable_auth_module();

    let expected_xrd_to_buyback_fund = xrd_to_unstake * BUYBACK_FUND_PERCENTAGE / dec!(100);
    let expected_xrd_to_protocol = xrd_to_unstake * (dec!(100) - BUYBACK_FUND_PERCENTAGE) / dec!(100);
    assert!(
        xrd_to_buyback_fund == expected_xrd_to_buyback_fund,
        "XRD in account: {}, should be {}",
        xrd_to_buyback_fund,
        expected_xrd_to_buyback_fund
    );
    assert!(
        xrd_to_protocol == expected_xrd_to_protocol,
        "Invested XRD amount: {}, should be {}",
        xrd_to_protocol,
        expected_xrd_to_protocol
    );

    let protocols = common.fund_manager.fund_details(&mut common.env)?;
    let protocol_value = protocols.get(&protocol_name).expect("Protocol not found");
    let expected_protocol_value = expected_xrd_to_protocol * XRD_PRICE;
    assert!(
        *protocol_value == expected_protocol_value,
        "Protocol value is {}, should be {}",
        *protocol_value,
        expected_protocol_value
    );

    let fund_unit_supply = FUND_UNIT_INITIAL_SUPPLY + fund_unit_amount_to_distribute;
    let (fund_unit_net_price, fund_unit_gross_price) =
        common.fund_manager.fund_unit_value(&mut common.env)?;
    let expected_fund_unit_gross_price = *protocol_value / fund_unit_supply;
    let expected_fund_unit_net_price = expected_fund_unit_gross_price *
        (dec!(100) - WITHDRAWAL_FEE_PERCENTAGE) / dec!(100);
    assert!(
        fund_unit_gross_price == expected_fund_unit_gross_price,
        "Fund unit gross price: {}, should be {}",
        fund_unit_gross_price,
        expected_fund_unit_gross_price
    );
    assert!(
        fund_unit_net_price == expected_fund_unit_net_price,
        "Fund unit net price: {}, should be {}",
        fund_unit_net_price,
        expected_fund_unit_net_price
    );

    Ok(())
}

