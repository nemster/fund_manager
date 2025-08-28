use scrypto::prelude::*;
use crate::common::*;
use scrypto_interface::*;

// Info about a pool state
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

// How many NFTs can be withdrawn in a single operation
static NON_FUNGIBLES_PER_WITHDRAW: u32 = 100;

// This blueprint is a wrapper to talk to the Surge protocol using the DefiProtocolInterfaceStub
// interface.
// Here "coin" is the coin used to provide liquidity to the Surge protocol (as an example XRD); to
// provide multiple coins, multiple instances of this component are required.
// "other coin" is not supported
// "token" is the token returned by the Surge protocol upon deposits.
#[blueprint_with_traits]
mod surge_wrapper {

    extern_blueprint! {
        "package_tdx_2_1phyewk3m6aeycqmmmk5easfmk7mg97sn20p2yvd499rj5y5xrxzdcc",
        Exchange {
            fn add_liquidity(&self, payment: Bucket) -> Bucket;
            fn remove_liquidity(&self, lp_token: Bucket) -> Bucket;
            fn get_pool_details(&self) -> PoolDetails;
        }
    }

    extern_blueprint! {
        "package_tdx_2_1phjqhqsp286r7nc4e47kyeyus7drxwcf3965u693fzpe0krcpmt6hu",
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

    struct SurgeWrapper {
        coin_address: ResourceAddress,      // The coin to provide liquidity to Surge
        token_address: ResourceAddress,     // The token Surge returns for provided liquidity
        account: Global<Account>,           // The account to hold the tokens
        account_badge_vault: NonFungibleVault,  // Badge to manage the Account
        exchange_component: Global<Exchange>,       // Surge main component
        wrapper_component: Global<TokenWrapper>,    // Surge additional component to wrap/unwrap
                                                    // coins
    }

    impl SurgeWrapper {

        // Instantiate a global SurgeWrapper component
        pub fn new(
            coin_address: ResourceAddress,  // The coin to provide liquidity to Surge
            token_address: ResourceAddress, // The token Surge returns for provided liquidity
            account: Global<Account>,       // The account to hold the tokens
            account_badge_bucket: NonFungibleBucket,    // Badge to manage the Account
            exchange_component: Global<Exchange>,       // Surge main component
            wrapper_component: Global<TokenWrapper>,    // Surge additional component
            fund_manager_badge_address: ResourceAddress,    // God's badge
            admin_badge_address: ResourceAddress,       // Admins' badge
        ) -> Global<SurgeWrapper> {

            // Instantiate and globalize the component
            Self {
                coin_address: coin_address,
                token_address: token_address,
                account: account,
                account_badge_vault: NonFungibleVault::with_bucket(account_badge_bucket),
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

                // If the resource to withdraw is a fungible, take the whole balance from the account
                true => {
                    let (coin_bucket, _) = self.take_from_account(coin_address, Decimal::MAX);

                    coin_bucket
                },

                // If we have to withdraw a non fungible, take up to NON_FUNGIBLES_PER_WITHDRAW
                // NFTs
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
            mut amount: Decimal,                // The maxim amount to withdraw
        ) -> (
            Bucket,     // Bucket of the requested resource
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
                amount = amount.checked_round(divisibility, RoundingMode::ToZero).unwrap();
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

    impl DefiProtocolInterfaceTrait for SurgeWrapper {

        // Deposit tokens and eventually coins
        fn deposit_all(
            &mut self,
            token: Bucket,                          // Token to deposit
            coin: Option<FungibleBucket>,           // Eventual coin to deposit
            _other_coin: Option<FungibleBucket>,    // Not supported
        ) -> (
            Decimal,                // Total coin amount
            Option<Decimal>         // None
        ) {
            // Deposit tokens
            self.account.try_deposit_or_abort(token, None);

            // Deposit coin if needed and return total deposited amount
            if coin.is_some() {
                self.deposit_coin(coin.unwrap(), None, None, None)
            } else {
                self.get_coin_amounts()
            }
        }

        // Withdraw tokens from the account
        fn withdraw_all(
            &mut self,
        ) -> (
            Bucket,                 // LP tokens
            Option<FungibleBucket>, // None (the Account is not supposed to hold coins)
            Option<FungibleBucket>, // None
        ) {
            // Get the tokens
            let (token_bucket, _) = self.take_from_account(
                self.token_address,
                Decimal::MAX
            );

            // Return just the token
            (token_bucket, None, None)
        }

        // Deposit coins in the Surge protocol
        fn deposit_coin(
            &mut self,
            coin: FungibleBucket,                   // Coins to deposit
            _other_coin: Option<FungibleBucket>,    // Not suppported
            _message: Option<String>,               // Not needed
            _signature: Option<String>,             // Not needed
        ) -> (
            Decimal,                // Total coin amount
            Option<Decimal>         // None
        ) {
            // Wrap the coins
            let wrapped_coin_bucket = self.wrapper_component.wrap(coin.into());

            // Deposit the wrapped coins in Surge and get the tokens
            let token_bucket = self.exchange_component.add_liquidity(wrapped_coin_bucket);

            // Deposit the tokens in the bucket
            self.account.try_deposit_or_abort(
                token_bucket,
                None
            );

            // REturn the total amount of coins deposited
            self.get_coin_amounts()
        }

        // Withdraw coins from the Surge prtocol
        fn withdraw_coin(
            &mut self,
            amount: Decimal,        // The maximum amount to withdraw
            _other_coin_to_coin_price_ratio: Option<Decimal>,   // Not needed
        ) -> (
            FungibleBucket,         // Coins
            Option<FungibleBucket>, // None
            Decimal,                // Remaining coin amount
            Option<Decimal>         // None
        ) {
            let pool_details = self.exchange_component.get_pool_details();

            // Compute the token amount to use in order to get amount coins
            let token_amount_to_withdraw = pool_details.lp_supply
                * (amount / pool_details.base_tokens_amount);

            // Take tokens from the Account
            let (token_bucket, remaining_tokens) = self.take_from_account(
                self.token_address,
                token_amount_to_withdraw
            );

            // Compute the remaining coin amount
            let remaining_coin_amount = pool_details.base_tokens_amount
                * (remaining_tokens / pool_details.lp_supply);
            
            // Remove liquidity and unwrap the received coins
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

        // Get the number of coins in this position
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

        // Withdraw the badge used to manage the Account; this component will no loger be able to
        // work correctly.
        // This method is called by the FundManager when a DeFi protocol position is removed from
        // the list.
        fn withdraw_account_badge(&mut self) -> NonFungibleBucket {
            self.account_badge_vault.take_non_fungible(
                &self.account_badge_vault.non_fungible_local_id()
            )
        }
    }
}
