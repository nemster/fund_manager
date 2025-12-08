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

- MultiOracleWrapper package: `package_rdx1p5ardguj2pr93sccv73russradn52fu80xsf3j4hdffgl0h8zttua6`
- MultiOracleWrapper component: `component_rdx1cqssup6xpx6uwn8wrawljtlq0qncvl0fara2xvrjmee2vajpmqq7jn`

- MultiDexWrapper package: `package_rdx1p43r79f9rr64hs6vk977uqxx6ds3c0xx23lmzw7yzlfttngd60vwgc`
- MultiDexWrapper component: `component_rdx1cq03dl2atp9jl7w0vkc46udgsukkr56n2tmugsw3g5vklkttq88xh7`

- WeftWrapper package: `package_rdx1ph2cg8m2l79dddyupun4kgmjvxzg3xytjd5cmn5celg77xe2gqyy4x`
- LSULP@Weft component: `component_rdx1cp63nqsx3lsny4hpvd0lyma7802cepykxwrswehd0kdnpxas8hrcw0`
- LSULP@Weft account: `account_rdx1299rdyg5hymutdpf6s76laa0gvtr798eezthwmq07j7l2p6x82qcp6`
- hUSDC@Weft component: `component_rdx1cpeeae00r9dl7zqjwsut40rgp97rq4yk8c8q70pz96575uflgcvnnp`
- hUSDC@Weft account: `account_rdx12xfguwk24ja9jnqdgvtvz399t3neh3nhcadqpp93aguczchkq5nsqh`

- RootFinanceWrapper package: `package_rdx1ph4k5tzazsanc536a4swcsh2sqeft2wquvuc5tf3vr8vdutzk04wy3`
- xUSDC@Root component: `component_rdx1cz3qgfjwa4alts2m74k9wf0ecsh4n6fj6kwuafxwku7fm43vmvjlaf`
- xUSDC@Root account: `account_rdx12xkn2kka07k37gqn4vd5lx2y9zeqaa3djvmezusq5zyv2fetw723gs`
- xUSDT@Root component: `component_rdx1crjy22rvz83ujqxydwrajldxy33he68namweljeprx5sk75c0cppyd`
- xUSDT@Root account: `account_rdx12xlua8tdzuv8lvepqtcfmqu0rxhmpk4zv078quwtw89tjujtmtmxud`

- OciswapLpPool2Wrapper package: `package_rdx1pkw7natk0488kjuw77zypmulz2lg0stt3056e290j0j5mmv44r9828`
- fUSD/XRD@Ociswap component: `component_rdx1cplxt80h3t24k8dfwhetlkd3gr4nmluqzz028kgz90uufj936xretr`
- fUSD/XRD@Ociswap account: `account_rdx12yrx58uq4ntjjx7wh853qtd8vrtjzqptagkwldvxdpy4rvth2ysphs`

- CaviarnineLpWrapper package: `package_rdx1p48tlgrksqhx3yghxjr66h2dgz3gqrtvj4t5kt0e7yqux84a8uzurg`
- fUSD/xUSDC@Caviarnine component: `component_rdx1crkl95v8ccvpx3wz8yd9qvkdl063hfwfqtzy2rqrumsg6rrrp732cz`
- fUSD/xUSDC@Caviarnine account: `account_rdx12x95s98uyexfcgq8s5r63xzups4hsjx8a7nyrsxm60wxk3dqg5u2js`

- FluxWrapper package: `package_rdx1pk09n7x6vuf4ynywa7eps2fjqvhtrtukm0zmg9jfm7xmrdec0mxksf`
- fUSD/LSULP@Flux component: `component_rdx1crgwrzxppp6w95y5m6vygkujyzt3273kq4y4v4x7wq9d8v8u5l7wsl` (not tested)
- fUSD/LSULP@Flux account: `account_rdx12xxp5gh9kxyhz8w83499ag53ptzhxykjp4ak8c46p5nwjw5xc37ctl`

- SurgeWrapper package: `package_rdx1p5066w4vddkvq9dcd0tpmwngpktx3zzy07v42h6w4qyp4j6tty0n34`
- xUSDC@Surge component: `component_rdx1czascaqverqn397t4rug07pj89t7vmjue69hvc3lcdc9yatud5s7fh`
- xUSDC@Surge account: `account_rdx12x7tcdyquxjl74nx6mk75ysdsw4ry6x4efgy8f3rg3u3fl0dyar6us`

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
    Address("package_rdx1p5ardguj2pr93sccv73russradn52fu80xsf3j4hdffgl0h8zttua6")
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
    Address("component_rdx1cqssup6xpx6uwn8wrawljtlq0qncvl0fara2xvrjmee2vajpmqq7jn")
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
    Address("component_rdx1cqssup6xpx6uwn8wrawljtlq0qncvl0fara2xvrjmee2vajpmqq7jn")
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
    Address("component_rdx1cqssup6xpx6uwn8wrawljtlq0qncvl0fara2xvrjmee2vajpmqq7jn")
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
    Address("component_rdx1cqssup6xpx6uwn8wrawljtlq0qncvl0fara2xvrjmee2vajpmqq7jn")
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
    Address("component_rdx1cqssup6xpx6uwn8wrawljtlq0qncvl0fara2xvrjmee2vajpmqq7jn")
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
    Address("component_rdx1cqssup6xpx6uwn8wrawljtlq0qncvl0fara2xvrjmee2vajpmqq7jn")
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
    Address("component_rdx1cqssup6xpx6uwn8wrawljtlq0qncvl0fara2xvrjmee2vajpmqq7jn")
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

### Tell the MultiOracleWrapper component that the price of hETH must be asked to the XRD/hETH pool
```
CALL_METHOD
    Address("account_rdx1289mytexylv27ey3xty93lskxyjnxat6d5r3ldsljwygtw8gwyusmj")
    "create_proof_of_non_fungibles"
    Address("resource_rdx1nthnjx8ltdk26c8vmvlajtfk4xzy4dlnayqmq7v5l5arurfrh5mp5a")
    Array<NonFungibleLocalId>(NonFungibleLocalId("#1#"))
;
CALL_METHOD
    Address("component_rdx1cqssup6xpx6uwn8wrawljtlq0qncvl0fara2xvrjmee2vajpmqq7jn")
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
    Address("component_rdx1cqssup6xpx6uwn8wrawljtlq0qncvl0fara2xvrjmee2vajpmqq7jn")
    "add_oracle"
    Address("resource_rdx1tk3fxrz75ghllrqhyq8e574rkf4lsq2x5a0vegxwlh3defv225cth3")
    None
    None
    Some(Address("resource_rdx1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxradxrd"))
    Some(Address("component_rdx1crpq83nf76ea2dkkjxfwr426qvmpu9pyakh58ay3eyswe4ps5yn3q2"))
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

### Tell the MultiDexWrapper component to use a DefiPlaza pool for hUSDC <-> XRD swaps
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

### Tell the MultiDexWrapper component to use a DefiPlaza pool for xUSDT <-> XRD swaps
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
    Address("resource_rdx1thrvr3xfs2tarm2dl9emvs26vjqxu6mqvfgvqjne940jv0lnrrg7rw")
    "defiplaza_pool"
    Address("component_rdx1crhrzxe6x35hwx3wmnnw0g8qs84p2hle6ud7n2q4ffzp0udluqm8hj")
    true
;
```

## WeftWrapper

### Instantiate the LSULP@Weft wrapper component
```
CALL_METHOD
    Address("account_rdx1289mytexylv27ey3xty93lskxyjnxat6d5r3ldsljwygtw8gwyusmj")
    "withdraw_non_fungibles"
    Address("resource_rdx1nfxxxxxxxxxxaccwnrxxxxxxxxx006664022062xxxxxxxxxaccwnr")
    Array<NonFungibleLocalId>(
        NonFungibleLocalId("[514a369114b937c5b429d43daff7af43163f14f9c897776c0ff4bdf50746]")
    )
;
TAKE_ALL_FROM_WORKTOP
    Address("resource_rdx1nfxxxxxxxxxxaccwnrxxxxxxxxx006664022062xxxxxxxxxaccwnr")
    Bucket("account_badge")
;
CALL_FUNCTION
    Address("package_rdx1ph2cg8m2l79dddyupun4kgmjvxzg3xytjd5cmn5celg77xe2gqyy4x")
    "WeftWrapper"
    "new"
    Address("resource_rdx1thksg5ng70g9mmy9ne7wz0sc7auzrrwy7fmgcxzel2gvp8pj0xxfmf")
    Address("resource_rdx1t4p82pms6r20k87rscms728tekujacd0sgxyysk7yvl0jgf56gvjuc")
    Address("resource_rdx1tk3fxrz75ghllrqhyq8e574rkf4lsq2x5a0vegxwlh3defv225cth3")
    Address("resource_rdx1nt3vrt8xtdal6gn7ddv0zfzvxpqylxyfmr97setz8r3amhhk90yqmg")
    Address("component_rdx1czmr02yl4da709ceftnm9dnmag7rthu0tu78wmtsn5us9j02d9d0xn")
    Address("component_rdx1crys4t0nvfjzwvsa2pt3zgsmaaaqql8squkannhzcfh36j6u993dnz")
    Address("resource_rdx1th9ul6k57hmfx8u26lgfhz8c7wl4j9jk7knl4crjzec0t8fdgl7sgf")
    Address("resource_rdx1nthnjx8ltdk26c8vmvlajtfk4xzy4dlnayqmq7v5l5arurfrh5mp5a")
    Address("resource_rdx1t5s0qfsgsfhlf8q3sutt8j500dk2el4p3dk4psxyagqxdfwuhrm56k")
    Address("account_rdx1299rdyg5hymutdpf6s76laa0gvtr798eezthwmq07j7l2p6x82qcp6")
    Bucket("account_badge")
;
```

### Authorize admin #2# to add the LSULP@Weft wrapper component in the FundManager
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
    1u8
    Some("LSULP@Weft")
    None
    None
;
```

### Add the LSULP@Weft wrapper component in the FundManager
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
    "add_defi_protocol"
    Proof("admin_proof")
    "LSULP@Weft"
    Address("resource_rdx1thksg5ng70g9mmy9ne7wz0sc7auzrrwy7fmgcxzel2gvp8pj0xxfmf")
    Some(Address("resource_rdx1tk3fxrz75ghllrqhyq8e574rkf4lsq2x5a0vegxwlh3defv225cth3"))
    1u8
    Address("component_rdx1cp63nqsx3lsny4hpvd0lyma7802cepykxwrswehd0kdnpxas8hrcw0")
    None
    false
;
```

### Instantiate the hUSDC@Weft wrapper component
```
CALL_METHOD
    Address("account_rdx1289mytexylv27ey3xty93lskxyjnxat6d5r3ldsljwygtw8gwyusmj")
    "withdraw_non_fungibles"
    Address("resource_rdx1nfxxxxxxxxxxaccwnrxxxxxxxxx006664022062xxxxxxxxxaccwnr")
    Array<NonFungibleLocalId>(
        NonFungibleLocalId("[51928e3acaacba594c0d4316c144a55c679bc677c75a0084b1ea398162f6]")
    )
;
TAKE_ALL_FROM_WORKTOP
    Address("resource_rdx1nfxxxxxxxxxxaccwnrxxxxxxxxx006664022062xxxxxxxxxaccwnr")
    Bucket("account_badge")
;
CALL_FUNCTION
    Address("package_rdx1ph2cg8m2l79dddyupun4kgmjvxzg3xytjd5cmn5celg77xe2gqyy4x")
    "WeftWrapper"
    "new"
    Address("resource_rdx1thxj9m87sn5cc9ehgp9qxp6vzeqxtce90xm5cp33373tclyp4et4gv")
    Address("resource_rdx1t4kxe9n00hgzng02myj6a320qxcma2umxj8ygr795cc5m0hsj3p4l2")
    Address("resource_rdx1tk3fxrz75ghllrqhyq8e574rkf4lsq2x5a0vegxwlh3defv225cth3")
    Address("resource_rdx1nt3vrt8xtdal6gn7ddv0zfzvxpqylxyfmr97setz8r3amhhk90yqmg")
    Address("component_rdx1czmr02yl4da709ceftnm9dnmag7rthu0tu78wmtsn5us9j02d9d0xn")
    Address("component_rdx1crys4t0nvfjzwvsa2pt3zgsmaaaqql8squkannhzcfh36j6u993dnz")
    Address("resource_rdx1th9ul6k57hmfx8u26lgfhz8c7wl4j9jk7knl4crjzec0t8fdgl7sgf")
    Address("resource_rdx1nthnjx8ltdk26c8vmvlajtfk4xzy4dlnayqmq7v5l5arurfrh5mp5a")
    Address("resource_rdx1t5s0qfsgsfhlf8q3sutt8j500dk2el4p3dk4psxyagqxdfwuhrm56k")
    Address("account_rdx12xfguwk24ja9jnqdgvtvz399t3neh3nhcadqpp93aguczchkq5nsqh")
    Bucket("account_badge")
; 
```

### Authorize admin #2# to add the hUSDC@Weft wrapper component in the FundManager
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
    1u8
    Some("hUSDC@Weft")
    None
    None
;
```

### Add the hUSDC@Weft wrapper component in the FundManager
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
    "add_defi_protocol"
    Proof("admin_proof")
    "hUSDC@Weft"
    Address("resource_rdx1thxj9m87sn5cc9ehgp9qxp6vzeqxtce90xm5cp33373tclyp4et4gv")
    Some(Address("resource_rdx1tk3fxrz75ghllrqhyq8e574rkf4lsq2x5a0vegxwlh3defv225cth3"))
    1u8
    Address("component_rdx1cpeeae00r9dl7zqjwsut40rgp97rq4yk8c8q70pz96575uflgcvnnp")
    None
    false
;
```

## RootFinanceWrapper

### Instantiate the xUSDC@Root component
```
CALL_METHOD
    Address("account_rdx1289mytexylv27ey3xty93lskxyjnxat6d5r3ldsljwygtw8gwyusmj")
    "withdraw_non_fungibles"
    Address("resource_rdx1nfxxxxxxxxxxaccwnrxxxxxxxxx006664022062xxxxxxxxxaccwnr")
    Array<NonFungibleLocalId>(
        NonFungibleLocalId("[51ad355add7fad1f2013ab1b4f994428b20ef62d9337917200a088c5272b]")
    )
;
TAKE_ALL_FROM_WORKTOP
    Address("resource_rdx1nfxxxxxxxxxxaccwnrxxxxxxxxx006664022062xxxxxxxxxaccwnr")
    Bucket("account_badge")
;
CALL_FUNCTION
    Address("package_rdx1ph4k5tzazsanc536a4swcsh2sqeft2wquvuc5tf3vr8vdutzk04wy3")
    "RootFinanceWrapper"
    "new"
    Address("resource_rdx1t4upr78guuapv5ept7d7ptekk9mqhy605zgms33mcszen8l9fac8vf")
    Address("resource_rdx1ngekvyag42r0xkhy2ds08fcl7f2ncgc0g74yg6wpeeyc4vtj03sa9f")
    Address("account_rdx12xkn2kka07k37gqn4vd5lx2y9zeqaa3djvmezusq5zyv2fetw723gs")
    Bucket("account_badge")
    Address("component_rdx1crwusgp2uy9qkzje9cqj6pdpx84y94ss8pe7vehge3dg54evu29wtq")
    Address("component_rdx1cqlfmwmhdmp0ln4gaera4skn3yz30p4k5ssv7lqflgh0rjeakwzs9f")
    Address("resource_rdx1th9ul6k57hmfx8u26lgfhz8c7wl4j9jk7knl4crjzec0t8fdgl7sgf")
    Address("resource_rdx1nthnjx8ltdk26c8vmvlajtfk4xzy4dlnayqmq7v5l5arurfrh5mp5a")
;
```

### Authorize admin #2# to add the xUSDC@Root component in the FundManager
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
    1u8
    Some("xUSDC@Root")
    None
    None
;
```

### Add the xUSDC@Root component in the FundManager
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
    "add_defi_protocol"
    Proof("admin_proof")
    "xUSDC@Root"
    Address("resource_rdx1t4upr78guuapv5ept7d7ptekk9mqhy605zgms33mcszen8l9fac8vf")
    None
    1u8
    Address("component_rdx1cz3qgfjwa4alts2m74k9wf0ecsh4n6fj6kwuafxwku7fm43vmvjlaf")
    None
    false
;
```

### Instantiate the xUSDT@Root component
```
CALL_METHOD
    Address("account_rdx1289mytexylv27ey3xty93lskxyjnxat6d5r3ldsljwygtw8gwyusmj")
    "withdraw_non_fungibles"
    Address("resource_rdx1nfxxxxxxxxxxaccwnrxxxxxxxxx006664022062xxxxxxxxxaccwnr")
    Array<NonFungibleLocalId>(
        NonFungibleLocalId("[51bfce9d6d17187fb32102f09d838f19afb0daa263fc7071cb71cab9724b]")
    )
;
TAKE_ALL_FROM_WORKTOP
    Address("resource_rdx1nfxxxxxxxxxxaccwnrxxxxxxxxx006664022062xxxxxxxxxaccwnr")
    Bucket("account_badge")
;
CALL_FUNCTION
    Address("package_rdx1ph4k5tzazsanc536a4swcsh2sqeft2wquvuc5tf3vr8vdutzk04wy3")
    "RootFinanceWrapper"
    "new"
    Address("resource_rdx1thrvr3xfs2tarm2dl9emvs26vjqxu6mqvfgvqjne940jv0lnrrg7rw")
    Address("resource_rdx1ngekvyag42r0xkhy2ds08fcl7f2ncgc0g74yg6wpeeyc4vtj03sa9f")
    Address("account_rdx12xlua8tdzuv8lvepqtcfmqu0rxhmpk4zv078quwtw89tjujtmtmxud")
    Bucket("account_badge")
    Address("component_rdx1crwusgp2uy9qkzje9cqj6pdpx84y94ss8pe7vehge3dg54evu29wtq")
    Address("component_rdx1cqz0f5znwhyy2d4q2rhncetm5tfpvu2c73kvfertktkw33drxcawk8")
    Address("resource_rdx1th9ul6k57hmfx8u26lgfhz8c7wl4j9jk7knl4crjzec0t8fdgl7sgf")
    Address("resource_rdx1nthnjx8ltdk26c8vmvlajtfk4xzy4dlnayqmq7v5l5arurfrh5mp5a")
;
```

### Authorize admin #2# to add the xUSDT@Root component in the FundManager
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
    1u8
    Some("xUSDT@Root")
    None
    None
;
```

### Add the xUSDT@Root component in the FundManager
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
    "add_defi_protocol"
    Proof("admin_proof")
    "xUSDT@Root"
    Address("resource_rdx1thrvr3xfs2tarm2dl9emvs26vjqxu6mqvfgvqjne940jv0lnrrg7rw")
    None
    1u8
    Address("component_rdx1crjy22rvz83ujqxydwrajldxy33he68namweljeprx5sk75c0cppyd")
    None
    false
; 
```

## OciswapLpPool2Wrapper

### Instantiate the fUSD/XRD@Ociswap component
```
CALL_METHOD
    Address("account_rdx1289mytexylv27ey3xty93lskxyjnxat6d5r3ldsljwygtw8gwyusmj")
    "withdraw_non_fungibles"
    Address("resource_rdx1nfxxxxxxxxxxaccwnrxxxxxxxxx006664022062xxxxxxxxxaccwnr")
    Array<NonFungibleLocalId>(
        NonFungibleLocalId("[51066a1f80acd7291bceb9e9102da760d721002bea2cefb586684951b177]")
    )
;
TAKE_ALL_FROM_WORKTOP
    Address("resource_rdx1nfxxxxxxxxxxaccwnrxxxxxxxxx006664022062xxxxxxxxxaccwnr")
    Bucket("account_badge")
;
CALL_FUNCTION
    Address("package_rdx1pkw7natk0488kjuw77zypmulz2lg0stt3056e290j0j5mmv44r9828")
    "OciswapLpPool2Wrapper"
    "new"
    Address("resource_rdx1t49wa75gve8ehvejr760g3pgvkawsgsgq0u3kh7vevzk0g0cnsmscq")
    Address("resource_rdx1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxradxrd")
    Address("resource_rdx1tkc32nfsvwnt7ysq2sdgyq6w2g4s69w86yzk8zwegf7pcvya905ctv")
    Address("account_rdx12yrx58uq4ntjjx7wh853qtd8vrtjzqptagkwldvxdpy4rvth2ysphs")
    Bucket("account_badge")
    Address("component_rdx1cpmacy5gwzswse56jprvlfhrpnt3mplswupu7qtq8pdz2hywy5uaqd")
    Address("resource_rdx1th9ul6k57hmfx8u26lgfhz8c7wl4j9jk7knl4crjzec0t8fdgl7sgf")
    Address("resource_rdx1nthnjx8ltdk26c8vmvlajtfk4xzy4dlnayqmq7v5l5arurfrh5mp5a")
;
```

### Authorize admin #2# to add the fUSD/XRD@Ociswap component in the FundManager
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
    1u8
    Some("fUSD/XRD@Ociswap")
    None
    None
;
```

### Add the fUSD/XRD@Ociswap component in the FundManager
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
    "add_defi_protocol"
    Proof("admin_proof")
    "fUSD/XRD@Ociswap"
    Address("resource_rdx1t49wa75gve8ehvejr760g3pgvkawsgsgq0u3kh7vevzk0g0cnsmscq")
    Some(Address("resource_rdx1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxradxrd"))
    1u8
    Address("component_rdx1cplxt80h3t24k8dfwhetlkd3gr4nmluqzz028kgz90uufj936xretr")
    None
    true
;
```

## CaviarnineLpWrapper

### Instantiate the fUSD/xUSDC@Caviarnine component
```
CALL_METHOD
    Address("account_rdx1289mytexylv27ey3xty93lskxyjnxat6d5r3ldsljwygtw8gwyusmj")
    "withdraw_non_fungibles"
    Address("resource_rdx1nfxxxxxxxxxxaccwnrxxxxxxxxx006664022062xxxxxxxxxaccwnr")
    Array<NonFungibleLocalId>(
        NonFungibleLocalId("[518b4814fc264c9c20078507a8985c0c2b7848c7efa641c0dbd3dc6b45a0]")
    )
;
TAKE_ALL_FROM_WORKTOP
    Address("resource_rdx1nfxxxxxxxxxxaccwnrxxxxxxxxx006664022062xxxxxxxxxaccwnr")
    Bucket("account_badge")
;
CALL_FUNCTION
    Address("package_rdx1p48tlgrksqhx3yghxjr66h2dgz3gqrtvj4t5kt0e7yqux84a8uzurg")
    "CaviarnineLpWrapper"
    "new"
    Address("resource_rdx1t49wa75gve8ehvejr760g3pgvkawsgsgq0u3kh7vevzk0g0cnsmscq")
    Address("resource_rdx1t4upr78guuapv5ept7d7ptekk9mqhy605zgms33mcszen8l9fac8vf")
    Address("resource_rdx1ng2m9cn34czt73x0zjjxhzrpddt5kr6juyfyxrk4uc4gudhy2nkyxy")
    Address("account_rdx12x95s98uyexfcgq8s5r63xzups4hsjx8a7nyrsxm60wxk3dqg5u2js")
    Bucket("account_badge")
    Address("component_rdx1cqmx9aqpr36anp960xes8f4wp7skc6pya6k9ra2jtlmlv24qslmwxf")
    Address("resource_rdx1th9ul6k57hmfx8u26lgfhz8c7wl4j9jk7knl4crjzec0t8fdgl7sgf")
    Address("resource_rdx1nthnjx8ltdk26c8vmvlajtfk4xzy4dlnayqmq7v5l5arurfrh5mp5a")
    Array<Tuple>(
        Tuple(0i32, Decimal("1"))
    )
;
```

## Authorize admin #2# to add the fUSD/xUSDC@Caviarnine component in the FundManager
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
    1u8
    Some("fUSD/xUSDC@Caviarnine")
    None
    None
;
```

### Add the fUSD/xUSDC@Caviarnine component in the FundManager
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
    "add_defi_protocol"
    Proof("admin_proof")
    "fUSD/xUSDC@Caviarnine"
    Address("resource_rdx1t49wa75gve8ehvejr760g3pgvkawsgsgq0u3kh7vevzk0g0cnsmscq")
    Some(Address("resource_rdx1t4upr78guuapv5ept7d7ptekk9mqhy605zgms33mcszen8l9fac8vf"))
    1u8
    Address("component_rdx1crkl95v8ccvpx3wz8yd9qvkdl063hfwfqtzy2rqrumsg6rrrp732cz")
    None
    true
;
```

## FluxWrapper

### Instantiate the fUSD/LSULP@Flux component
```
CALL_METHOD
    Address("account_rdx1289mytexylv27ey3xty93lskxyjnxat6d5r3ldsljwygtw8gwyusmj")
    "withdraw_non_fungibles"
    Address("resource_rdx1nfxxxxxxxxxxaccwnrxxxxxxxxx006664022062xxxxxxxxxaccwnr")
    Array<NonFungibleLocalId>(
        NonFungibleLocalId("[518c1a22e5b189711dc78d4a5ea2910ac57312d20d7b63e2ba0d26e93a86]")
    )
;
TAKE_ALL_FROM_WORKTOP
    Address("resource_rdx1nfxxxxxxxxxxaccwnrxxxxxxxxx006664022062xxxxxxxxxaccwnr")
    Bucket("account_badge")
;
CALL_FUNCTION
    Address("package_rdx1pk09n7x6vuf4ynywa7eps2fjqvhtrtukm0zmg9jfm7xmrdec0mxksf")
    "FluxWrapper"
    "new"
    Address("resource_rdx1t49wa75gve8ehvejr760g3pgvkawsgsgq0u3kh7vevzk0g0cnsmscq")
    Address("resource_rdx1thksg5ng70g9mmy9ne7wz0sc7auzrrwy7fmgcxzel2gvp8pj0xxfmf")
    Address("resource_rdx1tksgc3j8ylrjjqgtny3l4dsfnpepch32hndyk20uptplqk8zuezk0z")
    Address("account_rdx12xxp5gh9kxyhz8w83499ag53ptzhxykjp4ak8c46p5nwjw5xc37ctl")
    Bucket("account_badge")
    Address("component_rdx1cpkye6pp2643ghalcppdxks6kymyu5gla87gf7sk34k0vg7xu57jaj")
    Address("resource_rdx1th9ul6k57hmfx8u26lgfhz8c7wl4j9jk7knl4crjzec0t8fdgl7sgf")
    Address("resource_rdx1nthnjx8ltdk26c8vmvlajtfk4xzy4dlnayqmq7v5l5arurfrh5mp5a")
;
```

## Authorize admin #2# to add the fUSD/LSULP@Flux component in the FundManager
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
    1u8
    Some("fUSD/LSULP@Flux")
    None
    None
;
```

### Add the fUSD/LSULP@Flux component in the FundManager
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
    "add_defi_protocol"
    Proof("admin_proof")
    "fUSD/LSULP@Flux"
    Address("resource_rdx1t49wa75gve8ehvejr760g3pgvkawsgsgq0u3kh7vevzk0g0cnsmscq")
    Some(Address("resource_rdx1thksg5ng70g9mmy9ne7wz0sc7auzrrwy7fmgcxzel2gvp8pj0xxfmf"))
    1u8
    Address("component_rdx1crgwrzxppp6w95y5m6vygkujyzt3273kq4y4v4x7wq9d8v8u5l7wsl")
    None
    false
;
```

## SurgeWrapper

### Instantiate the xUSDC@Surge component
```
CALL_METHOD
    Address("account_rdx1289mytexylv27ey3xty93lskxyjnxat6d5r3ldsljwygtw8gwyusmj")
    "withdraw_non_fungibles"
    Address("resource_rdx1nfxxxxxxxxxxaccwnrxxxxxxxxx006664022062xxxxxxxxxaccwnr")
    Array<NonFungibleLocalId>(
        NonFungibleLocalId("[51bcbc3480e1a5ff5666d6edea120d83aa3268d5ca5043a623447914fded]")
    )
;
TAKE_ALL_FROM_WORKTOP
    Address("resource_rdx1nfxxxxxxxxxxaccwnrxxxxxxxxx006664022062xxxxxxxxxaccwnr")
    Bucket("account_badge")
;
CALL_FUNCTION
    Address("package_rdx1p5066w4vddkvq9dcd0tpmwngpktx3zzy07v42h6w4qyp4j6tty0n34")
    "SurgeWrapper"
    "new"
    Address("resource_rdx1t4upr78guuapv5ept7d7ptekk9mqhy605zgms33mcszen8l9fac8vf")
    Address("resource_rdx1t48x0z68dm6z422wxyctj5wvnt2nh95lvmly65vxzywdkd24zypl5d")
    Address("account_rdx12x7tcdyquxjl74nx6mk75ysdsw4ry6x4efgy8f3rg3u3fl0dyar6us")
    Bucket("account_badge")
    Address("component_rdx1cz5dduz6flgsmx7frc0854nk545s69nryvgq0y02r2mlm3tsryk6xx")
    Address("component_rdx1czqcwcqyv69y9s6xfk443250ruragewa0vj06u5ke04elcu9kae92n")
    Address("resource_rdx1th9ul6k57hmfx8u26lgfhz8c7wl4j9jk7knl4crjzec0t8fdgl7sgf")
    Address("resource_rdx1nthnjx8ltdk26c8vmvlajtfk4xzy4dlnayqmq7v5l5arurfrh5mp5a")
;
```

## Authorize admin #2# to add the xUSDC@Surge component in the FundManager
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
    1u8
    Some("xUSDC@Surge")
    None
    None
;
```

### Add the xUSDC@Surge component in the FundManager
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
    "add_defi_protocol"
    Proof("admin_proof")
    "xUSDC@Surge"
    Address("resource_rdx1t4upr78guuapv5ept7d7ptekk9mqhy605zgms33mcszen8l9fac8vf")
    None
    1u8
    Address("component_rdx1czascaqverqn397t4rug07pj89t7vmjue69hvc3lcdc9yatud5s7fh")
    None
    false
;
```

