use scrypto::prelude::*;
use crate::common::*;
use scrypto_interface::*;

#[derive(ScryptoSbor)]
enum CDPType {
    Standard
}

#[derive(ScryptoSbor, NonFungibleData)]
struct CollaterizedDebtPositionData {
    key_image_url: String,
    name: String,
    description: String,
    minted_at: i64,
    updated_at: i64,
    cdp_type: CDPType,
    collaterals: IndexMap<ResourceAddress, PreciseDecimal>,
    loans: IndexMap<ResourceAddress, PreciseDecimal>,
    liquidable: Option<Decimal>
}

#[blueprint_with_traits]
mod root_finance_wrapper {

    extern_blueprint! {
        "package_tdx_2_1pk07dw324vwcvr75dk2p39yjt33jc6ndvd5zmn8p5k66a6zwjshdnr",
        LendingMarket {
            fn remove_collateral(&mut self, cdp_proof: Proof, withdraw_details: Vec<(ResourceAddress, Decimal, bool)>) -> Vec<Bucket>;
            fn add_collateral(&mut self, cdp_proof: Proof, deposits: Vec<Bucket>);
        }
    }

    enable_method_auth! {
        roles {
            fund_manager => updatable_by: [];
        },
        methods {
            deposit_protocol_token => restrict_to: [fund_manager];
            withdraw_protocol_token => restrict_to: [fund_manager];
            deposit_coin => restrict_to: [fund_manager];
            withdraw_coin => restrict_to: [fund_manager];
        }
    }

    struct RootFinanceWrapper {
        coin_address: ResourceAddress,
        token_vault: NonFungibleVault,
        component_address: Global<LendingMarket>,
    }

    impl RootFinanceWrapper {

        pub fn new(
            coin_address: ResourceAddress, // Example coin: xUSDC
            token_address: ResourceAddress, // Root receipt
            component_address: Global<LendingMarket>,
            fund_manager_badge_address: ResourceAddress,
        ) -> Global<RootFinanceWrapper> {
            Self {
                coin_address: coin_address,
                token_vault: NonFungibleVault::new(token_address),
                component_address: component_address,
            }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .roles(roles!(
                    fund_manager => rule!(require(fund_manager_badge_address));
                ))
                .globalize()
        }

    }

    impl DefiProtocolInterfaceTrait for RootFinanceWrapper {

        fn deposit_protocol_token(
            &mut self,
            token: Bucket,
        ) -> (Option<Decimal>, Option<Decimal>) {
            assert!(
                self.token_vault.amount() == Decimal::ZERO,
                "There's already a Root receipt in the vault",
            );

            let root_receipt = NonFungibleBucket(token);

            let non_fungible_data = root_receipt.non_fungible::<CollaterizedDebtPositionData>().data();

            self.token_vault.put(root_receipt);

            let number_of_coins = non_fungible_data.collaterals.len();

            if number_of_coins == 0 {
                (Some(Decimal::ZERO), None)

            } else if number_of_coins == 1 {
                let (address, amount) = non_fungible_data.collaterals.get_index(0).unwrap();

                assert!(
                    *address == self.coin_address,
                    "This Root receipt contains a different coin from the one managed by this wrapper"
                );

                (amount.checked_truncate(RoundingMode::ToNegativeInfinity), None)

            } else {
                Runtime::panic("Multiple coins in the Root receipt".to_string());
            }
        }

        fn withdraw_protocol_token(
            &mut self,
            _amount: Option<Decimal>,
        ) -> Bucket {
            self.token_vault.take_all().into()
        }

        fn deposit_coin(
            &mut self,
            coin: FungibleBucket,
            _other_coin: Option<FungibleBucket>,
        ) {
            let proof = self.token_vault.create_proof_of_non_fungibles(
                &self.token_vault.non_fungible_local_ids(1)
            );

            self.component_address.add_collateral(
                proof.into(),
                vec![coin.into()],
            );
        }

        fn withdraw_coin(
            &mut self,
            mut amount: Option<Decimal>,
        ) -> (FungibleBucket, Option<FungibleBucket>) {
            let proof = self.token_vault.create_proof_of_non_fungibles(
                &self.token_vault.non_fungible_local_ids(1)
            );

            if amount.is_none() {
                let non_fungible_data = self.token_vault.non_fungible::<CollaterizedDebtPositionData>().data();
                amount = non_fungible_data.collaterals.get_index(0)
                    .expect("No coins in this Root receipt")
                    .1
                    .checked_truncate(RoundingMode::ToNegativeInfinity);
            }

            let coin_bucket = self.component_address.remove_collateral(
                proof.into(),
                vec![(
                    self.coin_address,
                    amount.unwrap(),
                    false
                )]
            )
                .pop()
                .unwrap();

            (FungibleBucket(coin_bucket), None)
        }
    }
}
