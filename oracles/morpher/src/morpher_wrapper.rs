use scrypto::prelude::*;
use crate::common::*;
use scrypto_interface::*;

#[derive(ScryptoSbor)]
struct PriceMessage {
    market_id: String,
    price: Decimal,
    nonce: u64,
    created_at: u64,
}

#[derive(ScryptoSbor)]
struct LastPriceData {
    price: Decimal,
    created_at: u64,
}

#[blueprint_with_traits]
#[types(ResourceAddress, LastPriceData)]
mod morpher_wrapper {

    extern_blueprint! {
        "package_tdx_2_1p58lk25kdv698akrx3tq9dgejmns57530uyyvh8tuan2k3lcetcxhj",
        MorpherOracle {
            fn check_price_input(&mut self, message: String, signature: String) -> PriceMessage;
        }
    }

    enable_method_auth! {
        roles {
            fund_manager => updatable_by: [];
        },
        methods {
           get_price  => restrict_to: [fund_manager];
           update_settings => restrict_to: [OWNER];
           add_market_id => restrict_to: [OWNER];
        }
    }

    struct MorpherWrapper {
        oracle_component: Global<MorpherOracle>,
        market_id_to_resource_address: HashMap<String, ResourceAddress>,
        price_lifetime: u64,
        cache_data: bool,
        last_prices: KeyValueStore<ResourceAddress, LastPriceData>,
    }

    impl MorpherWrapper {

        pub fn new(
            oracle_component: Global<MorpherOracle>,
            market_id_to_resource_address: HashMap<String, ResourceAddress>,
            fund_manager_badge_address: ResourceAddress,
            admin_badge_address: ResourceAddress,
            price_lifetime: u64,
            cache_data: bool,
        ) -> Global<MorpherWrapper> {
            Self {
                oracle_component: oracle_component,
                market_id_to_resource_address: market_id_to_resource_address,
                price_lifetime: price_lifetime,
                cache_data: cache_data,
                last_prices: KeyValueStore::new_with_registered_type(),
            }
                .instantiate()
                .prepare_to_globalize(OwnerRole::Fixed(rule!(require(admin_badge_address))))
                .roles(roles!(
                    fund_manager => rule!(require(fund_manager_badge_address));
                ))
                .globalize()
        }

        pub fn update_settings(
            &mut self,
            price_lifetime: u64,
            cache_data: bool
        ) {
            self.price_lifetime = price_lifetime;
            self.cache_data = cache_data;
        }

        pub fn add_market_id(
            &mut self,
            market_id: String,
            resource_address: ResourceAddress
        ) {
            self.market_id_to_resource_address.insert(
                market_id,
                resource_address
            );
        }
    }

    impl OracleInterfaceTrait for MorpherWrapper {

        fn get_price(
            &mut self,
            coin_address: ResourceAddress,
            message: Option<String>,
            signature: Option<String>,
        ) -> Decimal {
            let now: u64 = Clock::current_time_rounded_to_seconds().seconds_since_unix_epoch.try_into().unwrap();

            if self.cache_data {
                let last_price = self.last_prices.get(&coin_address);

                if last_price.is_some() {
                    if last_price.as_ref().unwrap().created_at + self.price_lifetime <= now {
                        return last_price.unwrap().price;
                    }
                }
            }

            let price_message = self.oracle_component.check_price_input(
                message.unwrap(),
                signature.unwrap(),
            );

            assert!(
                price_message.created_at + self.price_lifetime >= now,
                "This price is out of date!"
            );

            assert!(
                *self.market_id_to_resource_address.get(&price_message.market_id).expect("Unknown market") == coin_address,
                "Mismatched resource address",
            );

            if self.cache_data {
                self.last_prices.insert(
                    coin_address,
                    LastPriceData {
                        price: price_message.price,
                        created_at: price_message.created_at,
                    }
                );
            }

            price_message.price
        }
    }
}
