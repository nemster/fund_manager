use scrypto::prelude::*;
use crate::common::*;
use scrypto_interface::*;

static ACCEPTABLE_REMAININGS_RATIO: u8 = 10;

#[blueprint_with_traits]
mod ociswap_precision_pool_wrapper {

    extern_blueprint! {
        "package_tdx_2_1pha4sgrwdc09ymqy6e8gpe7h0e652p5l22esxd2t8u82290cjq45ma",
        PrecisionPool {
            fn swap(&mut self, input_bucket: Bucket) -> (Bucket, Bucket); // (output_bucket, remainings)
        }
    }

    enable_method_auth! {
        roles {
            fund_manager => updatable_by: [];
        },
        methods {
            swap => restrict_to: [fund_manager];
        }
    }

    struct OciswapPrecisionPoolWrapper {
        component_address: Global<PrecisionPool>,
        input_coin_vault: FungibleVault,
    }

    impl OciswapPrecisionPoolWrapper {

        pub fn new(
            component_address: Global<PrecisionPool>,
            fund_manager_badge_address: ResourceAddress,
            input_coin_address: ResourceAddress,
        ) -> Global<OciswapPrecisionPoolWrapper> {
            Self {
                component_address: component_address,
                input_coin_vault: FungibleVault::new(input_coin_address),
            }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .roles(roles!(
                    fund_manager => rule!(require(fund_manager_badge_address));
                ))
                .globalize()
        }
    }

    impl DexInterfaceTrait for OciswapPrecisionPoolWrapper {

        fn swap(
            &mut self,
            mut input_bucket: FungibleBucket,
        ) -> FungibleBucket {
            let input_bucket_amount = input_bucket.amount();

            input_bucket.put(self.input_coin_vault.take_all());

            let (output_bucket, remainings) = self.component_address.swap(input_bucket.into());

            assert!(
                remainings.amount() < input_bucket_amount / ACCEPTABLE_REMAININGS_RATIO,
                "Swap failed",
            );

            self.input_coin_vault.put(FungibleBucket(remainings));

            FungibleBucket(output_bucket)
        }
    }
}
