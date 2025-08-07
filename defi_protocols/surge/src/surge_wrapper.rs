use scrypto::prelude::*;
use crate::common::*;
use scrypto_interface::*;

#[derive(ScryptoSbor)]
struct PoolDetails {
    base_tokens_amount: Decimal,
    virtual_balance: Decimal,
    unrealized_pool_funding: Decimal,
    pnl_snap: Decimal,
    skew_ratio: Decimal,
    skew_ratio_cap: Decimal,
    lp_supply: Decimal,
    lp_price: Decimal,
}

#[blueprint_with_traits]
mod surge_wrapper {

    extern_blueprint! {
        "package_tdx_2_1pk6dp0yek7ctv4evkhk78lc2af8ha8wd70ntkxva49cres0nl0pd2x",
        Exchange {
            fn add_liquidity(&self, payment: Bucket) -> Bucket;
            fn remove_liquidity(&self, lp_token: Bucket) -> Bucket;
            fn get_pool_details(&self) -> PoolDetails;
        }
    }

    extern_blueprint! {
        "package_tdx_2_1pkddk9u36afsazvfad3af09gvs0l5kmk560v9n9ejj5z99x35scnyn",
        TokenWrapper {
            fn wrap(&mut self, child_token: Bucket) -> Bucket;
            fn unwrap(&mut self, parent_token: Bucket, child_resource: ResourceAddress) -> Bucket;
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

            get_coin_amounts => PUBLIC;
        }
    }

    struct SurgeWrapper {
        coin_address: ResourceAddress,
        token_vault: FungibleVault,
        exchange_component: Global<Exchange>,
        wrapper_component: Global<TokenWrapper>,
    }

    impl SurgeWrapper {

        pub fn new(
            coin_address: ResourceAddress,
            token_address: ResourceAddress,
            exchange_component: Global<Exchange>,
            wrapper_component: Global<TokenWrapper>,
            fund_manager_badge_address: ResourceAddress,
        ) -> Global<SurgeWrapper> {
            Self {
                coin_address: coin_address,
                token_vault: FungibleVault::new(token_address),
                exchange_component: exchange_component,
                wrapper_component: wrapper_component,
            }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .roles(roles!(
                    fund_manager => rule!(require(fund_manager_badge_address));
                ))
                .globalize()
        }

    }

    impl DefiProtocolInterfaceTrait for SurgeWrapper {

        fn deposit_protocol_token(
            &mut self,
            token: Bucket,
        ) -> (
            Decimal,                // Total coin amount
            Option<Decimal>         // None
        ) {
            self.token_vault.put(FungibleBucket(token));

            self.get_coin_amounts()
        }

        fn withdraw_protocol_token(
            &mut self,
            amount: Option<Decimal>,
        ) -> (
            Bucket,                 // LP tokens
            Decimal,                // Total coin amount
            Option<Decimal>         // None
        ) {
            match amount {
                Some(amount) => match amount > self.token_vault.amount() {
                    true => {
                        let token_bucket = self.token_vault.take_all();

                        let (coin_amount, _) = self.get_coin_amounts();

                        (token_bucket.into(), coin_amount, None)
                    },
                    false => {
                        let token_bucket = self.token_vault.take(amount);

                        let (coin_amount, _) = self.get_coin_amounts();

                        (token_bucket.into(), coin_amount, None)
                    },
                },
                None => (
                    self.token_vault.take_all().into(),
                    Decimal::ZERO,
                    None
                ),
            }
        }

        fn deposit_coin(
            &mut self,
            coin: FungibleBucket,
            _other_coin: Option<FungibleBucket>,
            _message: Option<String>,
            _signature: Option<String>,
        ) -> (
            Decimal,                // Total coin amount
            Option<Decimal>         // None
        ) {
            let wrapped_coin_bucket = self.wrapper_component.wrap(coin.into());

            self.token_vault.put(
                FungibleBucket(
                    self.exchange_component.add_liquidity(wrapped_coin_bucket)
                )
            );

            self.get_coin_amounts()
        }

        fn withdraw_coin(
            &mut self,
            amount: Option<Decimal>,
            _other_coin_to_coin_price_ratio: Option<Decimal>,
        ) -> (
            FungibleBucket,
            Option<FungibleBucket>,
            Decimal,                // Total coin amount
            Option<Decimal>         // None
        ) {
            let (token_bucket, remaining_coin_amount)  = match amount {
                Some(amount) => {
                    let pool_details = self.exchange_component.get_pool_details();

                    let requested_token_amount = pool_details.lp_supply * (amount / pool_details.base_tokens_amount);

                    let available_token_amount = self.token_vault.amount();

                    match requested_token_amount > available_token_amount {
                        true => (
                            self.token_vault.take_all(),
                            Decimal::ZERO
                        ),
                        false => (
                            self.token_vault.take(requested_token_amount),
                            pool_details.base_tokens_amount *
                                ((available_token_amount - requested_token_amount) / pool_details.lp_supply)
                        ),
                    }
                },
                None => (
                    self.token_vault.take_all(),
                    Decimal::ZERO
                ),
            };

            let wrapped_coin_bucket = self.exchange_component.remove_liquidity(
                token_bucket.into(),
            );

            let coin_bucket = FungibleBucket(
                self.wrapper_component.unwrap(
                    wrapped_coin_bucket,
                    self.coin_address
                )
            );

            (
                coin_bucket,
                None,
                remaining_coin_amount,
                None
            )
        }

        fn get_coin_amounts(&mut self) -> (
            Decimal,                // Total coin amount
            Option<Decimal>         // None
        ) {
            let pool_details = self.exchange_component.get_pool_details();

            let token_amount = self.token_vault.amount();

            (
                pool_details.base_tokens_amount * (token_amount / pool_details.lp_supply),
                None
            )
        }
    }
}
