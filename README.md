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

## Fund value
It is possible to query the value of the investment of all of the DeFi protocols by calling the `fund_details` method of the `FundManager` component.  
On each deposit or withdraw operation on a DeFi protocol, the `FundManager` updates the value of the investment in that protocol and emits appropriate one of these events: `LsuUnstakeCompletedEvent`, `WithdrawFromFundEvent`, `AdminDepositEvent`.  
If a protocol is not used for a while its value can be obsolete because of price change of the invested coins or because of the yelds; in this case the bot can invoke the `update_defi_protocols_info` method to have the value updated. This method also emits the `ProtocolValueUpdateEvent`.  

The value of a fund unit can be obtained by dividing the total value of the fund by the fund unit current supply (gross value) and applying the withdrawal fee (net value).  
The `fund_unit_value` method returns both values.  

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
- the remaining value of the DeFi protocol it withdrew from  
This method returns one or two buckets of coins used by a DeFi protocol or the requested coin.  

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
`<MORPHER_MESSAGE>` the message for the Morpher oracle regarding `<COIN_RESOURCE_ADDRESS>`. The Map can be empty if the price is provided by a different oracle from Morpher.  
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
`<BOT_BADGE>` is the resource address of the badge held by the bot account.  
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
`<BOT_BADGE>` is the resource address of the badge held by the bot account.  
`<FUND_MANAGER_COMPONENT_ADDRESS>` the address of the fund manager component.  

### finish\_unstake
Compleses the unstake of LSUs and invests the resulting XRD in one of the available DeFi protocols. It also sends part of the XRD to the account managing the buyback fund and mints new fund units to reward stakers.  
This method emits a `LsuUnstakeCompletedEvent` reporting:  
- the amount of unstaked XRD  
- the name of the DeFi protocol it invested in  
- the number of new fund units that will be distributed  
- the new total value of the DeFi protocol it invested in

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
`<BOT_BADGE>` is the resource address of the badge held by the bot account.  
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
`<BOT_BADGE>` is the resource address of the badge held by the bot account.  
`<FUND_MANAGER_COMPONENT_ADDRESS>` the address of the fund manager component.  
`<RECIPIENT_ADDRESS>` the account address of a recipient of the airdrop.  
`<AMOUNT>` the number of fund units to send to `<RECIPIENT_ADDRESS>`.  
`<MORE_STAKERS>` must be `false` if the airdrop is completed, `true` if there will be more calls to this method.  

### update\_defi\_protocols\_info
This method can ask FundManager to update the estimation of the dollar value of the investment in some DeFi protocols and/or the desired percentage of value to invest in each DeFi protocol.  
It emits a `ProtocolValueUpdateEvent` for each coin whose value is updated. The event contains:  
- the name of the protocol  
- the updated value of the coins invested in the protocol  

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
    Set<String>(
        "<PROTOCOL_NAME>",
        ...
    )
    Map<String, Decimal>(
        "<PROTOCOL_NAME>" => <DESIRED_PERCENTAGE>u8,
        ...
    )
;
```

`<ACCOUNT>` is the bot account.  
`<BOT_BADGE>` is the resource address of the badge held by the bot account.  
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
`<BOT_BADGE>` is the resource address of the badge held by the bot account.  
`<ORACLE_COMPONENT_ADDRESS>` the address of the oracle component.  
`<COIN_RESOURCE_ADDRESS>` the resource address of the coin whose price information needs to be set.  
`<PRICE>` dollar value of the coin for the FixedPrice oracle. In case of a FixedMultiplier oracle the whole line should be `None`.  
`<PRICE_MULTIPLIER>` multiplier to apply to the price of the reference coin for the FixedMultiplier oracle. In case of a FixedPrice oracle the whole line should be `None`.  

### get\_incentives
Withdraws WEFT incentives from WEFT Finance to the WEFT wrapper component.  

```
CALL_METHOD
    Address("<ACCOUNT>")
    "create_proof_of_amount"
    Address("<BOT_BADGE>")
    Decimal("1")
;
CALL_METHOD
    Address("<WEFT_WRAPPER_ADDRESS>")
    "get_incentives"
    <INCENTIVE_TYPE>u8
    Decimal("<AMOUNT>")
;
```

`<ACCOUNT>` is the bot account.  
`<BOT_BADGE>` is the resource address of the badge held by the bot account.  
`<WEFT_WRAPPER_ADDRESS>` the component address of the weft wrapper.  
`<INCENTIVE_TYPE>` WEFT Finance supports different type of incentives, for liquidity providers this must be 4.  
`<AMOUNT>` the amount of WEFT to withdraw from the WEFT component; it can be read from the WEFT Claimer NFT in the Account used by the wrapper.  

## Admin callable methods

### authorize\_admin\_operation
Allow another admin to perform a restricted operation.  
The authorization persists until the operation is performed or two days has passed (timeout).  
Allowers must agree, not just on the operation to perform, but on most of the parameter to pass to it too. As en example, for `mint_admin_badge` both the authorizers and the admin that executes the operation must pass the same `<RECEIVER_ACCOUNT>`.  

```
CALL_METHOD
    Address("<ACCOUNT>")
    "create_proof_of_non_fungibles"
    Address("<ADMIN_BADGE>")
    Array<NonFungibleLocalId>(NonFungibleLocalId("<MY_ADMIN_BADGE_ID>"));
;
POP_FROM_AUTH_ZONE
    Proof("admin_proof")
;
CALL_METHOD
    Address("<FUND_MANAGER_COMPONENT_ADDRESS>")
    "authorize_admin_operation"
    Proof("admin_proof")
    <ADMIN_BADGE_ID>u8
    <AUTHORIZED_OPERATION>u8
    Some("<PROTOCOL_NAME>")
    Some(Decimal("<WITHDRAWAL_FEE>"))
    Some(Address("<RECEIVER_ACCOUNT>"))
;
```

`<ACCOUNT>` is the admin account.  
`<ADMIN_BADGE>` is the resource address of the badge held by the admin account.  
`<MY_ADMIN_BADGE_ID>` is the numeric identifier of the admin badge owned by the account that is executing this transaction.  
`<FUND_MANAGER_COMPONENT_ADDRESS>` the address of the fund manager component.  
`<ADMIN_BADGE_ID>` is the numeric identifier of the admin badge to be allowed (obviously must be different from `<MY_ADMIN_BADGE_ID>`).   
`<AUTHORIZED_OPERATION>` a number representing the operation to authorize:  
0 -> `withdraw_validator_badge`  
1 -> `add_defi_protocol`  
2 -> `remove_defi_protocol`  
3 -> `set_dex_component`  
4 -> `decrease_min_authorizers`  
5 -> `increase_min_authorizers`  
6 -> `mint_admin_badge`  
7 -> `set_oracle_component`  
8 -> `withdraw_fund_manager_badge`  
9 -> `set_withdrawal_fee`  
10 -> `mint_bot_badge`  
11 -> `set_buyback_fund`  
`<PROTOCOL_NAME>` is the name of the protocol to add/remove for `add_defi_protocol` and `remove_defi_protocol` operations, `None` for all the other operations.  
`<PERCENTAGE>` is the percentage to set for `set_withdrawal_fee` or `set_buyback_fund` operations, `None` for all the other operations.  
`<RECEIVER_ACCOUNT>` is the account address that will receive the badge for the `mint_admin_badge` and `mint_bot_badge` operations, `None` for all the other operations.  

### withdraw\_validator\_badge
The Validator badge is usually deposited in the FundManager component, this method lets an authorized admin withdraw it.  
Returns a bucket containing the Validator badge.  

```
CALL_METHOD
    Address("<ACCOUNT>")
    "create_proof_of_non_fungibles"
    Address("<ADMIN_BADGE>")
    Array<NonFungibleLocalId>(NonFungibleLocalId("<MY_ADMIN_BADGE_ID>"));
;
POP_FROM_AUTH_ZONE
    Proof("admin_proof")
;
CALL_METHOD
    Address("<FUND_MANAGER_COMPONENT_ADDRESS>")
    "withdraw_validator_badge"
    Proof("admin_proof")
;
CALL_METHOD
    Address("<ACCOUNT>")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP")
;
```

`<ACCOUNT>` is the admin account.  
`<ADMIN_BADGE>` is the resource address of the badge held by the admin account.  
`<MY_ADMIN_BADGE_ID>` is the numeric identifier of the admin badge owned by the account that is executing this transaction.  
`<FUND_MANAGER_COMPONENT_ADDRESS>` the address of the fund manager component.  

### deposit\_validator\_badge
Put the Validator badge back in the FundManager component.  
This operation does not need authorization by a different admin.  

```
CALL_METHOD
    Address("<ACCOUNT>")
    "withdraw_non_fungibles"
    Address("<VALIDATOR_BADGE>")
    Array<NonFungibleLocalId>(NonFungibleLocalId("<VALIDATOR_BADGE_ID>"))
;
TAKE_ALL_FROM_WORKTOP
    Address("<VALIDATOR_BADGE>")
    Bucket("validator_badge")
;
CALL_METHOD
    Address("<ACCOUNT>")
    "create_proof_of_non_fungibles"
    Address("<ADMIN_BADGE>")
    Array<NonFungibleLocalId>(NonFungibleLocalId("<MY_ADMIN_BADGE_ID>"));
;
CALL_METHOD
    Address("<FUND_MANAGER_COMPONENT_ADDRESS>")
    "deposit_validator_badge"
    Bucket("validator_badge")
;
```

`<ACCOUNT>` is the admin account.  
`<VALIDATOR_BADGE>` is the resource address of Validator badges.  
`<VALIDATOR_BADGE_ID>` is the unique identifier of a Validator badge.  
`<ADMIN_BADGE>` is the resource address of the badge held by the admin account.  
`<MY_ADMIN_BADGE_ID>` is the numeric identifier of the admin badge owned by the account that is executing this transaction.  
`<FUND_MANAGER_COMPONENT_ADDRESS>` the address of the fund manager component.  

### add\_defi\_protocol
This method allows an authorized admin to add a new DeFi protocol to the ones managed by the FundManager.  

```
CALL_METHOD
    Address("<ACCOUNT>")
    "create_proof_of_non_fungibles"
    Address("<ADMIN_BADGE>")
    Array<NonFungibleLocalId>(NonFungibleLocalId("<MY_ADMIN_BADGE_ID>"));
;
POP_FROM_AUTH_ZONE
    Proof("admin_proof")
;
CALL_METHOD
    Address("<FUND_MANAGER_COMPONENT_ADDRESS>")
    "add_defi_protocol"
    Proof("admin_proof")
    "<PROTOCOL_NAME>"
    Address("<COIN_ADDRESS>")
    Address("<TOKEN_ADDRESS>")
    Some(Address("<OTHER_COIN_ADDRESS>"))
    <DESIRED_PERCENTAGE>u8
    Address("<COMPONENT_ADDRESS>")
    Some(Address("<MORPHER_COIN_ADDRESS>"))
;
```

`<ACCOUNT>` is the admin account.  
`<ADMIN_BADGE>` is the resource address of the badge held by the admin account.  
`<MY_ADMIN_BADGE_ID>` is the numeric identifier of the admin badge owned by the account that is executing this transaction.  
`<FUND_MANAGER_COMPONENT_ADDRESS>` the address of the fund manager component.  
`<PROTOCOL_NAME>` is a conventional name that will be used to identify this protocol. Is a protocol with such a name already exists the new one will replace the existing one and take all of the liquidity from it (so `<TOKEN_ADDRESS>` must be the same).  
`<COIN_ADDRESS>` the resource address of the coin that will be deposited in this protocol.  
`<TOKEN_ADDRESS>` the resource address of the receipt that the protocol returns when a deposit operation happens. It can be both a fungible (WEFT) or a non fungible (Root Finance).  
`<OTHER_COIN_ADDRESS>` if the protocol allows depositing more two coins togheter (as an example a dex pool), this is the resource address of the second coin to be deposited. Otherwise the line must be `None`.  
`<DESIRED_PERCENTAGE>` the percentage value share of the fund that must be deposited in this protocol.  
`<COMPONENT_ADDRESS>` the address of the wrapper component implementing the `DefiProtocol` interface for this protocol.  
`<MORPHER_COIN_ADDRESS>` some protocols (Flux) need data from the Morpher oracle when performing operations on them. This is the resource address of the coin whose data are needed by the protocol. If this is not the case the line must be `None`.  

### remove\_defi\_protocol
This method allows an authorized admin to remove a DeFi protocol wrapper from the FundManager.  
Warning: the admin will receive all of the liquidity (tokens) in the protocol so it's advisable to set the desired percentage to zero and let users withdraw the liquidity before authorizing this operation.  

```
CALL_METHOD
    Address("<ACCOUNT>")
    "create_proof_of_non_fungibles"
    Address("<ADMIN_BADGE>")
    Array<NonFungibleLocalId>(NonFungibleLocalId("<MY_ADMIN_BADGE_ID>"));
;
POP_FROM_AUTH_ZONE
    Proof("admin_proof")
;
CALL_METHOD
    Address("<FUND_MANAGER_COMPONENT_ADDRESS>")
    "remove_defi_protocol"
    Proof("admin_proof")
    "<PROTOCOL_NAME>"
;
CALL_METHOD
    Address("<ACCOUNT>")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP")
;
```

`<ACCOUNT>` is the admin account.  
`<ADMIN_BADGE>` is the resource address of the badge held by the admin account.  
`<MY_ADMIN_BADGE_ID>` is the numeric identifier of the admin badge owned by the account that is executing this transaction.  
`<FUND_MANAGER_COMPONENT_ADDRESS>` the address of the fund manager component.  
`<PROTOCOL_NAME>` is the name of the protocol to remove.  

### set\_dex\_component
This method allows an authorized admin to replace the dex component used by FundManager.  

```
CALL_METHOD
    Address("<ACCOUNT>")
    "create_proof_of_non_fungibles"
    Address("<ADMIN_BADGE>")
    Array<NonFungibleLocalId>(NonFungibleLocalId("<MY_ADMIN_BADGE_ID>"));
;
POP_FROM_AUTH_ZONE
    Proof("admin_proof")
;
CALL_METHOD
    Address("<FUND_MANAGER_COMPONENT_ADDRESS>")
    "set_dex_component"
    Proof("admin_proof")
    Address("<DEX_COMPONENT>")
;
```

`<ACCOUNT>` is the admin account.  
`<ADMIN_BADGE>` is the resource address of the badge held by the admin account.  
`<MY_ADMIN_BADGE_ID>` is the numeric identifier of the admin badge owned by the account that is executing this transaction.  
`<FUND_MANAGER_COMPONENT_ADDRESS>` the address of the fund manager component.  
`<DEX_COMPONENT>` the address of the new dex component to use.  

### decrease\_min\_authorizers
This method decreases by one the minimum number of admins required to authorize an admin to perform a restricted operation.  
Warning: reducing this number to zero will make authorizations no longer required.  

```
CALL_METHOD
    Address("<ACCOUNT>")
    "create_proof_of_non_fungibles"
    Address("<ADMIN_BADGE>")
    Array<NonFungibleLocalId>(NonFungibleLocalId("<MY_ADMIN_BADGE_ID>"));
;
POP_FROM_AUTH_ZONE
    Proof("admin_proof")
;
CALL_METHOD
    Address("<FUND_MANAGER_COMPONENT_ADDRESS>")
    "decrease_min_authorizers"
    Proof("admin_proof")
;
```

`<ACCOUNT>` is the admin account.  
`<ADMIN_BADGE>` is the resource address of the badge held by the admin account.  
`<MY_ADMIN_BADGE_ID>` is the numeric identifier of the admin badge owned by the account that is executing this transaction.  
`<FUND_MANAGER_COMPONENT_ADDRESS>` the address of the fund manager component.  

### increase\_min\_authorizers
This method increases by one the minimum number of admins required to authorize an admin to perform a restricted operation.  
It is not possible to increase this number above the number of existing admin badges - 1.  

```
CALL_METHOD
    Address("<ACCOUNT>")
    "create_proof_of_non_fungibles"
    Address("<ADMIN_BADGE>")
    Array<NonFungibleLocalId>(NonFungibleLocalId("<MY_ADMIN_BADGE_ID>"));
;
POP_FROM_AUTH_ZONE
    Proof("admin_proof")
;
CALL_METHOD
    Address("<FUND_MANAGER_COMPONENT_ADDRESS>")
    "increase_min_authorizers"
    Proof("admin_proof")
;
```

`<ACCOUNT>` is the admin account.  
`<ADMIN_BADGE>` is the resource address of the badge held by the admin account.  
`<MY_ADMIN_BADGE_ID>` is the numeric identifier of the admin badge owned by the account that is executing this transaction.  
`<FUND_MANAGER_COMPONENT_ADDRESS>` the address of the fund manager component.  

### mint\_admin\_badge
Mints a new admin badge and sends it to the specified account.  

```
CALL_METHOD
    Address("<ACCOUNT>")
    "create_proof_of_non_fungibles"
    Address("<ADMIN_BADGE>")
    Array<NonFungibleLocalId>(NonFungibleLocalId("<MY_ADMIN_BADGE_ID>"));
;
POP_FROM_AUTH_ZONE
    Proof("admin_proof")
;
CALL_METHOD
    Address("<FUND_MANAGER_COMPONENT_ADDRESS>")
    "mint_admin_badge"
    Proof("admin_proof")
    Address("<RECEIVER_ACCOUNT>")
;
```

`<ACCOUNT>` is the admin account.  
`<ADMIN_BADGE>` is the resource address of the badge held by the admin account.  
`<MY_ADMIN_BADGE_ID>` is the numeric identifier of the admin badge owned by the account that is executing this transaction.  
`<FUND_MANAGER_COMPONENT_ADDRESS>` the address of the fund manager component.  
`<RECEIVER_ACCOUNT>` is the account address that will receive the admin badge.  

### mint\_bot\_badge
Mints a new bot badge and sends it to the specified account.  

```
CALL_METHOD
    Address("<ACCOUNT>")
    "create_proof_of_non_fungibles"
    Address("<ADMIN_BADGE>")
    Array<NonFungibleLocalId>(NonFungibleLocalId("<MY_ADMIN_BADGE_ID>"));
;
POP_FROM_AUTH_ZONE
    Proof("admin_proof")
;
CALL_METHOD
    Address("<FUND_MANAGER_COMPONENT_ADDRESS>")
    "mint_bot_badge"
    Proof("admin_proof")
    Address("<RECEIVER_ACCOUNT>")
;
```

`<ACCOUNT>` is the admin account.  
`<ADMIN_BADGE>` is the resource address of the badge held by the admin account.  
`<MY_ADMIN_BADGE_ID>` is the numeric identifier of the admin badge owned by the account that is executing this transaction.  
`<FUND_MANAGER_COMPONENT_ADDRESS>` the address of the fund manager component.  
`<RECEIVER_ACCOUNT>` is the account address that will receive the bot badge.  

### set\_oracle\_component
Replaces the current oracle componet with the specified one.  

```
CALL_METHOD
    Address("<ACCOUNT>")
    "create_proof_of_non_fungibles"
    Address("<ADMIN_BADGE>")
    Array<NonFungibleLocalId>(NonFungibleLocalId("<MY_ADMIN_BADGE_ID>"));
;
POP_FROM_AUTH_ZONE
    Proof("admin_proof")
;
CALL_METHOD
    Address("<FUND_MANAGER_COMPONENT_ADDRESS>")
    "set_oracle_component"
    Proof("admin_proof")
    Address("<COMPONENT_ADDRESS>")
;
```

`<ACCOUNT>` is the admin account.  
`<ADMIN_BADGE>` is the resource address of the badge held by the admin account.  
`<MY_ADMIN_BADGE_ID>` is the numeric identifier of the admin badge owned by the account that is executing this transaction.  
`<FUND_MANAGER_COMPONENT_ADDRESS>` the address of the fund manager component.  
`<COMPONENT_ADDRESS>` the address of the new oracle component; it must implement the `Oracle` interface.  

### withdraw\_fund\_manager\_badge
Withdraws the fund manager badge from the FundManager component.  
Warning: the admin that receives the fund manager badge can do almost anything, included stealing all funds. Moreover the FundManager component will not work without this badge. Authorize this operation only in case of emergency.  

```
CALL_METHOD
    Address("<ACCOUNT>")
    "create_proof_of_non_fungibles"
    Address("<ADMIN_BADGE>")
    Array<NonFungibleLocalId>(NonFungibleLocalId("<MY_ADMIN_BADGE_ID>"));
;
POP_FROM_AUTH_ZONE
    Proof("admin_proof")
;
CALL_METHOD
    Address("<FUND_MANAGER_COMPONENT_ADDRESS>")
    "withdraw_fund_manager_badge"
    Proof("admin_proof")
;
CALL_METHOD
    Address("<ACCOUNT>")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP")
;
```

`<ACCOUNT>` is the admin account.  
`<ADMIN_BADGE>` is the resource address of the badge held by the admin account.  
`<MY_ADMIN_BADGE_ID>` is the numeric identifier of the admin badge owned by the account that is executing this transaction.  
`<FUND_MANAGER_COMPONENT_ADDRESS>` the address of the fund manager component.  

### deposit\_fund\_manager\_badge
Puts the fund manager badge back to its place in the FundManager component.  

``` 
CALL_METHOD
    Address("<ACCOUNT>")
    "withdraw"
    Address("<FUND_MANAGER_BADGE>")
    Decimal("1")
;
TAKE_ALL_FROM_WORKTOP
    Address("<FUND_MANAGER_BADGE>")
    Bucket("fund_manager_badge")
;   
CALL_METHOD
    Address("<ACCOUNT>") 
    "create_proof_of_non_fungibles"
    Address("<ADMIN_BADGE>")
    Array<NonFungibleLocalId>(NonFungibleLocalId("<MY_ADMIN_BADGE_ID>"));
;
CALL_METHOD
    Address("<FUND_MANAGER_COMPONENT_ADDRESS>")
    "deposit_fund_manager_badge"
    Bucket("fund_manager_badge")
;
```

`<ACCOUNT>` is the admin account.
`<FUND_MANAGER_BADGE>` is the resource address of the fund manager badge.   
`<ADMIN_BADGE>` is the resource address of the badge held by the admin account.  
`<MY_ADMIN_BADGE_ID>` is the numeric identifier of the admin badge owned by the account that is executing this transaction.  
`<FUND_MANAGER_COMPONENT_ADDRESS>` the address of the fund manager component.  

### set\_withdrawal\_fee
Updates the percentage fee that users leave in the protocol during a withdraw operation.  

``` 
CALL_METHOD
    Address("<ACCOUNT>")
    "create_proof_of_non_fungibles"
    Address("<ADMIN_BADGE>")
    Array<NonFungibleLocalId>(NonFungibleLocalId("<MY_ADMIN_BADGE_ID>"));
;
POP_FROM_AUTH_ZONE
    Proof("admin_proof") 
;
CALL_METHOD
    Address("<FUND_MANAGER_COMPONENT_ADDRESS>")
    "set_withdrawal_fee"
    Proof("admin_proof")
    Decimal("<PERCENTAGE>")
;
```

`<ACCOUNT>` is the admin account.  
`<ADMIN_BADGE>` is the resource address of the badge held by the admin account.  
`<MY_ADMIN_BADGE_ID>` is the numeric identifier of the admin badge owned by the account that is executing this transaction.  
`<FUND_MANAGER_COMPONENT_ADDRESS>` the address of the fund manager component.  
`<PERCENTAGE>` is the percentage fee to set.  

### set\_buyback\_fund
Updates the percentage of XRD allocated to the buyback fund and the account that manages it.  

``` 
CALL_METHOD
    Address("<ACCOUNT>")
    "create_proof_of_non_fungibles"
    Address("<ADMIN_BADGE>")
    Array<NonFungibleLocalId>(NonFungibleLocalId("<MY_ADMIN_BADGE_ID>"));
;
POP_FROM_AUTH_ZONE
    Proof("admin_proof")
;
CALL_METHOD
    Address("<FUND_MANAGER_COMPONENT_ADDRESS>")
    "set_buyback_fund"
    Proof("admin_proof")
    Decimal("<PERCENTAGE>")
    Address("<RECEIVER_ACCOUNT>")
;
```

`<ACCOUNT>` is the admin account.  
`<ADMIN_BADGE>` is the resource address of the badge held by the admin account.  
`<MY_ADMIN_BADGE_ID>` is the numeric identifier of the admin badge owned by the account that is executing this transaction.  
`<FUND_MANAGER_COMPONENT_ADDRESS>` the address of the fund manager component.  
`<PERCENTAGE>` is the percentage fee to set.  
`<RECEIVER_ACCOUNT>` is the account address that will manage the buyback fund.  

### deposit\_coin
Deposit coins (eventually other coins too) in a DeFi protocol and eventually get the equivalent amount of fund units.  
This metod emits a AdminDepositEvent event containing:  
- the name of the protocol the coins were invested in  
- the new total value of the protocol  

```
CALL_METHOD
    Address("<ACCOUNT>")
    "withdraw"
    Address("<COIN_ADDRESS>")
    Decimal("<COIN_AMOUNT>")
;
TAKE_ALL_FROM_WORKTOP
    Address("<COIN_ADDRESS>")
    Bucket("coin_bucket")
; 
CALL_METHOD
    Address("<ACCOUNT>")
    "withdraw"
    Address("<OTHER_COIN_ADDRESS>")
    Decimal("<OTHER_COIN_AMOUNT>")
;
TAKE_ALL_FROM_WORKTOP
    Address("<OTHER_COIN_ADDRESS>")
    Bucket("other_coin_bucket")
; 
CALL_METHOD
    Address("<ACCOUNT>")
    "create_proof_of_non_fungibles"
    Address("<ADMIN_BADGE>")
    Array<NonFungibleLocalId>(NonFungibleLocalId("<MY_ADMIN_BADGE_ID>"));
;
CALL_METHOD
    Address("<FUND_MANAGER_COMPONENT_ADDRESS>")
    "deposit_coin"
    "<PROTOCOL_NAME>"
    Bucket("coin_bucket")
    Some(Bucket("other_coin_bucket"))
    Map<ResourceAddress, Tuple>(
        Address("<COIN_ADDRESS>") => ("<MORPHER_MESSAGE>", "<MORPHER_SIGNATURE>"),
        ...
    )
    <MINT_FUND_UNITS>
;
```

`<ACCOUNT>` is the admin account.  
`<COIN_ADDRESS>` the resource address of the coin to deposit.  
`<COIN_AMOUNT>` the amount of coins to deposit.  
`<OTHER_COIN_ADDRESS>` the resource address of the other coin to deposit (if any).  
`<OTHER_COIN_AMOUNT>` the amount of other coins to deposit (if any).  
`<ADMIN_BADGE>` is the resource address of the badge held by the admin account.  
`<MY_ADMIN_BADGE_ID>` is the numeric identifier of the admin badge owned by the account that is executing this transaction.  
`<FUND_MANAGER_COMPONENT_ADDRESS>` the address of the fund manager component.  
`<PROTOCOL_NAME>` is the name of the protocol to deposit coins in.  
`<MORPHER_MESSAGE>` the message for the Morpher oracle regarding the deposited coin(s), if needed by the DeFi protocol.  
`<MORPHER_SIGNATURE>` the signature of `<MORPHER_MESSAGE>`.  
`<MINT_FUND_UNITS>` whether to mint fund units of the same value of the deposited coins (`true` or `false`).  

### deposit\_protocol\_token
Deposit tokens in a DeFi protocol and eventually get the equivalent amount of fund units.  
This metod emits a AdminDepositEvent event containing:  
- the name of the protocol the tokens were deposited in  
- the new total value of the protocol  

```
CALL_METHOD
    Address("<ACCOUNT>")
    "withdraw"
    Address("<TOKEN_ADDRESS>")
    Decimal("<TOKEN_AMOUNT>")
;
TAKE_ALL_FROM_WORKTOP
    Address("<TOKEN_ADDRESS>")
    Bucket("token_bucket")
; 
CALL_METHOD
    Address("<ACCOUNT>")
    "create_proof_of_non_fungibles"
    Address("<ADMIN_BADGE>")
    Array<NonFungibleLocalId>(NonFungibleLocalId("<MY_ADMIN_BADGE_ID>"));
;
CALL_METHOD
    Address("<FUND_MANAGER_COMPONENT_ADDRESS>")
    "deposit_protocol_token"
    "<PROTOCOL_NAME>"
    Bucket("token_bucket")
    Map<ResourceAddress, Tuple>(
        Address("<COIN_ADDRESS>") => ("<MORPHER_MESSAGE>", "<MORPHER_SIGNATURE>"),
        ...
    )
    <MINT_FUND_UNITS>
;
```

`<ACCOUNT>` is the admin account.  
`<TOKEN_ADDRESS>` the resource address of the token to deposit.  
`<TOKEN_AMOUNT>` the amount of token to deposit.  
`<ADMIN_BADGE>` is the resource address of the badge held by the admin account.  
`<MY_ADMIN_BADGE_ID>` is the numeric identifier of the admin badge owned by the account that is executing this transaction.  
`<FUND_MANAGER_COMPONENT_ADDRESS>` the address of the fund manager component.  
`<PROTOCOL_NAME>` is the name of the protocol to deposit coins in.  
`<MORPHER_MESSAGE>` the message for the Morpher oracle, if needed by the DeFi protocol.  
`<MORPHER_SIGNATURE>` the signature of `<MORPHER_MESSAGE>`.  
`<MINT_FUND_UNITS>` whether to mint fund units of the same value of the deposited coins (`true` or `false`).  

### register\_validator
Registers or unregisters the Validator.  
An admin does't need other admins' authorization to call this method.  

```
CALL_METHOD
    Address("<ACCOUNT>")
    "create_proof_of_non_fungibles"
    Address("<ADMIN_BADGE>")
    Array<NonFungibleLocalId>(NonFungibleLocalId("<MY_ADMIN_BADGE_ID>"));
;
CALL_METHOD
    Address("<FUND_MANAGER_COMPONENT_ADDRESS>")
    "register_validator"
    <REGISTER>
;
```

`<ACCOUNT>` is the admin account.  
`<ADMIN_BADGE>` is the resource address of the badge held by the admin account.  
`<MY_ADMIN_BADGE_ID>` is the numeric identifier of the admin badge owned by the account that is executing this transaction.  
`<FUND_MANAGER_COMPONENT_ADDRESS>` the address of the fund manager component.  
`<REGISTER>` `true` for registerning the Validator, `false` to unregister it.  

### signal\_protocol\_update\_readiness
Votes for a Radix protcol update.  
An admin does't need other admins' authorization to call this method.  

```
CALL_METHOD
    Address("<ACCOUNT>")
    "create_proof_of_non_fungibles"
    Address("<ADMIN_BADGE>")
    Array<NonFungibleLocalId>(NonFungibleLocalId("<MY_ADMIN_BADGE_ID>"));
;
CALL_METHOD
    Address("<FUND_MANAGER_COMPONENT_ADDRESS>")
    "signal_protocol_update_readiness"
    "<VOTE>"
;
```

`<ACCOUNT>` is the admin account.  
`<ADMIN_BADGE>` is the resource address of the badge held by the admin account.  
`<MY_ADMIN_BADGE_ID>` is the numeric identifier of the admin badge owned by the account that is executing this transaction.  
`<FUND_MANAGER_COMPONENT_ADDRESS>` the address of the fund manager component.  
`<VOTE>` is the code name of the update.  

### update\_node\_key
Associates the Validator with a Radix node.  
An admin does't need other admins' authorization to call this method.  

```
CALL_METHOD
    Address("<ACCOUNT>")
    "create_proof_of_non_fungibles"
    Address("<ADMIN_BADGE>")
    Array<NonFungibleLocalId>(NonFungibleLocalId("<MY_ADMIN_BADGE_ID>"));
;
CALL_METHOD
    Address("<FUND_MANAGER_COMPONENT_ADDRESS>")
    "update_node_key"
    "<KEY>"
;
```

`<ACCOUNT>` is the admin account.  
`<ADMIN_BADGE>` is the resource address of the badge held by the admin account.  
`<MY_ADMIN_BADGE_ID>` is the numeric identifier of the admin badge owned by the account that is executing this transaction.  
`<FUND_MANAGER_COMPONENT_ADDRESS>` the address of the fund manager component.  
`<KEY>` is a string representation of the node key.  

### whithdraw\_unexpected\_coin
Since the DeFi wrappers use Accounts to store DeFi protocols tokens it's possible that someone sends unexpected coins to one of these accounts; this method allows an admin to withdraw these coins. It can't be used to withdraw any coin that the DeFi protocol wrapper is supposed to handle.  
The coin can be both a fungible or a non fungible; in case of a fungible the full balance will be withdrawn, in case of a non fungible up do 100 NFTs will be withdrawn in one transaction.  
An admin does't need other admins' authorization to call this method.  

```
CALL_METHOD
    Address("<ACCOUNT>")
    "create_proof_of_non_fungibles"
    Address("<ADMIN_BADGE>")
    Array<NonFungibleLocalId>(NonFungibleLocalId("<MY_ADMIN_BADGE_ID>"));
;
CALL_METHOD
    Address("<ANY_DEFI_WRAPPER_COMPONENT_ADDRESS>")
    "whithdraw_unexpected_coin"
    Address("<COIN_ADDRESS>")
;
```

`<ACCOUNT>` is the admin account.  
`<ADMIN_BADGE>` is the resource address of the badge held by the admin account.  
`<MY_ADMIN_BADGE_ID>` is the numeric identifier of the admin badge owned by the account that is executing this transaction.  
`<ANY_DEFI_WRAPPER_COMPONENT_ADDRESS>` the address of one of the DeFi protocol wrappers.  
`<COIN_ADDRESS>` is the resource address of the coin to withdraw.  

## Disclaimer
Untested software, for educational purposes only, no warranty.  

## RIP Dan Hughes
We will never forget you.  
