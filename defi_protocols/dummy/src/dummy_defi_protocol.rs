use scrypto::prelude::*;
use crate::common::*;
use scrypto_interface::*;

#[blueprint_with_traits]
mod dummy_defi_protocol {

    enable_method_auth! {
        roles {
            fund_manager => updatable_by: [];
            admin => updatable_by: [fund_manager];
        },
        methods {
            deposit_all => restrict_to: [fund_manager];
            withdraw_all => restrict_to: [fund_manager];
            deposit_coin => restrict_to: [fund_manager];
            withdraw_coin => restrict_to: [fund_manager];
            withdraw_account_badge => restrict_to: [fund_manager];
            get_coin_amounts => PUBLIC;
        }
    }

    struct DummyDefiProtocol {
        token_vault: Vault,
        coin_vault: FungibleVault,
        other_coin_vault: Option<FungibleVault>,
        account_badge_vault: NonFungibleVault,
    }

    impl DummyDefiProtocol {

        pub fn new(
            token_address: ResourceAddress,
            coin_address: ResourceAddress,
            other_coin_address: Option<ResourceAddress>,
            account_badge: NonFungibleBucket,
            fund_manager_badge_address: ResourceAddress,
            admin_badge_address: ResourceAddress,
        ) -> Global<DummyDefiProtocol> {

            Self {
                token_vault: Vault::new(token_address),
                coin_vault: FungibleVault::new(coin_address),
                other_coin_vault: match other_coin_address {
                    None => None,
                    Some(other_coin_address) => Some(FungibleVault::new(other_coin_address)),
                },
                account_badge_vault: NonFungibleVault::with_bucket(account_badge),
            }
                .instantiate()
                .prepare_to_globalize(OwnerRole::Fixed(rule!(require(admin_badge_address))))
                .roles(roles!(
                    fund_manager => rule!(require(fund_manager_badge_address));
                    admin => rule!(require(admin_badge_address));
                ))
                .globalize()
        }
    }

    impl DefiProtocolInterfaceTrait for DummyDefiProtocol {

        fn deposit_all(
            &mut self,
            token: Bucket,
            coin: Option<FungibleBucket>,
            other_coin: Option<FungibleBucket>,
        ) -> (
            Decimal,
            Option<Decimal>
        ) {
            self.token_vault.put(token);

            if coin.is_some() {
                self.coin_vault.put(coin.unwrap());
            }

            if other_coin.is_some() {
                self.other_coin_vault.as_mut().unwrap().put(other_coin.unwrap());
            }

            self.get_coin_amounts()
        }

        fn withdraw_all(
            &mut self,
        ) -> (
            Bucket,
            Option<FungibleBucket>,
            Option<FungibleBucket>
        ) {
            let other_coin_bucket = match self.other_coin_vault.as_mut() {
                None => None,
                Some(other_coin_vault) => Some(other_coin_vault.take_all()),
            };

            (
                self.token_vault.take_all(),
                Some(self.coin_vault.take_all()),
                other_coin_bucket,
            )
        }

        fn deposit_coin(
            &mut self,
            coin: FungibleBucket,
            other_coin: Option<FungibleBucket>,
            _message: Option<String>,
            _signature: Option<String>,
        ) -> (
            Decimal,
            Option<Decimal>
        ) {
            self.coin_vault.put(coin);

            if other_coin.is_some() {
                self.other_coin_vault.as_mut().unwrap().put(other_coin.unwrap());
            }

            self.get_coin_amounts()
        }

        fn withdraw_coin(
            &mut self,
            mut amount: Decimal,
            other_coin_to_coin_price_ratio: Option<Decimal>,
        ) -> (
            FungibleBucket,
            Option<FungibleBucket>,
            Decimal,
            Option<Decimal>
        ) {
            let available_coin_amount = self.coin_vault.amount();

            let coin_bucket = match amount > available_coin_amount {
                true => {
                    amount -= available_coin_amount;
                    self.coin_vault.take(available_coin_amount)
                },
                false => {
                    let coin_bucket = self.coin_vault.take(amount);
                    amount = Decimal::ZERO;
                    coin_bucket
                },
            };

            let (other_coin_bucket, other_coin_amount) = match self.other_coin_vault.as_mut() {
                None => (None, None),
                Some(other_coin_vault) => match amount == Decimal::ZERO {
                    true => (
                        None,
                        Some(other_coin_vault.amount())
                    ),
                    false => {
                        amount /= other_coin_to_coin_price_ratio.unwrap();

                        let available_other_coin_amount = other_coin_vault.amount();

                        match amount > available_coin_amount {
                            true => (
                                Some(other_coin_vault.take(available_other_coin_amount)),
                                Some(Decimal::ZERO)
                            ),
                            false => (
                                Some(other_coin_vault.take(amount)),
                                Some(other_coin_vault.amount())
                            )
                        }
                    },
                },
            };

            (
                coin_bucket,
                other_coin_bucket,
                self.coin_vault.amount(),
                other_coin_amount
            )
        }

        fn withdraw_account_badge(&mut self) -> NonFungibleBucket {
            self.account_badge_vault.take_all()
        }

        fn get_coin_amounts(&mut self) -> (
            Decimal,
            Option<Decimal>
        ) {
            let other_coin_amount = match &self.other_coin_vault {
                None => None,
                Some(other_coin_vault) => Some(other_coin_vault.amount()),
            };

            (
                self.coin_vault.amount(),
                other_coin_amount
            )
        }
    }
}
