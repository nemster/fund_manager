use scrypto::prelude::*;
use crate::common::*;
use scrypto_interface::*;

static ACCEPTABLE_REMAININGS_RATIO: u8 = 10;

#[derive(ScryptoSbor)]
struct CoinsCouple {
    from: ResourceAddress,
    to: ResourceAddress,
}

#[derive(ScryptoSbor, Clone)]
enum PoolType {
    OciswapPool2,
    OciswapPrecisionPool,
    CaviarninePool,
}

#[derive(ScryptoSbor, Clone)]
struct DexPool {
    pool_type: PoolType,
    component: Global<AnyComponent>,
}

#[blueprint_with_traits]
#[types(
    CoinsCouple,
    DexPool,
    ResourceAddress,
    Vault,
)]
mod multi_dex_wrapper {

/*
    extern_blueprint! {
        "package_tdx_2_1p5qntnqluczzjjnm577mfp7p5jd3qm2sv0qzkqklgkrypcnspw3dff",
        Pool {
            fn swap(&mut self, input_bucket: Bucket) -> Bucket;
        }
    }

    extern_blueprint! {
        "package_tdx_2_1pha4sgrwdc09ymqy6e8gpe7h0e652p5l22esxd2t8u82290cjq45ma",
        PrecisionPool {
            fn swap(&mut self, input_bucket: Bucket) -> (Bucket, Bucket); // (output_bucket, remainings)
        }
    }

    extern_blueprint! {
        "package_tdx_2_1p4g09xagmsyql6r65a70c94n6qgvk6ffx9q0z5g3vnqmrsr96627vg",
        QuantaSwap {
            fn swap(&mut self, tokens: Bucket) -> (Bucket, Bucket); // (output_bucket, remainings)
        }
    }
*/

    enable_method_auth! {
        roles {
            fund_manager => updatable_by: [];
        },
        methods {
            swap => restrict_to: [fund_manager];
            add_pool => restrict_to: [OWNER];
        }
    }

    struct MultiDexWrapper {
        pools: KeyValueStore<CoinsCouple, DexPool>,
        remainings: KeyValueStore<ResourceAddress, Vault>,
    }

    impl MultiDexWrapper {

        pub fn new(
            fund_manager_badge_address: ResourceAddress,
            admin_badge_address: ResourceAddress,
        ) -> Global<MultiDexWrapper> {
            Self {
                pools: KeyValueStore::new_with_registered_type(),
                remainings: KeyValueStore::new_with_registered_type(),
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
            from: ResourceAddress,
            to: ResourceAddress,
            pool_type: String,
            component: Global<AnyComponent>,
            opposite_too: bool,
        ) {
            let pool = match pool_type.as_str() {
                "ociswap_pool2" => DexPool {
                    pool_type: PoolType::OciswapPool2,
                    component: component,
                },
                "ociswap_precision_pool" => DexPool {
                    pool_type: PoolType::OciswapPrecisionPool,
                    component: component,
                },
                "caviarnine_pool" => DexPool {
                    pool_type: PoolType::CaviarninePool,
                    component: component,
                },
                _ => { Runtime::panic("Unrecognized pool type".to_string()); },
            };

            self.pools.insert(
                CoinsCouple {
                    from: from,
                    to: to,
                },
                pool.clone()
            );

            if opposite_too {
                self.pools.insert(
                    CoinsCouple {
                        from: to,
                        to: from,
                    },
                    pool
                );
            }
        }
    }

    impl DexInterfaceTrait for MultiDexWrapper {

        fn swap(
            &mut self,
            mut input_bucket: Bucket,
            output_resource: ResourceAddress,
        ) -> Bucket {
            let input_bucket_amount = input_bucket.amount();
            let input_resource = input_bucket.resource_address();

            match self.remainings.get_mut(&input_resource) {
                Some(mut vault) =>
                    if vault.amount() > Decimal::ZERO {
                        input_bucket.put(vault.take_all());
                    },
                None => {},
            }

            let coins_couple = CoinsCouple {
                from: input_resource,
                to: output_resource,
            };

            let output_bucket: Bucket;

            let pool = self.pools.get_mut(&coins_couple);

            match pool {
                Some(dex_pool) => {
                    match dex_pool.pool_type {
                        PoolType::OciswapPool2 => {
                            output_bucket = dex_pool.component
                                .call::<Bucket, Bucket>(
                                    "swap",
                                    &input_bucket
                                );

                            input_bucket = Bucket::new(input_resource);
                        },
                        _ => (output_bucket, input_bucket) = dex_pool.component
                            .call::<Bucket, (Bucket, Bucket)>(
                                "swap",
                                &input_bucket
                            ),
                    }
                },
                None => {
                    if input_resource != XRD && output_resource != XRD {
                        drop(pool);

                        let xrd_bucket = self.swap(input_bucket, XRD);

                        output_bucket = self.swap(xrd_bucket, output_resource);

                        input_bucket = Bucket::new(input_resource);
                    } else {
                        Runtime::panic("Pool not found".to_string());
                    }
                },
            };

            assert!(
                input_bucket.amount() < input_bucket_amount / ACCEPTABLE_REMAININGS_RATIO,
                "Swap failed",
            );

            let remainings_vault = self.remainings.get_mut(&input_resource);

            match remainings_vault {
                Some(mut vault) => vault.put(input_bucket),
                None => {
                    drop(remainings_vault);

                    self.remainings.insert(
                        input_resource,
                        Vault::with_bucket(input_bucket)
                    );
                },
            }

            output_bucket
        }
    }
}
