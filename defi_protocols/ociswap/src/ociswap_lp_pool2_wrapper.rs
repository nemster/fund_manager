use scrypto::prelude::*;
use crate::common::*;
use scrypto_interface::*;

static NON_FUNGIBLES_PER_WITHDRAW: u32 = 100;

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
            admin => updatable_by: [fund_manager];
        },
        methods {
            deposit_protocol_token => restrict_to: [fund_manager];
            withdraw_protocol_token => restrict_to: [fund_manager];
            deposit_coin => restrict_to: [fund_manager];
            withdraw_coin => restrict_to: [fund_manager];

            // The fund_manager component will never call these methods, they can only be used in
            // case of an emergency by the admins
            withdraw_account_badge => restrict_to: [fund_manager];
            deposit_account_badge => restrict_to: [fund_manager];

            // Withdraw any unexpected coin in the account
            whithdraw_unexpected_coin => restrict_to: [admin];

            get_coin_amounts => PUBLIC;
        }
    }

    struct OciswapLpPool2Wrapper {
        a_address: ResourceAddress,
        b_address: ResourceAddress,
        lp_token_address: ResourceAddress,
        account: Global<Account>,
        account_badge_vault: NonFungibleVault, // Badge to manage the Account
        component_address: Global<Pool>,
        pool: Global<TwoResourcePool>,
    }

    impl OciswapLpPool2Wrapper {

        pub fn new(
            a_address: ResourceAddress,
            b_address: ResourceAddress,
            lp_token_address: ResourceAddress,
            account: Global<Account>,
            account_badge_vault: NonFungibleVault, // Badge to manage the Account
            component_address: Global<Pool>,
            fund_manager_badge_address: ResourceAddress,
            admin_badge_address: ResourceAddress,
        ) -> Global<OciswapLpPool2Wrapper> {
            Self {
                a_address: a_address,
                b_address: b_address,
                lp_token_address: lp_token_address,
                account: account,
                account_badge_vault: account_badge_vault,
                component_address: component_address,
                pool: component_address.liquidity_pool(),
            }
                .instantiate()
                .prepare_to_globalize(OwnerRole::Fixed(rule!(require(admin_badge_address))))
                .roles(roles!(
                    fund_manager => rule!(require(fund_manager_badge_address));
                    admin => rule!(require(admin_badge_address));
                ))
                .globalize()
        }

        // Emergency procedure to get the control of the Account
        pub fn withdraw_account_badge(&mut self) -> NonFungibleBucket {
            self.account_badge_vault.take_non_fungible(
                &self.account_badge_vault.non_fungible_local_id()
            )
        }

        // Give the control of the Account back to the component
        pub fn deposit_account_badge(&mut self, badge_bucket: NonFungibleBucket) {
            assert!(
                self.account_badge_vault.amount() == Decimal::ZERO && badge_bucket.amount() == Decimal::ONE,
                "Only one badge can be deposited",
            );

            self.account_badge_vault.put(badge_bucket);
        }

        // Withdraw any unexpected fungible or non fungible in the account
        pub fn whithdraw_unexpected_coin(
            &mut self,
            coin_address: ResourceAddress,
        ) -> Bucket {
            assert!(
                coin_address != self.a_address &&
                coin_address != self.b_address &&
                coin_address != self.lp_token_address,
                "You can't withdraw this coin",
            );

            match coin_address.is_fungible() {
                true => {
                    let (coin_bucket, _) = self.take_from_account(coin_address, Decimal::MAX);

                    coin_bucket
                },
                false => {
                    let ids = self.account.non_fungible_local_ids(
                        coin_address,
                        NON_FUNGIBLES_PER_WITHDRAW,
                    );

                    self.account_badge_vault.authorize_with_non_fungibles(
                        &self.account_badge_vault.non_fungible_local_ids(1),
                        || self.account.withdraw_non_fungibles(
                            coin_address,
                            ids,
                        )
                    )
                        .into()
                }
            }
        }

        fn take_from_account(
            &mut self,
            resource_address: ResourceAddress,
            mut amount: Decimal,
        ) -> (Bucket, Decimal) {
            let available_amount = self.account.balance(resource_address);
            if amount > available_amount {
                amount = available_amount;
            } else {
                let divisibility = ResourceManager::from_address(resource_address)
                    .resource_type()
                    .divisibility()
                    .unwrap();

                amount = amount.checked_round(divisibility, RoundingMode::ToNegativeInfinity).unwrap();
            }

            match amount > Decimal::ZERO {
                true => {
                    let bucket = self.account_badge_vault.authorize_with_non_fungibles(
                        &self.account_badge_vault.non_fungible_local_ids(1),
                        || self.account.withdraw(
                            resource_address,
                            amount,
                        )
                    );

                    (bucket, available_amount - amount)
                },
                false => (
                    Bucket::new(resource_address),
                    available_amount,
                ),
            }
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
            self.account.try_deposit_or_abort(token, None);

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
            let amount = amount.unwrap_or(Decimal::MAX);

            let (lp_token_bucket, _) = self.take_from_account(
                self.lp_token_address,
                amount,
            );

            let (a_amount, b_amount) = self.get_coin_amounts();

            (lp_token_bucket, a_amount, b_amount)
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
            let coin_amount = coin.amount();

            let (a_bucket, _) = self.take_from_account(
                self.a_address,
                Decimal::MAX,
            );
                
            coin.put(FungibleBucket(a_bucket));

            if other_coin.is_none() {
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
            } else if coin_amount == Decimal::ZERO {
                let (b_bucket, _) = self.take_from_account(
                    self.b_address,
                    Decimal::MAX,
                );
                
                other_coin.as_mut().unwrap().put(FungibleBucket(b_bucket));

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

            let (lp_tokens, remainings) = self.component_address.add_liquidity(
                coin.into(),
                other_coin.unwrap().into(),
            );

            match remainings {
                Some(remainings) => {
                    self.account.try_deposit_batch_or_abort(vec![remainings, lp_tokens], None);
                },
                None => {
                    self.account.try_deposit_or_abort(lp_tokens, None);
                },
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
            let (mut a_bucket, mut remaining_a_amount) = self.take_from_account(
                self.a_address,
                amount.unwrap_or(Decimal::MAX),
            );

            let b_amount_to_withdraw = match amount {
                Some(amount) => (amount - a_bucket.amount()) / other_coin_to_coin_price_ratio.unwrap(),
                None => Decimal::MAX,
            };
            let (mut b_bucket, mut remaining_b_amount) = self.take_from_account(
                self.b_address,
                b_amount_to_withdraw,
            );

            let lp_token_available_amount = self.account.balance(self.lp_token_address);
            if lp_token_available_amount > Decimal::ZERO {

                let lp_token_amount_to_withdraw = match amount {
                    Some(amount) => {
                        let amounts = self.pool.get_redemption_value(lp_token_available_amount);

                        let reedemeble_a_amount = *amounts.get(&self.a_address).unwrap();
                        let reedemeble_b_amount = *amounts.get(&self.b_address).unwrap();

                        remaining_a_amount += reedemeble_a_amount;
                        remaining_b_amount += reedemeble_b_amount;

                        let reedemeble_a_amount_equivalent = reedemeble_a_amount +
                            reedemeble_b_amount * other_coin_to_coin_price_ratio.unwrap();

                        let a_amount_equivalent_to_withdraw = amount
                            - a_bucket.amount()
                            - b_bucket.amount() * other_coin_to_coin_price_ratio.unwrap();

                        lp_token_available_amount * (a_amount_equivalent_to_withdraw / reedemeble_a_amount_equivalent)
                    }
                    None => Decimal::MAX,
                };

                let (lp_token_bucket, _) = self.take_from_account(
                    self.lp_token_address,
                    lp_token_amount_to_withdraw
                );

                if lp_token_bucket.amount() > Decimal::ZERO {
                    let (a, b) = self.component_address.remove_liquidity(lp_token_bucket);

                    remaining_a_amount -= a.amount();
                    remaining_b_amount -= b.amount();

                    a_bucket.put(a);
                    b_bucket.put(b);
                }
            }

            (
                FungibleBucket(a_bucket),
                Some(FungibleBucket(b_bucket)),
                remaining_a_amount,
                Some(remaining_b_amount)
            )
        }

        fn get_coin_amounts(&mut self) -> (
            Decimal,                // Total coin amount
            Option<Decimal>         // Total other coin amount
        ) {
            let amounts = self.pool.get_redemption_value(
                self.account.balance(self.lp_token_address)
            );

            let a_amount = *amounts.get(&self.a_address).unwrap_or(&Decimal::ZERO) +
                self.account.balance(self.a_address);

            let b_amount = *amounts.get(&self.b_address).unwrap_or(&Decimal::ZERO) +
                self.account.balance(self.b_address);

            (
                a_amount,
                Some(b_amount),
            )
        }
    }
}
