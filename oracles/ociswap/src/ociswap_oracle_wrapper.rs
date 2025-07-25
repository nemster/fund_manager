use scrypto::prelude::*;
use crate::common::*;
use scrypto_interface::*;

#[derive(ScryptoSbor)]
pub struct ObservationInterval {
    pub start: u64,
    pub end: u64,
    pub price_sqrt: Decimal,
}

#[derive(ScryptoSbor)]
struct CoinCouple {
    x: ResourceAddress,
    y: ResourceAddress,
}

#[blueprint_with_traits]
#[types(ResourceAddress, CoinCouple, Global<PrecisionPool>)]
mod ociswap_oracle_wrapper {

    extern_blueprint! {
        "package_tdx_2_1pha4sgrwdc09ymqy6e8gpe7h0e652p5l22esxd2t8u82290cjq45ma",
        PrecisionPool {
            fn observation_intervals(&self, intervals: Vec<(u64, u64)>) -> Vec<ObservationInterval>;
        }
    }

    enable_method_auth! {
        roles {
            fund_manager => updatable_by: [];
        },
        methods {
           get_price  => restrict_to: [fund_manager];
           add_pool => restrict_to: [OWNER];
           remove_pool => restrict_to: [OWNER];
        }
    }

    struct OciswapOracleWrapper {
        observation_time: u64,
        reference_coin: ResourceAddress,
        pools: KeyValueStore<CoinCouple, Global<PrecisionPool>>, 
    }

    impl OciswapOracleWrapper {

        pub fn new(
            observation_time: u64,
            reference_coin: ResourceAddress,
            admin_badge_address: ResourceAddress,
            fund_manager_badge_address: ResourceAddress,
        ) -> Global<OciswapOracleWrapper> {
            Self {
                observation_time: observation_time,
                reference_coin: reference_coin,
                pools: KeyValueStore::new_with_registered_type(),
            }
                .instantiate()
                .prepare_to_globalize(OwnerRole::Fixed(rule!(require(admin_badge_address))))
                .roles(roles!(
                    fund_manager => rule!(require(fund_manager_badge_address));
                ))
                .globalize()
        }

        pub fn add_pool(
            &mut self,
            coin_x: ResourceAddress,
            coin_y: ResourceAddress,
            pool: Global<PrecisionPool>,
        ) {
            self.pools.insert(
                CoinCouple {
                    x: coin_x,
                    y: coin_y,
                },
                pool
            );
        }

        pub fn remove_pool(
            &mut self,
            coin_x: ResourceAddress,
            coin_y: ResourceAddress,
        ) {
            self.pools.remove(
                &CoinCouple {
                    x: coin_x,
                    y: coin_y,
                }
            );
        }
    }

    impl OracleInterfaceTrait for OciswapOracleWrapper {

        fn get_price(
            &mut self,
            coin_address: ResourceAddress,
            _message: Option<String>,
            _signature: Option<String>,
        ) -> Decimal {
            if coin_address == self.reference_coin {
                return Decimal::ONE;
            }

            let interval_end = Clock::current_time_rounded_to_seconds().seconds_since_unix_epoch.try_into().unwrap();
            let intervals = vec![(interval_end - self.observation_time, interval_end)];

            let mut price: Decimal;

            let mut pool = self.pools.get(
                &CoinCouple {
                    x: self.reference_coin,
                    y: XRD,
                }
            );

            if pool.is_some() {
                let price_sqrt = pool.unwrap().observation_intervals(intervals.clone())[0].price_sqrt;
                price = price_sqrt * price_sqrt;
            } else {
                pool = self.pools.get(
                    &CoinCouple {
                        x: XRD,
                        y: self.reference_coin,
                    }
                );

                let price_sqrt = pool.expect("XRD pool not found").observation_intervals(intervals.clone())[0].price_sqrt;
                price = Decimal::ONE / (price_sqrt * price_sqrt);
            }

            if coin_address == XRD {
                return price;
            }

            pool = self.pools.get(
                &CoinCouple {
                    x: XRD,
                    y: coin_address,
                }
            );

            if pool.is_some() {
                let price_sqrt = pool.unwrap().observation_intervals(intervals.clone())[0].price_sqrt;
                price *= price_sqrt * price_sqrt;
            } else {
                pool = self.pools.get(
                    &CoinCouple {
                        x: coin_address,
                        y: XRD,
                    }
                );

                let price_sqrt = pool.expect("Pool not found").observation_intervals(intervals)[0].price_sqrt;
                price /= price_sqrt * price_sqrt;
            }

            price
        }
    }
}
