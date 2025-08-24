use scrypto::prelude::*;
use scrypto_interface::*;

define_interface! {
    DefiProtocol impl [ScryptoStub, Trait, ScryptoTestStub] {

        fn deposit_all(
            &mut self,
            token: Bucket,
            coin: Option<FungibleBucket>,
            other_coin: Option<FungibleBucket>,
        ) -> (
            Decimal,                // Total coin amount
            Option<Decimal>         // Total other coin amount
        );

        // Withdraw all of the protocol tokens and coins from the component
        fn withdraw_all(&mut self) -> (
            Bucket,                 // Tokens
            Option<FungibleBucket>, // Coins
            Option<FungibleBucket>  // Other coins
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
            amount: Decimal,
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

        // Get the control of the Account; to use when a wrapper is definitively dismissed
        fn withdraw_account_badge(&mut self) -> NonFungibleBucket;
    }
}

define_interface! {
    Dex impl [ScryptoStub, Trait, ScryptoTestStub] {

        fn swap(
            &mut self,
            input_bucket: Bucket,           // Bucket of coins to swap
            output_resource: ResourceAddress,   // Output coins resource address
            use_remainings: bool,           // Whether to use remainings of previous partial swaps
        ) -> Bucket;                        // Bucket of output_resource
    }
}

define_interface! {
    Oracle impl [ScryptoStub, Trait, ScryptoTestStub] {

        fn get_price(
            &mut self,
            coin_address: ResourceAddress,  // The coin to get the price of
            morpher_data: HashMap<ResourceAddress, (String, String)>,   // Eventual Morpher data
        ) -> Decimal;                       // coin price
    }
}
