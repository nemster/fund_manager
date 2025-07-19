use scrypto::prelude::*;
use scrypto_interface::*;

define_interface! {
    DefiProtocol impl [ScryptoStub, Trait, ScryptoTestStub] {

        fn deposit_protocol_token(
            &mut self,
            token: Bucket,
        );

        fn withdraw_protocol_token(
            &mut self,
            amount: Option<Decimal>,
        ) -> Bucket;

        fn deposit_coin(
            &mut self,
            coin: FungibleBucket,
        );

        fn withdraw_coin(
            &mut self,
            amount: Option<Decimal>,
        ) -> FungibleBucket;
    }
}

define_interface! {
    Dex impl [ScryptoStub, Trait, ScryptoTestStub] {

        fn swap(
            &mut self,
            input_bucket: FungibleBucket,
        ) -> FungibleBucket;
    }
}
