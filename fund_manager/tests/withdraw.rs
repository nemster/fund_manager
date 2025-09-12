mod common;
use common::*;
use scrypto_test::prelude::*;
// use scrypto_test::prelude::ApplicationError::PanicMessage;

#[test]
fn test_withdraw() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    let protocol_name = "My protocol".to_string();
    let a_address = common.a_bucket.resource_address(&mut common.env)?;
    common.create_and_add_defi_protocol(
        protocol_name.clone(),
        a_address,
        None,
        100u8,
        true
    )?;

    let deposited_a_amount = common.a_bucket.amount(&mut common.env)?;
    common.env.disable_auth_module();
    let fund_unit_bucket = common.fund_manager.deposit_coin(
        protocol_name.clone(),
        common.a_bucket.take(deposited_a_amount, &mut common.env)?,
        None,
        HashMap::new(),
        true,
        &mut common.env
    )?
        .unwrap();
    common.env.enable_auth_module();

    let protocols = common.fund_manager.fund_details(&mut common.env)?;
    let protocol_value = protocols.get(&protocol_name).expect("Protocol not found");
    let expected_protocol_value = deposited_a_amount * A_PRICE;
    assert!(
        *protocol_value == expected_protocol_value,
        "Protocol value: {}, was supposed to be {}",
        protocol_value,
        expected_protocol_value
    );

    let mut fund_unit_amount = fund_unit_bucket.amount(&mut common.env)?;
    let fund_unit_expected_amount = protocol_value;
    assert!(
        fund_unit_amount == *fund_unit_expected_amount,
        "Fund unit amount: {}, was supposed to be {}",
        fund_unit_amount,
        fund_unit_expected_amount
    );

    let (fund_unit_net_price, fund_unit_gross_price) =
        common.fund_manager.fund_unit_value(&mut common.env)?;
    let fund_unit_expected_gross_price = *protocol_value / (FUND_UNIT_INITIAL_SUPPLY + fund_unit_amount);
    assert!(
        fund_unit_gross_price == fund_unit_expected_gross_price,
        "Fund unit gross price: {}, was supposed to be {}",
        fund_unit_gross_price,
        fund_unit_expected_gross_price
    );

    let fund_unit_expected_net_price = (fund_unit_gross_price * (100 - WITHDRAWAL_FEE_PERCENTAGE)) / dec!(100);
    assert!(
        fund_unit_net_price == fund_unit_expected_net_price,
        "Fund unit net price: {}, was supposed to be {}",
        fund_unit_net_price,
        fund_unit_expected_net_price
    );

    let mut fund_unit_burned_amount = fund_unit_amount / dec!(2);
    let (a_bucket, _, fund_unit_returned) = common.fund_manager.withdraw(
        fund_unit_bucket.take(fund_unit_burned_amount, &mut common.env)?,
        None, HashMap::new(),
        &mut common.env
    )?;

    let fund_unit_returned_amount = match fund_unit_returned {
        Some(bucket) => bucket.amount(&mut common.env)?,
        None => Decimal::ZERO,
    };
    fund_unit_burned_amount -= fund_unit_returned_amount;

    let withdrawn_a_amount = a_bucket.amount(&mut common.env)?;
    let expected_withdrawn_a_amount = fund_unit_net_price * fund_unit_burned_amount / A_PRICE;
    assert!(
        withdrawn_a_amount == expected_withdrawn_a_amount,
        "Withdrawn A amount: {}, was supposed to be {}",
        withdrawn_a_amount,
        expected_withdrawn_a_amount,
    );

    let withdrawal_fee = 10u8;
    for n in 1..=MIN_AUTHORIZERS {
        common.authorize_admin_operation(
            n,
            MIN_AUTHORIZERS + 1,
            SET_WITHDRAWAL_FEE,
            None,
            Some(withdrawal_fee),
            None,
        )?;
    }

    let proof = common.create_admin_proof(MIN_AUTHORIZERS + 1)?;

    common.fund_manager.set_withdrawal_fee(
        proof,
        withdrawal_fee,
        &mut common.env
    )?;

    let protocols = common.fund_manager.fund_details(&mut common.env)?;
    let protocol_value = protocols.get(&protocol_name).expect("Protocol not found");
    let expected_protocol_value = (deposited_a_amount - withdrawn_a_amount) * A_PRICE;
    assert!(
        *protocol_value == expected_protocol_value,
        "Protocol value: {}, was supposed to be {}",
        protocol_value,
        expected_protocol_value
    );

    Ok(())
}

// TODO: wrong coin
