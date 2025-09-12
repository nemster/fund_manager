mod common;
use common::*;
use scrypto_test::prelude::*;
use scrypto_test::prelude::ApplicationError::PanicMessage;

#[test]
fn test_authorize_self() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    let result = common.authorize_admin_operation(
        1u8,
        1u8,
        WITHDRAW_VALIDATOR_BADGE,
        None,
        None,
        None,
    );

    if result.is_ok() {
        return Err(RuntimeError::ApplicationError(
            PanicMessage("Admin authorized himself".to_string())
        ));
    }

    Ok(())
}

#[test]
fn test_non_existent_admin() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    let result = common.authorize_admin_operation(
        1u8,
        NUMBER_OF_ADMINS + 1,
        WITHDRAW_VALIDATOR_BADGE,
        None,
        None,
        None,
    );

    if result.is_ok() {
        return Err(RuntimeError::ApplicationError(
            PanicMessage("Admin authorized a non existent admin".to_string())
        ));
    }

    Ok(())
}

#[test]
fn test_non_existent_operation() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    let result = common.authorize_admin_operation(
        1u8,
        2u8,
        MAX_OPERATION + 1,
        None,
        None,
        None,
    );

    if result.is_ok() {
        return Err(RuntimeError::ApplicationError(
            PanicMessage("Admin authorized a non existent operation".to_string())
        ));
    }

    Ok(())
}

#[test]
fn test_wrong_proof() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    // Wrong proof
    let proof = common.account_badge_bucket.create_proof_of_non_fungibles(
        indexset!(NonFungibleLocalId::Integer(1u64.into())),
        &mut common.env
    )?;

    let result = common.fund_manager.authorize_admin_operation(
        proof.into(),
        2u8,
        WITHDRAW_VALIDATOR_BADGE,
        None,
        None,
        None,
        &mut common.env
    );

    if result.is_ok() {
        return Err(RuntimeError::ApplicationError(
            PanicMessage("Wrong badge accepted".to_string())
        ));
    }

    Ok(())
}

#[test]
fn test_multiple_authorizations() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    common.authorize_admin_operation(
        1u8,
        2u8,
        WITHDRAW_VALIDATOR_BADGE,
        None,
        None,
        None,
    )?;

    // Same auth as the previous one
    let result = common.authorize_admin_operation(
        1u8,
        2u8,
        WITHDRAW_VALIDATOR_BADGE,
        None,
        None,
        None,
    );

    if result.is_ok() {
        return Err(RuntimeError::ApplicationError(
            PanicMessage("Duplicate authorization accepted".to_string())
        ));
    }

    Ok(())
}

#[test]
// Check that the maximum vector size (50) is respected
fn test_vector_size() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    let max_vector_size = 50u8;

    let mut authorizations: u8 = 0;

    for authorizer in 1..=NUMBER_OF_ADMINS {
        for authorized in 1..=NUMBER_OF_ADMINS {
            if authorizer != authorized {
                for operation in 0..MAX_OPERATION {
                    let result = common.authorize_admin_operation(
                        authorizer,
                        authorized,
                        operation,
                        None,
                        None,
                        None,
                    );

                    authorizations += 1;

                    if result.is_err() {
                        if authorizations <= max_vector_size {
                            return Err(RuntimeError::ApplicationError(
                                PanicMessage("Failed authorization number ".to_string() + &authorizations.to_string())
                            ));
                        }
                    } else {
                        if authorizations > max_vector_size {
                            return Err(RuntimeError::ApplicationError(
                                PanicMessage("Succeded authorization number ".to_string() + &authorizations.to_string())
                            ));
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

#[test]
fn test_wrong_admin() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    for n in 1..=MIN_AUTHORIZERS {
        common.authorize_admin_operation(
            n,
            MIN_AUTHORIZERS + 1,
            WITHDRAW_VALIDATOR_BADGE,
            None,
            None,
            None,
        )?;
    }

    // Not the authorized admin
    let proof = common.create_admin_proof(1u8)?;

    let result = common.fund_manager.withdraw_validator_badge(
        proof,
        &mut common.env
    );

    if result.is_ok() {
        return Err(
            RuntimeError::ApplicationError(
                PanicMessage("Validator badge withdrawn by an unauthorized admin".to_string())
            )
        );
    }

    Ok(())
}
