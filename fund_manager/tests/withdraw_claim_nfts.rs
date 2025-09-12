mod common;
use common::*;
use scrypto_test::prelude::*;
use scrypto_test::prelude::ApplicationError::PanicMessage;

#[test]
fn test_withdraw_claim_nfts() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    common.create_and_add_defi_protocol(
        "protocol name".to_string(),
        XRD,
        None,
        100u8,
        true
    )?;

    let xrd_to_unstake = dec!(100);

    common.env.disable_auth_module();

    common.fund_manager.start_unlock_owner_stake_units(
        xrd_to_unstake,
        &mut common.env
    )?;

    let (_, claim_nft_id) = common.fund_manager.start_unstake(&mut common.env)?;

    for n in 1..=MIN_AUTHORIZERS {
        common.authorize_admin_operation(
            n,
            MIN_AUTHORIZERS + 1,
            WITHDRAW_CLAIM_NFTS,
            None,
            None,
            None,
        )?;
    }

    let proof = common.create_admin_proof(MIN_AUTHORIZERS + 1)?;

    let claim_nfts_bucket = common.fund_manager.withdraw_claim_nfts(
        proof,
        &mut common.env
    )?;

    // Check that the claim NFT is there
    let _ = claim_nfts_bucket.take_non_fungibles(
        indexset!(claim_nft_id),
        &mut common.env
    )?;

    Ok(())
}

#[test]
fn test_no_auth() -> Result<(), RuntimeError> {

    let mut common = Common::new().unwrap();

    let proof = common.create_admin_proof(MIN_AUTHORIZERS + 1)?;

    let result = common.fund_manager.withdraw_claim_nfts(
        proof,
        &mut common.env
    );

    if result.is_ok() {
        return Err(
            RuntimeError::ApplicationError(
                PanicMessage("Authorization bypassed in withdraw_claim_nfts".to_string())
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
            WITHDRAW_CLAIM_NFTS,
            None,
            None,
            None,
        )?;
    }

    // Wrong proof
    let proof = common.account_badge_bucket.create_proof_of_non_fungibles(
        indexset!(NonFungibleLocalId::Integer(u64::from(MIN_AUTHORIZERS + 1).into())),
        &mut common.env
    )?;

    let result = common.fund_manager.withdraw_claim_nfts(
        proof.into(),
        &mut common.env
    );

    if result.is_ok() {
        return Err(
            RuntimeError::ApplicationError(
                PanicMessage("Wrong proof accepted in withdraw_claim_nfts".to_string())
            )
        );
    }

    Ok(())
}

