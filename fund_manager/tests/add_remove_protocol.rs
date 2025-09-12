mod common;
use common::*;
use scrypto_test::prelude::*;
use scrypto_test::prelude::ApplicationError::PanicMessage;

#[test]
fn test_add_remove_protocol() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    let protocol_name = "My protocol".to_string();
    let a_address = common.a_bucket.resource_address(&mut common.env)?;
    let a_amount = common.a_bucket.amount(&mut common.env)?;
    common.create_and_add_defi_protocol(
        protocol_name.clone(),
        a_address,
        None,
        100u8,
        true
    )?;

    common.env.disable_auth_module();
    common.fund_manager.deposit_coin(
        protocol_name.clone(),
        common.a_bucket.take(a_amount, &mut common.env)?,
        None,
        HashMap::new(),
        false,
        &mut common.env
    )?;
    common.env.enable_auth_module();

    // The protocol with the same name replaces the old one and receives its liquidity but it also
    // supports B coin deposits
    let b_address = common.b_bucket.resource_address(&mut common.env)?;
    let b_amount = common.b_bucket.amount(&mut common.env)?;
    common.create_and_add_defi_protocol(
        protocol_name.clone(),
        a_address,
        Some(b_address),
        100u8,
        true
    )?;

    common.env.disable_auth_module();
    common.fund_manager.deposit_coin(
        protocol_name.clone(),
        common.a_bucket.take(Decimal::ZERO, &mut common.env)?,
        Some(common.b_bucket.take(b_amount, &mut common.env)?),
        HashMap::new(),
        false,
        &mut common.env
    )?;
    common.env.enable_auth_module();


    // Check that both the deposits went into the new protocol
    let protocols = common.fund_manager.fund_details(&mut common.env)?;
    let protocol_value = protocols.get(&protocol_name).expect("Protocol not found");
    let expected_protocol_value = a_amount * A_PRICE + b_amount * B_PRICE;
    assert!(
        *protocol_value == expected_protocol_value,
        "A protocol value: {}, should be {}",
        protocol_value,
        expected_protocol_value
    );

    for n in 1..=MIN_AUTHORIZERS {
        common.authorize_admin_operation(
            n,
            NUMBER_OF_ADMINS,
            REMOVE_DEFI_PROTOCOL,
            Some(protocol_name.clone()),
            None,
            None,
        )?;
    }

    let proof = common.create_admin_proof(NUMBER_OF_ADMINS)?;

    common.fund_manager.remove_defi_protocol(
        proof,
        protocol_name.clone(),
        false,
        &mut common.env
    )?;

    let protocols = common.fund_manager.fund_details(&mut common.env)?;
    assert!(
        protocols.len() == 0,
        "Protocol has not been removed"
    );

    Ok(())
}

#[test]
fn test_no_auth1() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    let protocol = common.create_defi_protocol(
        XRD,
        None
    )?;

    let proof = common.create_admin_proof(1u8)?;

    let result = common.fund_manager.add_defi_protocol(
        proof,
        "protocol name".to_string(),
        XRD,
        None,
        100u8,
        protocol.into(),
        None,
        false,
        &mut common.env
    );

    if result.is_ok() {
        return Err(
            RuntimeError::ApplicationError(
                PanicMessage("Authorization bypassed in add_defi_protocol".to_string())
            )
        );
    }

    Ok(())
}

#[test]
fn test_no_auth2() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    let protocol_name = "My protocol".to_string();
    let a_address = common.a_bucket.resource_address(&mut common.env)?;
    common.create_and_add_defi_protocol(
        protocol_name.clone(),
        a_address,
        None,
        100u8,
        true
    )?;

    let proof = common.create_admin_proof(1u8)?;

    let result = common.fund_manager.remove_defi_protocol(
        proof,
        protocol_name,
        false,
        &mut common.env
    );

    if result.is_ok() {
        return Err(
            RuntimeError::ApplicationError(
                PanicMessage("Authorization bypassed in remove_defi_protocol".to_string())
            )
        );
    }

    Ok(())
}

#[test]
fn test_wrong_auth1() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    let protocol = common.create_defi_protocol(
        XRD,
        None
    )?;

    let protocol_name = "My protocol".to_string();

    // Protocol name is missing
    for n in 1..=MIN_AUTHORIZERS {
        common.authorize_admin_operation(
            n,
            MIN_AUTHORIZERS + 1,
            ADD_DEFI_PROTOCOL,
            None,
            None,
            None,
        )?;
    }

    let proof = common.create_admin_proof(MIN_AUTHORIZERS + 1)?;

    let result = common.fund_manager.add_defi_protocol(
        proof,
        protocol_name,
        XRD,
        None,
        100u8,
        protocol.into(),
        None,
        false,
        &mut common.env
    );

    if result.is_ok() {
        return Err(
            RuntimeError::ApplicationError(
                PanicMessage("Incomplete authorization accepted in add_defi_protocol".to_string())
            )
        );
    }

    Ok(())
}

#[test]
fn test_wrong_proof1() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    let protocol = common.create_defi_protocol(
        XRD,
        None
    )?;

    let protocol_name = "My protocol".to_string();

    for n in 1..=MIN_AUTHORIZERS {
        common.authorize_admin_operation(
            n,
            MIN_AUTHORIZERS + 1,
            ADD_DEFI_PROTOCOL,
            Some(protocol_name.clone()),
            None,
            None,
        )?;
    }

    // Wrong proof
    let proof = common.account_badge_bucket.create_proof_of_non_fungibles(
        indexset!(NonFungibleLocalId::Integer(u64::from(MIN_AUTHORIZERS + 1).into())),
        &mut common.env
    )?;

    let result = common.fund_manager.add_defi_protocol(
        proof.into(),
        protocol_name,
        XRD,
        None,
        100u8,
        protocol.into(),
        None,
        false,
        &mut common.env
    );

    if result.is_ok() {
        return Err(
            RuntimeError::ApplicationError(
                PanicMessage("Wrong proof accepted in add_defi_protocol".to_string())
            )
        );
    }

    Ok(())
}

#[test]
fn test_wrong_auth2() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    let protocol_name = "My protocol".to_string();
    let a_address = common.a_bucket.resource_address(&mut common.env)?;
    common.create_and_add_defi_protocol(
        protocol_name.clone(),
        a_address,
        None,
        100u8,
        true
    )?;

    // Protocol name is missing
    for n in 1..=MIN_AUTHORIZERS {
        common.authorize_admin_operation(
            n,
            MIN_AUTHORIZERS + 1,
            REMOVE_DEFI_PROTOCOL,
            None,
            None,
            None,
        )?;
    }

    let proof = common.create_admin_proof(MIN_AUTHORIZERS + 1)?;

    let result = common.fund_manager.remove_defi_protocol(
        proof.into(),
        protocol_name,
        false,
        &mut common.env
    );

    if result.is_ok() {
        return Err(
            RuntimeError::ApplicationError(
                PanicMessage("Incomplete authorization accepted in remove_defi_protocol".to_string())
            )
        );
    }

    Ok(())
}

#[test]
fn test_wrong_proof2() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    let protocol_name = "My protocol".to_string();
    let a_address = common.a_bucket.resource_address(&mut common.env)?;
    common.create_and_add_defi_protocol(
        protocol_name.clone(),
        a_address,
        None,
        100u8,
        true
    )?;

    for n in 1..=MIN_AUTHORIZERS {
        common.authorize_admin_operation(
            n,
            MIN_AUTHORIZERS + 1,
            REMOVE_DEFI_PROTOCOL,
            Some(protocol_name.clone()),
            None,
            None,
        )?;
    }   

    // Wrong proof
    let proof = common.account_badge_bucket.create_proof_of_non_fungibles(
        indexset!(NonFungibleLocalId::Integer(u64::from(MIN_AUTHORIZERS + 1).into())),
        &mut common.env
    )?;

    let result = common.fund_manager.remove_defi_protocol(
        proof.into(),
        protocol_name,
        false,
        &mut common.env
    );

    if result.is_ok() {
        return Err(
            RuntimeError::ApplicationError(
                PanicMessage("Wrong proof accepted in remove_defi_protocol".to_string())
            )
        );
    }

    Ok(())
}

#[test]
fn test_remove_non_existing_protocol() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    let protocol_name = "My protocol".to_string();
    common.create_and_add_defi_protocol(
        protocol_name,
        XRD,
        None,
        100u8,
        true
    )?; 

    // Different protocol name
    let other_protocol_name = "Another protocol".to_string();
    for n in 1..=MIN_AUTHORIZERS {
        common.authorize_admin_operation(
            n,
            MIN_AUTHORIZERS + 1,
            REMOVE_DEFI_PROTOCOL,
            Some(other_protocol_name.clone()),
            None,
            None,
        )?;
    }

    let proof = common.create_admin_proof(MIN_AUTHORIZERS + 1)?;

    let result = common.fund_manager.remove_defi_protocol(
        proof, 
        other_protocol_name,
        false,
        &mut common.env
    );

    if result.is_ok() {
        return Err(
            RuntimeError::ApplicationError(
                PanicMessage("Successfully removed non existent protocol".to_string())
            )
        );
    }

    Ok(())
}

