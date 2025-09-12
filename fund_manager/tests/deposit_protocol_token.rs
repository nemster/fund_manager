mod common;
use common::*;
use scrypto_test::prelude::*;
use scrypto_test::prelude::ApplicationError::PanicMessage;

#[test]
fn test_deposit_protocol_token() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    let a_protocol_name = "A".to_string();
    let a_address = common.a_bucket.resource_address(&mut common.env)?;
    common.create_and_add_defi_protocol(
        a_protocol_name.clone(),
        a_address,
        None,
        100u8,
        true
    )?;

    common.env.disable_auth_module();
    common.fund_manager.deposit_protocol_token(
        a_protocol_name,
        common.token_bucket,
        HashMap::new(),
        false,
        &mut common.env
    )?;
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
        100u8,
        true
    )?;

    let result = common.fund_manager.deposit_protocol_token(
        a_protocol_name,
        common.token_bucket,
        HashMap::new(),
        false,
        &mut common.env
    );

    if result.is_ok() {
        return Err(
            RuntimeError::ApplicationError(
                PanicMessage("Auth bypassed for deposit_protocol_token".to_string())
            )
        );
    }

    Ok(())
}

#[test]
fn test_non_existing_protocol() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    let a_protocol_name = "A".to_string();
    let a_address = common.a_bucket.resource_address(&mut common.env)?;
    common.create_and_add_defi_protocol(
        a_protocol_name,
        a_address,
        None,
        100u8,
        true
    )?;

    // Different protocol name from the created one
    let result = common.fund_manager.deposit_protocol_token(
        "B".to_string(),
        common.token_bucket,
        HashMap::new(),
        false,
        &mut common.env
    );

    if result.is_ok() {
        return Err(
            RuntimeError::ApplicationError(
                PanicMessage("Successfully deposited token in non existing protocol".to_string())
            )
        );
    }

    Ok(())
}

