use scrypto::prelude::*;
use crate::common::*;
use scrypto_interface::*;
use std::ops::DerefMut;

#[derive(ScryptoSbor)]
struct PriceMessage {
    market_id: String,
    price: Decimal,
    nonce: u64,
    created_at: u64,
}

#[derive(ScryptoSbor)]
pub struct ObservationInterval {
    start: u64,
    end: u64,
    price_sqrt: Decimal,
}

#[derive(ScryptoSbor, Clone)]
pub enum OracleType {
    FixedPrice {
        price: Decimal,
    },
    FixedMultiplier {
        multiplier: Decimal,
        reference_coin: ResourceAddress
    },
    Ociswap {
        component: Global<AnyComponent>,
        reference_coin: ResourceAddress,
        reverse: bool,
    },
    Morpher {
        market_id: String,
    },
}

#[blueprint_with_traits]
#[types(
    ResourceAddress,
    OracleType,
)]
mod multi_oracle_wrapper {

    extern_blueprint! {
        "package_tdx_2_1p58lk25kdv698akrx3tq9dgejmns57530uyyvh8tuan2k3lcetcxhj",
        MorpherOracle {
            fn check_price_input(&mut self, message: String, signature: String) -> PriceMessage;
        }
    }

/*
    extern_blueprint! {
        "package_tdx_2_1pha4sgrwdc09ymqy6e8gpe7h0e652p5l22esxd2t8u82290cjq45ma",
        PrecisionPool {
            fn observation_intervals(&self, intervals: Vec<(u64, u64)>) -> Vec<ObservationInterval>;
        }
    }

    extern_blueprint! {
        "package_tdx_2_1p5qntnqluczzjjnm577mfp7p5jd3qm2sv0qzkqklgkrypcnspw3dff",
        Pool {
            fn observation_intervals(&self, intervals: Vec<(u64, u64)>) -> Vec<ObservationInterval>;
        }
    }
*/

    enable_method_auth! {
        roles {
            fund_manager => updatable_by: [];
            bot => updatable_by: [fund_manager];
        },
        methods {
           get_price  => PUBLIC;
          
           update_price => restrict_to: [bot];

           update_settings => restrict_to: [OWNER];
           add_oracle => restrict_to: [OWNER];
           remove_oracle => restrict_to: [OWNER];
        }
    }

    struct MultiOracleWrapper {
        oracles: KeyValueStore<ResourceAddress, OracleType>,
        morpher_component: Global<MorpherOracle>,
        observation_time: u64,
        price_lifetime: u64,
    }

    impl MultiOracleWrapper {

        pub fn new(
            fund_manager_badge_address: ResourceAddress,
            admin_badge_address: ResourceAddress,
            bot_badge_address: ResourceAddress,
            morpher_component: Global<MorpherOracle>,
            observation_time: u64,
            price_lifetime: u64,
        ) -> Global<MultiOracleWrapper> {
            Self {
                oracles: KeyValueStore::new_with_registered_type(),
                morpher_component: morpher_component,
                observation_time: observation_time,
                price_lifetime: price_lifetime,
            }
                .instantiate()
                .prepare_to_globalize(OwnerRole::Fixed(rule!(require(admin_badge_address))))
                .roles(roles!(
                    fund_manager => rule!(require(fund_manager_badge_address));
                    bot => rule!(require(bot_badge_address));
                ))
                .globalize()
        }

        pub fn update_price(
            &mut self,
            coin_address: ResourceAddress,
            fixed_price: Option<Decimal>,
            fixed_multiplier: Option<Decimal>,
        ) {
            match self.oracles.get_mut(&coin_address).expect("Unknown coin").deref_mut() {
                OracleType::FixedPrice { price } => { *price = fixed_price.unwrap(); },
                OracleType::FixedMultiplier { multiplier, .. } => { *multiplier = fixed_multiplier.unwrap(); },
                _ => Runtime::panic("Can't update this oracle type".to_string()),
            }
        }

        pub fn update_settings(
            &mut self,
            morpher_component: Option<Global<MorpherOracle>>,
            observation_time: Option<u64>,
            price_lifetime: Option<u64>,
        ) {
            if morpher_component.is_some() {
                self.morpher_component = morpher_component.unwrap();
            }

            if observation_time.is_some() {
                self.observation_time = observation_time.unwrap();
            }

            if price_lifetime.is_some() {
                self.price_lifetime = price_lifetime.unwrap();
            }
        }

        pub fn add_oracle(
            &mut self,
            coin_address: ResourceAddress,
            fixed_price: Option<Decimal>,
            fixed_multiplier: Option<Decimal>,
            reference_coin: Option<ResourceAddress>,
            ociswap_component: Option<Global<AnyComponent>>,
            ociswap_reverse: Option<bool>,
            morpher_market_id: Option<String>,
        ) {
            if fixed_price.is_some() {
                self.oracles.insert(
                    coin_address,
                    OracleType::FixedPrice {
                        price: fixed_price.unwrap(),
                    }
                );
            } else if fixed_multiplier.is_some() {
                self.oracles.insert(
                    coin_address,
                    OracleType::FixedMultiplier {
                        multiplier: fixed_multiplier.unwrap(),
                        reference_coin: reference_coin.unwrap(),
                    }
                );
            } else if ociswap_component.is_some() {
                self.oracles.insert(
                    coin_address,
                    OracleType::Ociswap {
                        component: ociswap_component.unwrap(),
                        reference_coin: reference_coin.unwrap(),
                        reverse: ociswap_reverse.unwrap(),
                    }
                );
            } else if morpher_market_id.is_some() {
                self.oracles.insert(
                    coin_address,
                    OracleType::Morpher {
                        market_id: morpher_market_id.unwrap(),
                    }
                );
            } else {
                Runtime::panic("Can't understand oracle type".to_string());
            }
        }

        pub fn remove_oracle(
            &mut self,
            coin_address: ResourceAddress,
        ) {
            self.oracles.remove(&coin_address);
        }

    }

    impl OracleInterfaceTrait for MultiOracleWrapper {

        fn get_price(
            &mut self,
            coin_address: ResourceAddress,
            morpher_data: HashMap<ResourceAddress, (String, String)>,
        ) -> Decimal {
            let oracle = self.oracles.get(&coin_address).expect("Coin not found").clone();

            match oracle {
                OracleType::FixedPrice { price } => price,

                OracleType::FixedMultiplier { multiplier, reference_coin } =>
                    multiplier * self.get_price(reference_coin, morpher_data),

                OracleType::Ociswap { component, reference_coin, reverse } => {
                    let interval_end = Clock::current_time_rounded_to_seconds()
                        .seconds_since_unix_epoch.try_into().unwrap();
                    let intervals = vec![(interval_end - self.observation_time, interval_end)];

                    let price_sqrt = component.call::<Vec<(u64, u64)>, Vec<ObservationInterval>>(
                        "observation_intervals",
                        &intervals
                    )[0].price_sqrt;

                    match reverse {
                        false => self.get_price(reference_coin, morpher_data) * price_sqrt * price_sqrt,
                        true => self.get_price(reference_coin, morpher_data) / (price_sqrt * price_sqrt),
                    }
                },

                OracleType::Morpher { market_id } => {
                    let now: u64 = Clock::current_time_rounded_to_seconds()
                        .seconds_since_unix_epoch.try_into().unwrap();

                    let (message, signature) = morpher_data.get(&coin_address).expect("Missing Morpher data");

                    let price_message = self.morpher_component.check_price_input(
                        message.clone(),
                        signature.clone(),
                    );

                    assert!(
                        price_message.created_at + self.price_lifetime >= now,
                        "This price is out of date!"
                    );

                    assert!(
                        price_message.market_id == *market_id,
                        "Mismatched resource address",
                    );

                    price_message.price
                },
            }
        }
    }
}
