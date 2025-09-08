mod common;
use common::*;
use scrypto_test::prelude::*;
use scrypto_test::prelude::ApplicationError::PanicMessage;

#[test]
// Test withdraw and deposit of the fund manager badge
fn test_fund_manager_badge() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    for n in 1..=MIN_AUTHORIZERS {
        common.authorize_admin_operation(
            n,
            MIN_AUTHORIZERS + 1,
            8u8,
            None,
            None,
            None,
        )?;
    }

    let proof = common.create_admin_proof(MIN_AUTHORIZERS + 1)?;

    let fund_manager_badge_bucket = common.fund_manager.withdraw_fund_manager_badge(
        proof,
        &mut common.env
    )?;

    let resource_address = fund_manager_badge_bucket.resource_address(&mut common.env)?;
    assert!(
        resource_address == common.fund_manager_badge_address,
        "Wrong badge received"
    );

    let amount = fund_manager_badge_bucket.amount(&mut common.env)?;
    assert!(
        amount == Decimal::ONE,
        "One fund manager badge was expected"
    );

    common.env.disable_auth_module();
    common.fund_manager.deposit_fund_manager_badge(fund_manager_badge_bucket, &mut common.env)?;
    common.env.enable_auth_module();

    Ok(())
}

#[test]
// Test withdraw_fund_manager_badge without other admins' authorization
fn test_withdraw_no_auth() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    let proof = common.create_admin_proof(MIN_AUTHORIZERS + 1)?;

    let result1 = common.fund_manager.withdraw_fund_manager_badge(
        proof.clone(&mut common.env)?,
        &mut common.env
    );

    match result1 {
        Err(RuntimeError::ApplicationError(_)) => {},
        _ => return Err(
            RuntimeError::ApplicationError(
                PanicMessage("Autorization bypassed in withdraw_fund_manager_badge".to_string())
            )
        ),
    }

    Ok(())
}

#[test]
// Test deposit_fund_manager_badge operation with auth module
fn test_deposit_no_auth() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    for n in 1..=MIN_AUTHORIZERS {
        common.authorize_admin_operation(
            n,
            MIN_AUTHORIZERS + 1,
            8u8,
            None,
            None,
            None,
        )?;
    }

    let proof = common.create_admin_proof(MIN_AUTHORIZERS + 1)?;

    let fund_manager_badge_bucket = common.fund_manager.withdraw_fund_manager_badge(
        proof,
        &mut common.env
    )?;

    let result2 = common.fund_manager.deposit_fund_manager_badge(
        fund_manager_badge_bucket,
        &mut common.env
    );

    match result2 {
        Err(RuntimeError::SystemModuleError(SystemModuleError::AuthError(_))) => {},
        _ => return Err(
            RuntimeError::ApplicationError(
                PanicMessage("Authentication bypassed in deposit_fund_manager_badge".to_string())
            )
        ),
    }

    Ok(())
}

#[test]
// Test deposit_fund_manager_badge with a wrong badge
fn test_deposit_wrong_badge() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    for n in 1..=MIN_AUTHORIZERS {
        common.authorize_admin_operation(
            n,
            MIN_AUTHORIZERS + 1,
            8u8,
            None,
            None,
            None,
        )?;
    }

    let proof = common.create_admin_proof(MIN_AUTHORIZERS + 1)?;

    let _fund_manager_badge_bucket = common.fund_manager.withdraw_fund_manager_badge(
        proof,
        &mut common.env
    )?;

    let wrong_badge_bucket = common.a_bucket.take(dec!(1), &mut common.env)?;

    common.env.disable_auth_module();
    let result2 = common.fund_manager.deposit_fund_manager_badge(
        wrong_badge_bucket,
        &mut common.env
    );
    common.env.enable_auth_module();

    match result2 {
        Err(_) => {},
        _ => return Err(
            RuntimeError::ApplicationError(
                PanicMessage("Deposited wrong badge in deposit_fund_manager_badge".to_string())
            )
        ),
    }

    Ok(())
}

#[test]
// Test divisibility of the fund manager badge
fn test_divisibility() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    for n in 1..=MIN_AUTHORIZERS {
        common.authorize_admin_operation(
            n,
            MIN_AUTHORIZERS + 1,
            8u8,
            None,
            None,
            None,
        )?;
    }

    let proof = common.create_admin_proof(MIN_AUTHORIZERS + 1)?;

    let fund_manager_badge_bucket = common.fund_manager.withdraw_fund_manager_badge(
        proof,
        &mut common.env
    )?;

    let result = fund_manager_badge_bucket.take(dec!("0.5"), &mut common.env);

    match result {
        Err(_) => {},
        _ => return Err(
            RuntimeError::ApplicationError(
                PanicMessage("Successfully divided fund manager badge".to_string())
            )
        ),
    }

    Ok(())
}
