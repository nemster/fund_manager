use scrypto::prelude::*;
use crate::common::*;
use scrypto_interface::*;
use std::ops::DerefMut;

// Morpher price information struct
#[derive(ScryptoSbor, Debug)]
struct PriceMessage {
    market_id: String,
    price: Decimal,
    nonce: u64,
    created_at: u64,
}

// Ociswap price information struct
#[derive(ScryptoSbor, Debug)]
pub struct ObservationInterval {
    start: u64,
    end: u64,
    price_sqrt: Decimal,
}

// Information about one single oracle
#[derive(ScryptoSbor, Clone, Debug)]
pub enum OracleType {
    FixedPrice {
        price: Decimal, // Fixed price
    },
    FixedMultiplier {
        multiplier: Decimal, // The multiplier to apply to the reference coin price
        reference_coin: ResourceAddress,
    },
    Ociswap {
        component: Global<AnyComponent>,    // Ociswap pool address
        reference_coin: ResourceAddress,    // Reference coin
        reverse: bool,  // Whether the pool returns coin price against reference coin or the
                        // opposite
    },
    Morpher {
        market_id: String, // String identifier of the market (e.g. "GATEIO:XRD_USDT")
    },
}

// This blueprint wraps some of the available price oracles on Radix (Ociswap and Morpher) and
// defines two very simple additional oracles (FixedPrice and FixedMultiplier).
//
// FixedPrice always return the same number (e.g. xUSDC -> 1) while FixedMultiplier returns the
// price of another coin multiplied by a fixed factor (e.g. LSULP -> 1.15 XRD).
//
// Ociswap can either be a PrecisionPool or a new (Pool2) pool. Older pools are not supported.
//
// This oracle is intended to get the USD price of a coin.
// Since FixedMultiplier and Ociswap only handle price of a coin relative to another resource
// address, multiple internal steps can be needed internally to get the USD price.
// The different steps can involve different oracle types.
//
// Only one oracle type can be added for each resource address.
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

    enable_method_auth! {
        roles {
            fund_manager => updatable_by: [];
            bot => updatable_by: [fund_manager];
        },
        methods {
            // public method
            get_price  => PUBLIC;
          
            // bot callable methods
            update_price => restrict_to: [bot];

            // Admins' methods
            update_settings => restrict_to: [OWNER];
            add_oracle => restrict_to: [OWNER];
            remove_oracle => restrict_to: [OWNER];
        }
    }

    struct MultiOracleWrapper {
        oracles: KeyValueStore<ResourceAddress, OracleType>,    // Infromation about he oracle to
                                                                // use for each coin
        morpher_component: Global<MorpherOracle>,   // Morpher component address
        observation_time: u64,                      // Ociswap's oracle observation time
        price_lifetime: u64,                        // Morpher oracle information lifetime
    }

    impl MultiOracleWrapper {

        // Instantiate and globalize a MultiOracleWrapper component
        pub fn new(
            fund_manager_badge_address: ResourceAddress,    // God's badge address
            admin_badge_address: ResourceAddress,       // Owners' badge address
            bot_badge_address: ResourceAddress,         // Bot badge address
            morpher_component: Global<MorpherOracle>,   // Morpher component address
            observation_time: u64,                      // Ociswap oracle observation time
            price_lifetime: u64,                        // Morpher oracle information lifetime
        ) -> Global<MultiOracleWrapper> {

            // Instantiate and globalize the component
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

        // By invoking this method the bot can update price information for the FixedPrice and
        // FixedMultiplier oracle types
        pub fn update_price(
            &mut self,
            coin_address: ResourceAddress,      // Coin whose price information needs update
            fixed_price: Option<Decimal>,       // New fixed price or None
            fixed_multiplier: Option<Decimal>,  // New fized multiplier or None
        ) {
            match self.oracles.get_mut(&coin_address).expect("Unknown coin").deref_mut() {
                OracleType::FixedPrice { price } => { *price = fixed_price.unwrap(); },
                OracleType::FixedMultiplier { multiplier, .. } => { *multiplier = fixed_multiplier.unwrap(); },
                _ => Runtime::panic("Can't update this oracle type".to_string()),
            }
        }

        // Update global setting
        pub fn update_settings(
            &mut self,
            morpher_component: Option<Global<MorpherOracle>>,   // New Morpher component address
                                                                // or None
            observation_time: Option<u64>,  // New Ociswap oracle observation time
            price_lifetime: Option<u64>,    // New Morpher oracle information lifetime
        ) {
            // Only change non None information
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

        // Add or replace the oracle to use for a given coin
        pub fn add_oracle(
            &mut self,
            coin_address: ResourceAddress,      // The coin whose oracle has to be added
            fixed_price: Option<Decimal>,               // Fixed price or None
            fixed_multiplier: Option<Decimal>,          // Fixed multiplier or None
            reference_coin: Option<ResourceAddress>,    // Reference coin (for FixedMultiplier or
                                                        // Ociswap) or None
            ociswap_component: Option<Global<AnyComponent>>,    // Ociswap pool
            ociswap_reverse: Option<bool>,      // Whether to reverse Ociswap oracle price
            morpher_market_id: Option<String>,  // Market id for the Morpher oracle
        ) {
            // Add a FixedPrice oracle
            if fixed_price.is_some() {
                self.oracles.insert(
                    coin_address,
                    OracleType::FixedPrice {
                        price: fixed_price.unwrap(),
                    }
                );

            // Add a FixedMultiplier oracle
            } else if fixed_multiplier.is_some() {
                self.oracles.insert(
                    coin_address,
                    OracleType::FixedMultiplier {
                        multiplier: fixed_multiplier.unwrap(),
                        reference_coin: reference_coin.unwrap(),
                    }
                );

            // Add an Ociswap oracle
            } else if ociswap_component.is_some() {
                self.oracles.insert(
                    coin_address,
                    OracleType::Ociswap {
                        component: ociswap_component.unwrap(),
                        reference_coin: reference_coin.unwrap(),
                        reverse: ociswap_reverse.unwrap(),
                    }
                );

            // Add a Morpher oracle market
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

        // Remove the oracle to use for a given coin
        pub fn remove_oracle(
            &mut self,
            coin_address: ResourceAddress, // The coin whose oracle will be removed
        ) {
            self.oracles.remove(&coin_address);
        }

    }

    impl OracleInterfaceTrait for MultiOracleWrapper {

        // Returns the USD price of the given coin
        // This method can recursively call itself if only a relative price is known
        fn get_price(
            &mut self,
            coin_address: ResourceAddress, // The coin to get the price of
            morpher_data: HashMap<ResourceAddress, (String, String)>, // Eventual Morpher data
        ) -> Decimal {

            // Find the oracle to use for the given coin
            let oracle = self.oracles.get(&coin_address).expect("Coin not found").clone();

            // Use the found oracle type
            match oracle {
                OracleType::FixedPrice { price } => price,

                OracleType::FixedMultiplier { multiplier, reference_coin } =>
                    multiplier * self.get_price(reference_coin, morpher_data),

                OracleType::Ociswap { component, reference_coin, reverse } => {

                    // Ociswap oracle requires a time interval to return an average price
                    let interval_end = Clock::current_time_rounded_to_seconds()
                        .seconds_since_unix_epoch.try_into().unwrap();
                    let intervals = vec![(interval_end - self.observation_time, interval_end)];

                    // Ociswap returns the sware root of the requested price
                    let price_sqrt = component.call::<Vec<(u64, u64)>, Vec<ObservationInterval>>(
                        "observation_intervals",
                        &intervals
                    )[0].price_sqrt;

                    // Is it a/b or b/a price?
                    match reverse {
                        false => self.get_price(reference_coin, morpher_data) * price_sqrt * price_sqrt,
                        true => self.get_price(reference_coin, morpher_data) / (price_sqrt * price_sqrt),
                    }
                },

                OracleType::Morpher { market_id } => {
                    let now: u64 = Clock::current_time_rounded_to_seconds()
                        .seconds_since_unix_epoch.try_into().unwrap();

                    // Extract message and signature for this coin from the morpher_data HashMap
                    let (message, signature) = morpher_data.get(&coin_address).expect("Missing Morpher data");

                    // Let the Morpher component verify price information
                    let price_message = self.morpher_component.check_price_input(
                        message.clone(),
                        signature.clone(),
                    );

                    // How old is this price information?
                    assert!(
                        price_message.created_at + self.price_lifetime >= now,
                        "This price is out of date!"
                    );

                    // Make sure that the price information is related to the requested coin and
                    // whoever built the transaction manifest is not trying to cheat
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
