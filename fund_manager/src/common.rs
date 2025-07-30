use scrypto::prelude::*;
use scrypto_interface::*;

define_interface! {
    DefiProtocol impl [ScryptoStub, Trait, ScryptoTestStub] {

        fn deposit_protocol_token(
            &mut self,
            token: Bucket,
        ) -> (Option<Decimal>, Option<Decimal>);

        fn withdraw_protocol_token(
            &mut self,
            amount: Option<Decimal>,
        ) -> Bucket;

        fn deposit_coin(
            &mut self,
            coin: FungibleBucket,
            other_coin: Option<FungibleBucket>,
            message: Option<String>,
            signature: Option<String>,
        );

        fn withdraw_coin(
            &mut self,
            amount: Option<Decimal>,
            other_coin_to_coin_price_ratio: Option<Decimal>,
        ) -> (FungibleBucket, Option<FungibleBucket>);
    }
}

define_interface! {
    Dex impl [ScryptoStub, Trait, ScryptoTestStub] {

        fn swap(
            &mut self,
            input_bucket: Bucket,
            output_resource: ResourceAddress,
        ) -> Bucket;
    }
}

define_interface! {
    Oracle impl [ScryptoStub, Trait, ScryptoTestStub] {

        fn get_price(
            &mut self,
            coin_address: ResourceAddress,
            message: Option<String>,
            signature: Option<String>,
        ) -> Decimal;
    }
}
