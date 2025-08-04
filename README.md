# Fund Manager
Fund Manager is a software to manage a fund that unstakes LSU rewards for a Validator owner and invests them in Radix DeFi protocols.  
Validator stakers are rewarded with fund units that represent a share of the fund and can be exchanged with invested coins.  

## Blueprints
This software is composed of multiple blueprints:  
- `FundManager` is the main blueprint, it must be instatiated first.  
- `MultiDexWrapper` is a consistent interface towards pools from Ociswap, Caviarnine and DefiPlaza.  
- `MultiOracleWrapper` is a unique interface towards Morpher, Ociswap and two simple internal oracles.  
The software also contains the `DefiProtocol` interface that can be used to talk to different protocols; the current implementation of the interface are:  
- `FluxWrapper`  for managing liquidity in the Flux protocol.  
- `OciswapLpPool2Wrapper` for managing liquidity in the newest Ociswap pools.  
- `SurgeWrapper` for managing liquidity provided to Surge.  
- `WeftWrapper` for managing liquidity in Weft Finance.  
- `RootFinanceWRapper` for managing liquidity provided to Root Finance.  

## Actors and badges

### Unauthenticated user
A user can exchange his fund units for whatever coin the contract will give him (or ask to swap it for a specific coin).  
The exchanged fund units are burned.  
The withdraw happens taking funds from only one DeFi protocol.  
If there's not enough liquidity in any protocol the unused fund units will be returned to the user; the user can execute new withdrawals to convert them too.  
A withdrawal fee makes everyone else a bit richer when someone withdraws: I actually withdraw just a part of my share of the fund value, the remaining part increases the value of the remaining fund units.  

### Bot
The bot badge allows to perform everyday's operations such as unstaking from the Validator and distributing freshly minted fund units.  
The distribution happens in a "push" way (an AccountLocker is used) and follows a snapshot happened 5 weeks before so everyone gets the exact value of the XRD he contributed to the Validator rewards.  
This badge can be held by a backend so that everything happens automatically without human intervention.  
The bot badge will also tell the component about how we want to share the funds among the different DeFi protocols; changing this setting will not directly move funds from one protocol to another, it will influence the future decisions about which protocol to withdraw from and which protocol to deposit to the future unstaked XRD.  

### Admin
There can be multiple admin badges; these allow to set metadata for the component and the coins.  
A single admin can't steal funds or alter the component functionality.  
There's a sort of multisignature system through which some admins can allow other admins to perform extraordinary tasks:  
- Withdraw the Validator badge to perform operations such as node maintenance.  
- Add/remove/replace DeFi protocol adapters, this will allow to fix bugs and also support eventual future DeFi protocols will appear.  
- Replace the DEX adapter to fix bugs and support any future DEX will appear.  
- Increase/decrease the minimum number of cosigners for multisig operations and mint new admin badges if the team grows/shrinks.  
- Replace the oracle adapter to fix bugs and support any future oracle will appear.  
- Set the withdrawal fee percentage.  
- Withdraw the fund manager badge.  
The admin badge is not fungible: one admin authorizes one single admin to perform one single operation.  

### Fund manager
There will be just one fund manager badge and will be locked in the main component.  
It is almighty: who owns it can do everything including stealing all funds and changing the rules of the game.  
Multiple admins can agree to withdraw it in an emergency situation.  

## Public methods

### withdraw
Exchange fund units for any coin in the fund or a specific coin.  
The method emits the `WithdrawFromFundEvent` that contains:  
- the amount of fund units burnt  
- the name of the DeFi protocols the withdraw happened from  

```
CALL_METHOD
    Address("<ACCOUNT>")
    "withdraw"
    Address("<FUND_UNIT_RESOURCE_ADDRESS>")
    Decimal("<AMOUNT>")
;
TAKE_ALL_FROM_WORKTOP
    Address("<FUND_UNIT_RESOURCE_ADDRESS>")
    Bucket("fund_units")
;
CALL_METHOD
    Address("<FUND_MANAGER_COMPONENT_ADDRESS>")
    "withdraw"
    Bucket("fund_units")
    Some(Address("<WANTED_COIN_RESOURCE_ADDRESS>"))
    Map<ResourceAddress, Tuple>(
        Address("<COIN_RESOURCE_ADDRESS>") => ("<MORPHER_MESSAGE>", "<MORPHER_SIGNATURE>"),
        ...
    )
;
CALL_METHOD
    Address("<ACCOUNT>")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP")
;
```

`<ACCOUNT>` is the user account.  
`<FUND_UNIT_RESOURCE_ADDRESS>` is the resource address of the fund units managed by the fund component.  
`<AMOUNT>` the amount of fund units the user wants to exchange.  
`<FUND_MANAGER_COMPONENT_ADDRESS>` the address of the fund manager component.  
`<WANTED_COIN_RESOURCE_ADDRESS>` is the address of the coin the user wants to receive. Replace the whole line with `None` if any coin is acceptable.  
`<COIN_RESOURCE_ADDRESS>` the resource address of a coin that is listed on the Morpher oracle.  
`<MORPHER_MESSAGE>` the message for the Morpher oracle regarding `<COIN_RESOURCE_ADDRESS>`.  
`<MORPHER_SIGNATURE>` the signature of `<MORPHER_MESSAGE>`.  

### fund\_unit\_value
Returns the net and the gross (including withdrawal fee) dollar value of a fund unit.  
A preview of the transaction is enough to get the values; it is not necessary to consume fees actually executing it.  

```
CALL_METHOD
    Address("<FUND_MANAGER_COMPONENT_ADDRESS>")
    "fund_unit_value"
;
```

`<FUND_MANAGER_COMPONENT_ADDRESS>` the address of the fund manager component.  

### fund\_details
Returns an HashMap containing the amount invested in each DeFi protocol.
A preview of the transaction is enough to get the values; it is not necessary to consume fees actually executing it.  

```
CALL_METHOD
    Address("<FUND_MANAGER_COMPONENT_ADDRESS>")
    "fund_details"
;
```

`<FUND_MANAGER_COMPONENT_ADDRESS>` the address of the fund manager component.  

### get\_price
Reurns the dollar price of a coin.  
A preview of the transaction is enough to get the values; it is not necessary to consume fees actually executing it.  

```
CALL_METHOD
    Address("<ORACLE_COMPONENT_ADDRESS>")
    "get_price"
    Address("<COIN_RESOURCE_ADDRESS>")
    Map<ResourceAddress, Tuple>(
        Address("<COIN_RESOURCE_ADDRESS>") => ("<MORPHER_MESSAGE>", "<MORPHER_SIGNATURE>"),
        ...
    )
;
```

`<ORACLE_COMPONENT_ADDRESS>` the address of the oracle wrapper component.  
`<COIN_RESOURCE_ADDRESS>` the resource address of the coin the user wants to know the value of.  
`<MORPHER_MESSAGE>` the message for the Morpher oracle regarding `<COIN_RESOURCE_ADDRESS>`. The Map can be empty if the price is provided by a differnt oracle from Morpher.  
`<MORPHER_SIGNATURE>` the signature of `<MORPHER_MESSAGE>`.  

## Bot callable methods

### start\_unlock\_owner\_stake\_units
Starts the unlock of owner LSUs on the Validator.  

```
CALL_METHOD
    Address("<ACCOUNT>")
    "create_proof_of_amount"
    Address("<BOT_BADGE>")
    Decimal("1")
;
CALL_METHOD
    Address("<FUND_MANAGER_COMPONENT_ADDRESS>")
    "start_unlock_owner_stake_units"
    Decimal("<AMOUNT>")
;
```

`<ACCOUNT>` is the bot account.  
`<BOT_BADGE>` is the badge held by the bot account.  
`<FUND_MANAGER_COMPONENT_ADDRESS>` the address of the fund manager component.  
`<AMOUNT>` the amount of LSU to unlock.  

### start\_unstake
Completes the unlock of the Validator owner LSUs and starts their unstake.  
This method emits a `LsuUnstakeStartedEvent` that contains:
- the amount of LSU that are being unstaked  
- the NonFungibleId of the minted Claim NFT  

```
CALL_METHOD
    Address("<ACCOUNT>")
    "create_proof_of_amount"
    Address("<BOT_BADGE>")
    Decimal("1")
;
CALL_METHOD
    Address("<FUND_MANAGER_COMPONENT_ADDRESS>")
    "start_unstake"
;
```

`<ACCOUNT>` is the bot account.  
`<BOT_BADGE>` is the badge held by the bot account.  
`<FUND_MANAGER_COMPONENT_ADDRESS>` the address of the fund manager component.  

### finish\_unstake
Compleses the unstake of LSUs and invests the resulting XRD in one of the available DeFi protocols.  
It also mints new fund units to reward stakers.  
This method emits a `LsuUnstakeCompletedEvent` reporting:  
- the amount of unstaked XRD  
- the name of the DeFi protocol it invested in  
- the number of new fund units that will be distributed  

```
CALL_METHOD
    Address("<ACCOUNT>")
    "create_proof_of_amount"
    Address("<BOT_BADGE>")
    Decimal("1")
;
CALL_METHOD
    Address("<FUND_MANAGER_COMPONENT_ADDRESS>")
    "finish_unstake"
    "<CLAIM_NFT_ID>"
    Map<ResourceAddress, Tuple>(
        Address("<COIN_RESOURCE_ADDRESS>") => ("<MORPHER_MESSAGE>", "<MORPHER_SIGNATURE>"),
        ...
    )
;
```

`<ACCOUNT>` is the bot account.  
`<BOT_BADGE>` is the badge held by the bot account.  
`<FUND_MANAGER_COMPONENT_ADDRESS>` the address of the fund manager component.  
`<CLAIM_NFT_ID>` the NonFungibleId of the Claim NFT to complete the unstake.  
`<COIN_RESOURCE_ADDRESS>` the resource address of a coin that is listed on the Morpher oracle.  
`<MORPHER_MESSAGE>` the message for the Morpher oracle regarding `<COIN_RESOURCE_ADDRESS>`.  
`<MORPHER_SIGNATURE>` the signature of `<MORPHER_MESSAGE>`.  

### fund\_units\_distribution
Airdrops the fund units minted when an unstake is completed.  
This method can be called more than once in the number of stakers is high; it is risky to call it with a list of more than 60 stakers to reward.  

```
CALL_METHOD
    Address("<ACCOUNT>")
    "create_proof_of_amount"
    Address("<BOT_BADGE>")
    Decimal("1")
;
CALL_METHOD
    Address("<FUND_MANAGER_COMPONENT_ADDRESS>")
    "fund_units_distribution"
    Map<Address, Decimal>(
        Address("<RECIPIENT_ADDRESS>") => Decimal("<AMOUNT>"),
        ...
    )
    <MORE_STAKERS>
;
```

`<ACCOUNT>` is the bot account.  
`<BOT_BADGE>` is the badge held by the bot account.  
`<FUND_MANAGER_COMPONENT_ADDRESS>` the address of the fund manager component.  
`<RECIPIENT_ADDRESS>` the account address of a recipient of the airdrop.  
`<AMOUNT>` the number of fund units to send to `<RECIPIENT_ADDRESS>`.  
`<MORE_STAKERS>` must be `false` if the airdrop is completed, `true` if there will be more calls to this method.  

### update\_defi\_protocols\_info
This method can update the estimation of the dollar value of the investment in each DeFi protocol and/or the desired percentage of value to invest in each DeFi protocol.  

```
CALL_METHOD
    Address("<ACCOUNT>")
    "create_proof_of_amount"
    Address("<BOT_BADGE>")
    Decimal("1")
;
CALL_METHOD
    Address("<FUND_MANAGER_COMPONENT_ADDRESS>")
    "update_defi_protocols_info"
    Map<String, Decimal>(
        Address("<PROTOCOL_NAME>") => Decimal("<VALUE>"),
        ...
    )
    Map<String, Decimal>(
        Address("<PROTOCOL_NAME>") => <DESIRED_PERCENTAGE>u8,
        ...
    )
```

`<ACCOUNT>` is the bot account.  
`<BOT_BADGE>` is the badge held by the bot account.  
`<FUND_MANAGER_COMPONENT_ADDRESS>` the address of the fund manager component.  
`<PROTOCOL_NAME>` is one of the DeFi protocols whose information need to be updates.  
`<VALUE>` the dollar value of the investment in `<PROTOCOL_NAME>`.  
`<DESIRED_PERCENTAGE>` the desired percentage of the fund value to be invested in `<PROTOCOL_NAME>`.  


### update\_price
Updates the price for the FixedPrice or the FixedMultiplier oracles.  
FixedPrice is an oracle that always returns the same number (example: xUSDC -> 1).  
FixedMultiplier is an oracle that returns the price of a reference coin multiplied by a constant (example: LSULP -> 1.128 * the price of XRD).  

```
CALL_METHOD
    Address("<ACCOUNT>")
    "create_proof_of_amount"
    Address("<BOT_BADGE>")
    Decimal("1")
;
CALL_METHOD
    Address("<ORACLE_COMPONENT_ADDRESS>")
    "update_price"
    Address("<COIN_RESOURCE_ADDRESS>")
    Some(Decimal("<PRICE>"))
    Some(Decimal("<PRICE_MULTIPLIER>"))
;
```

`<ACCOUNT>` is the bot account.  
`<BOT_BADGE>` is the badge held by the bot account.  
`<ORACLE_COMPONENT_ADDRESS>` the address of the oracle component.  
`<COIN_RESOURCE_ADDRESS>` the resource address of the coin whose price information needs to be set.  
`<PRICE>` dollar value of the coin for the FixedPrice oracle. In case of a FixedMultiplier oracle the whole line should be `None`.  
`<PRICE_MULTIPLIER>` multiplier to apply to the price of the reference coin for the FixedMultiplier oracle. In case of a FixedPrice oracle the whole line should be `None`.  

## Admin callable methods
TODO...  

## Disclaimer
Untested software, for educational purposes only, no warranty.  

## RIP Dan Hughes
We will never forget you.  
