use scrypto::prelude::*;
use crate::common::*;
use scrypto_interface::*;

#[derive(ScryptoSbor)]
pub struct StabilityPoolInfoReturn {
    pub collateral: ResourceAddress,
    pub payout_split: Option<Decimal>,
    pub liquidity_rewards_split: Option<Decimal>,
    pub stability_pool_split: Option<Decimal>,
    pub allow_pool_buys: bool,
    pub pool_buy_price_modifier: Option<Decimal>,
    pub liquidity_rewards: Decimal,
    pub pool: Global<TwoResourcePool>,
    pub collateral_amount: Decimal,
    pub fusd_amount: Decimal,
    pub latest_lowest_interests: Vec<Decimal>,
    pub last_lowest_interests_update: Instant,
}

#[blueprint_with_traits]
mod flux_wrapper {

    extern_blueprint! {
        "package_tdx_2_1p42tqez7qegpjgz26vnfjrc92vcuqx7ghwezu49qlh785qjz40y9t5",
        StabilityPools {
            fn contribute_to_pool(
                &mut self,
                collateral: ResourceAddress,
                contribution: Bucket,
                deposit_leftover: bool,
                message: String,
                signature: String,
            ) -> (Bucket, Option<FungibleBucket>, Option<Bucket>);

            fn withdraw_from_pool(
                &mut self,
                collateral: ResourceAddress,
                tokens: Bucket,
            ) -> (Bucket, Bucket);

            fn get_stability_pool_infos(
                &mut self,
                resource_addresses: Option<Vec<ResourceAddress>>
            ) -> Vec<StabilityPoolInfoReturn>;
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

    struct FluxWrapper {
        fusd_address: ResourceAddress,
        coin_address: ResourceAddress,
        token_vault: FungibleVault,
        component_address: Global<StabilityPools>,
        pool: Global<TwoResourcePool>,
    }

    impl FluxWrapper {

        pub fn new(
            fusd_address: ResourceAddress,
            coin_address: ResourceAddress, // Example coin: LSULP
            token_address: ResourceAddress, // Example token: lsulpFUSD
            mut component_address: Global<StabilityPools>,
            fund_manager_badge_address: ResourceAddress,
        ) -> Global<FluxWrapper> {
            Self {
                fusd_address: fusd_address,
                coin_address: coin_address,
                token_vault: FungibleVault::new(token_address),
                component_address: component_address,
                pool: component_address.get_stability_pool_infos(Some(vec![coin_address])).pop().unwrap().pool,
            }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .roles(roles!(
                    fund_manager => rule!(require(fund_manager_badge_address));
                ))
                .globalize()
        }

    }

    impl DefiProtocolInterfaceTrait for FluxWrapper {

        fn deposit_protocol_token(
            &mut self,
            token: Bucket,
        ) -> (Option<Decimal>, Option<Decimal>) {
            let token_amount = token.amount();

            self.token_vault.put(FungibleBucket(token));

            let amounts = self.pool.get_redemption_value(token_amount);

            (
                amounts.get(&self.fusd_address).copied(),
                amounts.get(&self.coin_address).copied(),
            )
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
            message: Option<String>,
            signature: Option<String>,
        ) {
            let (token_bucket, _, _) = self.component_address.contribute_to_pool(
                self.coin_address,
                coin.into(),
                true,
                message.expect("Message needed"),
                signature.expect("Signature needed"),
            );

            self.token_vault.put(FungibleBucket(token_bucket));
        }

        fn withdraw_coin(
            &mut self,
            amount: Option<Decimal>,
            other_coin_to_coin_price_ratio: Option<Decimal>,
        ) -> (FungibleBucket, Option<FungibleBucket>) {
            match amount {
                Some(amount) => {
                    let amounts = self.pool.get_redemption_value(Decimal::ONE);

                    // Amount of coins withdrawn by returning one pool unit
                    let fusd_amount = amounts.get(&self.fusd_address).unwrap_or(&Decimal::ZERO);
                    let coin_amount = amounts.get(&self.coin_address).unwrap_or(&Decimal::ZERO);

                    let mut token_amount = amount / (*fusd_amount + *coin_amount * other_coin_to_coin_price_ratio.unwrap());
                    if token_amount > self.token_vault.amount() {
                        token_amount = self.token_vault.amount();
                    }

                    let buckets = self.component_address.withdraw_from_pool(
                        self.coin_address,
                        self.token_vault.take(token_amount).into()
                    );

                    (FungibleBucket(buckets.0), Some(FungibleBucket(buckets.1)))
                },

                None => {
                    let buckets = self.component_address.withdraw_from_pool(
                        self.coin_address,
                        self.token_vault.take_all().into()
                    );

                    (FungibleBucket(buckets.0), Some(FungibleBucket(buckets.1)))
                },
            }
        }
    }
}
