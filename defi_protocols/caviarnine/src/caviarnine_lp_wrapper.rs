use scrypto::prelude::*;
use crate::common::*;
use scrypto_interface::*;

// How many NFTs can be withdrawn in a single operation
static NON_FUNGIBLES_PER_WITHDRAW: u32 = 100;

#[derive(ScryptoSbor, Debug)]
pub struct ShapePosition(i32, Decimal);

#[derive(ScryptoSbor, NonFungibleData)]
struct LiquidityReceipt {
    liquidity_claims: HashMap<u32, Decimal>,
}

// This blueprint is a wrapper to deposit and withdraw liquidity in QuantaSwap (Caviarnine) pools
// using the DefiProtocolInterface interface.
// Here "coin" and "other coin" are the two tokens managed by the pool; those are also called
// "x_token" and "y_token".
// "token" is the non fungible liquidity receipt returned by the QuantaSwap pools upon deposits.
#[blueprint_with_traits]
mod caviarnine_lp_wrapper {

    extern_blueprint! {
        "package_tdx_2_1p4g09xagmsyql6r65a70c94n6qgvk6ffx9q0z5g3vnqmrsr96627vg",
        QuantaSwap {
            fn add_liquidity(
                &mut self,
                tokens_x: Bucket,
                tokens_y: Bucket,
                positions: Vec<(u32, Decimal, Decimal)>,
            ) -> (
                Bucket,     // Liquidity receipt
                Bucket,     // Tokens x that were not used
                Bucket      // Tokens y that were not used
            );

            fn add_liquidity_to_receipt(
                &mut self,
                liquidity_receipt: Bucket,
                tokens_x: Bucket,
                tokens_y: Bucket,
                positions: Vec<(u32, Decimal, Decimal)>,
            ) -> (
                Bucket,     // Updated liquidity receipt
                Bucket,     // Tokens x that were not used
                Bucket      // Tokens y that were not used
            );

            fn get_active_tick(&self) -> Option<u32>;

            fn get_redemption_value(&self, liquidity_receipt_id: NonFungibleLocalId) -> (
                Decimal,    // Amount of tokens x
                Decimal     // Amount of tokens y
            );

            fn remove_specific_liquidity(&mut self, liquidity_receipt: Bucket, claims: Vec<(u32, Decimal)>) -> (
                Bucket,     // Updated liquidity receipt
                Bucket,     // Tokens x that were removed
                Bucket      // Tokens y that were removed
            );

            fn get_price(&self) -> Option<Decimal>;
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
            update_shape => restrict_to: [admin];

            // Public method
            get_coin_amounts => PUBLIC;
        }
    }

    struct CaviarnineLpWrapper {
        x_address: ResourceAddress,             // First coin managed by the pool
        y_address: ResourceAddress,             // Second coin managed by the pool
        lp_token_address: ResourceAddress,      // LP tokens of the pool
        account: Global<Account>,               // Account used to store LP tokens and eventual x
                                                // and y remainings
        account_badge_vault: NonFungibleVault,  // Badge to manage the Account
        component_address: Global<QuantaSwap>,  // QuantaSwap component
        shape: Vec<ShapePosition>,              // Shape of the positions to deposit
    }

    impl CaviarnineLpWrapper {

        // Instantiate and globalize a CaviarnineLpWrapper component
        pub fn new(
            x_address: ResourceAddress,
            y_address: ResourceAddress,
            lp_token_address: ResourceAddress,
            account: Global<Account>,
            account_badge_bucket: NonFungibleBucket,
            component_address: Global<QuantaSwap>,
            fund_manager_badge_address: ResourceAddress,
            admin_badge_address: ResourceAddress,
            shape: Vec<ShapePosition>,
        ) -> Global<CaviarnineLpWrapper> {

            // Instantiate and globalize the component
            Self {
                x_address: x_address,
                y_address: y_address,
                lp_token_address: lp_token_address,
                account: account,
                account_badge_vault: NonFungibleVault::with_bucket(account_badge_bucket),
                component_address: component_address,
                shape: shape,
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

        // Private method to withdraw fungibles from the Account
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

        pub fn update_shape(
            &mut self,
            shape: Vec<ShapePosition>
        ) {
            self.shape = shape;
        }

    }

    impl DefiProtocolInterfaceTrait for CaviarnineLpWrapper {

        // Deposit LP tokens, x and y coins in the Account.
        fn deposit_all(
            &mut self,
            token: Bucket,                          // Liquidity receipt bucket
            coin: Option<FungibleBucket>,           // X token bucket
            other_coin: Option<FungibleBucket>,     // Y token bucket
        ) -> (
            Decimal,            // Available x token amount
            Option<Decimal>     // Available y token amount
        ) {
            if coin.is_some() || other_coin.is_some() {

                // If multiple buckets must to be deposited, create a vector and deposit them in a
                // single Account operation
                let mut buckets = vec![token];

                if coin.is_some() {
                    assert!(
                        coin.as_ref().unwrap().resource_address() == self.x_address,
                        "Wrong x coin provided"
                    );

                    buckets.push(coin.unwrap().into());
                }

                if other_coin.is_some() {
                    assert!(
                        other_coin.as_ref().unwrap().resource_address() == self.y_address,
                        "Wrong y coin provided"
                    );

                    buckets.push(other_coin.unwrap().into());
                }

                self.account.try_deposit_batch_or_abort(buckets, None);
            } else {
                assert!(
                    token.resource_address() == self.lp_token_address,
                    "Wrong token provided"
                );

                // Deposit just the liquidity receipt
                self.account.try_deposit_or_abort(token, None);
            }

            assert!(
                self.account.balance(self.lp_token_address) <= Decimal::ONE,
                "This wrapper can't manage multiple liquidity receipts"
            );

            // Return the x and y tokens availability
            self.get_coin_amounts()
        }

        // Withdraw liquidity receipt, x and y tokens from the Account.
        fn withdraw_all(
            &mut self,
        ) -> (
            Bucket,                     // Liquidity receipt
            Option<FungibleBucket>,     // X coin
            Option<FungibleBucket>      // Y coin
        ) {

            // Withdraw the liquidity receipt (if exists)
            let id = self.account.non_fungible_local_ids(
                self.lp_token_address,
                1u32,
            );
            let liquidity_receipt_bucket = match id.len() == 1 {
                true => self.account_badge_vault.authorize_with_non_fungibles(
                    &self.account_badge_vault.non_fungible_local_ids(1),
                    || self.account.withdraw_non_fungibles(
                        self.lp_token_address,
                        id,
                    )
                        .into()
                ),
                false => Bucket::new(self.lp_token_address),
            };

            let (x_bucket, _) = self.take_from_account(
                self.x_address,
                Decimal::MAX,
            );

            let (y_bucket, _) = self.take_from_account(
                self.y_address,
                Decimal::MAX,
            );

            (
                liquidity_receipt_bucket,
                Some(FungibleBucket(x_bucket)),
                Some(FungibleBucket(y_bucket)),
            )
        }

        // Deposit x and y tokens in the pool
        fn deposit_coin(
            &mut self,
            coin: FungibleBucket,               // X token
            other_coin: Option<FungibleBucket>, // Y token
            _message: Option<String>,
            _signature: Option<String>,
        ) -> (
            Decimal,                // Available x tokens
            Option<Decimal>         // Available y tokens
        ) {

            // TODO: swap?

            let (mut x_bucket, _) = self.take_from_account(
                self.x_address,
                Decimal::MAX,
            );
            x_bucket.put(coin.into());

            let (mut y_bucket, _) = self.take_from_account(
                self.y_address,
                Decimal::MAX,
            ); 
            if other_coin.is_some() {
                y_bucket.put(other_coin.unwrap().into());
            }

            let active_tick = self.component_address.get_active_tick().unwrap();

            let mut positions = Vec::<(u32, Decimal, Decimal)>::new();

            let mut x_amount = x_bucket.amount();
            let mut y_amount = y_bucket.amount();

            let mut share_in_active_tick = Decimal::ONE;

            if x_amount != Decimal::ZERO && y_amount != Decimal::ZERO {

                let price = self.component_address.get_price().unwrap();

                for shape_position in &self.shape {
                    if shape_position.0 == 0i32 {
                        share_in_active_tick = shape_position.1;
                        break;
                    }
                }

                let (
                    x_amount_in_active_bin,
                    y_amount_in_active_bin
                ) = match y_amount * price > x_amount {
                    true => match x_amount == Decimal::ZERO {
                        true => (Decimal::ZERO, y_amount * share_in_active_tick),
                        false => {
                            let x_amount_in_active_bin = x_amount * share_in_active_tick;

                            (
                                x_amount_in_active_bin,
                                x_amount_in_active_bin * price
                            )
                        },
                    },
                    false => match y_amount == Decimal::ZERO {
                        true => (x_amount * share_in_active_tick, Decimal::ZERO),
                        false => {
                            let y_amount_in_active_bin = y_amount * share_in_active_tick;

                            (
                                y_amount_in_active_bin * price,
                                y_amount_in_active_bin
                            )
                        },
                    },
                };

                x_amount -= x_amount_in_active_bin;
                y_amount -= y_amount_in_active_bin;

                positions.push((active_tick, x_amount_in_active_bin, y_amount_in_active_bin));
            }

            for shape_position in &self.shape {
                if shape_position.0 < 0i32 {
                    if shape_position.0.unsigned_abs() < active_tick {
                        positions.push((
                            active_tick - shape_position.0.unsigned_abs(),
                            Decimal::ZERO,

                            // The division is safe because if share_in_active_tick is 1 there must
                            // be no other ticks in the shape
                            y_amount * shape_position.1 / (Decimal::ONE - share_in_active_tick)
                        ));
                    }
                } else if shape_position.0 > 0i32 {
                    // TODO: how to check that shape_position.0 + active_tick doesn't excced the
                    // maximum tick number?
                    positions.push((
                        active_tick + shape_position.0.unsigned_abs(),
                        x_amount * shape_position.1 / (Decimal::ONE - share_in_active_tick),
                        Decimal::ZERO
                    ));
                }
            }

            let id = self.account.non_fungible_local_ids(
                self.lp_token_address,
                1u32,
            );
            let liquidity_receipt = match id.len() == 1 {
                true => {
                    let mut liquidity_receipt = self.account_badge_vault.authorize_with_non_fungibles(
                        &self.account_badge_vault.non_fungible_local_ids(1),
                        || self.account.withdraw_non_fungibles(
                            self.lp_token_address,
                            id,
                        )
                    )
                        .into();

                    (
                        liquidity_receipt,
                        x_bucket,
                        y_bucket
                    ) = self.component_address.add_liquidity_to_receipt(
                        liquidity_receipt,
                        x_bucket,
                        y_bucket,
                        positions
                    );

                    liquidity_receipt
                },
                false => {
                    let liquidity_receipt: Bucket;

                    (
                        liquidity_receipt,
                        x_bucket,
                        y_bucket
                    ) = self.component_address.add_liquidity(
                        x_bucket,
                        y_bucket,
                        positions
                    );

                    liquidity_receipt
                },
            };


            self.account.try_deposit_batch_or_abort(
                vec![
                    liquidity_receipt,
                    x_bucket,
                    y_bucket
                ],
                None
            );

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

            let id = self.account.non_fungible_local_ids(
                self.lp_token_address,
                1u32,
            );
            if id.len() == 1 {
                let (
                    mut x_amount_in_pool,
                    mut y_amount_in_pool
                ) = self.component_address.get_redemption_value(id[0].clone());

                if amount > Decimal::ZERO {

                    let liquidity_receipt: Bucket = self.account_badge_vault.authorize_with_non_fungibles(
                        &self.account_badge_vault.non_fungible_local_ids(1),
                        || self.account.withdraw_non_fungibles(
                            self.lp_token_address,
                            id,
                        )
                    )
                        .into();

                    let liquidity_claims = liquidity_receipt
                        .as_non_fungible()
                        .non_fungible::<LiquidityReceipt>()
                        .data()
                        .liquidity_claims;

                    // TODO: remove all liquidity from ticks currently out of shape first?

                    let share_to_withdraw =
                        amount / 
                        (x_amount_in_pool +
                        y_amount_in_pool * other_coin_to_coin_price_ratio.unwrap());

                    let mut liquidity_to_withdraw = Vec::<(u32, Decimal)>::new();
                    for (tick, claim) in liquidity_claims {
                        liquidity_to_withdraw.push((
                            tick,
                            claim * share_to_withdraw
                        ));
                    }
 
                    let (
                        liquidity_receipt,
                        x_token_from_pool,
                        y_token_from_pool
                    ) = self.component_address.remove_specific_liquidity(
                        liquidity_receipt,
                        liquidity_to_withdraw
                    );

                    x_amount_in_pool -= x_token_from_pool.amount();
                    y_amount_in_pool -= y_token_from_pool.amount();

                    x_bucket.put(x_token_from_pool);
                    y_bucket.put(y_token_from_pool);

                    self.account.try_deposit_or_abort(liquidity_receipt, None);
                }

                remaining_x_amount += x_amount_in_pool;
                remaining_y_amount += y_amount_in_pool;
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

        // Return the number of all available x and y coins, both in the Account and in the pool
        fn get_coin_amounts(&mut self) -> (
            Decimal,                // Total coin amount
            Option<Decimal>         // Total other coin amount
        ) {
            let mut x_amount = self.account.balance(self.x_address);
            let mut y_amount = self.account.balance(self.y_address);

            let id = self.account.non_fungible_local_ids(
                self.lp_token_address,
                1u32,
            );
            if id.len() == 1 {
                let (
                    x_amount_in_pool,
                    y_amount_in_pool
                ) = self.component_address.get_redemption_value(id[0].clone());
 
                x_amount += x_amount_in_pool;
                y_amount += y_amount_in_pool;
            }

            (
                x_amount,
                Some(y_amount),
            )
        }
    }
}
