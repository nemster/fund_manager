#![allow(dead_code)]

use scrypto_test::prelude::*;
use dummy_validator::dummy_validator::dummy_validator_test::*;
use fund_manager::fund_manager::fund_manager_test::*;
use dummy_dex_and_oracle::dummy_dex_and_oracle::dummy_dex_and_oracle_test::*;
use dummy_defi_protocol::dummy_defi_protocol::dummy_defi_protocol_test::*;

pub static BUYBACK_FUND_PERCENTAGE: u8 = 20;
pub static WITHDRAWAL_FEE_PERCENTAGE: u8 = 20;
pub static NUMBER_OF_ADMINS: u8 = 3;
pub static MIN_AUTHORIZERS: u8 = 1;
pub static FUND_UNIT_INITIAL_SUPPLY: Decimal = dec!(100);

pub static XRD_PRICE: Decimal = Decimal::ONE;
pub static A_PRICE: Decimal = dec!(2);
pub static B_PRICE: Decimal = Decimal::ONE;
pub static C_PRICE: Decimal = dec!("0.5");

pub static WITHDRAW_VALIDATOR_BADGE: u8 = 0;
pub static ADD_DEFI_PROTOCOL: u8 = 1;
pub static REMOVE_DEFI_PROTOCOL: u8 = 2;
pub static SET_DEX_COMPONENT: u8 = 3;
pub static DECREASE_MIN_AUTHORIZERS: u8 = 4;
pub static INCREASE_MIN_AUTHORIZERS: u8 = 5;
pub static MINT_ADMIN_BADGE: u8 = 6;
pub static SET_ORACLE_COMPONENT: u8 = 7;
pub static WITHDRAW_FUND_MANAGER_BADGE: u8 = 8;
pub static SET_WITHDRAWAL_FEE: u8 = 9;
pub static MINT_BOT_BADGE: u8 = 10;
pub static SET_BUYBACK_FUND: u8 = 11;
pub static WITHDRAW_CLAIM_NFTS: u8 = 12;
pub static MAX_OPERATION: u8 = 12;

#[derive(ScryptoSbor, NonFungibleData)]
pub struct Empty {
}

pub struct Common {
    pub env: TestEnvironment<InMemorySubstateDatabase>,

    pub validator: DummyValidator,
    pub validator_owner_badge_address: ResourceAddress,
    pub lsu_address: ResourceAddress,
    pub claim_nft_address: ResourceAddress,

    pub account: Reference,

    pub fund_manager: FundManager,
    pub fund_manager_badge_address: ResourceAddress,
    pub admin_badges: NonFungibleBucket,
    pub fund_units: FungibleBucket,
    pub bot_badge_address: ResourceAddress,

    pub dex_and_oracle_package: PackageAddress,
    pub dex_and_oracle: DummyDexAndOracle,

    pub token_bucket: Bucket,
    pub a_bucket: FungibleBucket,
    pub b_bucket: FungibleBucket,
    pub c_bucket: FungibleBucket,
    pub account_badge_bucket: NonFungibleBucket,

    pub defi_protocol_package: PackageAddress,
}

impl Common {
    pub fn new() -> Result<Self, RuntimeError> {

        // Create a DummyValidator and initalize it with 1000000 XRD
        let mut env = TestEnvironmentBuilder::new()
            .build();

        let xrd_bucket = BucketFactory::create_fungible_bucket(
            XRD,
            dec![1000000],
            Mock,
            &mut env
        )?;
        let dummy_validator_package = PackageFactory::compile_and_publish(
            "../validator/",
            &mut env,
            CompileProfile::Standard,
        )?;
        let (
            dummy_validator,
            validator_owner_badge_bucket,
            lsu_address,
            claim_nft_address
        ) = DummyValidator::new(
            xrd_bucket,
            dummy_validator_package,
            &mut env
        )?;
        let validator_owner_badge_address = validator_owner_badge_bucket.resource_address(&mut env)?;

        // Create the buyback Account
        let account = env.call_function_typed::<_, AccountCreateOutput>(
                ACCOUNT_PACKAGE,
                ACCOUNT_BLUEPRINT,
                ACCOUNT_CREATE_IDENT,
                &AccountCreateInput {},
            )?
            .0
            .0;

        // Create the FundManager component
        let fund_manager_package = PackageFactory::compile_and_publish(
            this_package!(),
            &mut env,
            CompileProfile::Standard,
        )?;
        let (
            mut fund_manager,
            fund_manager_badge_address,
            admin_badges,
            fund_units,
            bot_badge_address,
        ) = FundManager::new(
            dummy_validator.into(),
            validator_owner_badge_bucket.resource_address(&mut env)?,
            claim_nft_address,
            WITHDRAWAL_FEE_PERCENTAGE,
            BUYBACK_FUND_PERCENTAGE,
            account.into(),
            NUMBER_OF_ADMINS,
            MIN_AUTHORIZERS,
            FUND_UNIT_INITIAL_SUPPLY,
            fund_manager_package,
            &mut env,
        )?;

        // Deposit the validator owner badge in the FundManager component
        env.disable_auth_module();
        fund_manager.deposit_validator_badge(
            validator_owner_badge_bucket,
            &mut env,
        )?;
        env.enable_auth_module();

        // Create the DummyDexAndOracle component
        let dummy_dex_and_oracle_package = PackageFactory::compile_and_publish(
            "../dex_and_oracle/",
            &mut env,
            CompileProfile::Standard,
        )?;
        let mut dummy_dex_and_oracle = DummyDexAndOracle::new(
            fund_manager_badge_address,
            admin_badges.resource_address(&mut env)?,
            dummy_dex_and_oracle_package,
            &mut env
        )?;

        // Deposit 1000000 XRD in the dex
        let xrd_bucket = BucketFactory::create_fungible_bucket(
            XRD,
            dec![1000000],
            Mock,
            &mut env
        )?;
        dummy_dex_and_oracle.deposit(
            xrd_bucket.into(),
            &mut env
        )?;

        // Create 100 dummy protocol tokens
        let token_bucket = ResourceBuilder::new_fungible(OwnerRole::None)
            .divisibility(18)
            .mint_initial_supply(100, &mut env)?;

        // Create coin A, deposit 1000000 A in the dex and set its price
        let a_bucket = ResourceBuilder::new_fungible(OwnerRole::None)
            .divisibility(18)
            .burn_roles(Some(BurnRoles {
                burner: Some(AccessRule::AllowAll),
                burner_updater: Some(AccessRule::DenyAll),
            }))
            .mint_initial_supply(2000000, &mut env)?;
        dummy_dex_and_oracle.deposit(
            a_bucket.take(dec!(1000000), &mut env)?.into(),
            &mut env
        )?;
        dummy_dex_and_oracle.set_price(
            a_bucket.resource_address(&mut env)?,
            A_PRICE,
            &mut env
        )?;

        // Create coin B, deposit 1000000 B in the dex and set its price
        let b_bucket = ResourceBuilder::new_fungible(OwnerRole::None)
            .divisibility(18)
            .burn_roles(Some(BurnRoles {
                burner: Some(AccessRule::AllowAll),
                burner_updater: Some(AccessRule::DenyAll),
            }))
            .mint_initial_supply(2000000, &mut env)?;
        dummy_dex_and_oracle.deposit(
            b_bucket.take(dec!(1000000), &mut env)?.into(),
            &mut env
        )?;
        dummy_dex_and_oracle.set_price(
            b_bucket.resource_address(&mut env)?,
            B_PRICE,
            &mut env
        )?;

        // Create coin C, deposit 1000000 C in the dex and set its price
        let c_bucket = ResourceBuilder::new_fungible(OwnerRole::None)
            .divisibility(18)
            .mint_initial_supply(2000000, &mut env)?;
        dummy_dex_and_oracle.deposit(
            c_bucket.take(dec!(1000000), &mut env)?.into(),
            &mut env
        )?;
        dummy_dex_and_oracle.set_price(
            c_bucket.resource_address(&mut env)?,
            C_PRICE,
            &mut env
        )?;

        // Create 100 dummy account badges
        let mut account_badges_specification = vec![];
        for n in 1u64..=100 {
            account_badges_specification.push(
                (IntegerNonFungibleLocalId::from(n), Empty {})
            );
        }
        let account_badge_bucket = ResourceBuilder::new_integer_non_fungible::<Empty>(OwnerRole::None)
            .mint_initial_supply(account_badges_specification, &mut env)?;

        // Create the DummyDefiProtocol package
        let dummy_defi_protocol_package = PackageFactory::compile_and_publish(
            "../defi_protocols/dummy/",
            &mut env,
            CompileProfile::Standard,
        )?;

        let mut common = Self {
            env: env,
            validator: dummy_validator,
            validator_owner_badge_address: validator_owner_badge_address,
            lsu_address: lsu_address,
            claim_nft_address: claim_nft_address,
            account: account.into(),
            fund_manager: fund_manager,
            fund_manager_badge_address: fund_manager_badge_address,
            admin_badges: admin_badges,
            fund_units: fund_units,
            bot_badge_address: bot_badge_address,
            dex_and_oracle_package: dummy_dex_and_oracle_package,
            dex_and_oracle: dummy_dex_and_oracle,
            token_bucket: token_bucket.into(),
            a_bucket: a_bucket,
            b_bucket: b_bucket,
            c_bucket: c_bucket,
            account_badge_bucket: account_badge_bucket,
            defi_protocol_package: dummy_defi_protocol_package,
        };

        // Set the DummyDexAndOracle component as the dex to be used by the FundManager component
        for n in 1..=MIN_AUTHORIZERS {
            common.authorize_admin_operation(
                n,
                MIN_AUTHORIZERS + 1,
                SET_DEX_COMPONENT,
                None,
                None,
                None,
            )?;
        }
        let proof = common.create_admin_proof(MIN_AUTHORIZERS + 1)?;
        fund_manager.set_dex_component(
            proof,
            common.dex_and_oracle.into(),
            &mut common.env
        )?;

        // Set the DummyDexAndOracle component as the oracle to be used by the FundManager component
        for n in 1..=MIN_AUTHORIZERS {
            common.authorize_admin_operation(
                n,
                MIN_AUTHORIZERS + 1,
                SET_ORACLE_COMPONENT,
                None,
                None,
                None
            )?;
        }
        let proof = common.create_admin_proof(MIN_AUTHORIZERS + 1)?;
        common.fund_manager.set_oracle_component(
            proof,
            common.dex_and_oracle.into(),
            &mut common.env
        )?;

        // Mint a bot badge and send it to the account
        for n in 1..=MIN_AUTHORIZERS {
            common.authorize_admin_operation(
                n,
                MIN_AUTHORIZERS + 1,
                MINT_BOT_BADGE,
                None,
                None,
                Some(common.account)
            )?;
        }
        let proof = common.create_admin_proof(MIN_AUTHORIZERS + 1)?;
        common.fund_manager.mint_bot_badge(
            proof,
            common.account,
            &mut common.env
        )?;

        Ok(common)
    }

    pub fn create_admin_proof(
        &mut self,
        admin_id: u8,
    ) -> Result<Proof, RuntimeError> {

        // Create an IndexSet containig the id of the admin badge
        let mut ids = IndexSet::<NonFungibleLocalId>::new();
        ids.insert(NonFungibleLocalId::Integer(u64::from(admin_id).into()));
        
        // Create the requested proof
        let proof = self.admin_badges.create_proof_of_non_fungibles(
            ids,
            &mut self.env
        )?;

        Ok(proof.into())
    }

    pub fn authorize_admin_operation(
        &mut self,
        allower_admin_id: u8,
        allowed_admin_id: u8,
        authorized_operation: u8,
        protocol_name: Option<String>,
        percentage: Option<u8>,
        account_address: Option<Reference>,
    ) -> Result<(), RuntimeError> {

        // Create a proof for the allower
        let proof = self.create_admin_proof(allower_admin_id)?;

        // Authorize the specified operation
        self.fund_manager.authorize_admin_operation(
            proof,
            allowed_admin_id,
            authorized_operation,
            protocol_name,
            percentage,
            account_address,
            &mut self.env
        )?;

        Ok(())
    }

    pub fn create_defi_protocol(
        &mut self,
        coin: ResourceAddress,
        other_coin: Option<ResourceAddress>,
    ) -> Result<DummyDefiProtocol, RuntimeError> {

        // Take one NFT out of account_badge_bucket
        let mut account_badge_local_id = self.account_badge_bucket
            .non_fungible_local_ids(&mut self.env)?;
        account_badge_local_id.truncate(1);
        let account_badge_bucket = self.account_badge_bucket.take_non_fungibles(
            account_badge_local_id,
            &mut self.env
        )?;

        // Instantiate a DummyDefiProtocol component
        let dummy_defi_protocol = DummyDefiProtocol::new(
            self.token_bucket.resource_address(&mut self.env)?,
            coin,
            other_coin,
            account_badge_bucket,
            self.fund_manager_badge_address,
            self.admin_badges.resource_address(&mut self.env)?,
            self.defi_protocol_package,
            &mut self.env
        )?;

        Ok(dummy_defi_protocol)
    }

    pub fn create_and_add_defi_protocol(
        &mut self,
        protocol_name: String,
        coin: ResourceAddress,
        other_coin: Option<ResourceAddress>,
        desired_percentage: u8,
        allow_other_coin_input: bool,
    ) -> Result<(), RuntimeError> {

        // Create the DummyDefiProtocol component
        let defi_protocol = self.create_defi_protocol(
            coin,
            other_coin
        )?;

        // Make the requested number of authorization for the add_defi_protocol operation
        for n in 1..=MIN_AUTHORIZERS {
            self.authorize_admin_operation(
                n,
                MIN_AUTHORIZERS + 1,
                ADD_DEFI_PROTOCOL,
                Some(protocol_name.clone()),
                None,
                None,
            )?;
        }

        // Create the proof for the authorized admin
        let proof = self.create_admin_proof(MIN_AUTHORIZERS + 1)?;

        // Register the new defi protocol in the FundManager component
        self.fund_manager.add_defi_protocol(
            proof,
            protocol_name,
            coin,
            other_coin,
            desired_percentage,
            defi_protocol.into(),
            None,
            allow_other_coin_input,
            &mut self.env
        )?;

        Ok(())
    }

}
