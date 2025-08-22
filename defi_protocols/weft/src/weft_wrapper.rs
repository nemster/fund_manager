use scrypto::prelude::*;
use crate::common::*;
use scrypto_interface::*;

/* This blueprint implements the DefiProtocolInterfaceTrait interface and allows to communicate
 * with the WEFT Finance components.
 *
 * Here we call "coin" one of the coins we will provide to WEFT (such as xUSDC) and "token" its
 * version wrapped by WEFT (such as w2-xUSDC).
 * The "other_coin" is the WEFT coin.
 *
 * A component from this blueprint can handle just one coin; we will create multiple components from
 * this blueprint to deposit multiple coins in WEFT.
 *
 * Instead of using a Vault to hold tokens this blueprint uses an Account so that the WEFT
 * incentive system is happy. */

static NON_FUNGIBLES_PER_WITHDRAW: u32 = 100;

#[blueprint_with_traits]
mod weft_wrapper {

    // Main WEFT Finance blueprint
    extern_blueprint! {
        "package_tdx_2_1pk02rsgrec4dv3fhtw2ltmy3g80325wlusl76tjwhjpj48qtk8c80n",
        LendingPool {
            fn deposit(&mut self, buckets: Vec<Bucket>) -> Vec<Bucket>;
            fn withdraw(&mut self, buckets: Vec<Bucket>) -> Vec<Bucket>;
            fn get_deposit_unit_ratio(&mut self, resources: IndexSet<ResourceAddress>) -> IndexMap<ResourceAddress, Option<PreciseDecimal>>;
        }
    }

    // Blueprint that handles WEFT Finance incentives
    extern_blueprint! {
        "package_tdx_2_1p4qm5qpdj4tt3fr6cl0cqsmmk4h8ag2y0450zzvvzaqyyrf6a9e4p7",
        WeftTokenClaimer {
            fn claim(&mut self, claim_type: u8, amount: Decimal, proof: Proof) -> Bucket;
        }
    }

    enable_method_auth! {
        roles {
            fund_manager => updatable_by: []; // The fund_manager component
            bot => updatable_by: [fund_manager]; // The backend
            admin => updatable_by: [fund_manager]; // Any admin
        },
        methods {
            // DefiProtocolInterfaceTrait implementation
            deposit_all => restrict_to: [fund_manager];
            withdraw_all => restrict_to: [fund_manager];
            deposit_coin => restrict_to: [fund_manager];
            withdraw_coin => restrict_to: [fund_manager];
            withdraw_account_badge => restrict_to: [fund_manager];

            // Admin callable methods
            deposit_account_badge => restrict_to: [admin];
            whithdraw_unexpected_coin => restrict_to: [admin];

            // Collect WEFT incentives (WEFT coins)
            get_incentives => restrict_to: [bot];

            // Return coin and WEFT coin amounts
            get_coin_amounts => PUBLIC;
        }
    }

    struct WeftWrapper {
        coin_address: ResourceAddress, // Example coin: xUSDC
        token_address: ResourceAddress, // Example token: w2-xUSDC
        weft_claimer_nft_address: ResourceAddress, // The badge used to collect incentives

        account: Global<Account>, // Account to hold tokens and incentives
        account_badge_vault: NonFungibleVault, // Badge to manage the Account

        weft_coin_address: ResourceAddress, // Resource address of the WEFT coin

        component_address: Global<LendingPool>, // Main WEFT component
        claimer_component_address: Global<WeftTokenClaimer>, // WEFT component that manages
                                                             // incentives
    }

    impl WeftWrapper {

        // Component constuctor
        pub fn new(
            coin_address: ResourceAddress, // Example coin: xUSDC
            token_address: ResourceAddress, // Example token: w2-xUSDC
            weft_coin_address: ResourceAddress, // Resource address of the WEFT coin
            weft_claimer_nft_address: ResourceAddress, // The address of the badge used to collect
                                                       // incentives
            component_address: Global<LendingPool>, // Main WEFT component
            claimer_component_address: Global<WeftTokenClaimer>, // WEFT component that handles
                                                                 // incentives
            fund_manager_badge_address: ResourceAddress, // God's badge
            admin_badge_address: ResourceAddress, // Admins' badge
            bot_badge_address: ResourceAddress, // Backend's badge
            account: Global<Account>, // The account that will be used as a vault
            account_badge: NonFungibleBucket, // The badge to manage the account
        ) -> Global<WeftWrapper> {

            // Instantiate the component, globalize and return it
            Self {
                account: account,
                account_badge_vault: NonFungibleVault::with_bucket(account_badge),
                coin_address: coin_address,
                token_address: token_address,
                weft_coin_address: weft_coin_address,
                component_address: component_address,
                claimer_component_address: claimer_component_address,
                weft_claimer_nft_address: weft_claimer_nft_address,
            }
                .instantiate()
                .prepare_to_globalize(OwnerRole::Fixed(rule!(require(admin_badge_address))))
                .roles(roles!(
                    fund_manager => rule!(require(fund_manager_badge_address));
                    admin => rule!(require(admin_badge_address));
                    bot => rule!(require(bot_badge_address));
                ))
                .globalize()
        }

        /* The backend can call this method to make the component collect incentives
         * The incentive_type for liquididy providers is 4
         * The amount of incentives to collect can be obtained from the Claimer NFT non fungible
         * data */
        pub fn get_incentives(&mut self, incentive_type: u8, amount: Decimal) {
            // The Account needs badge authentication to create the proof of the Claimer badge it
            // holds
            let claimer_proof = self.account_badge_vault.authorize_with_non_fungibles(
                &self.account_badge_vault.non_fungible_local_ids(1),
                || self.account.create_proof_of_non_fungibles(
                    self.weft_claimer_nft_address,
                    self.account.non_fungible_local_ids(
                        self.weft_claimer_nft_address,
                        1
                    )
                )
            );

            // Get incentives and deposit them in the Account
            self.account.try_deposit_or_abort(
                self.claimer_component_address.claim(
                    incentive_type,
                    amount,
                    claimer_proof.into()
                ),
                None
            );
        }


        fn get_token_coin_ratio(&mut self) -> Decimal {
            let mut resources = IndexSet::new();
            resources.insert(self.coin_address);

            self.component_address.get_deposit_unit_ratio(resources)
                .get(&self.token_address)
                .unwrap()
                .unwrap()
                .checked_truncate(RoundingMode::ToZero)
                .unwrap()
        }

        // Withdraw any unexpected fungible or non fungible from the account
        pub fn whithdraw_unexpected_coin(
            &mut self,
            coin_address: ResourceAddress,
        ) -> Bucket {
            assert!(
                coin_address != self.coin_address &&
                coin_address != self.token_address &&
                coin_address != self.weft_claimer_nft_address &&
                coin_address != self.weft_coin_address,
                "You can't withdraw this coin",
            );

            match coin_address.is_fungible() {
                true => {
                    let (bucket, _) = self.take_from_account(
                        coin_address,
                        Decimal::MAX,
                    );

                    bucket
                }
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

        // Give the control of the Account back to the component
        pub fn deposit_account_badge(&mut self, badge_bucket: NonFungibleBucket) {
            assert!(
                self.account_badge_vault.amount() == Decimal::ZERO && badge_bucket.amount() == Decimal::ONE,
                "Only one badge can be deposited",
            );

            self.account_badge_vault.put(badge_bucket);
        }
    }

    impl DefiProtocolInterfaceTrait for WeftWrapper {

        // Use this method to deposit tokens and coins in this component
        fn deposit_all(
            &mut self,
            token: Bucket,
            coin: Option<FungibleBucket>,
            other_coin: Option<FungibleBucket>,
        ) -> (
            Decimal,                // Total coin amount
            Option<Decimal>         // Total WEFT coin amount
        ) {
            self.account.try_deposit_or_abort(token, None);

            if coin.is_some() {
                self.deposit_coin(coin.unwrap(), other_coin, None, None)
            } else if other_coin.is_some() {
                self.deposit_coin(FungibleBucket::new(self.coin_address), other_coin, None, None)
            } else {
                self.get_coin_amounts()
            }
        }

        // Use this method to withdraw all of the protocol tokens and coins from this component
        fn withdraw_all(
            &mut self,
        ) -> (
            Bucket,                 // Tokens
            Option<FungibleBucket>, // Coins
            Option<FungibleBucket>  // WEFT coins
        ) {

            // Withdraw tokens
            let (token_bucket, _) = self.take_from_account(
                self.token_address,
                Decimal::MAX,
            );

            // Withdraw WEFT coins
            let (weft_bucket, _) = self.take_from_account(
                self.weft_coin_address,
                Decimal::MAX,
            );

            (
                token_bucket,
                None,
                Some(FungibleBucket(weft_bucket))
            )
        }

        // The fund_manager invokes this method to deposit coins in the WEFT protocol.
        // It is also possible to deposit WEFT coins (other_coin) although the fund_manager is not
        // supposed to do that.
        fn deposit_coin(
            &mut self,
            coin: FungibleBucket, // The coin to provide to the WEFT protocol
            other_coin: Option<FungibleBucket>, // Eventual WEFT coins
            _message: Option<String>, // Unused
            _signature: Option<String>, // Unused
        ) -> (
            Decimal,                // Total coin amount
            Option<Decimal>         // Total WEFT coin amount
        ) {
            // Pass the bucket of coins to the WEFT component and expect a vector conteining a
            // bucket of tokens
            let token_bucket = self.component_address.deposit(
                vec![coin.into()]
            )
                .pop()
                .unwrap();

            // Deposit the tokens  in the Account
            self.account.try_deposit_or_abort(token_bucket.into(), None);

            // If other_coin is provided, ensure that this is a bucket of WEFT and deposit them in
            // the Account
            match other_coin {
                None => {},
                Some(bucket) => {
                    assert!(
                        bucket.resource_address() == self.weft_coin_address,
                        "Unknown coin"
                    );

                    self.account.try_deposit_or_abort(bucket.into(), None);
                },
            }

            self.get_coin_amounts()
        }

        // The fund_manager invokes this method when an user wants to exchange his fund units.
        // This method returns a bucket of coins and eventually a bucket of WEFT coins.
        // If available the WEFT coins will be preferred.
        fn withdraw_coin(
            &mut self,
            mut amount: Decimal,    // Amount of coins to withdraw
            other_coin_to_coin_price_ratio: Option<Decimal>, // WEFT/coin price ratio (this is
                                                             // used to return an equivalent amount
                                                             // of WEFT coins instead of coins
        ) -> (
            FungibleBucket, // Coin bucket
            Option<FungibleBucket>, // Eventual WEFT coin bucket
            Decimal,                // Total coin amount
            Option<Decimal>         // Total WEFT coin amount
        ) {

            // Get WEFT coins up to a value equivalent to amount coins
            let (weft_bucket, remaining_weft) = self.take_from_account(
                self.weft_coin_address,
                amount / other_coin_to_coin_price_ratio.unwrap()
            );

            // Update the amount to be withdrawn
            amount -= weft_bucket.amount() * other_coin_to_coin_price_ratio.unwrap();

            let token_coin_ratio = self.get_token_coin_ratio();

            // Get tokens up to a value equivalent to amount
            let (token_bucket, remaining_tokens) = self.take_from_account(
                self.token_address,
                amount * token_coin_ratio
            );

            // Send the tokens to the WEFT component to get the coins
            let coin_bucket = self.component_address.withdraw(
                vec![token_bucket]
            )
                .pop()
                .unwrap();

            // Return all of the coins and WEFT coins
            (
                FungibleBucket(coin_bucket),
                Some(FungibleBucket(weft_bucket)),
                remaining_tokens / token_coin_ratio,
                Some(remaining_weft),
            )
        }

        // Get the number of available coins and WEFT coins
        fn get_coin_amounts(&mut self) -> (
            Decimal,                // Total coin amount
            Option<Decimal>         // Total WEFT coin amount
        ) {
            (
                self.account.balance(self.token_address) / self.get_token_coin_ratio(),
                Some(self.account.balance(self.weft_coin_address))
            )
        }

        // Get the control of the Account
        fn withdraw_account_badge(&mut self) -> NonFungibleBucket {
            self.account_badge_vault.take_non_fungible(
                &self.account_badge_vault.non_fungible_local_id()
            )
        }
    }
}
