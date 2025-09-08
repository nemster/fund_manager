mod common;
use common::*;
use scrypto_test::prelude::*;
use scrypto_test::prelude::ApplicationError::PanicMessage;

#[test]
// Check that update_defi_protocols_value correctly update the protocol information
fn test_update_protocol_value() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    let a_protocol_name = "A".to_string();
    let a_address = common.a_bucket.resource_address(&mut common.env)?;
    let a_amount = common.a_bucket.amount(&mut common.env)?;
    common.create_and_add_defi_protocol(
        a_protocol_name.clone(),
        a_address,
        None,
        33u8,
        true
    )?;

    let b_protocol_name = "B".to_string();
    let b_address = common.b_bucket.resource_address(&mut common.env)?;
    let b_amount = common.b_bucket.amount(&mut common.env)?;
    common.create_and_add_defi_protocol(
        b_protocol_name.clone(),
        b_address,
        None,
        33u8,
        true
    )?;

    let c_protocol_name = "C".to_string();
    let c_address = common.c_bucket.resource_address(&mut common.env)?;
    let c_amount = common.c_bucket.amount(&mut common.env)?;
    common.create_and_add_defi_protocol(
        c_protocol_name.clone(),
        c_address,
        None,
        33u8,
        true
    )?;

    common.env.disable_auth_module();
    common.fund_manager.deposit_coin(
        a_protocol_name.clone(),
        common.a_bucket,
        None,
        HashMap::new(),
        false,
        &mut common.env
    )?;
    common.fund_manager.deposit_coin(
        b_protocol_name.clone(),
        common.b_bucket,
        None,
        HashMap::new(),
        false,
        &mut common.env
    )?;
    common.fund_manager.deposit_coin(
        c_protocol_name.clone(),
        common.c_bucket,
        None,
        HashMap::new(),
        false,
        &mut common.env
    )?;
    common.env.enable_auth_module();

    let mut protocols = common.fund_manager.fund_details(&mut common.env)?;

    let mut a_protocol_value = protocols.get(&a_protocol_name).expect("Protocol not found");
    let mut expected_a_protocol_value = a_amount * A_PRICE;
    assert!(
        *a_protocol_value == expected_a_protocol_value,
        "A protocol value: {}, should be {}",
        a_protocol_value,
        expected_a_protocol_value
    );

    let mut b_protocol_value = protocols.get(&b_protocol_name).expect("Protocol not found");
    let mut expected_b_protocol_value = b_amount * B_PRICE;
    assert!(
        *b_protocol_value == expected_b_protocol_value,
        "B protocol value: {}, should be {}",
        b_protocol_value,
        expected_b_protocol_value
    );

    let mut c_protocol_value = protocols.get(&c_protocol_name).expect("Protocol not found");
    let mut expected_c_protocol_value = c_amount * C_PRICE;
    assert!(
        *c_protocol_value == expected_c_protocol_value,
        "C protocol value: {}, should be {}",
        c_protocol_value,
        expected_c_protocol_value
    );

    let new_a_price = dec!(10);
    common.dex_and_oracle.set_price(
        a_address,
        new_a_price,
        &mut common.env
    )?;

    let new_b_price = dec!(20);
    common.dex_and_oracle.set_price(
        b_address,
        new_b_price,
        &mut common.env
    )?;

    let new_c_price = dec!(30);
    common.dex_and_oracle.set_price(
        c_address,
        new_c_price,
        &mut common.env
    )?;

    // Update only A and B protocols value
    let mut indexset = IndexSet::<String>::new();
    indexset.insert(a_protocol_name.clone());
    indexset.insert(b_protocol_name.clone());
    common.env.disable_auth_module();
    common.fund_manager.update_defi_protocols_value(
        indexset,
        HashMap::new(),
        &mut common.env
    )?;
    common.env.enable_auth_module();

    protocols = common.fund_manager.fund_details(&mut common.env)?;

    a_protocol_value = protocols.get(&a_protocol_name).expect("Protocol not found");
    expected_a_protocol_value = a_amount * new_a_price;
    assert!(
        *a_protocol_value == expected_a_protocol_value,
        "A protocol value: {}, should be {}",
        a_protocol_value,
        expected_a_protocol_value
    );

    b_protocol_value = protocols.get(&b_protocol_name).expect("Protocol not found");
    expected_b_protocol_value = b_amount * new_b_price;
    assert!(
        *b_protocol_value == expected_b_protocol_value,
        "B protocol value: {}, should be {}",
        b_protocol_value,
        expected_b_protocol_value
    );

    c_protocol_value = protocols.get(&c_protocol_name).expect("Protocol not found");
    expected_c_protocol_value = c_amount * C_PRICE;
    assert!(
        *c_protocol_value == expected_c_protocol_value,
        "C protocol value: {}, should be {}",
        c_protocol_value,
        expected_c_protocol_value
    );

    Ok(())
}

#[test]
// Check that badge authentication is needed
fn test_no_auth() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    let result = common.fund_manager.update_defi_protocols_value(
        IndexSet::new(),
        HashMap::new(),
        &mut common.env
    );

    match result {
        Err(RuntimeError::SystemModuleError(SystemModuleError::AuthError(_))) => {},
        _ => return Err(
            RuntimeError::ApplicationError(
                PanicMessage("Authentication bypassed in update_defi_protocols_value".to_string())
            )
        ),
    }

    Ok(())
}

#[test]
// Check that calling update_defi_protocols_value with a non existing protocol name panics
fn test_non_existing_protocol() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    let mut indexset = IndexSet::<String>::new();
    indexset.insert("not existing".to_string());

    common.env.disable_auth_module();
    let result = common.fund_manager.update_defi_protocols_value(
        indexset,
        HashMap::new(),
        &mut common.env
    );
    common.env.enable_auth_module();

    match result {
        Err(RuntimeError::ApplicationError(_)) => {},
        _ => return Err(
            RuntimeError::ApplicationError(
                PanicMessage("Silently ignored non existing protocol in update_defi_protocols_value".to_string())
            )
        ),
    }

    Ok(())
}
