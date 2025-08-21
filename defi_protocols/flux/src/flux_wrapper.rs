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

static NON_FUNGIBLES_PER_WITHDRAW: u32 = 100;

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
            admin => updatable_by: [fund_manager];
        },
        methods {
            deposit_all => restrict_to: [fund_manager];
            withdraw_all => restrict_to: [fund_manager];
            deposit_coin => restrict_to: [fund_manager];
            withdraw_coin => restrict_to: [fund_manager];
            withdraw_account_badge => restrict_to: [fund_manager];
            deposit_account_badge => restrict_to: [fund_manager];

            // Withdraw any unexpected coin in the account
            whithdraw_unexpected_coin => restrict_to: [admin];

            get_coin_amounts => PUBLIC;
        }
    }

    struct FluxWrapper {
        fusd_address: ResourceAddress,
        coin_address: ResourceAddress,
        token_address: ResourceAddress,
        component_address: Global<StabilityPools>,
        pool: Global<TwoResourcePool>,
        account: Global<Account>, // The account to hold the LP tokens
        account_badge_vault: NonFungibleVault, // Badge to manage the Account
    }

    impl FluxWrapper {

        pub fn new(
            fusd_address: ResourceAddress,
            coin_address: ResourceAddress, // Example coin: LSULP
            token_address: ResourceAddress, // Example token: lsulpFUSD
            account: Global<Account>, // The account to hold the LP tokens
            account_badge: NonFungibleBucket, // Badge to manage the Account
            mut component_address: Global<StabilityPools>,
            fund_manager_badge_address: ResourceAddress,
            admin_badge_address: ResourceAddress,
        ) -> Global<FluxWrapper> {
            Self {
                fusd_address: fusd_address,
                coin_address: coin_address,
                token_address: token_address,
                account: account,
                account_badge_vault: NonFungibleVault::with_bucket(account_badge),
                component_address: component_address,
                pool: component_address.get_stability_pool_infos(Some(vec![coin_address])).pop().unwrap().pool,
            }
                .instantiate()
                .prepare_to_globalize(OwnerRole::Fixed(rule!(require(admin_badge_address))))
                .roles(roles!(
                    fund_manager => rule!(require(fund_manager_badge_address));
                    admin => rule!(require(admin_badge_address));
                ))
                .globalize()
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
                coin_address != self.fusd_address &&
                coin_address != self.coin_address &&
                coin_address != self.token_address,
                "You can't withdraw this coin",
            );

            match coin_address.is_fungible() {
                true => {
                    let (coin_bucket, _) = self.take_from_account(
                      coin_address,
                        Decimal::MAX,
                    );

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

        // Private method to withdraw from the Account
        fn take_from_account(
            &mut self,
            resource_address: ResourceAddress,
            mut amount: Decimal,
        ) -> (
            Bucket,     // Bucket of the requested coin
            Decimal     // Remaining amount
        ) {
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

    impl DefiProtocolInterfaceTrait for FluxWrapper {

        fn deposit_all(
            &mut self,
            token: Bucket,
            _coin: Option<FungibleBucket>,
            _other_coin: Option<FungibleBucket>,
        ) -> (
            Decimal,
            Option<Decimal>
        ) {
            self.account.try_deposit_or_abort(token, None);

            self.get_coin_amounts()
        }

        fn withdraw_all(
            &mut self,
        ) -> (
            Bucket,             // LP tokens
            Option<FungibleBucket>,
            Option<FungibleBucket>,
        ) {
            let (token_bucket, _) = self.take_from_account(
                self.token_address,
                Decimal::MAX,
            );

            (
                token_bucket,
                None,
                None,
            )
        }

        fn deposit_coin(
            &mut self,
            coin: FungibleBucket,
            _other_coin: Option<FungibleBucket>,
            message: Option<String>,
            signature: Option<String>,
        ) -> (
            Decimal,                // Total fUSD amount
            Option<Decimal>         // Total other coin amount
        ) {
            let (token_bucket, _, _) = self.component_address.contribute_to_pool(
                self.coin_address,
                coin.into(),
                true,
                message.expect("Message needed"),
                signature.expect("Signature needed"),
            );

            self.account.try_deposit_or_abort(token_bucket, None);
            
            self.get_coin_amounts()
        }

        fn withdraw_coin(
            &mut self,
            amount: Decimal,
            other_coin_to_coin_price_ratio: Option<Decimal>,
        ) -> (
            FungibleBucket,
            Option<FungibleBucket>,
            Decimal,                    // FUSD amount remaining
            Option<Decimal>             // Other coin amount remaining
        ) {
            let amounts = self.pool.get_redemption_value(Decimal::ONE);

            // Amount of coins withdrawn by returning one pool unit
            let fusd_per_token = *amounts.get(&self.fusd_address).unwrap_or(&Decimal::ZERO);
            let coin_per_token = *amounts.get(&self.coin_address).unwrap_or(&Decimal::ZERO);

            let token_amount = amount / (fusd_per_token + coin_per_token * other_coin_to_coin_price_ratio.unwrap());

            let (token_bucket, remaining_tokens) = self.take_from_account(
                self.token_address,
                token_amount,
            );
            let buckets = self.component_address.withdraw_from_pool(
                self.coin_address,
                token_bucket
            );

            let remaining_fusd = remaining_tokens * fusd_per_token;
            let remaining_coin = remaining_tokens * coin_per_token;

            (
                FungibleBucket(buckets.0),
                Some(FungibleBucket(buckets.1)),
                remaining_fusd,
                Some(remaining_coin),
            )
        }

        fn withdraw_account_badge(&mut self) -> NonFungibleBucket {
            self.account_badge_vault.take_non_fungible(
                &self.account_badge_vault.non_fungible_local_id()
            )
        }

        fn get_coin_amounts(&mut self) -> (
            Decimal,                // Total fUSD amount
            Option<Decimal>         // Total other coin amount
        ) {
            let amounts = self.pool.get_redemption_value(
                self.account.balance(self.token_address)
            );

            (
                *amounts.get(&self.fusd_address).unwrap(),
                Some(*amounts.get(&self.coin_address).unwrap()),
            )
        }
    }
}
