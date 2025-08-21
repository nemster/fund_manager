use scrypto::prelude::*;
use crate::common::*;
use scrypto_interface::*;

// How many NFTs can be withdrawn in a single operation
static NON_FUNGIBLES_PER_WITHDRAW: u32 = 100;

// This blueprint is a wrapper to deposit and withdraw liquidity from the most reent Ociswap pools
// using the DefiProtocolInterface interface.
// Here "coin" and "other coin" are the two coins managed by the pool; those are also called "x"
// and "y" as Ociswap does.
// "token" is the LP token returned by the Ociswap pool upon deposits.
#[blueprint_with_traits]
mod ociswap_lp_pool2_wrapper {

    extern_blueprint! {
        "package_tdx_2_1p5qntnqluczzjjnm577mfp7p5jd3qm2sv0qzkqklgkrypcnspw3dff",
        Pool {
            fn add_liquidity(
                &mut self,
                x_bucket: Bucket,
                y_bucket: Bucket,
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

            // FundManager operations
            deposit_all => restrict_to: [fund_manager];
            withdraw_all => restrict_to: [fund_manager];
            deposit_coin => restrict_to: [fund_manager];
            withdraw_coin => restrict_to: [fund_manager];
            withdraw_account_badge => restrict_to: [fund_manager];

            // Single admin operations
            deposit_account_badge => restrict_to: [admin];
            whithdraw_unexpected_coin => restrict_to: [admin];

            // Public method
            get_coin_amounts => PUBLIC;
        }
    }

    struct OciswapLpPool2Wrapper {
        x_address: ResourceAddress,             // First coin managed by the pool
        y_address: ResourceAddress,             // Second coin managed by the pool
        lp_token_address: ResourceAddress,      // LP tokens of the pool
        account: Global<Account>,               // Account used to store LP tokens and eventual a
                                                // and b remainings
        account_badge_vault: NonFungibleVault,  // Badge to manage the Account
        component_address: Global<Pool>,        // Ociswap's component
        pool: Global<TwoResourcePool>,          // Pool used by Ociswap to hold coins
    }

    impl OciswapLpPool2Wrapper {

        // Instantiate and globalize an OciswapLpPool2Wrapper component
        pub fn new(
            x_address: ResourceAddress,
            y_address: ResourceAddress,
            lp_token_address: ResourceAddress,
            account: Global<Account>,
            account_badge_vault: NonFungibleVault,
            component_address: Global<Pool>,
            fund_manager_badge_address: ResourceAddress,
            admin_badge_address: ResourceAddress,
        ) -> Global<OciswapLpPool2Wrapper> {

            // Instantiate and globalize the component
            Self {
                x_address: x_address,
                y_address: y_address,
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

        // Give the control of the Account back to the component
        // This method can be used to restore normal operation of the component in case the
        // account badge has been withdran.
        // A single admin can perform this operation
        pub fn deposit_account_badge(&mut self, badge_bucket: NonFungibleBucket) {
            assert!(
                self.account_badge_vault.amount() == Decimal::ZERO && badge_bucket.amount() == Decimal::ONE,
                "Only one badge can be deposited",
            );

            self.account_badge_vault.put(badge_bucket);
        }

        // Withdraw any unexpected fungible or non fungible in the account.
        // LP tokens, x and y coins can't be withdrawn this way.
        // A single admin can perform this operation
        pub fn whithdraw_unexpected_coin(
            &mut self,
            coin_address: ResourceAddress,
        ) -> Bucket {

            // Make sure the admin isn't stealing from the fund
            assert!(
                coin_address != self.x_address &&
                coin_address != self.y_address &&
                coin_address != self.lp_token_address,
                "You can't withdraw this coin",
            );

            match coin_address.is_fungible() {

                // Withdraw the whole balance of a fungible
                true => {
                    let (coin_bucket, _) = self.take_from_account(coin_address, Decimal::MAX);

                    coin_bucket
                },

                // Withdraw up do NON_FUNGIBLES_PER_WITHDRAW NFTs
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
            resource_address: ResourceAddress,  // The resource to withdraw
            mut amount: Decimal,                // Maximum amount to withdraw
        ) -> (
            Bucket,     // Bucket of the requested coin
            Decimal,    // Remaining amount
        ) {
            // Make sure we are not trying to withdraw more than the available balance
            let available_amount = self.account.balance(resource_address);
            if amount > available_amount {
                amount = available_amount;
            } else {

                // Adjust the amount for coins that have limited divisibility
                let divisibility = ResourceManager::from_address(resource_address)
                    .resource_type()
                    .divisibility()
                    .unwrap();
                amount = amount.checked_round(divisibility, RoundingMode::ToNegativeInfinity).unwrap();
            }

            match amount > Decimal::ZERO {

                // If the amount to withdraw is bigger than zero, get it from the account using
                // the account badge
                true => {
                    let bucket = self.account_badge_vault.authorize_with_non_fungibles(
                        &self.account_badge_vault.non_fungible_local_ids(1),
                        || self.account.withdraw(
                            resource_address,
                            amount,
                        )
                    );

                    // Return the bucket and the remaining balance
                    (bucket, available_amount - amount)
                },

                // In case the amount to withdraw is zero, return an empty bucket and the available
                // balance
                false => (
                    Bucket::new(resource_address),
                    available_amount,
                ),
            }
        }
    }

    impl DefiProtocolInterfaceTrait for OciswapLpPool2Wrapper {

        // Deposit LP tokens, x and y coins in the Account.
        fn deposit_all(
            &mut self,
            token: Bucket,                          // LP token bucket
            coin: Option<FungibleBucket>,           // X coin bucket
            other_coin: Option<FungibleBucket>,     // Y coin bucket
        ) -> (
            Decimal,            // Available x coin amount
            Option<Decimal>     // Available y coin amount
        ) {
            if coin.is_some() || other_coin.is_some() {

                // If multiple buckets must to be deposited, create a vector and deposit them in a
                // single Account operation
                let mut buckets = vec![token];

                if coin.is_some() {
                    buckets.push(coin.unwrap().into());
                }

                if other_coin.is_some() {
                    buckets.push(other_coin.unwrap().into());
                }

                self.account.try_deposit_batch_or_abort(buckets, None);
            } else {

                // Deposit just the LP tokens
                self.account.try_deposit_or_abort(token, None);
            }

            // Return the x and y coin availability
            self.get_coin_amounts()
        }

        // Withdraw LP tokens, x and y coins from the Account.
        fn withdraw_all(
            &mut self,
        ) -> (
            Bucket,                     // LP tokens
            Option<FungibleBucket>,     // X coin
            Option<FungibleBucket>      // Y coin
        ) {
            let (lp_token_bucket, _) = self.take_from_account(
                self.lp_token_address,
                Decimal::MAX,
            );

            let (x_bucket, _) = self.take_from_account(
                self.x_address,
                Decimal::MAX,
            );

            let (y_bucket, _) = self.take_from_account(
                self.y_address,
                Decimal::MAX,
            );

            (
                lp_token_bucket,
                Some(FungibleBucket(x_bucket)),
                Some(FungibleBucket(y_bucket)),
            )
        }

        // Deposit x and y coins in the pool
        fn deposit_coin(
            &mut self,
            mut coin: FungibleBucket,               // X coin
            mut other_coin: Option<FungibleBucket>, // Y coin
            _message: Option<String>,
            _signature: Option<String>,
        ) -> (
            Decimal,                // Available x coins
            Option<Decimal>         // Available y coins
        ) {
            let coin_amount = coin.amount();

            // Add eventual x coins in the Account to the given bucket
            let (x_bucket, _) = self.take_from_account(
                self.x_address,
                Decimal::MAX,
            );
            coin.put(FungibleBucket(x_bucket));

            // If no y bucket have been provided, swap half of the x coins for y coins
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

            // Do the same if the y bucket has been provided but is empty
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
            }

            // Add eventual y coins in the Account to the bucket
            let (y_bucket, _) = self.take_from_account(
                self.y_address,
                Decimal::MAX,
            ); 
            other_coin.as_mut().unwrap().put(FungibleBucket(y_bucket));

            // If no x coins have been provided, swap half of the y coins for x coins
            if coin_amount == Decimal::ZERO {
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

            // Add liquidity to the pool and get LP tokens and eventual remainings, then put
            // everithing in the Account
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

            // Return available coins
            self.get_coin_amounts()
        }

        // Withdraw the specified amount of coin (or an equivalent value of y coin)
        fn withdraw_coin(
            &mut self,
            mut amount: Decimal,                                // x amount to withdraw
            other_coin_to_coin_price_ratio: Option<Decimal>,    // y/x price ratio
        ) -> (
            FungibleBucket,         // x bucket
            Option<FungibleBucket>, // y bucket
            Decimal,                // remaining x amount
            Option<Decimal>         // remaining y amount
        ) {
            // Take up to amount x coins from the Account and update the remaining value to
            // withdraw accordingly
            let (mut x_bucket, mut remaining_x_amount) = self.take_from_account(
                self.x_address,
                amount,
            );
            amount -= x_bucket.amount();

            // Take up to the y equivalent of amount from the Account
            let (mut y_bucket, mut remaining_y_amount) = self.take_from_account(
                self.y_address,
                amount / other_coin_to_coin_price_ratio.unwrap(),
            );
            amount -= y_bucket.amount() * other_coin_to_coin_price_ratio.unwrap();

            // If there are available LP tokens compute the available x and y coins
            let lp_token_available_amount = self.account.balance(self.lp_token_address);
            if lp_token_available_amount > Decimal::ZERO {
                let amounts = self.pool.get_redemption_value(lp_token_available_amount);
                let reedemeble_x_amount = *amounts.get(&self.x_address).unwrap_or(&Decimal::ZERO);
                let reedemeble_y_amount = *amounts.get(&self.y_address).unwrap_or(&Decimal::ZERO);

                remaining_x_amount += reedemeble_x_amount;
                remaining_y_amount += reedemeble_y_amount;

                // Compute the number of tokens to convert to fill amount
                let reedemeble_x_amount_equivalent = reedemeble_x_amount +
                    reedemeble_y_amount * other_coin_to_coin_price_ratio.unwrap();
                let lp_token_amount_to_withdraw = lp_token_available_amount
                    * (amount / reedemeble_x_amount_equivalent);

                // Take up to this numebr of tokens from the Account and convert them
                let (lp_token_bucket, _) = self.take_from_account(
                    self.lp_token_address,
                    lp_token_amount_to_withdraw
                );
                if lp_token_bucket.amount() > Decimal::ZERO {
                    let (x, y) = self.component_address.remove_liquidity(lp_token_bucket);

                    // Put all coins in the buckets and update the remaining amounts
                    remaining_x_amount -= x.amount();
                    remaining_y_amount -= y.amount();
                    x_bucket.put(x);
                    y_bucket.put(y);
                }
            }

            // Return all buckets and info
            (
                FungibleBucket(x_bucket),
                Some(FungibleBucket(y_bucket)),
                remaining_x_amount,
                Some(remaining_y_amount)
            )
        }

        // Get the control of the Account; the component will no loger be able to
        // work correctly.
        // This method is called by the FundManager when a DeFi protocol position is removed from
        // the list.
        fn withdraw_account_badge(&mut self) -> NonFungibleBucket {
            self.account_badge_vault.take_non_fungible(
                &self.account_badge_vault.non_fungible_local_id()
            )
        }

        // Return all available x and y coins, both in the Account and in the pool
        fn get_coin_amounts(&mut self) -> (
            Decimal,                // Total coin amount
            Option<Decimal>         // Total other coin amount
        ) {
            let amounts = self.pool.get_redemption_value(
                self.account.balance(self.lp_token_address)
            );

            let x_amount = *amounts.get(&self.x_address).unwrap_or(&Decimal::ZERO) +
                self.account.balance(self.x_address);

            let y_amount = *amounts.get(&self.y_address).unwrap_or(&Decimal::ZERO) +
                self.account.balance(self.y_address);

            (
                x_amount,
                Some(y_amount),
            )
        }
    }
}
