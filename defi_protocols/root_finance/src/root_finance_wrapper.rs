use scrypto::prelude::*;
use crate::common::*;
use scrypto_interface::*;

#[derive(ScryptoSbor)]
enum CDPType {
    Standard
}

#[derive(ScryptoSbor, NonFungibleData)]
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

static NON_FUNGIBLES_PER_WITHDRAW: u32 = 100;

#[blueprint_with_traits]
mod root_finance_wrapper {

    extern_blueprint! {
        "package_tdx_2_1pk07dw324vwcvr75dk2p39yjt33jc6ndvd5zmn8p5k66a6zwjshdnr",
        LendingMarket {
            fn remove_collateral(&mut self, cdp_proof: Proof, withdraw_details: Vec<(ResourceAddress, Decimal, bool)>) -> Vec<Bucket>;
            fn add_collateral(&mut self, cdp_proof: Proof, deposits: Vec<Bucket>);
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

            // TODO: how to manage root points?

            get_coin_amounts => PUBLIC;
        }
    }

    struct RootFinanceWrapper {
        coin_address: ResourceAddress,
        token_address: ResourceAddress,

        account: Global<Account>, // The account to hold the Root receipt and eventual incentives
        account_badge_vault: NonFungibleVault, // Badge to manage the Account

        component_address: Global<LendingMarket>,
    }

    impl RootFinanceWrapper {

        pub fn new(
            coin_address: ResourceAddress, // Example coin: xUSDC
            token_address: ResourceAddress, // Root receipt
            account: Global<Account>, // The account to hold the Root receipt
            account_badge: NonFungibleBucket, // Badge to manage the Account
            component_address: Global<LendingMarket>,
            fund_manager_badge_address: ResourceAddress,
            admin_badge_address: ResourceAddress,
        ) -> Global<RootFinanceWrapper> {
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
                coin_address != self.coin_address &&
                coin_address != self.token_address,
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

        fn create_root_receipt_proof(&self) -> Proof {
            let ids = self.account.non_fungible_local_ids(
                self.token_address,
                1,
            );

            self.account_badge_vault.authorize_with_non_fungibles(
                &self.account_badge_vault.non_fungible_local_ids(1),
                || self.account.create_proof_of_non_fungibles(
                    self.token_address,
                    ids
                )
            )
                .into()
        }

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

        fn deposit_protocol_token(
            &mut self,
            token: Bucket,
        ) -> (
            Decimal,                // Total coin amount
            Option<Decimal>         // None
        ) {
            assert!(
                self.account.balance(self.token_address) == Decimal::ZERO,
                "There's already a Root receipt in the account",
            );

            self.account.try_deposit_or_abort(token, None);

            self.get_coin_amounts()
        }

        fn withdraw_protocol_token(
            &mut self,
            _amount: Option<Decimal>,
        ) -> (
            Bucket,
            Decimal,                // Total coin amount
            Option<Decimal>         // None
        ) {
            let ids = self.account.non_fungible_local_ids(
                self.token_address,
                1,
            );
            let token_bucket = self.account_badge_vault.authorize_with_non_fungibles(
                &self.account_badge_vault.non_fungible_local_ids(1),
                || self.account.withdraw_non_fungibles(
                    self.token_address,
                    ids
                )
            );

            (
                token_bucket.into(),
                Decimal::ZERO,
                None
            )
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
            let proof = self.create_root_receipt_proof();

            self.component_address.add_collateral(
                proof.into(),
                vec![coin.into()],
            );

            self.get_coin_amounts()
        }

        fn withdraw_coin(
            &mut self,
            mut amount: Option<Decimal>,
            _other_coin_to_coin_price_ratio: Option<Decimal>,
        ) -> (
            FungibleBucket,
            Option<FungibleBucket>,
            Decimal,                // Total coin amount
            Option<Decimal>         // None
        ) {
            let proof = self.create_root_receipt_proof();

            let non_fungible_data = self.root_receipt_non_fungible_data();
            let available_amount = non_fungible_data.collaterals.get_index(0)
                .expect("No coins in this Root receipt")
                .1
                .checked_truncate(RoundingMode::ToNegativeInfinity)
                .unwrap();

            if amount.is_none() {
                amount = Some(available_amount)
            } else if amount.unwrap() > available_amount {
                amount = Some(available_amount);
            }

            let coin_bucket = self.component_address.remove_collateral(
                proof.into(),
                vec![(
                    self.coin_address,
                    amount.unwrap(),
                    false
                )]
            )
                .pop()
                .unwrap();

            (
                FungibleBucket(coin_bucket),
                None,
                available_amount - amount.unwrap(),
                None
            )
        }

        fn get_coin_amounts(&mut self) -> (
            Decimal,                // Total coin amount
            Option<Decimal>         // None
        ) {
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
