use scrypto::prelude::*;
use scrypto_interface::*;

define_interface! {
    DefiProtocol impl [ScryptoStub, Trait, ScryptoTestStub] {

        fn deposit_protocol_token(
            &mut self,
            token: Bucket,
        ) -> (
            Decimal,                // Total coin amount
            Option<Decimal>         // Total other coin amount
        );

        fn withdraw_protocol_token(
            &mut self,
            amount: Option<Decimal>,
        ) -> (
            Bucket,                 // Tokens
            Decimal,                // Total coin amount
            Option<Decimal>         // Total other coin amount
        );

        fn deposit_coin(
            &mut self,
            coin: FungibleBucket,
            other_coin: Option<FungibleBucket>,
            message: Option<String>,
            signature: Option<String>,
        ) -> (
            Decimal,                // Total coin amount
            Option<Decimal>         // Total other coin amount
        );

        fn withdraw_coin(
            &mut self,
            amount: Option<Decimal>,
            other_coin_to_coin_price_ratio: Option<Decimal>,
        ) -> (
            FungibleBucket,         // Coins
            Option<FungibleBucket>, // Other coins
            Decimal,                // Total coin amount
            Option<Decimal>         // Total other coin amount
        );

        fn get_coin_amounts(&mut self) -> (
            Decimal,                // Total coin amount
            Option<Decimal>         // Total other coin amount
        );
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
            morpher_data: HashMap<ResourceAddress, (String, String)>,
        ) -> Decimal;
    }
}
