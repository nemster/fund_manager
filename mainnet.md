# Mainnet deployment

- Validator: `validator_rdx1swez5cqmw4d6tls0mcldehnfhpxge0mq7cmnypnjz909apqqjgx6n9`
- Claim NFT: `resource_rdx1ngyn7ea28quxf44fsg4hxq6q6fhah2lwh5t32sarglygp9xl76c4tz`

- FundManager package: `package_rdx1p464xs2twhhw8ql85effprf0xpr7v24u2k6qumtrezwkvv8n9l8r2r`
- FundManager component: `component_rdx1cpd5eajj0rq9dcwuymdhjhcrn2k62xgn07msfj2xhk3rn8mn2gcuut`
- AccountLocker: `locker_rdx1drxckgmq4wqdh8zvs64dz2065wq4e9q8gfj7kkc8vehnwcxp0lu94d`
- FundManager badge: `resource_rdx1th9ul6k57hmfx8u26lgfhz8c7wl4j9jk7knl4crjzec0t8fdgl7sgf`
- Fund admin badge: `resource_rdx1nthnjx8ltdk26c8vmvlajtfk4xzy4dlnayqmq7v5l5arurfrh5mp5a`
- Fund unit: `resource_rdx1th38dkeamzmlhvv264tjk54gtvd2yn3x26kpa6c5ukspmyqd8rtgru`
- Fund bot badge: `resource_rdx1t5s0qfsgsfhlf8q3sutt8j500dk2el4p3dk4psxyagqxdfwuhrm56k`

- MultiOracleWrapper package: `package_rdx1pkkghh9jc7t32r82emylcsl0fvtrq2g2x524t9gyf29ltv6xz25m7n`
- MultiOracleWrapper component: `component_rdx1cp7ts7a6ty2kq8kcj3zcg93z7jga8a4hyvfmx5070dtgsuzk2rmygh`

- MultiDexWrapper package: `package_rdx1p43r79f9rr64hs6vk977uqxx6ds3c0xx23lmzw7yzlfttngd60vwgc`
- MultiDexWrapper component: `component_rdx1cq03dl2atp9jl7w0vkc46udgsukkr56n2tmugsw3g5vklkttq88xh7`

## FundManager

### Instantiate the FundManager component
```
CALL_FUNCTION
    Address("package_rdx1p464xs2twhhw8ql85effprf0xpr7v24u2k6qumtrezwkvv8n9l8r2r")
    "FundManager"
    "new"
    Address("validator_rdx1swez5cqmw4d6tls0mcldehnfhpxge0mq7cmnypnjz909apqqjgx6n9")
    Address("resource_rdx1ngyn7ea28quxf44fsg4hxq6q6fhah2lwh5t32sarglygp9xl76c4tz")
    20u8
    20u8
    Address("account_rdx129yg7ugqe9hulflzu0uf22sx9h5d5qrcg3v2599zl8sjwh4fawdygj")
    4u8
    1u8
    Decimal("0.1")
;
CALL_METHOD
    Address("account_rdx1289mytexylv27ey3xty93lskxyjnxat6d5r3ldsljwygtw8gwyusmj")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP")
;
```

## MultiOracleWrapper

### Instantiate the MultiOracleWrapper component
```
CALL_FUNCTION
    Address("package_rdx1pkkghh9jc7t32r82emylcsl0fvtrq2g2x524t9gyf29ltv6xz25m7n")
    "MultiOracleWrapper"
    "new"
    Address("resource_rdx1th9ul6k57hmfx8u26lgfhz8c7wl4j9jk7knl4crjzec0t8fdgl7sgf")
    Address("resource_rdx1nthnjx8ltdk26c8vmvlajtfk4xzy4dlnayqmq7v5l5arurfrh5mp5a")
    Address("resource_rdx1t5s0qfsgsfhlf8q3sutt8j500dk2el4p3dk4psxyagqxdfwuhrm56k")
    Address("component_rdx1cp07hrz378zfugcf6h8f9usct4zqx7rdgjhxjwphkzxyv9h7l2q04s")
    300u64
    180u64 
    Address("resource_rdx1thksg5ng70g9mmy9ne7wz0sc7auzrrwy7fmgcxzel2gvp8pj0xxfmf")
    Address("component_rdx1cppy08xgra5tv5melsjtj79c0ngvrlmzl8hhs7vwtzknp9xxs63mfp")
;
```

### Authorize admin #2# to set the oracle component in the FundManager
```
CALL_METHOD
    Address("account_rdx1289mytexylv27ey3xty93lskxyjnxat6d5r3ldsljwygtw8gwyusmj")
    "create_proof_of_non_fungibles"
    Address("resource_rdx1nthnjx8ltdk26c8vmvlajtfk4xzy4dlnayqmq7v5l5arurfrh5mp5a")
    Array<NonFungibleLocalId>(NonFungibleLocalId("#1#"))
;
POP_FROM_AUTH_ZONE
    Proof("admin_proof")
;
CALL_METHOD
    Address("component_rdx1cpd5eajj0rq9dcwuymdhjhcrn2k62xgn07msfj2xhk3rn8mn2gcuut")
    "authorize_admin_operation"
    Proof("admin_proof")
    2u8
    7u8
    None
    None
    None
;
```

### Set the MultiOracleWrapper component as the oracle to use in the FundManager
```
CALL_METHOD
    Address("account_rdx1289mytexylv27ey3xty93lskxyjnxat6d5r3ldsljwygtw8gwyusmj")
    "create_proof_of_non_fungibles"
    Address("resource_rdx1nthnjx8ltdk26c8vmvlajtfk4xzy4dlnayqmq7v5l5arurfrh5mp5a")
    Array<NonFungibleLocalId>(NonFungibleLocalId("#2#"))
;
POP_FROM_AUTH_ZONE
    Proof("admin_proof")
;
CALL_METHOD
    Address("component_rdx1cpd5eajj0rq9dcwuymdhjhcrn2k62xgn07msfj2xhk3rn8mn2gcuut")
    "set_oracle_component"
    Proof("admin_proof")
    Address("component_rdx1cp7ts7a6ty2kq8kcj3zcg93z7jga8a4hyvfmx5070dtgsuzk2rmygh")
;
```

### Tell the MultiOracleWrapper component that the price of xUSDC is always 1
```
CALL_METHOD
    Address("account_rdx1289mytexylv27ey3xty93lskxyjnxat6d5r3ldsljwygtw8gwyusmj")
    "create_proof_of_non_fungibles"
    Address("resource_rdx1nthnjx8ltdk26c8vmvlajtfk4xzy4dlnayqmq7v5l5arurfrh5mp5a")
    Array<NonFungibleLocalId>(NonFungibleLocalId("#1#"))
;
CALL_METHOD
    Address("component_rdx1cp7ts7a6ty2kq8kcj3zcg93z7jga8a4hyvfmx5070dtgsuzk2rmygh")
    "add_oracle"
    Address("resource_rdx1t4upr78guuapv5ept7d7ptekk9mqhy605zgms33mcszen8l9fac8vf")
    Some(Decimal("1"))
    None
    None
    None
    None
    None
;
```

### Tell the MultiOracleWrapper component that the price of xUSDT is always 1
```
CALL_METHOD
    Address("account_rdx1289mytexylv27ey3xty93lskxyjnxat6d5r3ldsljwygtw8gwyusmj")
    "create_proof_of_non_fungibles"
    Address("resource_rdx1nthnjx8ltdk26c8vmvlajtfk4xzy4dlnayqmq7v5l5arurfrh5mp5a")
    Array<NonFungibleLocalId>(NonFungibleLocalId("#1#"))
;
CALL_METHOD
    Address("component_rdx1cp7ts7a6ty2kq8kcj3zcg93z7jga8a4hyvfmx5070dtgsuzk2rmygh")
    "add_oracle"
    Address("resource_rdx1thrvr3xfs2tarm2dl9emvs26vjqxu6mqvfgvqjne940jv0lnrrg7rw")
    Some(Decimal("1"))
    None
    None
    None
    None
    None
;
```

### Tell the MultiOracleWrapper component that the price of hUSDC is always 1
```
CALL_METHOD
    Address("account_rdx1289mytexylv27ey3xty93lskxyjnxat6d5r3ldsljwygtw8gwyusmj")
    "create_proof_of_non_fungibles"
    Address("resource_rdx1nthnjx8ltdk26c8vmvlajtfk4xzy4dlnayqmq7v5l5arurfrh5mp5a")
    Array<NonFungibleLocalId>(NonFungibleLocalId("#1#"))
;
CALL_METHOD
    Address("component_rdx1cp7ts7a6ty2kq8kcj3zcg93z7jga8a4hyvfmx5070dtgsuzk2rmygh")
    "add_oracle"
    Address("resource_rdx1thxj9m87sn5cc9ehgp9qxp6vzeqxtce90xm5cp33373tclyp4et4gv")
    Some(Decimal("1"))
    None
    None
    None
    None
    None
;
```

### Tell the MultiOracleWrapper component that the price of fUSD is always 1
```
CALL_METHOD
    Address("account_rdx1289mytexylv27ey3xty93lskxyjnxat6d5r3ldsljwygtw8gwyusmj")
    "create_proof_of_non_fungibles"
    Address("resource_rdx1nthnjx8ltdk26c8vmvlajtfk4xzy4dlnayqmq7v5l5arurfrh5mp5a")
    Array<NonFungibleLocalId>(NonFungibleLocalId("#1#"))
;
CALL_METHOD
    Address("component_rdx1cp7ts7a6ty2kq8kcj3zcg93z7jga8a4hyvfmx5070dtgsuzk2rmygh")
    "add_oracle"
    Address("resource_rdx1t49wa75gve8ehvejr760g3pgvkawsgsgq0u3kh7vevzk0g0cnsmscq")
    Some(Decimal("1"))
    None
    None
    None
    None
    None
;
```

### Tell the MultiOracleWrapper component that the price of XRD must be asked to the XRD/hUSDC pool
```
CALL_METHOD
    Address("account_rdx1289mytexylv27ey3xty93lskxyjnxat6d5r3ldsljwygtw8gwyusmj")
    "create_proof_of_non_fungibles"
    Address("resource_rdx1nthnjx8ltdk26c8vmvlajtfk4xzy4dlnayqmq7v5l5arurfrh5mp5a")
    Array<NonFungibleLocalId>(NonFungibleLocalId("#1#"))
;
CALL_METHOD
    Address("component_rdx1cp7ts7a6ty2kq8kcj3zcg93z7jga8a4hyvfmx5070dtgsuzk2rmygh")
    "add_oracle"
    Address("resource_rdx1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxradxrd")
    None
    None
    Some(Address("resource_rdx1thxj9m87sn5cc9ehgp9qxp6vzeqxtce90xm5cp33373tclyp4et4gv"))
    Some(Address("component_rdx1czy2naejcqx8gv46zdsex2syuxrs4jnqzug58e66zr8wglxzvu97qr"))
    Some(false)
    None
;
```

### Tell the MultiOracleWrapper component that the price of hWBTC must be asked to the hWBTC/XRD pool
```
CALL_METHOD
    Address("account_rdx1289mytexylv27ey3xty93lskxyjnxat6d5r3ldsljwygtw8gwyusmj")
    "create_proof_of_non_fungibles"
    Address("resource_rdx1nthnjx8ltdk26c8vmvlajtfk4xzy4dlnayqmq7v5l5arurfrh5mp5a")
    Array<NonFungibleLocalId>(NonFungibleLocalId("#1#"))
;
CALL_METHOD
    Address("component_rdx1cp7ts7a6ty2kq8kcj3zcg93z7jga8a4hyvfmx5070dtgsuzk2rmygh")
    "add_oracle"
    Address("resource_rdx1t58kkcqdz0mavfz98m98qh9m4jexyl9tacsvlhns6yxs4r6hrm5re5")
    None
    None
    Some(Address("resource_rdx1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxradxrd"))
    Some(Address("component_rdx1crd7xk0nu07kj60artzz6evws7r6w69lwarf0nqmkxuwwluy5xjud0"))
    Some(false)
    None
;
```

### Tell the MultiOracleWrapper component that the price of hETC must be asked to the XRD/hETH pool
```
CALL_METHOD
    Address("account_rdx1289mytexylv27ey3xty93lskxyjnxat6d5r3ldsljwygtw8gwyusmj")
    "create_proof_of_non_fungibles"
    Address("resource_rdx1nthnjx8ltdk26c8vmvlajtfk4xzy4dlnayqmq7v5l5arurfrh5mp5a")
    Array<NonFungibleLocalId>(NonFungibleLocalId("#1#"))
;
CALL_METHOD
    Address("component_rdx1cp7ts7a6ty2kq8kcj3zcg93z7jga8a4hyvfmx5070dtgsuzk2rmygh")
    "add_oracle"
    Address("resource_rdx1th09yvv7tgsrv708ffsgqjjf2mhy84mscmj5jwu4g670fh3e5zgef0")
    None
    None
    Some(Address("resource_rdx1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxradxrd"))
    Some(Address("component_rdx1crumqsy0nu4pl3fwah3nkf8eg8qhltxenk83wh9tzlmr5jnsqs3x4c"))
    Some(true)
    None
;
```

### Tell the MultiOracleWrapper component that the price of WEFT must be asked to the WEFT/XRD pool
```
CALL_METHOD
    Address("account_rdx1289mytexylv27ey3xty93lskxyjnxat6d5r3ldsljwygtw8gwyusmj")
    "create_proof_of_non_fungibles"
    Address("resource_rdx1nthnjx8ltdk26c8vmvlajtfk4xzy4dlnayqmq7v5l5arurfrh5mp5a")
    Array<NonFungibleLocalId>(NonFungibleLocalId("#1#"))
;
CALL_METHOD
    Address("component_rdx1cp7ts7a6ty2kq8kcj3zcg93z7jga8a4hyvfmx5070dtgsuzk2rmygh")
    "add_oracle"
    Address("resource_rdx1tk3fxrz75ghllrqhyq8e574rkf4lsq2x5a0vegxwlh3defv225cth3")
    None
    None
    Some(Address("resource_rdx1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxradxrd"))
    Some(Address("component_rdx1crvtvnr02f5fl49jvap4rndlepfsgta455wcyteacr7dtfgzvqqw6n"))
    Some(false)
    None
;
```

## MultiDexWrapper

### Instantiate the MultiDexWrapper component
```
CALL_FUNCTION
    Address("package_rdx1p43r79f9rr64hs6vk977uqxx6ds3c0xx23lmzw7yzlfttngd60vwgc")
    "MultiDexWrapper"
    "new"
    Address("resource_rdx1th9ul6k57hmfx8u26lgfhz8c7wl4j9jk7knl4crjzec0t8fdgl7sgf")
    Address("resource_rdx1nthnjx8ltdk26c8vmvlajtfk4xzy4dlnayqmq7v5l5arurfrh5mp5a")
;
```

### Authorize admin #2# to set the dex component in the FundManager
```
CALL_METHOD
    Address("account_rdx1289mytexylv27ey3xty93lskxyjnxat6d5r3ldsljwygtw8gwyusmj")
    "create_proof_of_non_fungibles"
    Address("resource_rdx1nthnjx8ltdk26c8vmvlajtfk4xzy4dlnayqmq7v5l5arurfrh5mp5a")
    Array<NonFungibleLocalId>(NonFungibleLocalId("#1#"))
;
POP_FROM_AUTH_ZONE
    Proof("admin_proof")
;
CALL_METHOD
    Address("component_rdx1cpd5eajj0rq9dcwuymdhjhcrn2k62xgn07msfj2xhk3rn8mn2gcuut")
    "authorize_admin_operation"
    Proof("admin_proof")
    2u8
    3u8
    None
    None
    None
;
```

### Set the MultiDexWrapper component as dex to be used by the FundManager
```
CALL_METHOD
    Address("account_rdx1289mytexylv27ey3xty93lskxyjnxat6d5r3ldsljwygtw8gwyusmj")
    "create_proof_of_non_fungibles"
    Address("resource_rdx1nthnjx8ltdk26c8vmvlajtfk4xzy4dlnayqmq7v5l5arurfrh5mp5a")
    Array<NonFungibleLocalId>(NonFungibleLocalId("#2#"))
;
POP_FROM_AUTH_ZONE
    Proof("admin_proof")
;
CALL_METHOD
    Address("component_rdx1cpd5eajj0rq9dcwuymdhjhcrn2k62xgn07msfj2xhk3rn8mn2gcuut")
    "set_dex_component"
    Proof("admin_proof")
    Address("component_rdx1cq03dl2atp9jl7w0vkc46udgsukkr56n2tmugsw3g5vklkttq88xh7")
;
```

### Tell the MultiDexWrapper component to use an Ociswap pool for hUSDC <-> XRD swaps
```
CALL_METHOD
    Address("account_rdx1289mytexylv27ey3xty93lskxyjnxat6d5r3ldsljwygtw8gwyusmj")
    "create_proof_of_non_fungibles"
    Address("resource_rdx1nthnjx8ltdk26c8vmvlajtfk4xzy4dlnayqmq7v5l5arurfrh5mp5a")
    Array<NonFungibleLocalId>(NonFungibleLocalId("#1#"))
;
CALL_METHOD
    Address("component_rdx1cq03dl2atp9jl7w0vkc46udgsukkr56n2tmugsw3g5vklkttq88xh7")
    "add_pool"
    Address("resource_rdx1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxradxrd")
    Address("resource_rdx1t4upr78guuapv5ept7d7ptekk9mqhy605zgms33mcszen8l9fac8vf")
    "defiplaza_pool"
    Address("component_rdx1cqs6t5t70fcgrva6ws6gs84u29w3kecn6j0zkjg0u0x9szx0xnusxj")
    true
;
```

### Tell the MultiDexWrapper component to use a DefiPlaza pool for hETH <-> XRD swaps
```
CALL_METHOD
    Address("account_rdx1289mytexylv27ey3xty93lskxyjnxat6d5r3ldsljwygtw8gwyusmj")
    "create_proof_of_non_fungibles"
    Address("resource_rdx1nthnjx8ltdk26c8vmvlajtfk4xzy4dlnayqmq7v5l5arurfrh5mp5a")
    Array<NonFungibleLocalId>(NonFungibleLocalId("#1#"))
;
CALL_METHOD
    Address("component_rdx1cq03dl2atp9jl7w0vkc46udgsukkr56n2tmugsw3g5vklkttq88xh7")
    "add_pool"
    Address("resource_rdx1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxradxrd")
    Address("resource_rdx1th09yvv7tgsrv708ffsgqjjf2mhy84mscmj5jwu4g670fh3e5zgef0")
    "defiplaza_pool"
    Address("component_rdx1cq8nefdv75yqkgwqe9rhj436yr3z09du7g797y90prmwf9ugv0m8u2")
    true
;
```

### Tell the MultiDexWrapper component to use an Ociswap pool for hWBTC <-> XRD swaps
```
CALL_METHOD
    Address("account_rdx1289mytexylv27ey3xty93lskxyjnxat6d5r3ldsljwygtw8gwyusmj")
    "create_proof_of_non_fungibles"
    Address("resource_rdx1nthnjx8ltdk26c8vmvlajtfk4xzy4dlnayqmq7v5l5arurfrh5mp5a")
    Array<NonFungibleLocalId>(NonFungibleLocalId("#1#"))
;
CALL_METHOD
    Address("component_rdx1cq03dl2atp9jl7w0vkc46udgsukkr56n2tmugsw3g5vklkttq88xh7")
    "add_pool"
    Address("resource_rdx1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxradxrd")
    Address("resource_rdx1t58kkcqdz0mavfz98m98qh9m4jexyl9tacsvlhns6yxs4r6hrm5re5")
    "ociswap_precision_pool"
    Address("component_rdx1crd7xk0nu07kj60artzz6evws7r6w69lwarf0nqmkxuwwluy5xjud0")
    true
;
```

### Tell the MultiDexWrapper component to use an Ociswap pool for fUSD <-> XRD swaps
```
CALL_METHOD
    Address("account_rdx1289mytexylv27ey3xty93lskxyjnxat6d5r3ldsljwygtw8gwyusmj")
    "create_proof_of_non_fungibles"
    Address("resource_rdx1nthnjx8ltdk26c8vmvlajtfk4xzy4dlnayqmq7v5l5arurfrh5mp5a")
    Array<NonFungibleLocalId>(NonFungibleLocalId("#1#"))
;
CALL_METHOD
    Address("component_rdx1cq03dl2atp9jl7w0vkc46udgsukkr56n2tmugsw3g5vklkttq88xh7")
    "add_pool"
    Address("resource_rdx1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxradxrd")
    Address("resource_rdx1t49wa75gve8ehvejr760g3pgvkawsgsgq0u3kh7vevzk0g0cnsmscq")
    "ociswap_pool2"
    Address("component_rdx1cpmacy5gwzswse56jprvlfhrpnt3mplswupu7qtq8pdz2hywy5uaqd")
    true
;
```

### Tell the MultiDexWrapper component to use a DefiPlaza pool for xUSDC <-> XRD swaps
```
CALL_METHOD
    Address("account_rdx1289mytexylv27ey3xty93lskxyjnxat6d5r3ldsljwygtw8gwyusmj")
    "create_proof_of_non_fungibles"
    Address("resource_rdx1nthnjx8ltdk26c8vmvlajtfk4xzy4dlnayqmq7v5l5arurfrh5mp5a")
    Array<NonFungibleLocalId>(NonFungibleLocalId("#1#"))
;
CALL_METHOD
    Address("component_rdx1cq03dl2atp9jl7w0vkc46udgsukkr56n2tmugsw3g5vklkttq88xh7")
    "add_pool"
    Address("resource_rdx1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxradxrd")
    Address("resource_rdx1t4upr78guuapv5ept7d7ptekk9mqhy605zgms33mcszen8l9fac8vf")
    "defiplaza_pool"
    Address("component_rdx1czmha58h7vw0e4qpxz8ga68cq6h5fjm27w2z43r0n6k9x65nvrjp4g")
    true
;
```

### Tell the MultiDexWrapper component to use an Ociswap pool for WEFT <-> XRD swaps
```
CALL_METHOD
    Address("account_rdx1289mytexylv27ey3xty93lskxyjnxat6d5r3ldsljwygtw8gwyusmj")
    "create_proof_of_non_fungibles"
    Address("resource_rdx1nthnjx8ltdk26c8vmvlajtfk4xzy4dlnayqmq7v5l5arurfrh5mp5a")
    Array<NonFungibleLocalId>(NonFungibleLocalId("#1#"))
;
CALL_METHOD
    Address("component_rdx1cq03dl2atp9jl7w0vkc46udgsukkr56n2tmugsw3g5vklkttq88xh7")
    "add_pool"
    Address("resource_rdx1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxradxrd")
    Address("resource_rdx1tk3fxrz75ghllrqhyq8e574rkf4lsq2x5a0vegxwlh3defv225cth3")
    "ociswap_precision_pool"
    Address("component_rdx1crpq83nf76ea2dkkjxfwr426qvmpu9pyakh58ay3eyswe4ps5yn3q2")
    true
;
```

### Tell the MultiDexWrapper component to use a CaviarNine pool for LSULP <-> XRD swaps
```
CALL_METHOD
    Address("account_rdx1289mytexylv27ey3xty93lskxyjnxat6d5r3ldsljwygtw8gwyusmj")
    "create_proof_of_non_fungibles"
    Address("resource_rdx1nthnjx8ltdk26c8vmvlajtfk4xzy4dlnayqmq7v5l5arurfrh5mp5a")
    Array<NonFungibleLocalId>(NonFungibleLocalId("#1#"))
;
CALL_METHOD
    Address("component_rdx1cq03dl2atp9jl7w0vkc46udgsukkr56n2tmugsw3g5vklkttq88xh7")
    "add_pool"
    Address("resource_rdx1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxradxrd")
    Address("resource_rdx1thksg5ng70g9mmy9ne7wz0sc7auzrrwy7fmgcxzel2gvp8pj0xxfmf")
    "caviarnine_shape_pool"
    Address("component_rdx1crdhl7gel57erzgpdz3l3vr64scslq4z7vd0xgna6vh5fq5fnn9xas")
    true
;
```


