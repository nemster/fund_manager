mod common;
use common::*;
use scrypto_test::prelude::*;
use scrypto_test::prelude::ApplicationError::PanicMessage;

#[test]
fn test_change_dex() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    for n in 1..=MIN_AUTHORIZERS {
        common.authorize_admin_operation(
            n,
            MIN_AUTHORIZERS + 1,
            SET_ORACLE_COMPONENT,
            None,
            None,
            None,
        )?;
    }

    let proof = common.create_admin_proof(MIN_AUTHORIZERS + 1)?;

    common.fund_manager.set_oracle_component(
        proof,
        common.dex_and_oracle.into(),
        &mut common.env
    )?;

    Ok(())
}

#[test]
fn test_no_auth() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    let proof = common.create_admin_proof(MIN_AUTHORIZERS + 1)?;

    let result = common.fund_manager.set_oracle_component(
        proof,
        common.dex_and_oracle.into(),
        &mut common.env
    );

    if result.is_ok() {
        return Err(
            RuntimeError::ApplicationError(
                PanicMessage("Autorization bypassed in set_oracle_component".to_string())
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
            SET_ORACLE_COMPONENT,
            None,
            None,
            None,
        )?;
    }

    let proof = common.account_badge_bucket.create_proof_of_non_fungibles(
        indexset!(NonFungibleLocalId::Integer(u64::from(MIN_AUTHORIZERS + 1).into())),
        &mut common.env
    )?;

    let result = common.fund_manager.set_oracle_component(
        proof.into(),
        common.dex_and_oracle.into(),
        &mut common.env
    );

    if result.is_ok() {
        return Err(
            RuntimeError::ApplicationError(
                PanicMessage("Wrong proof accepted in set_oracle_component".to_string())
            )
        );
    }

    Ok(())
}

