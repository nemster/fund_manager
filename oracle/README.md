# MultiOracleWrapper

This oracle uses multiple sources to get $ price of existing tokens on the Radix platform.  
It can perform multiple steps to obtain the price of a coin. As an example `w2-LSULP` price depends on `LSULP` price, which depends on `XRD` price, which depends on `hUSDC` price.  
This component is free of use for everyone.  

## How to use it?
You can call invoke the `get_price` method from Scrypto to get the price as a return value:
```
extern_blueprint! {
    "package_rdx1pkkqwel9uey0zsdut9vcwh29e66lvez24jm9uxhnpj0maya0kt0lyp",
    MultiOracleWrapper {
        fn get_price(
            &mut self,
            coin_address: ResourceAddress,                            // The coin to get the price of
            morpher_data: HashMap<ResourceAddress, (String, String)>, // Unused
        ) -> Decimal;
    }
}
```

You can also invoke it from a transaction manifest and look for the `PriceUpdated` event(s).  
```
CALL_METHOD
    Address("component_rdx1crca7ztnmus92avl4e8gh90zntj48nh7k26zp5mhnastg7gt7fmd5c")
    "get_price"
    Address("<THE COIN YOU ARE INTERESTED IN>")
    Map<Address, Tuple>()
;
```

## Supported coins

### Stable coins
The oracle always returns a fixed price for these coins.

- `fUSD`: 1  
- `hUSDC`: 1  
- `xUSDC`: 1  
- `xUSDT`: 1  

### Coins listed on Ociswap
Only newest Ociswap pool include a reliable oracle.  

- `XRD`: obtained from the XRD/hUSDC pool  
- `hWBTC`: obtained from the hWBTC/XRD pool  
- `hETH`: obtained from the XRD/hETH pool  
- `WEFT`: obtained from the WEFT/XRD pool  
- `REDDICKS`: obtained from the REDDICKS/XRD pool  
- `ILIS`: obtained from the ILIS/XRD pool  
- `OCI`: obtained from the OCI/XRD pool  
- `WOWO`: obtained from the WOWO/XRD pool  
- `hBNB`: obtained from the hBNB/XRD pool  
- `xwBTC`: obtained from the xwBTC/XRD pool  
- `JWLXRD`: obtained from the XRD/JWLXRD pool  

### Other Radix ecosystem coins
Each of these ones are minted by a component that the oracle can ask information to.  

- `LSULP`: obtained from the Caviarnine LsuPool component  
- `FU`: obtained from the ADDIX+FOMO FundManager component  
- `SLP`: obtained from the Surge Exchange component  

### Weft wrapped coins
The oracle can ask the Weft LendingPool component the wrapped coin / base coin ratio.  

- `w2-hETH`: obtained from the Weft LendingPool component  
- `w2-hUSDC`: obtained from the Weft LendingPool component  
- `w2-hUSDT`: obtained from the Weft LendingPool component  
- `w2-hWBTC`: obtained from the Weft LendingPool component  
- `w2-LSULP`: obtained from the Weft LendingPool component  
- `w2-XRD`: obtained from the Weft LendingPool component  

### Validators LSUs
The oracle can ask a Validator about the LSU/XRD ratio.  


### Single resource pools
The oracle can ask a OneResourcePool (such as Defiplaza staking pools) the pool unit / staked coin ratio.  

-`sREDDICKS`: obtained from Defiplaza staking pool  
-`sWOWO`: obtained from Defiplaza staking pool  

### Two resources pools
Many Radix dApps use the TwoResourcePool component; the oracle can ask them the pool unit / coins ratios.  

### Multiple resources pools
Anyone using the MultiResourcePool component?  

