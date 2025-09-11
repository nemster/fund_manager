mod common;
use common::*;
use scrypto_test::prelude::*;
use scrypto_test::prelude::ApplicationError::PanicMessage;

#[test]
fn test_validator_operations() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    common.env.disable_auth_module();
    common.fund_manager.register_validator(
        true,
        &mut common.env
    )?;
    common.fund_manager.signal_protocol_update_readiness(
        "update name".to_string(),
        &mut common.env
    )?;
    common.fund_manager.update_node_key(
        "0342958027ec69a1a848651321c4dc6dd72a524d31752f76d8019e52eb6441e63a".to_string(),
        &mut common.env
    )?;
    common.env.enable_auth_module();

    Ok(())
}

#[test]
fn test_no_auth() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    let result = common.fund_manager.register_validator(
        true,
        &mut common.env
    );

    if result.is_ok() {
        return Err(
            RuntimeError::ApplicationError(
                PanicMessage("Authentication bypassed for register_validator operation".to_string())
            )
        );
    }

    let result = common.fund_manager.signal_protocol_update_readiness(
        "update name".to_string(),
        &mut common.env
    );
    
    if result.is_ok() {
        return Err(
            RuntimeError::ApplicationError(
                PanicMessage("Authentication bypassed for signal_protocol_update_readiness operation".to_string())
            )
        );
    }

    let result = common.fund_manager.update_node_key(
        "0342958027ec69a1a848651321c4dc6dd72a524d31752f76d8019e52eb6441e63a".to_string(),
        &mut common.env
    );

    if result.is_ok() {
        return Err(
            RuntimeError::ApplicationError(
                PanicMessage("Authentication bypassed for update_node_key operation".to_string())
            )
        );
    }

    Ok(())
}

