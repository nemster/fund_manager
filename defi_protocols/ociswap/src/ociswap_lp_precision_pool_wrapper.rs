use scrypto::prelude::*;
use crate::common::*;
use scrypto_interface::*;

const MAX_TICK: i32 = 887272;
const MIN_TICK: i32 = -MAX_TICK;

#[derive(ScryptoSbor, NonFungibleData)]
struct LiquidityPosition {
    liquidity: PreciseDecimal,
    left_bound: i32,
    right_bound: i32,
    shape_id: Option<NonFungibleLocalId>,
    added_at: u64,
    x_fee_checkpoint: PreciseDecimal,
    y_fee_checkpoint: PreciseDecimal,
    x_total_fee_checkpoint: PreciseDecimal,
    y_total_fee_checkpoint: PreciseDecimal,
    seconds_inside_checkpoint: i64,
}

#[blueprint_with_traits]
mod ociswap_lp_precision_pool_wrapper {

    extern_blueprint! {
        "package_tdx_2_1pha4sgrwdc09ymqy6e8gpe7h0e652p5l22esxd2t8u82290cjq45ma",
        PrecisionPool {
            fn add_liquidity(
                &mut self,
                left_bound: i32,
                right_bound: i32,
                x_bucket: Bucket,
                y_bucket: Bucket,
            ) -> (Bucket, Bucket, Bucket);
        }
    }

    enable_method_auth! {
        roles {
            fund_manager => updatable_by: [];
        },
        methods {
            deposit_protocol_token => restrict_to: [fund_manager];
            withdraw_protocol_token => restrict_to: [fund_manager];
            deposit_coin => restrict_to: [fund_manager];
            withdraw_coin => restrict_to: [fund_manager];
        }
    }

    struct OciswapLpPrecisionPoolWrapper {
        x_vault: FungibleVault,
        y_vault: FungibleVault,
        lp_token_vault: NonFungibleVault,
        component_address: Global<LendingPool>,
    }

    impl OciswapLpPrecisionPoolWrapper {

        pub fn new(
            x_address: ResourceAddress,
            y_address: ResourceAddress,
            lp_token_address: ResourceAddress,
            component_address: Global<PrecisionPool>,
            fund_manager_badge_address: ResourceAddress,
        ) -> Global<OciswapLpPrecisionPoolWrapper> {
            Self {
                x_vault: FungibleVault::new(x_address),
                y_vault: FungibleVault::new(y_address),
                lp_token_vault: NonFungibleVault::new(lp_token_address),
                component_address: component_address,
            }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .roles(roles!(
                    fund_manager => rule!(require(fund_manager_badge_address));
                ))
                .globalize()
        }

    }

    impl DefiProtocolInterfaceTrait for OciswapLpPrecisionPoolWrapper {

        fn deposit_protocol_token(
            &mut self,
            token: Bucket,
        ) -> (Option<Decimal>, Option<Decimal>) {
            // TODO
        }

        fn withdraw_protocol_token(
            &mut self,
            amount: Option<Decimal>,
        ) -> Bucket {
            // TODO
        }

        fn deposit_coin(
            &mut self,
            coin: FungibleBucket,
            _other_coin: Option<FungibleBucket>,
            _message: Option<String>,
            _signature: Option<String>,
        ) {
            // TODO
        }

        fn withdraw_coin(
            &mut self,
            amount: Option<Decimal>,
            _other_coin_to_coin_price_ratio: Option<Decimal>,
        ) -> (FungibleBucket, Option<FungibleBucket>) {
            // TODO
        }
    }
}
