mod common;
use common::*;
use scrypto_test::prelude::*;
use scrypto_test::prelude::ApplicationError::PanicMessage;

#[test]
fn test_set_percentages() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    let a_protocol_name = "A".to_string();
    let a_address = common.a_bucket.resource_address(&mut common.env)?;
    common.create_and_add_defi_protocol(
        a_protocol_name.clone(),
        a_address,
        None,
        50u8,
        true
    )?;

    let b_protocol_name = "B".to_string();
    let b_address = common.b_bucket.resource_address(&mut common.env)?;
    common.create_and_add_defi_protocol(
        b_protocol_name.clone(),
        b_address,
        None,
        50u8,
        true
    )?;

    let mut map = HashMap::<String, u8>::new();
    map.insert(a_protocol_name, 80u8);
    map.insert(b_protocol_name, 20u8);

    common.env.disable_auth_module();
    common.fund_manager.set_defi_protocols_percentage(map, &mut common.env)?;
    common.env.enable_auth_module();

    Ok(())
}

#[test]
fn test_no_auth() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    let a_protocol_name = "A".to_string();
    let a_address = common.a_bucket.resource_address(&mut common.env)?;
    common.create_and_add_defi_protocol(
        a_protocol_name.clone(),
        a_address,
        None,
        50u8,
        true
    )?;

    let mut map = HashMap::<String, u8>::new();
    map.insert(a_protocol_name, 80u8);

    let result = common.fund_manager.set_defi_protocols_percentage(map, &mut common.env);

    if result.is_ok() {
        return Err(
            RuntimeError::ApplicationError(
                PanicMessage("Auth bypassed in set_defi_protocols_percentage".to_string())
            )
        );
    }

    Ok(())
}

#[test]
fn test_wrong_percentage() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    let a_protocol_name = "A".to_string();
    let a_address = common.a_bucket.resource_address(&mut common.env)?;
    common.create_and_add_defi_protocol(
        a_protocol_name.clone(),
        a_address,
        None,
        50u8,
        true
    )?;

    // Percentage > 100
    let mut map = HashMap::<String, u8>::new();
    map.insert(a_protocol_name, 110u8);

    common.env.disable_auth_module();
    let result = common.fund_manager.set_defi_protocols_percentage(map, &mut common.env);
    common.env.enable_auth_module();

    if result.is_ok() {
        return Err(
            RuntimeError::ApplicationError(
                PanicMessage("Wrong percentage accepted in_defi_protocols_percentage".to_string())
            )
        );
    }

    Ok(())
}
