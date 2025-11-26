use scrypto::prelude::*;
use crate::common::*;
use scrypto_interface::*;

// How many NFTs can be withdrawn in a single operation
static NON_FUNGIBLES_PER_WITHDRAW: u32 = 100;

// This blueprint is a wrapper to deposit and withdraw liquidity in the HyperStake pool
// using the DefiProtocolInterface interface.
// Here "coin" is always LSULP and "other coin" is XRD.
// "token" is the HLP coin.
#[blueprint_with_traits]
mod hyperstake_wrapper {

    extern_blueprint! {
        "package_rdx1pk7qn3gm9g7s6ss93xgvmytua5awt7ujqkpmcse93zn4dvfel7s8rh",
        HyperStake {
            fn add_liquidity(&mut self, token_x: Bucket, token_y: Bucket) -> (Bucket, Option<Bucket>);
            fn remove_liquidity(&mut self, token_lp: Bucket) -> (Bucket, Bucket);
            fn swap(&mut self, input_token: Bucket) -> (Bucket, Bucket);
            fn get_redemption_value(&self, amount: Decimal) -> IndexMap<ResourceAddress, Decimal>;
            fn get_oracle_price(&self) -> Decimal;
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

    struct HyperstakeWrapper {
        lsulp_address: ResourceAddress,
        hlp_address: ResourceAddress,
        account: Global<Account>,               // Account used to store LP tokens and eventual
                                                // LSULP and XRD remainings
        account_badge_vault: NonFungibleVault,  // Badge to manage the Account
        component_address: Global<HyperStake>,  // HyperStake component
    }

    impl HyperstakeWrapper {

        // Instantiate and globalize an HyperstakeWrapper component
        pub fn new(
            lsulp_address: ResourceAddress,
            hlp_address: ResourceAddress,
            account: Global<Account>,
            account_badge_bucket: NonFungibleBucket,
            component_address: Global<HyperStake>,
            fund_manager_badge_address: ResourceAddress,
            admin_badge_address: ResourceAddress,
        ) -> Global<HyperstakeWrapper> {

            // Instantiate and globalize the component
            Self {
                lsulp_address: lsulp_address,
                hlp_address: hlp_address,
                account: account,
                account_badge_vault: NonFungibleVault::with_bucket(account_badge_bucket),
                component_address: component_address,
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
                coin_address != self.lsulp_address &&
                coin_address != XRD &&
                coin_address != self.hlp_address,
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
    }

    impl DefiProtocolInterfaceTrait for HyperstakeWrapper {

        // Deposit all coins in the Account.
        fn deposit_all(
            &mut self,
            token: Bucket,                          // HLP bucket
            coin: Option<FungibleBucket>,           // LSULP bucket
            other_coin: Option<FungibleBucket>,     // XRD bucket
        ) -> (
            Decimal,            // Available LSULP amount
            Option<Decimal>     // Available XRD amount
        ) {
            if coin.is_some() || other_coin.is_some() {

                // If multiple buckets must to be deposited, create a vector and deposit them in a
                // single Account operation
                let mut buckets = vec![token];

                if coin.is_some() {
                    assert!(
                        coin.as_ref().unwrap().resource_address() == self.lsulp_address,
                        "Wrong x coin provided"
                    );

                    buckets.push(coin.unwrap().into());
                }

                if other_coin.is_some() {
                    assert!(
                        other_coin.as_ref().unwrap().resource_address() == XRD,
                        "Wrong y coin provided"
                    );

                    buckets.push(other_coin.unwrap().into());
                }

                self.account.try_deposit_batch_or_abort(buckets, None);
            } else {
                assert!(
                    token.resource_address() == self.hlp_address,
                    "Wrong token provided"
                );

                // Deposit just the liquidity receipt
                self.account.try_deposit_or_abort(token, None);
            }

            self.get_coin_amounts()
        }

        // Withdraw all coins from the Account.
        fn withdraw_all(
            &mut self,
        ) -> (
            Bucket,                     // HLP
            Option<FungibleBucket>,     // LSULP
            Option<FungibleBucket>      // XRD
        ) {

            let (hlp_bucket, _) = self.take_from_account(
                self.hlp_address,
                Decimal::MAX,
            );

            let (lsulp_bucket, _) = self.take_from_account(
                self.lsulp_address,
                Decimal::MAX,
            );

            let (xrd_bucket, _) = self.take_from_account(
                XRD,
                Decimal::MAX,
            );

            (
                hlp_bucket,
                Some(FungibleBucket(lsulp_bucket)),
                Some(FungibleBucket(xrd_bucket)),
            )
        }

        // Deposit LSULP and/or XRD in the pool
        fn deposit_coin(
            &mut self,
            mut coin: FungibleBucket,               // LSULP
            mut other_coin: Option<FungibleBucket>, // XRD
            _message: Option<String>,
            _signature: Option<String>,
        ) -> (
            Decimal,                                // Available LSULP
            Option<Decimal>                         // Available XRD
        ) {

            // Check that the correct coins have been provided
            assert!(
                coin.resource_address() == self.lsulp_address,
                "Wrong coin provided"
            );
            if other_coin.is_some() {
                assert!(
                    other_coin.as_ref().unwrap().resource_address() == XRD,
                    "Wrong other coin provided"
                );
            } else {
                other_coin = Some(FungibleBucket::new(XRD));
            }
   
            // Compute the XRD/LSULP coin ratio desired by the pool (it is not dependant on price
            // for this pool)
            let amounts = self.component_address.get_redemption_value(Decimal::ONE);
            let xrd_lsulp_ratio = *amounts.get(&XRD).unwrap() /
                *amounts.get(&self.lsulp_address).unwrap();

            // Get the LSULP price in XRD
            let lsulp_price = self.component_address.get_oracle_price();

            let lsulp_amount = coin.amount();
            let xrd_amount = other_coin.as_ref().unwrap().amount();

            // If no XRD have been provided, swap part of the LSULP for XRD
            if xrd_amount == Decimal::ZERO {

                let (xrd, lsulp) = self.component_address.swap(
                    coin.take(xrd_lsulp_ratio * lsulp_amount / (xrd_lsulp_ratio + lsulp_price))
                        .into()
                );

                other_coin.as_mut().unwrap().put(FungibleBucket(xrd));
                coin.put(FungibleBucket(lsulp));

            // If no LSULP have been provided, swap part of the XRD for LSULP
            } else if lsulp_amount == Decimal::ZERO {

                let (lsulp, xrd) = self.component_address.swap(
                    other_coin.as_mut().unwrap().take(
                        lsulp_price * xrd_amount / (xrd_lsulp_ratio + lsulp_price)
                    ).into()
                );

                coin.put(FungibleBucket(lsulp));
                other_coin.as_mut().unwrap().put(FungibleBucket(xrd));
            }

            let (hlp, remainings) = self.component_address.add_liquidity(
                coin.into(),
                other_coin.unwrap().into()
            );
            match remainings {
                Some(remainings) => {
                    self.account.try_deposit_batch_or_abort(vec![hlp, remainings], None);
                },
                None => {
                    self.account.try_deposit_or_abort(hlp, None);
                },
            }

            // Return available coins
            self.get_coin_amounts()
        }

        // Withdraw the specified amount of coin (or an equivalent value of y coin)
        fn withdraw_coin(
            &mut self,
            mut amount: Decimal,                                // LSULP amount to withdraw
            other_coin_to_coin_price_ratio: Option<Decimal>,    // XRD/LSULP price ratio
        ) -> (
            FungibleBucket,                                     // LSULP bucket
            Option<FungibleBucket>,                             // XRD bucket
            Decimal,                                            // remaining LSULP amount
            Option<Decimal>                                     // remaining XRD amount
        ) {
            // Take up to amount LSULP coins from the Account and update the remaining value to
            // withdraw accordingly
            let (mut lsulp_bucket, mut remaining_lsulp_amount) = self.take_from_account(
                self.lsulp_address,
                amount,
            );
            amount -= lsulp_bucket.amount();

            // Take up to the XRD equivalent of amount LSULP from the Account
            let (mut xrd_bucket, mut remaining_xrd_amount) = self.take_from_account(
                XRD,
                amount / other_coin_to_coin_price_ratio.unwrap(),
            );
            amount -= xrd_bucket.amount() * other_coin_to_coin_price_ratio.unwrap();

            let hlp_amount = self.account.balance(self.hlp_address);
            if hlp_amount > Decimal::ZERO {

                let amounts = self.component_address.get_redemption_value(hlp_amount);
    
                let lsulp_deposited_amount = *amounts.get(&self.lsulp_address).unwrap();
                let xrd_deposited_amount = *amounts.get(&XRD).unwrap();

                remaining_lsulp_amount += lsulp_deposited_amount;
                remaining_xrd_amount += xrd_deposited_amount;

                if amount > Decimal::ZERO {

                    let total_deposited_lsulp_value = 
                        lsulp_deposited_amount +
                        xrd_deposited_amount * other_coin_to_coin_price_ratio.unwrap();

                    let (hlp_bucket, _) = self.take_from_account(
                        self.hlp_address,
                        amount * hlp_amount / total_deposited_lsulp_value
                    );

                    let (lsulp, xrd) = self.component_address.remove_liquidity(hlp_bucket);

                    remaining_lsulp_amount -= lsulp.amount();
                    remaining_xrd_amount -= xrd.amount();

                    lsulp_bucket.put(lsulp);
                    xrd_bucket.put(xrd);
                }
            }

            // Return all buckets and info
            (
                FungibleBucket(lsulp_bucket),
                Some(FungibleBucket(xrd_bucket)),
                remaining_lsulp_amount,
                Some(remaining_xrd_amount)
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

        // Return the number of all available coins, both in the Account and in the pool
        fn get_coin_amounts(&mut self) -> (
            Decimal,                // Total coin amount
            Option<Decimal>         // Total other coin amount
        ) {
            let mut lsulp_amount = self.account.balance(self.lsulp_address);
            let mut xrd_amount = self.account.balance(XRD);

            let hlp_amount = self.account.balance(self.hlp_address);
            if hlp_amount > Decimal::ZERO {
                let amounts = self.component_address.get_redemption_value(hlp_amount);

                lsulp_amount += *amounts.get(&self.lsulp_address).unwrap();
                xrd_amount += *amounts.get(&XRD).unwrap();
            }

            (
                lsulp_amount,
                Some(xrd_amount),
            )
        }
    }
}
