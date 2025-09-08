mod common;
use common::*;
use scrypto_test::prelude::*;
use scrypto_test::prelude::ApplicationError::PanicMessage;
use dummy_validator::dummy_validator::Owner;

#[test]
// Test withdraw and deposit of validator badge
fn test_validator_badge() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    for n in 1..=MIN_AUTHORIZERS {
        common.authorize_admin_operation(
            n,
            MIN_AUTHORIZERS + 1,
            0u8,
            None,
            None,
            None,
        )?;
    }

    let proof = common.create_admin_proof(MIN_AUTHORIZERS + 1)?;

    let validator_badge_bucket = common.fund_manager.withdraw_validator_badge(
        proof,
        &mut common.env
    )?;

    let resource_address = validator_badge_bucket.resource_address(&mut common.env)?;
    assert!(
        resource_address == common.validator_owner_badge_address,
        "Wrong badge received"
    );

    let amount = validator_badge_bucket.amount(&mut common.env)?;
    assert!(
        amount == Decimal::ONE,
        "One validator owner badge was expected"
    );

    common.env.disable_auth_module();
    common.fund_manager.deposit_validator_badge(validator_badge_bucket, &mut common.env)?;
    common.env.enable_auth_module();

    Ok(())
}

#[test]
// Test withdraw_validator_badge without other admins' authorization
fn test_withdraw_no_auth() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    let proof = common.create_admin_proof(MIN_AUTHORIZERS + 1)?;

    let result1 = common.fund_manager.withdraw_validator_badge(
        proof.clone(&mut common.env)?,
        &mut common.env
    );

    match result1 {
        Err(RuntimeError::ApplicationError(_)) => {},
        _ => return Err(
            RuntimeError::ApplicationError(
                PanicMessage("Autorization bypassed in withdraw_validator_badge".to_string())
            )
        ),
    }

    Ok(())
}

#[test]
// Test deposit_validator_badge operation with auth module
fn test_deposit_no_auth() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    for n in 1..=MIN_AUTHORIZERS {
        common.authorize_admin_operation(
            n,
            MIN_AUTHORIZERS + 1,
            0u8,
            None,
            None,
            None,
        )?;
    }

    let proof = common.create_admin_proof(MIN_AUTHORIZERS + 1)?;

    let validator_badge_bucket = common.fund_manager.withdraw_validator_badge(
        proof,
        &mut common.env
    )?;

    let result2 = common.fund_manager.deposit_validator_badge(
        validator_badge_bucket,
        &mut common.env
    );

    match result2 {
        Err(RuntimeError::SystemModuleError(SystemModuleError::AuthError(_))) => {},
        _ => return Err(
            RuntimeError::ApplicationError(
                PanicMessage("Authentication bypassed in deposit_validator_badge".to_string())
            )
        ),
    }

    Ok(())
}

#[test]
// Check that is not possible to deposit another validator badge if there's already one in place
fn test_multiple_validator_badges1() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    let validator_badge_bucket = BucketFactory::create_non_fungible_bucket(
        common.validator_owner_badge_address,
        vec![(NonFungibleLocalId::Integer(IntegerNonFungibleLocalId::from(2u64)), Owner {})],
        CreationStrategy::DisableAuthAndMint,
        &mut common.env,
    )?;

    common.env.disable_auth_module();
    let result = common.fund_manager.deposit_validator_badge(
        validator_badge_bucket,
        &mut common.env
    );
    common.env.enable_auth_module();

    match result {
        Err(RuntimeError::ApplicationError(_)) => {},
        _ => return Err(
            RuntimeError::ApplicationError(
                PanicMessage("Successful deposit of a second validator badge".to_string())
            )
        ),
    }

    Ok(())
}

#[test]
// Check that is not possible to deposit two validator badges at once
fn test_multiple_validator_badges2() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    for n in 1..=MIN_AUTHORIZERS {
        common.authorize_admin_operation(
            n,
            MIN_AUTHORIZERS + 1,
            0u8,
            None,
            None,
            None,
        )?;
    }

    let proof = common.create_admin_proof(MIN_AUTHORIZERS + 1)?;

    let _validator_badge_bucket = common.fund_manager.withdraw_validator_badge(
        proof,
        &mut common.env
    )?;

    let two_validator_badges_bucket = BucketFactory::create_non_fungible_bucket(
        common.validator_owner_badge_address,
        vec![
            (NonFungibleLocalId::Integer(IntegerNonFungibleLocalId::from(2u64)), Owner {}),
            (NonFungibleLocalId::Integer(IntegerNonFungibleLocalId::from(3u64)), Owner {})
        ],
        CreationStrategy::DisableAuthAndMint,
        &mut common.env,
    )?;

    common.env.disable_auth_module();
    let result = common.fund_manager.deposit_validator_badge(
        two_validator_badges_bucket,
        &mut common.env
    );
    common.env.enable_auth_module();

    match result {
        Err(RuntimeError::ApplicationError(_)) => {},
        _ => return Err(
            RuntimeError::ApplicationError(
                PanicMessage("Successful deposit of two validator badges".to_string())
            )
        ),
    }

    Ok(())
}

#[test]
// Test deposit_validator_badge with a wrong badge
fn test_deposit_wrong_badge() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    for n in 1..=MIN_AUTHORIZERS {
        common.authorize_admin_operation(
            n,
            MIN_AUTHORIZERS + 1,
            0u8,
            None,
            None,
            None,
        )?;
    }

    let proof = common.create_admin_proof(MIN_AUTHORIZERS + 1)?;

    let _validator_badge_bucket = common.fund_manager.withdraw_validator_badge(
        proof,
        &mut common.env
    )?;

    let mut ids = IndexSet::<NonFungibleLocalId>::new();
        ids.insert(NonFungibleLocalId::Integer(1u64.into()));
    let wrong_badge_bucket = common.account_badge_bucket.take_non_fungibles(
        ids,
        &mut common.env
    )?;

    common.env.disable_auth_module();
    let result2 = common.fund_manager.deposit_validator_badge(
        wrong_badge_bucket,
        &mut common.env
    );
    common.env.enable_auth_module();

    match result2 {
        Err(_) => {},
        _ => return Err(
            RuntimeError::ApplicationError(
                PanicMessage("Deposited wrong badge in deposit_validator_badge".to_string())
            )
        ),
    }

    Ok(())
}

