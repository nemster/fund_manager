use scrypto::prelude::*;
use crate::common::*;
use scrypto_interface::*;

#[blueprint_with_traits]
mod weft_wrapper {

    extern_blueprint! {
        "package_tdx_2_1pk02rsgrec4dv3fhtw2ltmy3g80325wlusl76tjwhjpj48qtk8c80n",
        LendingPool {
            fn deposit(&mut self, buckets: Vec<FungibleBucket>) -> Vec<FungibleBucket>;
            fn withdraw(&mut self, buckets: Vec<FungibleBucket>) -> Vec<FungibleBucket>;
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

    struct WeftWrapper {
        minimum_coin_amount: Decimal,
        token_vault: FungibleVault, // TODO: Replacing token_vault with an Account would allow
                                    // us to receive WEFT incentives. Can we do that in 
                                    // another way?
        component_address: Global<LendingPool>,
        coin_token_ratio: Decimal,
    }

    impl WeftWrapper {

        pub fn new(
            coin_address: ResourceAddress, // Example coin: xUSDC
            token_address: ResourceAddress, // Example token: w2-xUSDC
            component_address: Global<LendingPool>,
            fund_manager_badge_address: ResourceAddress,
        ) -> Global<WeftWrapper> {
            let coin_divisibility = ResourceManager::from_address(coin_address)
                .resource_type()
                .divisibility()
                .unwrap();

            Self {
                minimum_coin_amount: Decimal::ONE / 10.pow(coin_divisibility),
                token_vault: FungibleVault::new(token_address),
                component_address: component_address,
                coin_token_ratio: Decimal::ONE,
            }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .roles(roles!(
                    fund_manager => rule!(require(fund_manager_badge_address));
                ))
                .globalize()
        }

    }

    impl DefiProtocolInterfaceTrait for WeftWrapper {

        fn deposit_protocol_token(
            &mut self,
            token: Bucket,
        ) -> (Option<Decimal>, Option<Decimal>) {
            let token_amount = token.amount();

            self.token_vault.put(FungibleBucket(token));

            (Some(self.coin_token_ratio * token_amount), None)
        }

        fn withdraw_protocol_token(
            &mut self,
            amount: Option<Decimal>,
        ) -> Bucket {
            match amount {
                None => self.token_vault.take_all().into(),
                Some(mut amount) => {
                    if amount > self.token_vault.amount() {
                        amount = self.token_vault.amount();
                    }

                    self.token_vault.take(amount).into()
                },
            }
        }

        fn deposit_coin(
            &mut self,
            coin: FungibleBucket,
            _other_coin: Option<FungibleBucket>,
        ) {
            let coin_amount =  coin.amount();

            let token_bucket = self.component_address.deposit(
                vec![coin]
            )
                .pop()
                .unwrap();

            let token_amount = token_bucket.amount();

            self.token_vault.put(token_bucket);

            self.coin_token_ratio = coin_amount / token_amount;
        }

        fn withdraw_coin(
            &mut self,
            amount: Option<Decimal>,
        ) -> (FungibleBucket, Option<FungibleBucket>) {
            match amount {
                Some(amount) => {
                    let token_amount = match amount / self.coin_token_ratio > self.token_vault.amount() {
                        true => self.token_vault.amount(),
                        false => amount / self.coin_token_ratio,
                    };

                    let mut coin_bucket = self.component_address.withdraw(
                        vec![self.token_vault.take(token_amount)]
                    )
                        .pop()
                        .unwrap();

                    let excess_amount = coin_bucket.amount() - amount;

                    if excess_amount > self.minimum_coin_amount {
                        let excess_coin_bucket = coin_bucket.take_advanced(
                            excess_amount,
                            WithdrawStrategy::Rounded(RoundingMode::ToZero),
                        );

                        let mut excess_token_buckets = self.component_address.deposit(
                            vec![excess_coin_bucket]
                        );

                        self.token_vault.put(
                            excess_token_buckets.pop().unwrap()
                        );
                    }

                    (coin_bucket, None)
                },

                None => (
                    self.component_address.withdraw(
                        vec![self.token_vault.take_all()]
                    )
                        .pop()
                        .unwrap(),
                    None
                ),
            }
        }
    }
}
