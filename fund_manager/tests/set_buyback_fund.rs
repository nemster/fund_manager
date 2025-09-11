mod common;
use common::*;
use scrypto_test::prelude::*;
use scrypto_test::prelude::ApplicationError::PanicMessage;

#[test]
fn test_set_buyback_fund() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    for n in 1..=MIN_AUTHORIZERS {
        common.authorize_admin_operation(
            n,
            MIN_AUTHORIZERS + 1,
            SET_BUYBACK_FUND,
            None,
            Some(BUYBACK_FUND_PERCENTAGE),
            Some(common.account.into()),
        )?;
    }

    let proof = common.create_admin_proof(MIN_AUTHORIZERS + 1)?;

    common.fund_manager.set_buyback_fund(
        proof,
        BUYBACK_FUND_PERCENTAGE,
        common.account.into(),
        &mut common.env
    )?;

    Ok(())
}

#[test]
fn test_no_auth() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    let proof = common.create_admin_proof(MIN_AUTHORIZERS + 1)?;

    let result = common.fund_manager.set_buyback_fund(
        proof,
        BUYBACK_FUND_PERCENTAGE,
        common.account.into(),
        &mut common.env
    );

    if result.is_ok() {
        return Err(
            RuntimeError::ApplicationError(
                PanicMessage("Autorization bypassed in set_buyback_fund".to_string())
            )
        );
    }

    Ok(())
}

#[test]
fn test_wrong_auth1() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    for n in 1..=MIN_AUTHORIZERS {
        common.authorize_admin_operation(
            n,
            MIN_AUTHORIZERS + 1,
            SET_BUYBACK_FUND,
            None,
            None,
            Some(common.account.into()),
        )?;
    }

    let proof = common.create_admin_proof(MIN_AUTHORIZERS + 1)?;

    let result = common.fund_manager.set_buyback_fund(
        proof,
        BUYBACK_FUND_PERCENTAGE,
        common.account.into(),
        &mut common.env
    );

    if result.is_ok() {
        return Err(
            RuntimeError::ApplicationError(
                PanicMessage("Authorization without percentage accepted in set_buyback_fund".to_string())
            )
        );
    }

    Ok(())
}

#[test]
fn test_wrong_auth2() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    for n in 1..=MIN_AUTHORIZERS {
        common.authorize_admin_operation(
            n,
            MIN_AUTHORIZERS + 1,
            SET_BUYBACK_FUND,
            None,
            Some(BUYBACK_FUND_PERCENTAGE),
            None,
        )?;
    }

    let proof = common.create_admin_proof(MIN_AUTHORIZERS + 1)?;

    let result = common.fund_manager.set_buyback_fund(
        proof,
        BUYBACK_FUND_PERCENTAGE,
        common.account.into(),
        &mut common.env
    );

    if result.is_ok() {
        return Err(
            RuntimeError::ApplicationError(
                PanicMessage("Authorization without address accepted in set_buyback_fund".to_string())
            )
        );
    }

    Ok(())
}

#[test]
fn test_wrong_proof() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    for n in 1..=MIN_AUTHORIZERS {
        common.authorize_admin_operation(
            n,
            MIN_AUTHORIZERS + 1,
            SET_BUYBACK_FUND,
            None,
            None,
            Some(common.account.into()),
        )?;
    }

    let proof = common.account_badge_bucket.create_proof_of_non_fungibles(
        indexset!(NonFungibleLocalId::Integer(u64::from(MIN_AUTHORIZERS + 1).into())),
        &mut common.env
    )?;

    let result = common.fund_manager.set_buyback_fund(
        proof.into(),
        BUYBACK_FUND_PERCENTAGE,
        common.account.into(),
        &mut common.env
    );

    if result.is_ok() {
        return Err(
            RuntimeError::ApplicationError(
                PanicMessage("Wrong proof accepted in set_buyback_fund".to_string())
            )
        );
    }

    Ok(())
}

#[test]
fn test_wrong_percentage() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    for n in 1..=MIN_AUTHORIZERS {
        common.authorize_admin_operation(
            n,
            MIN_AUTHORIZERS + 1,
            SET_BUYBACK_FUND,
            None,
            Some(101u8),
            Some(common.account.into()),
        )?;
    }

    let proof = common.create_admin_proof(MIN_AUTHORIZERS + 1)?;

    let result = common.fund_manager.set_buyback_fund(
        proof,
        101u8,
        common.account.into(),
        &mut common.env
    );

    if result.is_ok() {
        return Err(
            RuntimeError::ApplicationError(
                PanicMessage("Wrong percentage accepted in set_buyback_fund".to_string())
            )
        );
    }

    Ok(())
}

