use scrypto::prelude::*;
use crate::common::*;
use scrypto_interface::*;

#[blueprint_with_traits]
mod ociswap_lp_pool2_wrapper {

    extern_blueprint! {
        "package_tdx_2_1p5qntnqluczzjjnm577mfp7p5jd3qm2sv0qzkqklgkrypcnspw3dff",
        Pool {
            fn add_liquidity(
                &mut self,
                a_bucket: Bucket,
                b_bucket: Bucket,
            ) -> (Bucket, Option<Bucket>);

            fn liquidity_pool(&self) -> Global<TwoResourcePool>;

            fn swap(&mut self, input_bucket: Bucket) -> Bucket;

            fn remove_liquidity(&mut self, lp_token: Bucket) -> (Bucket, Bucket);
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

    struct OciswapLpPool2Wrapper {
        a_vault: FungibleVault,
        b_vault: FungibleVault,
        lp_token_vault: FungibleVault,
        component_address: Global<Pool>,
        pool: Global<TwoResourcePool>,
    }

    impl OciswapLpPool2Wrapper {

        pub fn new(
            a_address: ResourceAddress,
            b_address: ResourceAddress,
            lp_token_address: ResourceAddress,
            component_address: Global<Pool>,
            fund_manager_badge_address: ResourceAddress,
        ) -> Global<OciswapLpPool2Wrapper> {
            Self {
                a_vault: FungibleVault::new(a_address),
                b_vault: FungibleVault::new(b_address),
                lp_token_vault: FungibleVault::new(lp_token_address),
                component_address: component_address,
                pool: component_address.liquidity_pool(),
            }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .roles(roles!(
                    fund_manager => rule!(require(fund_manager_badge_address));
                ))
                .globalize()
        }

    }

    impl DefiProtocolInterfaceTrait for OciswapLpPool2Wrapper {

        fn deposit_protocol_token(
            &mut self,
            token: Bucket,
        ) -> (
            Decimal,
            Option<Decimal>
        ) {
            self.lp_token_vault.put(FungibleBucket(token));

            self.get_coin_amounts()
        }

        fn withdraw_protocol_token(
            &mut self,
            amount: Option<Decimal>,
        ) -> (
            Bucket,
            Decimal,
            Option<Decimal>
        ) {
            match amount {
                Some(amount) => {
                    match amount > self.lp_token_vault.amount() {
                        true => (
                            self.lp_token_vault.take_all().into(),
                            self.a_vault.amount(),
                            Some(self.b_vault.amount())
                        ),
                        false => {
                            let token_bucket = self.lp_token_vault.take(amount);

                            let (a_amount, b_amount) = self.get_coin_amounts();

                            (token_bucket.into(), a_amount, b_amount)
                        },
                    }
                },
                None => (
                    self.lp_token_vault.take_all().into(),
                    self.a_vault.amount(),
                    Some(self.b_vault.amount())
                ),
            }
        }

        fn deposit_coin(
            &mut self,
            mut coin: FungibleBucket,
            mut other_coin: Option<FungibleBucket>,
            _message: Option<String>,
            _signature: Option<String>,
        ) -> (
            Decimal,
            Option<Decimal>
        ) {
            if other_coin.is_none() {
                coin.put(self.a_vault.take_all());

                other_coin = Some(
                    FungibleBucket(
                        self.component_address.swap(
                            coin.take_advanced(
                                coin.amount() / 2,
                                WithdrawStrategy::Rounded(RoundingMode::ToZero)
                            )
                                .into()
                        )
                    )
                );
            } else if other_coin.as_ref().unwrap().amount() == Decimal::ZERO {
                coin.put(self.a_vault.take_all());

                other_coin.as_mut().unwrap().put(
                    FungibleBucket(
                        self.component_address.swap(
                            coin.take_advanced(
                                coin.amount() / 2,
                                WithdrawStrategy::Rounded(RoundingMode::ToZero)
                            )
                                .into()
                        )
                    )
                );
            } else if coin.amount() == Decimal::ZERO {
                other_coin.as_mut().unwrap().put(self.b_vault.take_all());

                let other_coin_amount = other_coin.as_ref().unwrap().amount();

                coin.put(
                    FungibleBucket(
                        self.component_address.swap(
                            other_coin.as_mut().unwrap().take_advanced(
                                other_coin_amount / 2,
                                WithdrawStrategy::Rounded(RoundingMode::ToZero)
                            )
                                .into()
                        )
                    )
                );
            }

            let (token, remainings) = self.component_address.add_liquidity(
                coin.into(),
                other_coin.unwrap().into(),
            );

            self.lp_token_vault.put(FungibleBucket(token));

            match remainings {
                Some(remainings) => {
                    if remainings.resource_address() == self.a_vault.resource_address() {
                        self.a_vault.put(FungibleBucket(remainings));
                    } else {
                        self.b_vault.put(FungibleBucket(remainings));
                    }
                },
                None => {},
            }

            self.get_coin_amounts()
        }

        fn withdraw_coin(
            &mut self,
            amount: Option<Decimal>,
            other_coin_to_coin_price_ratio: Option<Decimal>,
        ) -> (
            FungibleBucket,
            Option<FungibleBucket>,
            Decimal,
            Option<Decimal>
        ) {
            match amount {
                Some(mut amount) => {
                    let mut a_amount: Decimal;
                    let mut b_amount: Decimal;

                    // Collect any unused a coin first and initialize a_amount with the amount that remains
                    // in a_vault
                    let mut a_bucket = match self.a_vault.amount() > Decimal::ZERO {
                        true => match amount > self.a_vault.amount() {
                            true => {
                                amount -= self.a_vault.amount();

                                a_amount = Decimal::ZERO;

                                self.a_vault.take_all()
                            },
                            false => {
                                let a_bucket = self.a_vault.take_advanced(
                                    amount,
                                    WithdrawStrategy::Rounded(RoundingMode::ToZero)
                                );

                                a_amount = self.a_vault.amount();

                                a_bucket
                            },
                        },
                        false => {
                            a_amount = Decimal::ZERO;

                            FungibleBucket::new(self.a_vault.resource_address())
                        },
                    };

                    // Collect any unused b coin first and initialize b_amount with the amount that remains
                    // in b_vault
                    let mut b_bucket = match self.b_vault.amount() > Decimal::ZERO {
                        true => match amount > self.b_vault.amount() * other_coin_to_coin_price_ratio.unwrap() {
                            true => {
                                amount -= self.b_vault.amount() * other_coin_to_coin_price_ratio.unwrap();

                                b_amount = Decimal::ZERO;

                                self.b_vault.take_all()
                            },
                            false => {
                                let b_bucket = self.b_vault.take_advanced(
                                    amount / other_coin_to_coin_price_ratio.unwrap(),
                                    WithdrawStrategy::Rounded(RoundingMode::ToZero)
                                );

                                b_amount = self.b_vault.amount();

                                b_bucket
                            },
                        },
                        false => {
                            b_amount = Decimal::ZERO;

                            FungibleBucket::new(self.b_vault.resource_address())
                        },
                    };

                    if self.lp_token_vault.amount() > Decimal::ZERO {
                        let amounts = self.pool.get_redemption_value(
                            self.lp_token_vault.amount()
                        );

                        let reedemeble_a_amount = *amounts.get(&self.a_vault.resource_address()).unwrap();
                        let reedemeble_b_amount = *amounts.get(&self.b_vault.resource_address()).unwrap();

                        let reedemeble_a_amount_equivalent = reedemeble_a_amount +
                            reedemeble_b_amount * other_coin_to_coin_price_ratio.unwrap();

                        let (a, b) = match reedemeble_a_amount_equivalent > amount {
                            true => self.component_address.remove_liquidity(
                                self.lp_token_vault.take(
                                    self.lp_token_vault.amount() * amount / reedemeble_a_amount_equivalent
                                )
                                    .into()
                            ),
                            false => self.component_address.remove_liquidity(
                                self.lp_token_vault.take_all().into()
                            ),
                        };

                        a_amount += reedemeble_a_amount - a.amount();
                        b_amount += reedemeble_b_amount - b.amount();

                        a_bucket.put(FungibleBucket(a));
                        b_bucket.put(FungibleBucket(b));
                    }

                    (
                        a_bucket,
                        Some(b_bucket),
                        a_amount,
                        Some(b_amount)
                    )
                },
                None => {
                    let mut a_bucket = self.a_vault.take_all();
                    let mut b_bucket = self.b_vault.take_all();

                    if self.lp_token_vault.amount() > Decimal::ZERO {
                        let (a, b) = self.component_address.remove_liquidity(
                            self.lp_token_vault.take_all().into()
                        );

                        a_bucket.put(FungibleBucket(a));
                        b_bucket.put(FungibleBucket(b));
                    }

                    (
                        a_bucket,
                        Some(b_bucket),
                        Decimal::ZERO,
                        Some(Decimal::ZERO)
                    )
                },
            }
        }

        fn get_coin_amounts(&mut self) -> (
            Decimal,                // Total coin amount
            Option<Decimal>         // Total other coin amount
        ) {
            let amounts = self.pool.get_redemption_value(
                self.lp_token_vault.amount()
            );

            let a_amount = *amounts.get(&self.a_vault.resource_address()).unwrap() +
                self.a_vault.amount();

            let b_amount = *amounts.get(&self.b_vault.resource_address()).unwrap() +
                self.b_vault.amount();

            (
                a_amount,
                Some(b_amount),
            )
        }
    }
}
