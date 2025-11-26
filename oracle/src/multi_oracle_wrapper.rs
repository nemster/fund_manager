use scrypto::prelude::*;
use crate::common::*;
use scrypto_interface::*;
use std::ops::DerefMut;

// Morpher price information struct
#[derive(ScryptoSbor)]
struct PriceMessage {
    market_id: String,
    price: Decimal,
    nonce: u64,
    created_at: u64,
}

// Ociswap price information struct
#[derive(ScryptoSbor)]
pub struct ObservationInterval {
    start: u64,
    end: u64,
    price_sqrt: Decimal,
}

// Price cache for a single coin
#[derive(ScryptoSbor, Clone)]
struct CachedPrice {
    last_update_time: u64,              // Last time the price cache was updated
    last_price: Decimal,                // Price cache
}

// Information about one single oracle
#[derive(ScryptoSbor, Clone)]
enum OracleType {
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
        cached_price: CachedPrice,
    },
    Morpher {
        market_id: String, // String identifier of the market (e.g. "GATEIO:XRD_USDT")
        cached_price: CachedPrice,
    },
    Lsu {
        validator: Global<Validator>,
    },
    OneResourcePoolUnit {
        pool: Global<OneResourcePool>,
        reference_coin: ResourceAddress,
    },
    TwoResourcePoolUnit {
        pool: Global<TwoResourcePool>,
    },
    MultiResourcePoolUnit {
        pool: Global<MultiResourcePool>,
    },
    Weft {
        reference_coin: ResourceAddress,    // Which coin this one is a Weft wrapped version
    },
}

#[derive(ScryptoSbor, ScryptoEvent)]
struct PriceUpdated {
    coin: ResourceAddress,
    price: Decimal,
}

// Info about a Surge pool
#[derive(ScryptoSbor)]
struct PoolDetails {
    base_tokens_amount: Decimal,
    virtual_balance: Decimal,
    unrealized_pool_funding: Decimal,
    pnl_snap: Decimal,
    skew_ratio: Decimal,
    skew_ratio_cap: Decimal,
    lp_supply: Decimal,
    lp_price: Decimal,
}

// This blueprint wraps some of the available price oracles on Radix (Ociswap and Morpher) and
// defines two very simple additional oracles (FixedPrice and FixedMultiplier); it can also query
// the LsuPool for the LSULP/XRD price.
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
//
// Ociswap and Morpher prices are cached to reduce the number of outgoing components calls. This is
// expecially useful for protocols that need both the XRD price and the price of some coin
// dependent from XRD; the XRD price will be computed just once.
#[blueprint_with_traits]
#[types(
    ResourceAddress,
    OracleType,
)]
#[events(
    PriceUpdated,
)]
mod multi_oracle_wrapper {

    extern_blueprint! {
        "package_rdx1p5xvvessslnpnfam9weyzldlxr7q06gen2t3d3waa0x760g7jwxhkd",
        MorpherOracle {
            fn check_price_input(&mut self, message: String, signature: String) -> PriceMessage;
        }
    }

    extern_blueprint! {
        "package_rdx1pkfrtmv980h85c9nvhxa7c9y0z4vxzt25c3gdzywz5l52g5t0hdeey",
        LsuPool {
            fn get_dex_valuation_xrd(&self) -> Decimal;
        }
    }

    extern_blueprint! {
        "package_tdx_2_1p4rv3hyae94tmyg36wru460wzcfjzajpw2zlt7ns5m7mswmchxud0l",
        FundManager {
            fn fund_unit_value(&self) -> (Decimal, Decimal);
        }
    }

    extern_blueprint! {
        "package_tdx_2_1phyewk3m6aeycqmmmk5easfmk7mg97sn20p2yvd499rj5y5xrxzdcc",
        Exchange {
            fn get_pool_details(&self) -> PoolDetails;
        }
    }

    extern_blueprint! {
        "package_tdx_2_1pk02rsgrec4dv3fhtw2ltmy3g80325wlusl76tjwhjpj48qtk8c80n",
        LendingPool {
            fn get_deposit_unit_ratio(&mut self, resources: IndexSet<ResourceAddress>) -> IndexMap<ResourceAddress, Option<PreciseDecimal>>;
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
        oracles: KeyValueStore<ResourceAddress, OracleType>,    // Information about he oracle to
                                                                // use for each coin
        morpher_component: Global<MorpherOracle>,   // Morpher component address
        observation_time: u64,                      // Ociswap's oracle observation time
        price_lifetime: u64,                        // Morpher oracle information lifetime
        lsulp: ResourceAddress,                     // LSULP resource address
        lsu_pool: Global<LsuPool>,                  // Caviarnine LSU pool 
        fund_unit: ResourceAddress,                 // Fund Unit resource address
        fund_manager: Global<FundManager>,          // FundManager component address
        surge_lp: ResourceAddress,                  // Surge LP coin resource address
        surge: Global<Exchange>,                    // Surge component address
        weft: Global<LendingPool>,                  // Weft component address
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
            lsulp: ResourceAddress,                     // LSULP resource address
            lsu_pool: Global<LsuPool>,                  // LSULP pool component address
            fund_unit: ResourceAddress,                 // Fund Unit resource address
            fund_manager: Global<FundManager>,          // FundManager component address
            surge_lp: ResourceAddress,                  // Surge LP coin resource address
            surge: Global<Exchange>,                    // Surge component address
            weft: Global<LendingPool>,                  // Weft component address
        ) -> Global<MultiOracleWrapper> {

            // Instantiate and globalize the component
            Self {
                oracles: KeyValueStore::new_with_registered_type(),
                morpher_component: morpher_component,
                observation_time: observation_time,
                price_lifetime: price_lifetime,
                lsulp: lsulp,
                lsu_pool: lsu_pool,
                fund_unit: fund_unit,
                fund_manager: fund_manager,
                surge_lp: surge_lp,
                surge: surge,
                weft: weft,
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
                OracleType::FixedPrice { price } => {
                    *price = fixed_price.unwrap();

                    Runtime::emit_event(
                        PriceUpdated {
                            coin: coin_address,
                            price: *price,
                        }
                    );
                },
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
            price_lifetime: Option<u64>,    // New Morpher oracle and cached information lifetime
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
                                                        // Ociswap or Weft) or None
            ociswap_component: Option<Global<AnyComponent>>,    // Ociswap pool
            ociswap_reverse: Option<bool>,      // Whether to reverse Ociswap oracle price
            morpher_market_id: Option<String>,  // Market id for the Morpher oracle
            validator: Option<Global<Validator>>,
            one_resource_pool: Option<Global<OneResourcePool>>,
            two_resource_pool: Option<Global<TwoResourcePool>>,
            multi_resource_pool: Option<Global<MultiResourcePool>>,
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
                        cached_price: CachedPrice {
                            last_update_time: 0,
                            last_price: Decimal::ONE,
                        },
                    }
                );

            // Add a Morpher oracle market
            } else if morpher_market_id.is_some() {
                self.oracles.insert(
                    coin_address,
                    OracleType::Morpher {
                        market_id: morpher_market_id.unwrap(),
                        cached_price: CachedPrice {
                            last_update_time: 0,
                            last_price: Decimal::ONE,
                        },
                    }
                );

            } else if validator.is_some() {
                self.oracles.insert(
                    coin_address,
                    OracleType::Lsu {
                        validator: validator.unwrap(),
                    }
                );

            } else if one_resource_pool.is_some() {
                self.oracles.insert(
                    coin_address,
                    OracleType::OneResourcePoolUnit {
                        pool: one_resource_pool.unwrap(),
                        reference_coin: reference_coin.unwrap(),
                    }
                );

            } else if two_resource_pool.is_some() {
                self.oracles.insert(
                    coin_address,
                    OracleType::TwoResourcePoolUnit {
                        pool: two_resource_pool.unwrap(),
                    }
                );

            } else if multi_resource_pool.is_some() {
                self.oracles.insert(
                    coin_address,
                    OracleType::MultiResourcePoolUnit {
                        pool: multi_resource_pool.unwrap(),
                    }
                );

            } else if reference_coin.is_some() {
                let reference_coin = reference_coin.unwrap();
                let index_set = indexset!(reference_coin);
                let out = self.weft.get_deposit_unit_ratio(index_set);
                for (coin, amount) in out.iter() {
                    if *coin == reference_coin && amount.is_some() {
                        self.oracles.insert(
                            coin_address,
                            OracleType::Weft {
                                reference_coin: reference_coin,
                            }
                        );
                    }
                }

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

        // Internal method to check if price cache is still valid
        fn get_cached_price(
            &self,
            cached_price: &CachedPrice,
            now: u64,
        ) -> Option<Decimal> {

            // If the cached value is still valid, return it
            if cached_price.last_update_time + self.price_lifetime >= now {
                return Some(cached_price.last_price);
            } else {
                return None;
            }
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
            let mut oracle = match self.oracles.get(&coin_address) {


                // If there's no available oracle for this coin
                None => {
                    let price;

                    // If the coin is LSULP, ask the LsuPool
                    if coin_address == self.lsulp {
                        let dex_valuation_xrd = self.lsu_pool.get_dex_valuation_xrd();
                        let lsulp_supply =
                            ResourceManager::from_address(self.lsulp).total_supply().unwrap();

                        OracleType::FixedMultiplier {
                            multiplier: dex_valuation_xrd / lsulp_supply,
                            reference_coin: XRD,
                        }

                    // If the coin is the fund unit get the net price from the FundManager component
                    } else if coin_address == self.fund_unit {
                        let (net_price, _) = self.fund_manager.fund_unit_value();
                        price = net_price;

                        Runtime::emit_event(
                            PriceUpdated {
                                coin: coin_address,
                                price: price,
                            }
                        );

                        return price;

                    // If the coin is the Surge LP, ask the Surge component
                    } else if coin_address == self.surge_lp {
                        price = self.surge.get_pool_details().lp_price;

                        Runtime::emit_event(
                            PriceUpdated {
                                coin: coin_address,
                                price: price,
                            }
                        );

                        return price;

                    } else {
                        Runtime::panic("No oracle available".to_string());
                    }
                },

                Some(oracle) => oracle.clone(),
            };

            // Get current time
            let now: u64 = Clock::current_time_rounded_to_seconds()
                .seconds_since_unix_epoch.try_into().unwrap();

            // Use the found oracle to get the price
            let (price, oracle_updated) = match oracle {

                OracleType::FixedPrice { price } => {
                    return price;
                },

                OracleType::FixedMultiplier { multiplier, reference_coin } => {
                    let price = multiplier * self.get_price(reference_coin, morpher_data);

                    Runtime::emit_event(
                        PriceUpdated {
                            coin: coin_address,
                            price: price,
                        }
                    );

                    return price;
                },

                OracleType::Ociswap {
                    component,
                    reference_coin,
                    reverse,
                    ref mut cached_price,
                } => {
                    // If the cached price is still valid, return it
                    match self.get_cached_price(cached_price, now) {
                        Some(price) => return price,
                        None => {},
                    };

                    // Ociswap oracle requires a time interval to return an average price
                    let intervals = vec![(now - self.observation_time, now)];

                    // Ociswap returns the square root of the requested price
                    let price_sqrt = component.call::<(Vec<(u64, u64)>, ), Vec<ObservationInterval>>(
                        "observation_intervals",
                        &(intervals, ),
                    )[0].price_sqrt;

                    // Is it a/b or b/a price?
                    let price = match reverse {
                        false => self.get_price(reference_coin, morpher_data) * price_sqrt * price_sqrt,
                        true => self.get_price(reference_coin, morpher_data) / (price_sqrt * price_sqrt),
                    };

                    // Update cache
                    // Cache will not work if observation_time / 2 > price_lifetime because newly
                    // collected data will already be osolete, so no need to store them
                    if self.observation_time / 2 <= self.price_lifetime {
                        cached_price.last_update_time = now - self.observation_time / 2;
                        cached_price.last_price = price;

                        (price, true)
                    } else {
                        (price, false)
                    }
                },

                OracleType::Morpher { ref market_id, ref mut cached_price } => {
                    // If the cached price is still valid, return it
                    match self.get_cached_price(cached_price, now) {
                        Some(price) => return price,
                        None => {},
                    };

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

                    // Update cache
                    cached_price.last_update_time = price_message.created_at;
                    cached_price.last_price = price_message.price;

                    (price_message.price, true)
                },

                OracleType::Lsu { ref validator } => {
                    // Get XRD/LSU ratio from the Validator and multiply it for XRD price
                    let price = self.get_price(XRD, morpher_data) * validator.get_redemption_value(Decimal::ONE);

                    (price, false)
                },

                OracleType::OneResourcePoolUnit { ref pool, reference_coin } => {
                    // Get coin/LP ratio from the pool and multiply it for coin price
                    let price = self.get_price(reference_coin, morpher_data) * pool.get_redemption_value(Decimal::ONE);

                    (price, false)
                },

                OracleType::TwoResourcePoolUnit { ref pool } => {
                    // The price is the sum of the coin/LP ratios multiplied by coin prices
                    let mut price = Decimal::ZERO;
                    for (coin, amount) in pool.get_redemption_value(Decimal::ONE).iter() {
                        price += self.get_price(*coin, morpher_data.clone()) * *amount;
                    }

                    (price, false)
                },

                OracleType::MultiResourcePoolUnit { ref pool } => {
                    // The price is the sum of the coin/LP ratios multiplied by coin prices
                    let mut price = Decimal::ZERO;
                    for (coin, amount) in pool.get_redemption_value(Decimal::ONE).iter() {
                        price += self.get_price(*coin, morpher_data.clone()) * *amount;
                    }

                    (price, false)
                },

                OracleType::Weft { reference_coin } => {
                    let mut price = Decimal::ZERO;
                    let out = self.weft.get_deposit_unit_ratio(indexset!(reference_coin));
                    for (wrapped_coin, amount) in out.iter() {
                        if *wrapped_coin == coin_address && amount.is_some() {
                            price = (self.get_price(reference_coin, morpher_data.clone()) / amount.unwrap())
                                .checked_truncate(RoundingMode::ToNearestMidpointTowardZero)
                                .unwrap();
                        }
                    }

                    (price, false)
                }
            };

            Runtime::emit_event(
                PriceUpdated {
                    coin: coin_address,
                    price: price,
                }
            );

            // If the oracle object has been modified, insert it back in the KVS
            if oracle_updated {
                self.oracles.insert(coin_address, oracle);
            }

            price
        }
    }
}
