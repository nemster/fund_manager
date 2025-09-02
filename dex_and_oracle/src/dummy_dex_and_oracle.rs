#![allow(unused_variables)]

use scrypto::prelude::*;
use crate::common::*;
use scrypto_interface::*;

#[derive(ScryptoSbor, Debug)]
struct Coin {
    vault: Vault,
    price: Decimal,
}

#[blueprint_with_traits]
mod dummy_dex_and_oracle {

    enable_method_auth! {
        roles {
            fund_manager => updatable_by: [];
        },
        methods {
            swap => restrict_to: [fund_manager];
            deposit => PUBLIC;
            get_price => PUBLIC;
            set_price => PUBLIC;
        }
    }

    struct DummyDexAndOracle {
        coins: KeyValueStore<ResourceAddress, Coin>,
    }

    impl DummyDexAndOracle {

        pub fn new(
            fund_manager_badge_address: ResourceAddress,
            admin_badge_address: ResourceAddress,
        ) -> Global<DummyDexAndOracle> {

            // Instantiate and globalize the component
            Self {
                coins: KeyValueStore::new(),
            }
                .instantiate()
                .prepare_to_globalize(OwnerRole::Fixed(rule!(require(admin_badge_address))))
                .roles(roles!(
                    fund_manager => rule!(require(fund_manager_badge_address));
                ))
                .globalize()
        }

        pub fn deposit(
            &mut self,
            bucket: Bucket
        ) {
            let coin = self.coins.get_mut(&bucket.resource_address());

            match coin {
                None => {
                    drop(coin);

                    self.coins.insert(
                        bucket.resource_address(),
                        Coin {
                            vault: Vault::with_bucket(bucket),
                            price: Decimal::ONE,
                        }
                    );
                },
                Some(mut coin) => coin.vault.put(bucket),
            }
        }

        pub fn set_price(
            &mut self,
            coin_address: ResourceAddress,
            price: Decimal,
        ) {
            let coin = self.coins.get_mut(&coin_address);

            match coin {
                None => {
                    drop(coin);

                    self.coins.insert(
                        coin_address,
                        Coin {
                            vault: Vault::new(coin_address),
                            price: price,
                        }
                    );
                },
                Some(mut coin) => coin.price = price,
            }
        }

    }

    impl DexInterfaceTrait for DummyDexAndOracle {

        fn swap(
            &mut self,
            input_bucket: Bucket,
            output_resource: ResourceAddress,
            _add_remainings: bool,
        ) -> Bucket {
            let input_amount = input_bucket.amount();
            let input_price = self.get_price(
                input_bucket.resource_address(),
                HashMap::new()
            );

            self.deposit(input_bucket);

            let output_price = self.get_price(
                output_resource,
                HashMap::new()
            );
            let output_amount = input_amount * input_price / output_price;

            self.coins.get_mut(&output_resource).unwrap().vault.take(output_amount)
        }

    }

    impl OracleInterfaceTrait for DummyDexAndOracle {

        fn get_price(
            &mut self,
            coin_address: ResourceAddress,
            _morpher_data: HashMap<ResourceAddress, (String, String)>,
        ) -> Decimal {
            let coin = self.coins.get(&coin_address);

            match coin {
                None => Decimal::ONE,
                Some(coin) => coin.price,
            }
        }

    }
}
