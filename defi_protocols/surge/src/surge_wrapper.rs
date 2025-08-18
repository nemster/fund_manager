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

static NON_FUNGIBLES_PER_WITHDRAW: u32 = 100;

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

    struct SurgeWrapper {
        coin_address: ResourceAddress,
        token_address: ResourceAddress,
        account: Global<Account>, // The account to hold the tokens
        account_badge_vault: NonFungibleVault, // Badge to manage the Account
        exchange_component: Global<Exchange>,
        wrapper_component: Global<TokenWrapper>,
    }

    impl SurgeWrapper {

        pub fn new(
            coin_address: ResourceAddress,
            token_address: ResourceAddress,
            account: Global<Account>, // The account to hold the tokens
            account_badge_vault: NonFungibleVault, // Badge to manage the Account
            exchange_component: Global<Exchange>,
            wrapper_component: Global<TokenWrapper>,
            fund_manager_badge_address: ResourceAddress,
            admin_badge_address: ResourceAddress,
        ) -> Global<SurgeWrapper> {
            Self {
                coin_address: coin_address,
                token_address: token_address,
                account: account,
                account_badge_vault: account_badge_vault,
                exchange_component: exchange_component,
                wrapper_component: wrapper_component,
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
                coin_address != self.token_address,
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

                amount = amount.checked_round(divisibility, RoundingMode::ToZero).unwrap();
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

    impl DefiProtocolInterfaceTrait for SurgeWrapper {

        fn deposit_protocol_token(
            &mut self,
            token: Bucket,
        ) -> (
            Decimal,                // Total coin amount
            Option<Decimal>         // None
        ) {
            self.account.try_deposit_or_abort(token, None);

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
                Some(amount) => {
                    let (token_bucket, _) = self.take_from_account(
                        self.token_address,
                        amount
                    );

                    let (coin_amount, _) = self.get_coin_amounts();

                    (token_bucket, coin_amount, None)
                },
                None => {
                    let (token_bucket, _) = self.take_from_account(
                        self.token_address,
                        Decimal::MAX
                    );

                    (token_bucket, Decimal::ZERO, None)
                },
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

            let token_bucket = self.exchange_component.add_liquidity(wrapped_coin_bucket);

            self.account.try_deposit_or_abort(
                token_bucket,
                None
            );

            self.get_coin_amounts()
        }

        fn withdraw_coin(
            &mut self,
            amount: Option<Decimal>,
            _other_coin_to_coin_price_ratio: Option<Decimal>,
        ) -> (
            FungibleBucket,
            Option<FungibleBucket>, // None
            Decimal,                // Total coin amount
            Option<Decimal>         // None
        ) {
            let (token_bucket, remaining_coin_amount)  = match amount {
                Some(amount) => {
                    let (token_bucket, remaining_tokens) = self.take_from_account(
                        self.token_address,
                        amount
                    );

                    let pool_details = self.exchange_component.get_pool_details();

                    let remaining_coin_amount = pool_details.base_tokens_amount
                        * (remaining_tokens / pool_details.lp_supply);
                    
                    (token_bucket, remaining_coin_amount)
                },
                None => self.take_from_account(
                    self.token_address,
                    Decimal::MAX
                ),
            };

            let wrapped_coin_bucket = self.exchange_component.remove_liquidity(
                token_bucket,
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

            let token_amount = self.account.balance(self.token_address);

            (
                pool_details.base_tokens_amount * (token_amount / pool_details.lp_supply),
                None
            )
        }
    }
}
