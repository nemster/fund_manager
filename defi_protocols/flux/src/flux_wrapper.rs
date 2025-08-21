use scrypto::prelude::*;
use crate::common::*;
use scrypto_interface::*;

// Struct containing information about a Flux pool
#[derive(ScryptoSbor, Debug)]
pub struct StabilityPoolInfoReturn {
    collateral: ResourceAddress,
    payout_split: Option<Decimal>,
    liquidity_rewards_split: Option<Decimal>,
    stability_pool_split: Option<Decimal>,
    allow_pool_buys: bool,
    pool_buy_price_modifier: Option<Decimal>,
    liquidity_rewards: Decimal,
    pool: Global<TwoResourcePool>,
    collateral_amount: Decimal,
    fusd_amount: Decimal,
    latest_lowest_interests: Vec<Decimal>,
    last_lowest_interests_update: Instant,
}

// How many NFTs can be withdrawn in a single operation
static NON_FUNGIBLES_PER_WITHDRAW: u32 = 100;

// This blueprint is a wrapper to talk to the Flux protocol using the DefiProtocolInterfaceStub
// interface.
// Here "coin" is the coin used to provide liquidity to the Flux protocol (as an example LSULP); to
// provide multiple coins, multiple instances of this component are required.
// "other coin" is fUSD; is is neved deposited but can be withdrawn.
// "token" is the LP token returned by the Flux protocol upon deposits.
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

    struct FluxWrapper {
        fusd_address: ResourceAddress, // Resource address of fUSD coin (other coin)
        coin_address: ResourceAddress, // Resource address of the coin to deposit in the pool
        token_address: ResourceAddress, // Resource address of the LP tokens
        component_address: Global<StabilityPools>, // Address of the Flux component
        pool: Global<TwoResourcePool>, // Address of the pool used by the Flux component
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
            mut component_address: Global<StabilityPools>, // Address of the Flux component
            fund_manager_badge_address: ResourceAddress, // God's badge
            admin_badge_address: ResourceAddress, // The badge that every admins holds
        ) -> Global<FluxWrapper> {

            // Create the FluxWrapper component and globalize it
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

        // Give the control of the Account back to the component.
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
        // LP tokens can't be withdrawn this way.
        // A single admin can perform this operation
        pub fn whithdraw_unexpected_coin(
            &mut self,
            coin_address: ResourceAddress,
        ) -> Bucket {

            // Make sure the admin isn't stealing funds
            assert!(
                coin_address != self.token_address,
                "You can't withdraw this coin",
            );

            match coin_address.is_fungible() {

                // Withdraw the whole balance of a fungible
                true => {
                    let (coin_bucket, _) = self.take_from_account(
                      coin_address,
                        Decimal::MAX,
                    );

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
            Decimal     // Remaining amount
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

    impl DefiProtocolInterfaceTrait for FluxWrapper {

        // Deposit LP tokens in the Account.
        // Depositing coins and fUSD is not supported by this method.
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

        // Withdraw LP tokens from the Account.
        // The Account is not supposed to hold coins and fUSD so those will not be withdrawn
        fn withdraw_all(
            &mut self,
        ) -> (
            Bucket,                     // LP tokens
            Option<FungibleBucket>,     // None
            Option<FungibleBucket>,     // None
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

        // Deposit coins in the Flux protocol. Depositing fUSD is not supported by this method.
        // Flux protocol requires message and signature about the XRD price from the Morpher
        // oracle.
        // This method returns the number of coins that can be withdrawn from the Flux pool.
        fn deposit_coin(
            &mut self,
            coin: FungibleBucket,
            _other_coin: Option<FungibleBucket>,
            message: Option<String>,
            signature: Option<String>,
        ) -> (
            Decimal,                // Total coin amount
            Option<Decimal>         // Total fUSD amount
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

        // This method withdraws coins and fUSD from the Flux pool
        fn withdraw_coin(
            &mut self,
            amount: Decimal, // Coin amount to withdraw (or fUSD equivalent value)
            other_coin_to_coin_price_ratio: Option<Decimal>, // coins per fUSD
        ) -> (
            FungibleBucket,             // Withdrawn coins
            Option<FungibleBucket>,     // Withdrawn fUSD
            Decimal,                    // Remaining coin amount
            Option<Decimal>             // Remaining fUSD amount
        ) {
            // Compute the amount of coins and fUSD withdrawn by returning one pool unit
            let amounts = self.pool.get_redemption_value(Decimal::ONE);
            let coin_per_token = *amounts.get(&self.coin_address).unwrap_or(&Decimal::ZERO);
            let fusd_per_token = *amounts.get(&self.fusd_address).unwrap_or(&Decimal::ZERO);

            // Compute the number of tokens to get amount coins (or fUSD equivalent value)
            let token_amount = amount
                / (coin_per_token + fusd_per_token * other_coin_to_coin_price_ratio.unwrap());

            // Get the tokens from the Account and send them to the Pool
            let (token_bucket, remaining_tokens) = self.take_from_account(
                self.token_address,
                token_amount,
            );
            let buckets = self.component_address.withdraw_from_pool(
                self.coin_address,
                token_bucket
            );

            // Compute the number of coins and fUSD that can be still withdrawn
            let remaining_fusd = remaining_tokens * fusd_per_token;
            let remaining_coin = remaining_tokens * coin_per_token;

            // Return buckets and information
            (
                FungibleBucket(buckets.0),
                Some(FungibleBucket(buckets.1)),
                remaining_coin,
                Some(remaining_fusd),
            )
        }

        // Withdraw the badge used to manage the Account; the component will no loger be able to
        // work correctly.
        // This method is called by the FundManager when a DeFi protocol position is removed from
        // the list.
        fn withdraw_account_badge(&mut self) -> NonFungibleBucket {
            self.account_badge_vault.take_non_fungible(
                &self.account_badge_vault.non_fungible_local_id()
            )
        }

        // Get the numebr of coins and fUSD that can be withdrawn from this component
        fn get_coin_amounts(&mut self) -> (
            Decimal,                // Total coin amount
            Option<Decimal>         // Total fUSD amount
        ) {
            // Ask the pool the redemption value of the available LP tokens
            let amounts = self.pool.get_redemption_value(
                self.account.balance(self.token_address)
            );

            (
                *amounts.get(&self.coin_address).unwrap_or(&Decimal::ZERO),
                Some(*amounts.get(&self.fusd_address).unwrap_or(&Decimal::ZERO)),
            )
        }
    }
}
