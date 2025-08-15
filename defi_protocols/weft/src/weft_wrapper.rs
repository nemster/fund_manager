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
            deposit_protocol_token => restrict_to: [fund_manager];
            withdraw_protocol_token => restrict_to: [fund_manager];
            deposit_coin => restrict_to: [fund_manager];
            withdraw_coin => restrict_to: [fund_manager];
            
            // The fund_manager component will never call these methods, they can only be used in
            // case of an emergency by withdrawing the fund_manager_badge
            withdraw_account_badge => restrict_to: [fund_manager];
            deposit_account_badge => restrict_to: [fund_manager];

            // Withdraw any unexpected coin in the account
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

        // Withdraw any unexpected fungible or non fungible in the account
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
                    let balance = self.account.balance(coin_address);

                    self.account_badge_vault.authorize_with_non_fungibles(
                        &self.account_badge_vault.non_fungible_local_ids(1),
                        || self.account.withdraw(
                            coin_address,
                            balance,
                        )
                    )
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
    }

    impl DefiProtocolInterfaceTrait for WeftWrapper {

        // Use this method to deposit tokens in this component; the fund_manager can do that when
        // the component is registered
        fn deposit_protocol_token(
            &mut self,
            token: Bucket          // Example token: w2-xUSDC
        ) -> (
            Decimal,                // Total coin amount
            Option<Decimal>         // Total WEFT coin amount
        ) {
            self.account.try_deposit_or_abort(token, None);

            self.get_coin_amounts()
        }

        // Use this method to withdraw tokens from this component; the fund_manager can do that
        // when the component is unregistered
        fn withdraw_protocol_token(
            &mut self,
            amount: Option<Decimal>, // The number of tokens to withdraw or None to withdraw all
                                     // of them
        ) -> (
            Bucket,                 // Withdrawn tokens
            Decimal,                // Total coin amount
            Option<Decimal>         // Total WEFT coin amount
        ) {
            // Number of tokens in the Account
            let available_token_amount = self.account.balance(self.token_address);

            let token_bucket = match amount {
                // Withdraw them all
                None => self.account_badge_vault.authorize_with_non_fungibles(
                    &self.account_badge_vault.non_fungible_local_ids(1),
                    || self.account.withdraw(
                        self.token_address,
                        available_token_amount
                    )
                ),
                // Withdraw the specified amount
                Some(mut amount) => {
                    // Make sure that the specified amount is no bigger than the number of tokens
                    // in the Account
                    if amount > available_token_amount {
                        amount = available_token_amount;
                    }

                    self.account_badge_vault.authorize_with_non_fungibles(
                        &self.account_badge_vault.non_fungible_local_ids(1),
                        || self.account.withdraw(
                            self.token_address,
                            amount
                        )
                    )
                },
            };

            let (coin_amount, weft_amount) = self.get_coin_amounts();

            (
                token_bucket,
                coin_amount,
                weft_amount
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
            amount: Option<Decimal>, // Amount of coins to withdraw or None to withdraw them all
            other_coin_to_coin_price_ratio: Option<Decimal>, // WEFT/coin price ratio (this is
                                                             // used to return an equivalent amount
                                                             // of WEFT coins instead of coins
        ) -> (
            FungibleBucket, // Coin bucket
            Option<FungibleBucket>, // Eventual WEFT coin bucket
            Decimal,                // Total coin amount
            Option<Decimal>         // Total WEFT coin amount
        ) {
            // Get the number of available tokens and WEFT coins
            let available_token_amount = self.account.balance(self.token_address);
            let mut available_weft = self.account.balance(self.weft_coin_address);

            let weft_bucket: Bucket;

            match amount {
                // If amount is specified
                Some(mut amount) => {

                    let token_coin_ratio = self.get_token_coin_ratio();
                    let mut available_coin = available_token_amount / token_coin_ratio;

                    // If there are enough WEFT coins to cover the amount equivalent value, those
                    // will be returned togheter with an empty coin bucket
                    if available_weft >= amount / other_coin_to_coin_price_ratio.unwrap() {
                        weft_bucket = self.account_badge_vault.authorize_with_non_fungibles(
                            &self.account_badge_vault.non_fungible_local_ids(1),
                            || self.account.withdraw(
                                self.weft_coin_address,
                                amount / other_coin_to_coin_price_ratio.unwrap()
                            )
                        );

                        available_weft -= weft_bucket.amount();

                        return (
                            FungibleBucket::new(self.coin_address),
                            Some(FungibleBucket(weft_bucket)),
                            available_coin,
                            Some(available_weft)
                        );

                    } else {

                        // There are not enough WEFT coins, get them all
                        weft_bucket = self.account_badge_vault.authorize_with_non_fungibles(
                            &self.account_badge_vault.non_fungible_local_ids(1),
                            || self.account.withdraw(
                                self.weft_coin_address,
                                available_weft
                            )
                        );

                        // Subtract the WEFT coin equivalent value withdrawn from the amount
                        amount -= available_weft * other_coin_to_coin_price_ratio.unwrap();

                        available_weft = Decimal::ZERO;
                    }

                    // Check if the available tokens are enough to cover the requested amount of
                    // coins
                    let token_amount = match amount < available_coin {

                        // if true we get the correct amount of tokens
                        true => amount * token_coin_ratio,

                        // If not we will withdraw all of the tokens
                        false => available_token_amount,
                    };

                    // Get the tokens
                    let token_bucket = self.account_badge_vault.authorize_with_non_fungibles(
                        &self.account_badge_vault.non_fungible_local_ids(1),
                        || self.account.withdraw(self.token_address, token_amount)
                    );

                    // Send the tokens to the WEFT component to get the coins
                    let coin_bucket = self.component_address.withdraw(
                        vec![token_bucket]
                    )
                        .pop()
                        .unwrap();

                    available_coin -= coin_bucket.amount();
                    
                    // Return coins and WEFT coins
                    (
                        FungibleBucket(coin_bucket),
                        Some(FungibleBucket(weft_bucket)),
                        available_coin,
                        Some(available_weft)
                    )
                },

                // If no amount was specified
                None => {

                    // Get all of the tokens from the Account
                    let token_bucket = self.account_badge_vault.authorize_with_non_fungibles(
                        &self.account_badge_vault.non_fungible_local_ids(1),
                        || self.account.withdraw(
                            self.token_address,
                            available_token_amount
                        )
                    );

                    // Send the tokens to the WEFT component to get the coins
                    let coin_bucket = self.component_address.withdraw(
                        vec![token_bucket]
                    )
                        .pop()
                        .unwrap();

                    // Get all of the WEFT coins from the Account
                    weft_bucket = self.account_badge_vault.authorize_with_non_fungibles(
                        &self.account_badge_vault.non_fungible_local_ids(1),
                        || self.account.withdraw(
                            self.weft_coin_address,
                            available_weft
                        )
                    );

                    // Return all of the coins and WEFT coins
                    (
                        FungibleBucket(coin_bucket),
                        Some(FungibleBucket(weft_bucket)),
                        Decimal::ZERO,
                        Some(Decimal::ZERO)
                    )
                },
            }
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
    }
}
