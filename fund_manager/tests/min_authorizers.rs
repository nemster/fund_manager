mod common;
use common::*;
use scrypto_test::prelude::*;
use scrypto_test::prelude::ApplicationError::PanicMessage;

#[test]
fn test_decrease_increase_min_autorizers() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    let mut min_authorizers = MIN_AUTHORIZERS;

    while min_authorizers > 0 {
        for n in 1..=min_authorizers {
            common.authorize_admin_operation(
                n,
                NUMBER_OF_ADMINS,
                DECREASE_MIN_AUTHORIZERS,
                None,
                None,
                None,
            )?;
        }

        let proof = common.create_admin_proof(NUMBER_OF_ADMINS)?;

        common.fund_manager.decrease_min_authorizers(
            proof,
            &mut common.env
        )?;

        min_authorizers -= 1;
    }

    let proof = common.create_admin_proof(NUMBER_OF_ADMINS)?;

    common.fund_manager.increase_min_authorizers(
        proof,
        &mut common.env
    )?;

    Ok(())
}

#[test]
fn test_increase_too_much() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    let mut min_authorizers = MIN_AUTHORIZERS;

    while min_authorizers < NUMBER_OF_ADMINS -1 {
        for n in 1..=min_authorizers {
            common.authorize_admin_operation(
                n,
                NUMBER_OF_ADMINS,
                INCREASE_MIN_AUTHORIZERS,
                None,
                None,
                None,
            )?;
        }

        let proof = common.create_admin_proof(NUMBER_OF_ADMINS)?;

        common.fund_manager.increase_min_authorizers(
            proof,
            &mut common.env
        )?;

        min_authorizers += 1;
    }

    for n in 1..=min_authorizers {
        common.authorize_admin_operation(
            n,
            NUMBER_OF_ADMINS,
            INCREASE_MIN_AUTHORIZERS,
            None,
            None,
            None,
        )?;
    }

    let proof = common.create_admin_proof(NUMBER_OF_ADMINS)?;

    let result = common.fund_manager.increase_min_authorizers(
        proof,
        &mut common.env
    );

    if result.is_ok() {
        return Err(
            RuntimeError::ApplicationError(
                PanicMessage("Successfully increased min authorizers too much".to_string())
            )
        );
    }

    Ok(())
}

#[test]
fn test_verify_increase() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    for n in 1..=MIN_AUTHORIZERS {
        common.authorize_admin_operation(
            n,
            NUMBER_OF_ADMINS,
            INCREASE_MIN_AUTHORIZERS,
            None,
            None,
            None,
        )?;
    }

    let proof = common.create_admin_proof(NUMBER_OF_ADMINS)?;

    common.fund_manager.increase_min_authorizers(
        proof,
        &mut common.env
    )?;

    for n in 1..=MIN_AUTHORIZERS {
        common.authorize_admin_operation(
            n,
            NUMBER_OF_ADMINS,
            DECREASE_MIN_AUTHORIZERS,
            None,
            None,
            None,
        )?;
    }

    let proof = common.create_admin_proof(NUMBER_OF_ADMINS)?;

    let result = common.fund_manager.decrease_min_authorizers(
        proof,
        &mut common.env
    );

    if result.is_ok() {
        return Err(
            RuntimeError::ApplicationError(
                PanicMessage("Min authorizers deos not really increse".to_string())
            )
        );
    }

    Ok(())
}
