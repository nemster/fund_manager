use scrypto::prelude::*;
use crate::common::*;
use scrypto_interface::*;

// Couple of coins managed by a dex
#[derive(ScryptoSbor, Debug)]
struct CoinsCouple {
    from: ResourceAddress,
    to: ResourceAddress,
}

// Type of dex
#[derive(ScryptoSbor, Clone, Debug)]
enum PoolType {
    OciswapPool2,
    OciswapPrecisionPool,
    CaviarninePool,
    DefiPlazaPool,
}

// Dex information
#[derive(ScryptoSbor, Clone, Debug)]
struct DexPool {
    pool_type: PoolType,
    component: Global<AnyComponent>,
}

// This blueprint wraps any number of pools in a single inteface; it can perform multi steps swaps
// involving different pool types.
// 
// Only one pool for ordered coin couple is supported (a->b pool can be different from b->a pool).
//
// Some pools can leave remainings of unswapped input coins; those remainigs are saved in the
// remainings KVS. The remainings can be withdrawn from the admins or used in the next swaps.
#[blueprint_with_traits]
#[types(
    CoinsCouple,
    DexPool,
    ResourceAddress,
    Vault,
)]
mod multi_dex_wrapper {

    enable_method_auth! {
        roles {
            fund_manager => updatable_by: [];
        },
        methods {
            swap => restrict_to: [fund_manager];
            add_pool => restrict_to: [OWNER];
            take_from_remainings => restrict_to: [OWNER];
        }
    }

    struct MultiDexWrapper {
        pools: KeyValueStore<CoinsCouple, DexPool>,         // All known pools
        remainings: KeyValueStore<ResourceAddress, Vault>,  // Vaults to store eventual unswapped coins
    }

    impl MultiDexWrapper {

        // Instantiate and globalize a MultiDexWrapper component
        pub fn new(
            fund_manager_badge_address: ResourceAddress,    // FundManager badge
            admin_badge_address: ResourceAddress,           // Owner badge
        ) -> Global<MultiDexWrapper> {

            // Instantiate and globalize the component
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

        // Add the pool to swap from "from" to "to" coins
        pub fn add_pool(
            &mut self,
            from: ResourceAddress,              // Input coin
            to: ResourceAddress,                // Output coin
            pool_type: String,                  // Pool type
            component: Global<AnyComponent>,    // Pool component address
            opposite_too: bool,                 // Whether to use the same pool for the opposite
                                                // swaps too
        ) {

            // Build the DexPool object
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
                "defiplaza_pool" => DexPool {
                    pool_type: PoolType::DefiPlazaPool,
                    component: component,
                },
                _ => { Runtime::panic("Unrecognized pool type".to_string()); },
            };

            // Insert the DexPool object in the pools KVS
            self.pools.insert(
                CoinsCouple {
                    from: from,
                    to: to,
                },
                pool.clone()
            );

            // If required insert a clone of the DexPool object in the KVS for the opposite swaps
            // too
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

        // Withdraw from one of the remainings vaults
        pub fn take_from_remainings(
            &mut self,
            resource: ResourceAddress,
        ) -> Option<Bucket> {

            // Find the remainings vault for the given resource; return None if not found
            let vault = self.remainings.get_mut(&resource);
            if vault.is_none() {
                return None;
            }

            // otherwise return a bucket with the full amount
            return Some(vault.unwrap().take_all());
        }
    }

    impl DexInterfaceTrait for MultiDexWrapper {

        // This method swaps the input_bucket for the output_resource
        fn swap(
            &mut self,
            mut input_bucket: Bucket,           // Bucket of coins to swap
            output_resource: ResourceAddress,   // Output coins resource address
            add_remainings: bool,   // Whether to use remainings of previous partial swaps
        ) -> Bucket {
            let input_resource = input_bucket.resource_address();

            // If add_remainings is set and there are old remainings of the input resource from the
            // previous swaps, add them to the input_bucket
            if add_remainings {
                let remainings_bucket = self.take_from_remainings(input_resource);
                if remainings_bucket.is_some() {
                    input_bucket.put(remainings_bucket.unwrap());
                }
            }

            // Find the dex pool to use for the swap
            let coins_couple = CoinsCouple {
                from: input_resource,
                to: output_resource,
            };
            let pool = self.pools.get_mut(&coins_couple);

            let mut output_bucket: Bucket;
            let mut remainings_bucket: Option<Bucket> = None;

            match pool {
                Some(ref dex_pool) => {
                    match dex_pool.pool_type {

                        // Ociswap latest pools only returns the output_bucket, no remainings
                        PoolType::OciswapPool2 => {
                            output_bucket = dex_pool.component
                                .call::<Bucket, Bucket>(
                                    "swap",
                                    &input_bucket
                                );
                        },

                        // DefiPlaza return the output bucket and eventually a remainings bucket
                        PoolType::DefiPlazaPool => {
                            (output_bucket, remainings_bucket) = dex_pool.component
                                .call::<Bucket, (Bucket, Option<Bucket>)>(
                                    "swap",
                                    &input_bucket
                                );
                        },

                        // Both Caviarnine pools and Ociswap precision pools always return a couple
                        // of buckets
                        _ => {
                            (output_bucket, input_bucket) = dex_pool.component
                                .call::<Bucket, (Bucket, Bucket)>(
                                    "swap",
                                    &input_bucket
                                );

                            remainings_bucket = Some(input_bucket);
                        },
                    }

                    drop(pool);
                },

                // If there's no direct input -> output pool try doing input -> XRD, XRD -> output
                None => {
                    if input_resource != XRD && output_resource != XRD {
                        drop(pool);

                        // Call recursivelly twice this method for the 2 phase swap
                        let xrd_bucket = self.swap(input_bucket, XRD, false);
                        output_bucket = self.swap(xrd_bucket, output_resource, false);

                    } else {
                        Runtime::panic("Pool not found".to_string());
                    }
                },
            };

            if remainings_bucket.is_some() {

                // Put remainings in the vault 
                let remainings_vault = self.remainings.get_mut(&input_resource);
                match remainings_vault {
                    Some(mut vault) => vault.put(remainings_bucket.unwrap()),
                    None => {
                        drop(remainings_vault);

                        self.remainings.insert(
                            input_resource,
                            Vault::with_bucket(remainings_bucket.unwrap())
                        );
                    },
                }
            }

            // If add_remainings is set and there are old remainings of the output resource from the
            // previous swaps, add them to the output_bucket
            if add_remainings {
                let remainings_bucket = self.take_from_remainings(output_resource);
                if remainings_bucket.is_some() {
                    output_bucket.put(remainings_bucket.unwrap());
                }
            }

            output_bucket
        }
    }
}
