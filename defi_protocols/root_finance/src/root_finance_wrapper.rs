use scrypto::prelude::*;
use crate::common::*;
use scrypto_interface::*;

// Non fungible data in the Root receipt
#[derive(ScryptoSbor, Debug)]
enum CDPType {
    Standard
}
#[derive(ScryptoSbor, NonFungibleData, Debug)]
struct CollaterizedDebtPositionData {
    key_image_url: String,
    name: String,
    description: String,
    minted_at: i64,
    updated_at: i64,
    cdp_type: CDPType,
    collaterals: IndexMap<ResourceAddress, PreciseDecimal>,
    loans: IndexMap<ResourceAddress, PreciseDecimal>,
    liquidable: Option<Decimal>
}

// How many NFTs can be withdrawn in a single operation
static NON_FUNGIBLES_PER_WITHDRAW: u32 = 100;

// This blueprint is a wrapper to talk to the Root Finance protocol using the DefiProtocolInterface
// interface.
// Here "coin" is the coin used to provide liquidity to the protocol; to
// provide multiple coins, multiple instances of this component are required.
// "other coin" is not used.
// "token" is the Root receipt: an NFT representing the added liquidity
#[blueprint_with_traits]
mod root_finance_wrapper {

    extern_blueprint! {
        "package_tdx_2_1pk07dw324vwcvr75dk2p39yjt33jc6ndvd5zmn8p5k66a6zwjshdnr",
        LendingMarket {
            fn remove_collateral(
                &mut self,
                cdp_proof: Proof,
                withdraw_details:
                Vec<(ResourceAddress, Decimal, bool)>
            ) -> Vec<Bucket>;

            fn add_collateral(&mut self, cdp_proof: Proof, deposits: Vec<Bucket>);

            fn create_cdp(
                &mut self,
                _name: Option<String>,
                _description: Option<String>,
                _key_image_url: Option<String>,
                deposits: Vec<Bucket>,
            ) -> Bucket;
        }
    }

    enable_method_auth! {
        roles {
            fund_manager => updatable_by: [];
            admin => updatable_by: [fund_manager];
        },
        methods {
            // FundManager callable methods
            deposit_all => restrict_to: [fund_manager];
            withdraw_all => restrict_to: [fund_manager];
            deposit_coin => restrict_to: [fund_manager];
            withdraw_coin => restrict_to: [fund_manager];
            withdraw_account_badge => restrict_to: [fund_manager];

            // Single admin callable methods
            deposit_account_badge => restrict_to: [admin];
            whithdraw_unexpected_coin => restrict_to: [admin];

            // TODO: how to manage root points?

            // Public method
            get_coin_amounts => PUBLIC;
        }
    }

    struct RootFinanceWrapper {
        coin_address: ResourceAddress,  // The coin managed by the Root Finance component
        token_address: ResourceAddress, // The Root receipt representing the added liquidity
        account: Global<Account>, // The account to hold the Root receipt and eventual incentives
        account_badge_vault: NonFungibleVault,      // Badge to manage the Account
        component_address: Global<LendingMarket>,   // Root Finance component
    }

    impl RootFinanceWrapper {

        // Instantiate a RootFinanceWrapper component
        pub fn new(
            coin_address: ResourceAddress,              // Example coin: xUSDC
            token_address: ResourceAddress,             // Root receipt address
            account: Global<Account>,                   // The account to hold the Root receipt
            account_badge: NonFungibleBucket,           // Badge to manage the Account
            component_address: Global<LendingMarket>,   // Root finance component
            fund_manager_badge_address: ResourceAddress,    // God's badge
            admin_badge_address: ResourceAddress,       // Admins' badge
        ) -> Global<RootFinanceWrapper> {

            // Instantiate and globalize the component
            Self {
                coin_address: coin_address,
                token_address: token_address,
                account: account,
                account_badge_vault: NonFungibleVault::with_bucket(account_badge),
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
            coin_address: ResourceAddress, // The fungible or non fungible to withdraw
        ) -> Bucket {

            // Make sure the admin isn't stealing from the fund
            assert!(
                coin_address != self.token_address,
                "You can't withdraw this coin",
            );

            match coin_address.is_fungible() {

                // If the resource to withdraw is fungible, withdraw the whole available amount
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

                // If the resource is not fungible, withdraw up to NON_FUNGIBLES_PER_WITHDRAW NFTs
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

        // Private method to create a proof of the Root receipt in the account
        fn create_root_receipt_proof(&self) -> Proof {

            // Get the non fungible local id of the Root receipt
            let ids = self.account.non_fungible_local_ids(
                self.token_address,
                1,
            );

            // The account badge is needed to create a proof of the NFT in the account
            self.account_badge_vault.authorize_with_non_fungibles(
                &self.account_badge_vault.non_fungible_local_ids(1),
                || self.account.create_proof_of_non_fungibles(
                    self.token_address,
                    ids
                )
            )
                .into()
        }

        // Private method to get information about the investmaent
        fn root_receipt_non_fungible_data(&self) -> CollaterizedDebtPositionData {
            let id = self.account.non_fungible_local_ids(
                self.token_address,
                1,
            )[0].clone();

            NonFungibleResourceManager::from(self.token_address)
                .get_non_fungible_data::<CollaterizedDebtPositionData>(&id)
        }
    }

    impl DefiProtocolInterfaceTrait for RootFinanceWrapper {

        // Deposit a Root receipt and eventually coins
        fn deposit_all(
            &mut self,
            token: Bucket,                          // Root receipt bucket
            coin: Option<FungibleBucket>,           // Coin bucket
            _other_coin: Option<FungibleBucket>,    // Not supported
        ) -> (
            Decimal,                // Total coin amount
            Option<Decimal>         // None
        ) {
            assert!(
                self.account.balance(self.token_address) == Decimal::ZERO,
                "There's already a Root receipt in the account",
            );

            // Deposit the Root receipt
            self.account.try_deposit_or_abort(token, None);

            // Deposit eventual coin in the Root component
            if coin.is_some() {
                self.deposit_coin(coin.unwrap(), None, None, None)
            } else {
                self.get_coin_amounts()
            }
        }

        // Withdraw the Root receipt
        fn withdraw_all(
            &mut self,
        ) -> (
            Bucket,                 // Root receipt
            Option<FungibleBucket>, // None
            Option<FungibleBucket>, // None
        ) {
            // If there's no Root receipt in the Account, there's nothing to withdraw
            if self.account.balance(self.token_address) == Decimal::ZERO {
                return (Bucket::new(self.token_address), None, None);
            }

            // Get the non fungible local id of the Root receipt
            let ids = self.account.non_fungible_local_ids(
                self.token_address,
                1,
            );

            // Get the Root receipt from the Account
            let token_bucket = self.account_badge_vault.authorize_with_non_fungibles(
                &self.account_badge_vault.non_fungible_local_ids(1),
                || self.account.withdraw_non_fungibles(
                    self.token_address,
                    ids
                )
            );

            (
                token_bucket.into(),
                None,
                None,
            )
        }

        // Invest coin in the Root Finance component
        fn deposit_coin(
            &mut self,
            coin: FungibleBucket,                   // Coins to invest
            _other_coin: Option<FungibleBucket>,    // Not supperted
            _message: Option<String>,               // Not used
            _signature: Option<String>,             // Not used
        ) -> (
            Decimal,                // Total coin amount
            Option<Decimal>         // None
        ) {
            if self.account.balance(self.token_address) == Decimal::ZERO {

                let coin_amount = coin.amount();

                // If there's no Root receipt in the Account, we have to mint one and deposit it in
                // the Account
                self.account.try_deposit_or_abort(
                    self.component_address.create_cdp(
                        None,
                        None,
                        None,
                        vec![coin.into()],
                    ),
                    None
                );
           
                (coin_amount, None)

            } else {

                // Create a Root receipt proof
                let proof = self.create_root_receipt_proof();

                // Add collateral associated to the Root receipt position
                self.component_address.add_collateral(
                    proof.into(),
                    vec![coin.into()],
                );

                // Return the total number of coin invested
                self.get_coin_amounts()
            }
        }

        // Get coins out of the Root Finance component
        fn withdraw_coin(
            &mut self,
            mut amount: Decimal,                                // Coin amount to withdraw
            _other_coin_to_coin_price_ratio: Option<Decimal>,   // Not used
        ) -> (
            FungibleBucket,         // Coin bucket
            Option<FungibleBucket>, // None
            Decimal,                // Remaining coin amount
            Option<Decimal>         // None
        ) {
            // If there's no Root receipt, there are no invested coins
            if self.account.balance(self.token_address) == Decimal::ZERO {
                return (
                    FungibleBucket::new(self.coin_address),
                    None,
                    Decimal::ZERO,
                    None
                );
            }

            // Create a Root receipt proof
            let proof = self.create_root_receipt_proof();

            // Read the available amount of coins from the Root receipt non fungible data
            let non_fungible_data = self.root_receipt_non_fungible_data();
            let available_amount = non_fungible_data.collaterals.get_index(0)
                .expect("No coins in this Root receipt")
                .1
                .checked_truncate(RoundingMode::ToNegativeInfinity)
                .unwrap();

            // It's not possible to withdraw more than the whole available amount
            if amount > available_amount {
                amount = available_amount;
            }

            // Get back the coins from the Root component
            let coin_bucket = self.component_address.remove_collateral(
                proof.into(),
                vec![(
                    self.coin_address,
                    amount,
                    false
                )]
            )
                .pop()
                .unwrap();

            (
                FungibleBucket(coin_bucket),
                None,
                available_amount - amount,
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

        // Get the numebr of coins that can be withdrawn from this component
        fn get_coin_amounts(&mut self) -> (
            Decimal,                // Total coin amount
            Option<Decimal>         // None
        ) {
            // If there's no Root receipt, there are no invested coins
            if self.account.balance(self.token_address) == Decimal::ZERO {
                return (Decimal::ZERO, None);
            }

            let non_fungible_data = self.root_receipt_non_fungible_data();

            match non_fungible_data.collaterals.len() {
                0 => (Decimal::ZERO, None),

                1 => {
                    let (address, amount) = non_fungible_data.collaterals.get_index(0).unwrap();
    
                    assert!(
                        *address == self.coin_address,
                        "The Root receipt contains a different coin from the one managed by this wrapper"
                    );

                    (amount.checked_truncate(RoundingMode::ToNegativeInfinity).unwrap(), None)
                },

                _ => Runtime::panic("Multiple coins in the Root receipt".to_string()),
            }
        }
    }
}
