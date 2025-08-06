use scrypto::prelude::*;
use crate::common::*;

static AUTHORIZATION_TIMEOUT: i64 = 172800; // Two days
static MAX_VECTOR_SIZE: usize = 30;

#[derive(ScryptoSbor, NonFungibleData)]
struct Admin {
}

#[derive(ScryptoSbor, PartialEq)]
#[repr(u8)]
pub enum AuthorizedOperation {
    WithdrawValidatorBadge      = 0,
    AddDefiProtocol             = 1,
    RemoveDefiProtocol          = 2,
    SetDexComponent             = 3,
    DecreaseMinAuthorizers      = 4,
    IncreaseMinAuthorizers      = 5,
    MintAdminBadge              = 6,
    SetOracleComponent          = 7,
    WithdrawFundManagerBadge    = 8,
    SetWithdrawalFee            = 9,
    MintBotBadge                = 10,
    SetBuybackFund              = 11,
}

#[derive(ScryptoSbor)]
struct Authorization {
    timestamp: i64,
    allower_admin_id: u8,
    allowed_admin_id: u8,
    authorized_operation: AuthorizedOperation,
    protocol_name: Option<String>,
    percentage: Option<u8>,
    account_address: Option<Global<Account>>,
}

#[derive(ScryptoSbor)]
struct DefiProtocol {
    value: Decimal,
    desired_percentage: u8,
    wrapper: DefiProtocolInterfaceScryptoStub,
    coin: ResourceAddress, // Example coin: xUSDC
    protocol_token: ResourceAddress, // Example protocol_token: w2-xUSDC
    needed_morpher_data: Option<ResourceAddress>,
    other_coin: Option<ResourceAddress>, // Only for protocols managing two coins, i.e. providing
                                         // liquidity to a Dex
}

#[derive(ScryptoSbor, ScryptoEvent)]
struct LsuUnstakeStartedEvent {
    lsu_amount: Decimal,
    claim_nft_id: NonFungibleLocalId,
}

#[derive(ScryptoSbor, ScryptoEvent)]
struct LsuUnstakeCompletedEvent {
    xrd_amount: Decimal,
    defi_protocol_name: String,
    fund_units_to_distribute: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent)]
struct WithdrawFromFundEvent {
    fund_unit_amount: Decimal,
    defi_protocol_name: String,
}

#[derive(ScryptoSbor, ScryptoEvent)]
struct MissingInfoEvent {
    defi_protocol_name: String,
}

#[blueprint]
#[events(
    LsuUnstakeStartedEvent,
    LsuUnstakeCompletedEvent,
    WithdrawFromFundEvent,
    MissingInfoEvent,
)]
#[types(
    String,
    DefiProtocol,
)]
mod fund_manager {

    enable_method_auth! {
        roles {
            bot => updatable_by: [OWNER];
        },
        methods {
            init => PUBLIC;

            // Multisig operations
            add_defi_protocol => PUBLIC;
            remove_defi_protocol => PUBLIC;
            set_dex_component => PUBLIC;
            withdraw_validator_badge => PUBLIC;
            decrease_min_authorizers => PUBLIC;
            increase_min_authorizers => PUBLIC;
            mint_admin_badge => PUBLIC;
            set_oracle_component => PUBLIC;
            withdraw_fund_manager_badge => PUBLIC;
            set_withdrawal_fee => PUBLIC;
            mint_bot_badge => PUBLIC;
            set_buyback_fund => PUBLIC;

            // Single admin operations
            authorize_admin_operation => PUBLIC;
            deposit_validator_badge => restrict_to: [OWNER];
            deposit_coin => restrict_to: [OWNER];
            deposit_protocol_token => restrict_to: [OWNER];
            deposit_fund_manager_badge => restrict_to: [OWNER];

            start_unlock_owner_stake_units => restrict_to: [bot];
            start_unstake => restrict_to: [bot];
            finish_unstake => restrict_to: [bot];
            fund_units_distribution => restrict_to: [bot];
            update_defi_protocols_info => restrict_to: [bot];

            withdraw => PUBLIC;
            fund_unit_value => PUBLIC;
            fund_details => PUBLIC;
        }
    }

    struct FundManager {
        admin_badge_resource_manager: NonFungibleResourceManager,
        bot_badge_resource_manager: FungibleResourceManager,
        fund_unit_resource_manager: FungibleResourceManager,
        validator_badge_vault: NonFungibleVault,
        authorization_vector: Vec<Authorization>,
        min_authorizers: u8,
        defi_protocols_list: Vec<String>,
        defi_protocols: KeyValueStore<String, DefiProtocol>,
        fund_manager_badge_vault: FungibleVault,
        validator: Global<Validator>,
        claim_nft_vault: NonFungibleVault,
        account_locker: Global<AccountLocker>,
        dex: Option<DexInterfaceScryptoStub>,
        total_value: Decimal,
        fund_units_vault: FungibleVault,
        fund_units_to_distribute: Decimal,
        oracle_component: Option<OracleInterfaceScryptoStub>,
        withdrawal_fee: u8,
        number_of_admins: u8,
        buyback_fund_percentage: u8,
        buyback_fund_account: Global<Account>,
    }

    impl FundManager {

        pub fn new(
            validator: Global<Validator>,
            claim_nft_address: ResourceAddress,
            withdrawal_fee: u8,
            buyback_fund_percentage: u8,
            buyback_fund_account: Global<Account>,
        ) -> Global<FundManager> {
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(FundManager::blueprint_id());

            let fund_manager_badge_bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(0)
                .metadata(metadata!(
                    roles {
                        metadata_setter => rule!(deny_all);
                        metadata_setter_updater => rule!(deny_all);
                        metadata_locker => rule!(deny_all);
                        metadata_locker_updater => rule!(deny_all);
                    },
                    init {
                        "name" => "Fund manager badge", updatable;
                    }
                ))
                .mint_roles(mint_roles!(
                    minter => rule!(deny_all);
                    minter_updater => rule!(deny_all);
                ))
                .mint_initial_supply(Decimal::ONE);
            let fund_manager_badge_address = fund_manager_badge_bucket.resource_address();

            let admin_badge_resource_manager = ResourceBuilder::new_integer_non_fungible::<Admin>(OwnerRole::None)
                .metadata(metadata!(
                    roles {
                        metadata_setter => rule!(require(fund_manager_badge_address));
                        metadata_setter_updater => rule!(require(fund_manager_badge_address));
                        metadata_locker => rule!(require(fund_manager_badge_address));
                        metadata_locker_updater => rule!(require(fund_manager_badge_address));
                    },
                    init {
                        "name" => "Fund admin badge", locked;
                    }
                ))
                .mint_roles(mint_roles!(
                    minter => rule!(require(global_caller(component_address)));
                    minter_updater => rule!(require(fund_manager_badge_address));
                ))
                .recall_roles(recall_roles!(
                    recaller => rule!(require(fund_manager_badge_address)); // Recallable
                    recaller_updater => rule!(require(fund_manager_badge_address));
                ))
                .create_with_no_initial_supply();
            let admin_badge_address = admin_badge_resource_manager.address();

            let fund_unit_resource_manager = ResourceBuilder::new_fungible(OwnerRole::Fixed(rule!(require(admin_badge_address))))
                .metadata(metadata!(
                    roles {
                        metadata_setter => rule!(require(admin_badge_address));
                        metadata_setter_updater => rule!(require(fund_manager_badge_address));
                        metadata_locker => rule!(require(fund_manager_badge_address));
                        metadata_locker_updater => rule!(require(fund_manager_badge_address));
                    },
                    init {
                        "name" => "Fund unit", updatable;
                    }
                ))
                .mint_roles(mint_roles!(
                    minter => rule!(require(global_caller(component_address)));
                    minter_updater => rule!(require(fund_manager_badge_address));
                ))
                .burn_roles(burn_roles!(
                    burner => rule!(require(global_caller(component_address)));
                    burner_updater => rule!(require(fund_manager_badge_address));
                ))
                .create_with_no_initial_supply();

            let bot_badge_resource_manager = ResourceBuilder::new_fungible(OwnerRole::Fixed(rule!(require(admin_badge_address))))
                .divisibility(0)
                .metadata(metadata!(
                    roles {
                        metadata_setter => rule!(require(admin_badge_address));
                        metadata_setter_updater => rule!(require(fund_manager_badge_address));
                        metadata_locker => rule!(require(fund_manager_badge_address));
                        metadata_locker_updater => rule!(require(fund_manager_badge_address));
                    },
                    init {
                        "name" => "Fund bot badge", locked;
                    }
                ))
                .mint_roles(mint_roles!(
                    minter => rule!(require(global_caller(component_address)));
                    minter_updater => rule!(require(fund_manager_badge_address));
                ))
                .withdraw_roles(withdraw_roles!(
                    withdrawer => rule!(deny_all); // Non transferable
                    withdrawer_updater => rule!(require(fund_manager_badge_address));
                ))
                .recall_roles(recall_roles!(
                    recaller => rule!(require(fund_manager_badge_address)); // Recallable
                    recaller_updater => rule!(require(fund_manager_badge_address));
                ))
                .create_with_no_initial_supply();

            let account_locker = Blueprint::<AccountLocker>::instantiate(
                OwnerRole::Fixed(rule!(require(admin_badge_address))),  // owner_role
                AccessRule::Protected(                                  // storer_role
                    CompositeRequirement::AnyOf(vec![
                        CompositeRequirement::BasicRequirement(
                            BasicRequirement::Require(
                                global_caller(component_address)
                            )
                        ),
                        require(admin_badge_address),
                    ])
                ),
                rule!(require(fund_manager_badge_address)),             // storer_updater_role
                rule!(deny_all),                                        // recoverer_role
                rule!(require(fund_manager_badge_address)),             // recoverer_updater_role
                None
            );

            Self {
                admin_badge_resource_manager: admin_badge_resource_manager,
                bot_badge_resource_manager: bot_badge_resource_manager,
                fund_unit_resource_manager: fund_unit_resource_manager,
                validator_badge_vault: NonFungibleVault::new(VALIDATOR_OWNER_BADGE),
                authorization_vector: vec![],
                min_authorizers: 0,
                defi_protocols_list: vec![],
                defi_protocols: KeyValueStore::new_with_registered_type(),
                fund_manager_badge_vault: FungibleVault::with_bucket(fund_manager_badge_bucket),
                validator: validator,
                claim_nft_vault: NonFungibleVault::new(claim_nft_address),
                account_locker: account_locker,
                dex: None,
                total_value: Decimal::ZERO,
                fund_units_vault: FungibleVault::new(fund_unit_resource_manager.address()),
                fund_units_to_distribute: Decimal::ZERO,
                oracle_component: None,
                withdrawal_fee: withdrawal_fee,
                number_of_admins: 0,
                buyback_fund_percentage: buyback_fund_percentage,
                buyback_fund_account: buyback_fund_account,
            }
                .instantiate()
                .prepare_to_globalize(OwnerRole::Fixed(rule!(require(admin_badge_address))))
                .roles(roles!(
                    bot => rule!(require(bot_badge_resource_manager.address()));
                ))
                .with_address(address_reservation)
                .globalize()
        }

        pub fn init(
            &mut self,
            number_of_admin_badges: u8,
            min_authorizers: u8,
            fund_units_initial_supply: Decimal,
        ) -> NonFungibleBucket {
            assert!(
                self.number_of_admins == 0,
                "Component already initialised",
            );

            assert!(
                number_of_admin_badges > 0,
                "Create at least one admin badge",
            );

            assert!(
                number_of_admin_badges > min_authorizers,
                "The minimum number of authorizers must be smaller than the number of badges",
            );

            self.min_authorizers = min_authorizers;

            let mut admin_badges_bucket = NonFungibleBucket::new(self.admin_badge_resource_manager.address());

            for n in 1..=number_of_admin_badges {
                admin_badges_bucket.put(
                    self.admin_badge_resource_manager.mint_non_fungible(
                        &NonFungibleLocalId::integer(n.into()),
                        Admin {},
                    )
                );
            }

            self.fund_units_to_distribute = fund_units_initial_supply;
            self.fund_units_vault.put(
                self.fund_unit_resource_manager.mint(fund_units_initial_supply)
            );
            
            self.number_of_admins = number_of_admin_badges;

            admin_badges_bucket
        }

        pub fn mint_bot_badge(
            &mut self,
            admin_proof: Proof,
            mut new_bot_account: Global<Account>,
        ) {
            self.check_operation_authorization(
                self.get_admin_id(admin_proof),
                AuthorizedOperation::MintBotBadge,
                None,
                None,
                Some(new_bot_account),
            );

            let bot_badge = self.bot_badge_resource_manager.mint(Decimal::ONE);

            new_bot_account.try_deposit_or_abort(
                bot_badge.into(),
                None
            );
        }

        pub fn deposit_validator_badge(
            &mut self,
            validator_badge: NonFungibleBucket,
        ) {
            assert!(
                self.validator_badge_vault.is_empty(),
                "There's already a validator badge",
            );

            self.validator_badge_vault.put(validator_badge);
        }

        pub fn deposit_fund_manager_badge(
            &mut self,
            fund_manager_badge: FungibleBucket,
        ) {
            self.fund_manager_badge_vault.put(fund_manager_badge);
        }

        fn get_admin_id(
            &self,
            admin_proof: Proof,
        ) -> u8 {
            let non_fungible = admin_proof.check_with_message(
                self.admin_badge_resource_manager.address(),
                "Incorrect proof",
            )
                .as_non_fungible()
                .non_fungible::<Admin>();

            u8::try_from(
                match &non_fungible.local_id() {
                    NonFungibleLocalId::Integer(local_id) => local_id.value(),
                    _ => Runtime::panic("Incorrect proof".to_string()),
                }
            )
                .unwrap()
        }

        fn purge_authorization_vector(&mut self) {
            let now = Clock::current_time_rounded_to_seconds().seconds_since_unix_epoch;

            self.authorization_vector.retain(|authorization| {
                authorization.timestamp + AUTHORIZATION_TIMEOUT > now
            });
        }

        pub fn authorize_admin_operation(
            &mut self,
            admin_proof: Proof,
            allowed_admin_id: u8,
            authorized_operation: AuthorizedOperation,
            protocol_name: Option<String>,
            percentage: Option<u8>,
            account_address: Option<Global<Account>>,
        ) {
            let allower_admin_id = self.get_admin_id(admin_proof);

            assert!(
                allower_admin_id != allowed_admin_id,
                "You can't authorize yourself",
            );

            self.purge_authorization_vector();

            assert!(
                self.authorization_vector.len() < MAX_VECTOR_SIZE,
                "Authorization vector is getting too big",
            );

            self.authorization_vector.push(
                Authorization {
                    timestamp: Clock::current_time_rounded_to_seconds().seconds_since_unix_epoch,
                    allower_admin_id: allower_admin_id,
                    allowed_admin_id: allowed_admin_id,
                    authorized_operation: authorized_operation,
                    protocol_name: protocol_name,
                    percentage: percentage,
                    account_address: account_address,
                }
            );
        }

        fn check_operation_authorization(
            &mut self,
            admin_id: u8,
            authorized_operation: AuthorizedOperation,
            protocol_name: Option<String>,
            percentage: Option<u8>,
            account_address: Option<Global<Account>>,
        ) {
            self.purge_authorization_vector();

            let authorizers_number = self.authorization_vector
                .iter()
                .filter(|&authorization| {
                    authorization.allowed_admin_id == admin_id &&
                        authorization.authorized_operation == authorized_operation &&
                        authorization.protocol_name == protocol_name &&
                        authorization.percentage == percentage &&
                        authorization.account_address == account_address
                })
                .count();

            assert!(
                authorizers_number >= self.min_authorizers.into(),
                "Operation not authorized",
            );

            self.authorization_vector.retain(|authorization| {
                authorization.allowed_admin_id != admin_id || authorization.authorized_operation != authorized_operation
            });
        }

        pub fn fund_unit_value(&self) -> (Decimal, Decimal) {
            let gross_value = self.total_value / self.fund_unit_resource_manager.total_supply().unwrap();

            (
                (gross_value * (100 - self.withdrawal_fee)) / 100, // net value
                gross_value
            )
        }

        pub fn fund_details(&self) -> HashMap<String, Decimal> {
            let mut protocols_value = HashMap::new();

            for name in self.defi_protocols_list.iter() {
                let defi_protocol = self.defi_protocols.get(&name).unwrap();

                protocols_value.insert(name.clone(), defi_protocol.value);
            }

            protocols_value
        }

        pub fn withdraw_validator_badge(
            &mut self,
            admin_proof: Proof,
        ) -> NonFungibleBucket {
            self.check_operation_authorization(
                self.get_admin_id(admin_proof),
                AuthorizedOperation::WithdrawValidatorBadge,
                None,
                None,
                None,
            );

            self.validator_badge_vault.take_all()
        }

        pub fn withdraw_fund_manager_badge(
            &mut self,
            admin_proof: Proof,
        ) -> FungibleBucket {
            self.check_operation_authorization(
                self.get_admin_id(admin_proof),
                AuthorizedOperation::WithdrawFundManagerBadge,
                None,
                None,
                None,
            );

            self.fund_manager_badge_vault.take_all()
        }

        pub fn increase_min_authorizers(
            &mut self,
            admin_proof: Proof,
        ) {
            self.check_operation_authorization(
                self.get_admin_id(admin_proof),
                AuthorizedOperation::IncreaseMinAuthorizers,
                None,
                None,
                None,
            );

            self.min_authorizers += 1;

            assert!(
                self.min_authorizers < self.number_of_admins,
                "The minimum number of authorizers must be smaller than the number of badges",
            );
        }

        pub fn decrease_min_authorizers(
            &mut self,
            admin_proof: Proof,
        ) {
            self.check_operation_authorization(
                self.get_admin_id(admin_proof),
                AuthorizedOperation::DecreaseMinAuthorizers,
                None,
                None,
                None,
            );

            self.min_authorizers -= 1;
        }

        pub fn mint_admin_badge(
            &mut self,
            admin_proof: Proof,
            mut new_admin_account: Global<Account>,
        ) {
            self.check_operation_authorization(
                self.get_admin_id(admin_proof),
                AuthorizedOperation::MintAdminBadge,
                None,
                None,
                Some(new_admin_account),
            );

            self.number_of_admins += 1;

            let admin_badge = self.admin_badge_resource_manager.mint_non_fungible(
                &NonFungibleLocalId::integer(self.number_of_admins.into()),
                Admin {},
            );

            new_admin_account.try_deposit_or_abort(
                admin_badge.into(),
                None
            );
        }

        pub fn start_unlock_owner_stake_units(
            &mut self,
            amount: Decimal,
        ) { 
            self.validator_badge_vault
                .authorize_with_non_fungibles(
                    &self.validator_badge_vault.non_fungible_local_ids(1),
                    || {
                        self.validator.start_unlock_owner_stake_units(amount);
                    }
                );
        }

        pub fn start_unstake(&mut self) { 
            let lsu_bucket = self.validator_badge_vault
                .authorize_with_non_fungibles(
                    &self.validator_badge_vault.non_fungible_local_ids(1),
                    || {
                        self.validator.finish_unlock_owner_stake_units()
                    }
                );

            let lsu_amount = lsu_bucket.amount();

            assert!(
                lsu_amount > Decimal::ZERO,
                "No LSU available"
            );

            let claim_nft_bucket = self.validator.unstake(lsu_bucket);

            Runtime::emit_event(
                LsuUnstakeStartedEvent {
                    lsu_amount: lsu_amount,
                    claim_nft_id: claim_nft_bucket.non_fungible_local_id(),
                }
            );
            
            self.claim_nft_vault.put(claim_nft_bucket);
        }

        fn find_where_to_deposit_to(&self) -> String {
            let mut smallest_percentage_diff: Decimal = dec!(101);
            let mut smallest_percentage_diff_name: Option<String> = None;

            for name in self.defi_protocols_list.iter() {
                let defi_protocol = self.defi_protocols.get(&name).unwrap();

                let percentage = 100 * defi_protocol.value / self.total_value;
                let percentage_diff: Decimal = percentage - defi_protocol.desired_percentage;

                if percentage_diff < smallest_percentage_diff {
                    smallest_percentage_diff = percentage_diff;
                    smallest_percentage_diff_name = Some(name.to_string());
                }
            }

            smallest_percentage_diff_name.unwrap()
        }

        pub fn finish_unstake(
            &mut self,
            claim_nft_id: String,
            morpher_data: HashMap<ResourceAddress, (String, String)>,
        ) {
            assert!(
                self.fund_units_vault.amount() == Decimal::ZERO,
                "Previous distribution was not finished",
            );

            let claim_nft_bucket = self.claim_nft_vault.take_non_fungible(
                &NonFungibleLocalId::String(StringNonFungibleLocalId::try_from(claim_nft_id).unwrap())
            );

            let mut bucket = self.validator.claim_xrd(claim_nft_bucket);

            let buyback_fund_bucket = bucket.take(
                (bucket.amount() * self.buyback_fund_percentage) / 100
            );
            self.buyback_fund_account.try_deposit_or_abort(
                buyback_fund_bucket.into(),
                None
            );

            let xrd_amount = bucket.amount();

            let (_, fund_unit_gross_value) = self.fund_unit_value();

            let defi_protocol_name = self.find_where_to_deposit_to();

            let defi_protocol = self.defi_protocols.get(&defi_protocol_name).unwrap();

            let (message, signature) = match defi_protocol.needed_morpher_data {
                Some(resource_address) => {
                    let morpher_data_needed_by_protocol = morpher_data.get(&resource_address).expect("Missing needed morpher data").clone();
                    
                    (Some(morpher_data_needed_by_protocol.0), Some(morpher_data_needed_by_protocol.1))
                },
                None => (None, None),
            };

            let bucket_value = xrd_amount * self.oracle_component.unwrap().get_price(
                defi_protocol.coin,
                morpher_data
            );

            if defi_protocol.coin == XRD {
                drop(defi_protocol);

                let mut defi_protocol = self.defi_protocols.get_mut(&defi_protocol_name).unwrap();

                self.fund_manager_badge_vault.authorize_with_amount(
                    1,
                    || defi_protocol.wrapper.deposit_coin(
                        bucket,
                        None,
                        message,
                        signature,
                    )
                );

                defi_protocol.value += bucket_value;
            } else if defi_protocol.other_coin == Some(XRD) {
                let defi_protocol_coin = defi_protocol.coin;

                drop(defi_protocol);

                let mut defi_protocol = self.defi_protocols.get_mut(&defi_protocol_name).unwrap();

                self.fund_manager_badge_vault.authorize_with_amount(
                    1,
                    || defi_protocol.wrapper.deposit_coin(
                        FungibleBucket::new(defi_protocol_coin),
                        Some(bucket),
                        message,
                        signature,
                    )
                );

                defi_protocol.value += bucket_value;
            } else {
                drop(defi_protocol);

                let mut defi_protocol = self.defi_protocols.get_mut(&defi_protocol_name).unwrap();

                bucket = self.fund_manager_badge_vault.authorize_with_amount(
                    1,
                    || FungibleBucket(self.dex.unwrap().swap(bucket.into(), defi_protocol.coin))
                );

                self.fund_manager_badge_vault.authorize_with_amount(
                    1,
                    || defi_protocol.wrapper.deposit_coin(
                        bucket,
                        None,
                        message,
                        signature,
                    )
                );

                defi_protocol.value += bucket_value;
            }

            self.total_value += bucket_value;

            self.fund_units_to_distribute = self.total_value / fund_unit_gross_value;

            self.fund_units_vault.put(
                self.fund_unit_resource_manager.mint(self.fund_units_to_distribute + Decimal::ONE)
            );

            Runtime::emit_event(
                LsuUnstakeCompletedEvent {
                    xrd_amount: xrd_amount,
                    defi_protocol_name: defi_protocol_name,
                    fund_units_to_distribute: self.fund_units_to_distribute,
                }
            );
        }

        pub fn fund_units_distribution (
            &mut self,
            stakers: IndexMap<Global<Account>, Decimal>,
            more_stakers: bool,
        ) {
            let mut distribution: IndexMap<Global<Account>, ResourceSpecifier> = IndexMap::new();
            for (account, share) in stakers.iter() {
                distribution.insert(
                    *account,
                    ResourceSpecifier::Fungible(*share * self.fund_units_to_distribute),
                );
            }

            let remainings = self.account_locker.airdrop(
                distribution,
                self.fund_units_vault.take_all().into(),
                true,
            );

            if more_stakers {
                self.fund_units_vault.put(FungibleBucket(remainings.unwrap()));
            } else if remainings.is_some() {
                remainings.unwrap().burn();
            }
        }

        pub fn add_defi_protocol(
            &mut self,
            admin_proof: Proof,
            name: String,
            coin: ResourceAddress,
            protocol_token: ResourceAddress,
            other_coin: Option<ResourceAddress>,
            desired_percentage: u8,
            wrapper: DefiProtocolInterfaceScryptoStub,
            needed_morpher_data: Option<ResourceAddress>,
        ) {
            self.check_operation_authorization(
                self.get_admin_id(admin_proof),
                AuthorizedOperation::AddDefiProtocol,
                Some(name.clone()),
                None,
                None,
            );

            let mut old_defi_protocol: Option<DefiProtocol> = None;

            if self.defi_protocols_list.iter().position(|n| *n == name).is_none() {
                assert!(
                    self.defi_protocols_list.len() < MAX_VECTOR_SIZE,
                    "Protocols list is getting too big",
                );

                self.defi_protocols_list.push(name.clone());
            } else {
                old_defi_protocol= self.defi_protocols.remove(&name);
            }

            let mut new_defi_protocol = DefiProtocol {
                value: Decimal::ZERO,
                desired_percentage: desired_percentage,
                wrapper: wrapper,
                coin: coin,
                protocol_token: protocol_token,
                other_coin: other_coin,
                needed_morpher_data: needed_morpher_data,
            };

            if old_defi_protocol.is_some() {
                new_defi_protocol.value = old_defi_protocol.as_ref().unwrap().value;

                self.fund_manager_badge_vault.authorize_with_amount(
                    1,
                    || {
                        let bucket = old_defi_protocol.unwrap().wrapper.withdraw_protocol_token(None);
                        new_defi_protocol.wrapper.deposit_protocol_token(bucket);
                    }
                );
            }

            self.defi_protocols.insert(
                name,
                new_defi_protocol,
            );
        }

        pub fn deposit_coin(
            &mut self,
            defi_protocol_name: String,
            coin_bucket: FungibleBucket,
            other_coin_bucket: Option<FungibleBucket>,
            morpher_data: HashMap<ResourceAddress, (String, String)>,
        ) {
            let mut buckets_value = coin_bucket.amount() * self.oracle_component.unwrap().get_price(
                coin_bucket.resource_address(),
                morpher_data.clone(),
            );

            if other_coin_bucket.is_some() {
                buckets_value +=
                    other_coin_bucket.as_ref().unwrap().amount() *
                    self.oracle_component.unwrap().get_price(
                        other_coin_bucket.as_ref().unwrap().resource_address(),
                        morpher_data.clone()
                    );
            }

            let mut defi_protocol = self.defi_protocols.get_mut(&defi_protocol_name).expect("Protocol not found");

            let (message, signature) = match defi_protocol.needed_morpher_data {
                Some(resource_address) => {
                    let morpher_data_needed_by_protocol = morpher_data.get(&resource_address).expect("Missing needed morpher data").clone();
                    
                    (Some(morpher_data_needed_by_protocol.0), Some(morpher_data_needed_by_protocol.1))
                },
                None => (None, None),
            };

            self.fund_manager_badge_vault.authorize_with_amount(
                1,
                || defi_protocol.wrapper.deposit_coin(
                    coin_bucket,
                    other_coin_bucket,
                    message,
                    signature,
                )
            );

            defi_protocol.value += buckets_value;
            self.total_value += buckets_value;
        }

        pub fn deposit_protocol_token(
            &mut self,
            defi_protocol_name: String,
            protocol_token_bucket: Bucket,
            morpher_data: HashMap<ResourceAddress, (String, String)>,
        ) {
            let mut added_value = Decimal::ZERO;

            let defi_protocol = self.defi_protocols.get(&defi_protocol_name).expect("Protocol not found");

            let coin1_price = self.oracle_component.unwrap().get_price(defi_protocol.coin, morpher_data.clone());

            let coin2_price = match defi_protocol.other_coin {
                Some(coin2) => Some(self.oracle_component.unwrap().get_price(coin2, morpher_data)),
                None => None,
            };

            drop(defi_protocol);

            let mut defi_protocol = self.defi_protocols.get_mut(&defi_protocol_name).unwrap();

            let (option_amount1, option_amount2) = self.fund_manager_badge_vault.authorize_with_amount(
                1,
                || defi_protocol.wrapper.deposit_protocol_token(protocol_token_bucket)
            );

            if option_amount1.is_some() {
                added_value += option_amount1.unwrap() * coin1_price;
            } else {
                Runtime::emit_event(
                    MissingInfoEvent {
                        defi_protocol_name: defi_protocol_name,
                    }
                );
            }

            if option_amount2.is_some() {
                added_value += option_amount2.unwrap() * coin2_price.unwrap();
            }

            defi_protocol.value += added_value;
            self.total_value += added_value;
        }

        pub fn remove_defi_protocol(
            &mut self,
            admin_proof: Proof,
            name: String,
        ) -> Bucket {
            self.check_operation_authorization(
                self.get_admin_id(admin_proof),
                AuthorizedOperation::RemoveDefiProtocol,
                Some(name.clone()),
                None,
                None,
            );

            self.defi_protocols_list.retain(|n| { *n != name });

            let mut defi_protocol = self.defi_protocols.remove(&name)
                .expect("Protocol not found");

            self.total_value -= defi_protocol.value;

            self.fund_manager_badge_vault.authorize_with_amount(
                1,
                || defi_protocol.wrapper.withdraw_protocol_token(None)
            )
        }

        pub fn update_defi_protocols_info(
            &mut self,
            defi_protocols_value: HashMap<String, Decimal>,
            defi_protocols_desired_percentage: HashMap<String, u8>,
        ) {
            let mut value_change = Decimal::ZERO;

            for (name, value) in defi_protocols_value.iter() {
                let mut defi_protocol = self.defi_protocols.get_mut(&name).expect("Not found");

                value_change += *value - defi_protocol.value;

                defi_protocol.value = *value;
            }

            self.total_value += value_change;

            for (name, percentage) in defi_protocols_desired_percentage.iter() {
                assert!(
                    *percentage <= 100,
                    "Pergentage out of 0 -100 range"
                );

                let mut defi_protocol = self.defi_protocols.get_mut(&name).expect("Not found");

                defi_protocol.desired_percentage = *percentage;
            }
        }

        fn find_where_to_withdraw_from(
            &self,
            amount: Decimal,
        ) -> (String, Decimal) {
            let mut defi_protocol_candidates: Vec<String> = vec![];

            for name in self.defi_protocols_list.iter() {
                let value = self.defi_protocols.get(&name).unwrap().value;

                if value >= amount {
                    defi_protocol_candidates.push(name.to_string());
                }
            }

            if defi_protocol_candidates.len() == 0 {
                let mut largest_value = Decimal::ZERO;
                let mut largest_value_name: Option<String> = None;

                for name in self.defi_protocols_list.iter() {
                    let value = self.defi_protocols.get(&name).unwrap().value;

                    if value > largest_value {
                        largest_value = value;
                        largest_value_name = Some(name.to_string());
                    }
                }

                return (largest_value_name.unwrap(), largest_value);
            }

            let mut largest_percentage_diff: Decimal = dec!(-101);
            let mut largest_percentage_diff_name: Option<String> = None;

            for name in defi_protocol_candidates.iter() {
                let defi_protocol = self.defi_protocols.get(&name).unwrap();
                let percentage = 100 * defi_protocol.value / self.total_value;
                let percentage_diff: Decimal = percentage - defi_protocol.desired_percentage;

                if percentage_diff > largest_percentage_diff {
                    largest_percentage_diff = percentage_diff;
                    largest_percentage_diff_name = Some(name.to_string());
                }
            }

            return (largest_percentage_diff_name.unwrap(), amount);
        }

        pub fn withdraw(
            &mut self,
            mut fund_units_bucket: FungibleBucket,
            swap_to: Option<ResourceAddress>,
            morpher_data: HashMap<ResourceAddress, (String, String)>,
        ) -> (
            FungibleBucket, // coin
            Option<FungibleBucket>, // other coin
            Option<FungibleBucket>, // fund units
        ) {
            assert!(
                fund_units_bucket.resource_address() == self.fund_unit_resource_manager.address(),
                "Wrong coin",
            );

            let (fund_unit_net_value, fund_unit_gross_value) = self.fund_unit_value();

            let (defi_protocol_name, withdrawable_value) = self.find_where_to_withdraw_from(
                fund_units_bucket.amount() * fund_unit_net_value
            );

            let defi_protocol = self.defi_protocols.get(&defi_protocol_name).unwrap();

            let coin_value = self.oracle_component.unwrap().get_price(defi_protocol.coin, morpher_data.clone());

            let (other_coin_to_coin_price_ratio, other_coin_value) = match defi_protocol.other_coin {
                Some(other_coin) => {
                    let other_coin_price = self.oracle_component.unwrap().get_price(other_coin, morpher_data);

                    (Some(other_coin_price / coin_value), Some(other_coin_price))
                },
                None => (None, None),
            };

            drop(defi_protocol);

            let mut defi_protocol = self.defi_protocols.get_mut(&defi_protocol_name).unwrap();

            let (mut coin_bucket, mut other_coin_bucket) = self.fund_manager_badge_vault.authorize_with_amount(
                1,
                || defi_protocol.wrapper.withdraw_coin(
                    Some(withdrawable_value / coin_value),
                    other_coin_to_coin_price_ratio,
                )
            );

            let mut coin_bucket_value = coin_bucket.amount() * coin_value;
            if other_coin_bucket.is_some() {
                coin_bucket_value += other_coin_bucket.as_ref().unwrap().amount() * other_coin_value.unwrap();
            }

            self.total_value -= coin_bucket_value;
            defi_protocol.value -= coin_bucket_value;

            if swap_to.is_some() {
                if swap_to.unwrap() != defi_protocol.coin {
                    coin_bucket = self.fund_manager_badge_vault.authorize_with_amount(
                        1,
                        || FungibleBucket(self.dex.unwrap().swap(coin_bucket.into(), swap_to.unwrap()))
                    );
                }

                if other_coin_bucket.is_some() && swap_to.unwrap() != defi_protocol.other_coin.unwrap() {
                    coin_bucket.put(
                        self.fund_manager_badge_vault.authorize_with_amount(
                            1,
                            || FungibleBucket(
                                self.dex.unwrap().swap(other_coin_bucket.unwrap().into(), swap_to.unwrap())
                            )
                        )
                    );

                    other_coin_bucket = None;
                }
            }

            let fund_units_to_burn = coin_bucket_value / fund_unit_gross_value;

            Runtime::emit_event(
                WithdrawFromFundEvent {
                    fund_unit_amount: fund_units_to_burn,
                    defi_protocol_name: defi_protocol_name,
                }
            );

            if fund_units_to_burn < fund_units_bucket.amount() {
                fund_units_bucket.take(fund_units_to_burn).burn();

                (coin_bucket, other_coin_bucket, Some(fund_units_bucket))
            } else {
                fund_units_bucket.burn();

                (coin_bucket, other_coin_bucket, None)
            }
        }

        pub fn set_dex_component(
            &mut self,
            admin_proof: Proof,
            dex: DexInterfaceScryptoStub,
        ) {
            self.check_operation_authorization(
                self.get_admin_id(admin_proof),
                AuthorizedOperation::SetDexComponent,
                None,
                None,
                None,
            );

            self.dex = Some(dex);
        }

        pub fn set_withdrawal_fee(
            &mut self,
            admin_proof: Proof,
            percentage: u8,
        ) {
            self.check_operation_authorization(
                self.get_admin_id(admin_proof),
                AuthorizedOperation::SetWithdrawalFee,
                None,
                Some(percentage),
                None,
            );

            assert!(
                percentage < 100,
                "Fee must be a number from 0 to 100"
            );

            self.withdrawal_fee = percentage;
        }

        pub fn set_oracle_component(
            &mut self,
            admin_proof: Proof,
            component: OracleInterfaceScryptoStub,
        ) {
            self.check_operation_authorization(
                self.get_admin_id(admin_proof),
                AuthorizedOperation::SetOracleComponent,
                None,
                None,
                None,
            );

            self.oracle_component = Some(component);
        }

        pub fn set_buyback_fund(
            &mut self,
            admin_proof: Proof,
            percentage: u8,
            account: Global<Account>,
        ) {
            self.check_operation_authorization(
                self.get_admin_id(admin_proof),
                AuthorizedOperation::SetBuybackFund,
                None,
                Some(percentage),
                Some(account),
            );

            assert!(
                percentage < 100,
                "Fee must be a number from 0 to 100"
            );

            self.buyback_fund_percentage = percentage;
            self.buyback_fund_account = account;
        }
    }
}
