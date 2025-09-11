use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData)]
pub struct Owner {
}

#[derive(ScryptoSbor, NonFungibleData)]
struct ClaimNft {
    amount: Decimal,
}

#[blueprint]
mod dummy_validator {

    enable_method_auth! {
        methods {
            start_unlock_owner_stake_units => restrict_to: [OWNER];
            finish_unlock_owner_stake_units => restrict_to: [OWNER];
            unstake => PUBLIC;
            claim_xrd => PUBLIC;
            register => restrict_to: [OWNER];
            unregister => restrict_to: [OWNER];
            signal_protocol_update_readiness => restrict_to: [OWNER];
            update_key => restrict_to: [OWNER];
        }
    }

    struct DummyValidator {
        xrd_vault: FungibleVault,
        unlocking: Vec<Decimal>,
        total_unstaking: Decimal,
        lsu_resource_manager: FungibleResourceManager,
        claim_nft_resource_manager: NonFungibleResourceManager,
        last_claim_nft_id: u64,
    }

    impl DummyValidator {

        pub fn new(
            xrd_bucket: FungibleBucket,
        ) -> (
            Global<DummyValidator>,
            NonFungibleBucket,
            ResourceAddress,
            ResourceAddress,
        ) {

            let (address_reservation, component_address) =
                Runtime::allocate_component_address(DummyValidator::blueprint_id());

            let owner_badge_bucket = ResourceBuilder::new_integer_non_fungible::<Owner>(
                OwnerRole::None
            )
                .mint_roles(mint_roles!(
                    minter => rule!(require(global_caller(component_address)));
                    minter_updater => rule!(deny_all);
                ))
                .mint_initial_supply(
                    vec![(IntegerNonFungibleLocalId::from(1u64), Owner {})]
                );
            let owner_badge_address = owner_badge_bucket.resource_address();

            let lsu_resource_manager = ResourceBuilder::new_fungible(OwnerRole::None)
                .mint_roles(mint_roles!(
                    minter => rule!(require(global_caller(component_address)));
                    minter_updater => rule!(deny_all);
                ))
                .burn_roles(burn_roles!(
                    burner => rule!(require(global_caller(component_address)));
                    burner_updater => rule!(deny_all);
                ))
                .create_with_no_initial_supply();
            let lsu_address = lsu_resource_manager.address();

            let claim_nft_resource_manager = ResourceBuilder::new_string_non_fungible::<ClaimNft>(
                OwnerRole::Fixed(AccessRule::DenyAll)
            )
                .mint_roles(mint_roles!(
                    minter => rule!(require(global_caller(component_address)));
                    minter_updater => rule!(deny_all);
                ))
                .burn_roles(burn_roles!(
                    burner => rule!(require(global_caller(component_address)));
                    burner_updater => rule!(deny_all);
                ))
                .create_with_no_initial_supply();
            let claim_nft_address = claim_nft_resource_manager.address();

            let dummy_validator = Self {
                xrd_vault: FungibleVault::with_bucket(xrd_bucket),
                unlocking: vec![],
                total_unstaking: Decimal::ZERO,
                lsu_resource_manager: lsu_resource_manager,
                claim_nft_resource_manager: claim_nft_resource_manager,
                last_claim_nft_id: 0u64,
            }
                .instantiate()
                .prepare_to_globalize(OwnerRole::Fixed(rule!(require(owner_badge_address))))
                .with_address(address_reservation)
                .globalize();

            (
                dummy_validator,
                owner_badge_bucket,
                lsu_address,
                claim_nft_address
            ) 
        }

        pub fn start_unlock_owner_stake_units(
            &mut self,
            amount: Decimal,
        ) {
            assert!(
                self.xrd_vault.amount() - self.total_unstaking >= amount,
                "not enough owner LSU to unlock",
            );

            self.unlocking.push(amount);
            self.total_unstaking += amount;
        }

        pub fn finish_unlock_owner_stake_units(&mut self) -> FungibleBucket {
            let amount = self.unlocking.remove(0);

            self.lsu_resource_manager.mint(amount)
        }

        pub fn unstake(
            &mut self,
            lsu_bucket: FungibleBucket,
        ) -> NonFungibleBucket {
            assert!(
                lsu_bucket.resource_address() == self.lsu_resource_manager.address(),
                "wrong LSU",
            );

            let amount = lsu_bucket.amount();

            lsu_bucket.burn();

            self.last_claim_nft_id += 1;
            self.claim_nft_resource_manager.mint_non_fungible(
                &NonFungibleLocalId::String(
                    StringNonFungibleLocalId::try_from(
                        self.last_claim_nft_id.to_string()
                    )
                        .unwrap()
                ),
                ClaimNft {amount: amount},
            )
        }

        pub fn claim_xrd(
            &mut self,
            claim_nft_bucket: NonFungibleBucket,
        ) -> FungibleBucket {
            assert!(
                claim_nft_bucket.resource_address() == self.claim_nft_resource_manager.address(),
                "wrong Claim NFT",
            );

            let mut amount = Decimal::ZERO;
            for nft in claim_nft_bucket.non_fungibles::<ClaimNft>() {
                amount += nft.data().amount;
            }

            self.total_unstaking -= amount;

            claim_nft_bucket.burn();

            self.xrd_vault.take(amount)
        }

        pub fn register(&self) {
        }

        pub fn unregister(&self) {
        }

        pub fn signal_protocol_update_readiness(
            &self,
            _vote: String,
        ) {
        }

        pub fn update_key(
            &self,
            _key: Secp256k1PublicKey,
        ) {
        }

    }
}
