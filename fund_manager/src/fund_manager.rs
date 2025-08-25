/*
 * This is the main blueprint of the FundManager software; it unstakes XRD from the Validator,
 * invests them in one of the DeFi protocols and mints and distributes new fund units to reward stakers.
 *
 * Multiple admins can manage the FundManager component; some operations can be performed by a
 * single admin, other ones require an authorization from other admins (sort of application level
 * multisignature).
 */
use scrypto::prelude::*;
use crate::common::*;

// How long an authorization from an admin lasts if not used by another admin.
static AUTHORIZATION_TIMEOUT: i64 = 172800; // Two days

// Maximum size for authorizations list and DeFi protocols list.
static MAX_VECTOR_SIZE: usize = 50;

// Acceptable value ratio that can be lost or gained when withdrawing
static ACCEPTABLE_VALUE_DIFFERENCE: Decimal = dec!("0.1");

// Admin badge NonFungibleData. Each one is just identified by a numeric id.
#[derive(ScryptoSbor, NonFungibleData)]
struct Admin {
}

// Admin operations that require authorization from other admins.
#[derive(ScryptoSbor, PartialEq, Debug)]
#[repr(u8)]
pub enum AuthorizedOperation {
    WithdrawValidatorBadge      = 0,    // withdraw_validator_badge method
    AddDefiProtocol             = 1,    // add_defi_protocol method
    RemoveDefiProtocol          = 2,    // remove_defi_protocol method
    SetDexComponent             = 3,    // set_dex_component method
    DecreaseMinAuthorizers      = 4,    // decrease_min_authorizers method
    IncreaseMinAuthorizers      = 5,    // increase_min_authorizers method
    MintAdminBadge              = 6,    // mint_admin_badge method
    SetOracleComponent          = 7,    // set_oracle_component method
    WithdrawFundManagerBadge    = 8,    // withdraw_fund_manager method
    SetWithdrawalFee            = 9,    // set_withdrawal_fee method
    MintBotBadge                = 10,   // mint_bot_badge method
    SetBuybackFund              = 11,   // set_buyback_fund method
}
impl From<u8> for AuthorizedOperation {
    fn from(orig: u8) -> Self {
        match orig {
            0  => return AuthorizedOperation::WithdrawValidatorBadge,
            1  => return AuthorizedOperation::AddDefiProtocol,
            2  => return AuthorizedOperation::RemoveDefiProtocol,
            3  => return AuthorizedOperation::SetDexComponent,
            4  => return AuthorizedOperation::DecreaseMinAuthorizers,
            5  => return AuthorizedOperation::IncreaseMinAuthorizers,
            6  => return AuthorizedOperation::MintAdminBadge,
            7  => return AuthorizedOperation::SetOracleComponent,
            8  => return AuthorizedOperation::WithdrawFundManagerBadge,
            9  => return AuthorizedOperation::SetWithdrawalFee,
            10 => return AuthorizedOperation::MintBotBadge,
            11 => return AuthorizedOperation::SetBuybackFund,
            _  => Runtime::panic("Unknown operation".to_string()),
        };
    }
}

// This struct represents the authorization from one admin (allower_admin_id) to another admin
// (allowed_admin_id) to perform an operation (authorized_operation).
// Depending on the operation some optional information can be required (protocol_name, percentage,
// account_address).
#[derive(ScryptoSbor, Debug)]
struct Authorization {
    timestamp: i64,
    allower_admin_id: u8,
    allowed_admin_id: u8,
    authorized_operation: AuthorizedOperation,
    protocol_name: Option<String>,
    percentage: Option<u8>,
    account_address: Option<Global<Account>>,
}

// This struct describes one of the FundManager investment in a DeFi protocol.
// Communication with the DeFi protocol happens through a wrapper component that implements the
// DefiProtocolInterfaceScryptoStub interface.
// Each DeFi protocol position can manage one or two coins. For DeFi protocols that support
// multiple coins, multiple instances of the wrapper will be created.
#[derive(ScryptoSbor, Debug)]
struct DefiProtocol {
    value: Decimal, // Investement value in USD
    desired_percentage: u8, // Desired percentage of the fund to invest in this protocol
    wrapper: DefiProtocolInterfaceScryptoStub,
    coin: ResourceAddress, // Example coin: xUSDC
    protocol_token: ResourceAddress, // Example protocol_token: w2-xUSDC
    needed_morpher_data: Option<ResourceAddress>, // Invoking Flux protocol methods requires data
                                                  // from the Morpher oracle
    other_coin: Option<ResourceAddress>, // Only for protocols managing two coins, i.e. providing
                                         // liquidity to a Dex
}

// This event is issued when the LSU unstake starts and a claim NFT is minted.
#[derive(ScryptoSbor, ScryptoEvent, Debug)]
struct LsuUnstakeStartedEvent {
    lsu_amount: Decimal,
    claim_nft_id: NonFungibleLocalId,
}

// This event is issued when the LSU unstake is completed and the resulting XRD have been invested
// in a DeFi protocol.
// It also contains the amount of new fund units that must be distributed.
#[derive(ScryptoSbor, ScryptoEvent, Debug)]
struct LsuUnstakeCompletedEvent {
    xrd_amount: Decimal,
    defi_protocol_name: String,
    fund_units_to_distribute: Decimal,
    protocol_value: Decimal,
    total_value: Decimal,
}

// This event is issued when a user swaps his fund units for the coins that were invested in a DeFi
// protocol.
// Fund units are burned while both protocol_value and fund total_value are updated.
#[derive(ScryptoSbor, ScryptoEvent, Debug)]
struct WithdrawFromFundEvent {
    fund_unit_amount: Decimal,
    defi_protocol_name: String,
    protocol_value: Decimal,
    total_value: Decimal,
}

// Admins are allowed to deposit coins (or DeFi protocol tokens) in one of the DeFi protocols and
// eventually mint new fund units; this event is emitted when this operation is performed.
#[derive(ScryptoSbor, ScryptoEvent, Debug)]
struct AdminDepositEvent {
    defi_protocol_name: String,
    protocol_value: Decimal,
    total_value: Decimal,
}

// This event is emitted when the bot asks the FundManager component to update the value estimate
// of one of the investments.
// The FundManager component asks the wrapper component about the held coins and the oracle about
// their value to update this information.
// The fund total value is updated too.
#[derive(ScryptoSbor, ScryptoEvent, Debug)]
struct ProtocolValueUpdateEvent {
    defi_protocol_name: String,
    protocol_value: Decimal,
    total_value: Decimal,
}

// This event is emitted when a protocol is removed from the fund.
#[derive(ScryptoSbor, ScryptoEvent, Debug)]
struct RemovedProtocolEvent {
    defi_protocol_name: String,
    total_value: Decimal,
}

#[blueprint]
#[events(
    LsuUnstakeStartedEvent,
    LsuUnstakeCompletedEvent,
    WithdrawFromFundEvent,
    AdminDepositEvent,
    ProtocolValueUpdateEvent,
    RemovedProtocolEvent,
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
            register_validator => restrict_to: [OWNER];
            signal_protocol_update_readiness => restrict_to: [OWNER];
            update_node_key => restrict_to: [OWNER];

            // Bot operations
            start_unlock_owner_stake_units => restrict_to: [bot];
            start_unstake => restrict_to: [bot];
            finish_unstake => restrict_to: [bot];
            fund_units_distribution => restrict_to: [bot];
            update_defi_protocols_value => restrict_to: [bot];
            set_defi_protocols_percentage => restrict_to: [bot];

            // Unauthenticated user operation
            withdraw => PUBLIC;
            fund_unit_value => PUBLIC;
            fund_details => PUBLIC;
        }
    }

    struct FundManager {
        // Resource managers for minting badges
        admin_badge_resource_manager: NonFungibleResourceManager,
        bot_badge_resource_manager: FungibleResourceManager,

        // Resource manager for minting fund units
        fund_unit_resource_manager: FungibleResourceManager,

        // Where to store the Validator owner badge
        validator_badge_vault: NonFungibleVault,

        // List of pending admin authorized operations (limited to MAX_VECTOR_SIZE)
        authorization_vector: Vec<Authorization>,

        // Minimum number of distinct admin authorizations needed for a multisig operation
        min_authorizers: u8,

        // List of names assigned to DeFi protocol positions (limited to MAX_VECTOR_SIZE)
        // This is needed because KeyValueStore is not iterable
        defi_protocols_list: Vec<String>,

        // Details about each DeFi protocol position
        defi_protocols: KeyValueStore<String, DefiProtocol>,

        // Where to store the fund manager badge that is needed to talk to the DeFi protocol
        // wrappers
        fund_manager_badge_vault: FungibleVault,

        // Address of the Validator
        validator: Global<Validator>,

        // A Vault to store claim NFTs of the LSU being unstaked
        claim_nft_vault: NonFungibleVault,

        // The AccountLocker to distribute the minted fund units
        account_locker: Global<AccountLocker>,

        // The address of the component that wrappes all of the available dexes
        dex: Option<DexInterfaceScryptoStub>,

        // Current estimated total value of the fund
        total_value: Decimal,

        // A Vault to store fund units that are being distributed.
        fund_units_vault: FungibleVault,

        // The total number of fund units in the current distribution batch
        fund_units_to_distribute: Decimal,

        // The address of the component that wrappes all of the available oracles
        oracle_component: Option<OracleInterfaceScryptoStub>,

        // Percentage fee for the withdraw oerations
        withdrawal_fee: u8,

        // Number of minted admin badges
        number_of_admins: u8,

        // Percentage of the unstaked XRD to send to the buyback fund
        buyback_fund_percentage: u8,

        // Address of the account managing the buyback fund
        buyback_fund_account: Global<Account>,
    }

    impl FundManager {

        // This function instantiates a globalized FundManager component
        pub fn new(
            validator: Global<Validator>,           // Validator address
            claim_nft_address: ResourceAddress,     // Validator's claim NFT address
            withdrawal_fee: u8,                     // Percentage withdrawal fee
            buyback_fund_percentage: u8,            // Percentage of XRD sent to the buyback fund
            buyback_fund_account: Global<Account>,  // Account managing the buyback fund
        ) -> Global<FundManager> {

            // Reserve a component address to set permissions
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(FundManager::blueprint_id());

            // Mint the one and only fund manager badge
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
                        "name" => "Fund manager badge", locked;
                    }
                ))
                .mint_roles(mint_roles!(
                    minter => rule!(deny_all);
                    minter_updater => rule!(deny_all);
                ))
                .mint_initial_supply(Decimal::ONE);
            let fund_manager_badge_address = fund_manager_badge_bucket.resource_address();

            // Create the resource manager to mint admin badges (those will be minted in the init
            // method).
            // Admin badges are non fungibles identified by a number, recallable by the fund
            // manager.
            let admin_badge_resource_manager = ResourceBuilder::new_integer_non_fungible::<Admin>(
                OwnerRole::Fixed(rule!(require(fund_manager_badge_address)))
            )
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
                    recaller => rule!(require(fund_manager_badge_address));
                    recaller_updater => rule!(require(fund_manager_badge_address));
                ))
                .create_with_no_initial_supply();
            let admin_badge_address = admin_badge_resource_manager.address();

            // Create the resource manager to mint fund units
            let fund_unit_resource_manager = ResourceBuilder::new_fungible(
                OwnerRole::Fixed(rule!(require(admin_badge_address)))
            )
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

            // Create the resource manager to mint bot badges.
            // Bot badges are fungibles with zero divisibility, non transferable and recallable by
            // the fund manager.
            let bot_badge_resource_manager = ResourceBuilder::new_fungible(
                OwnerRole::Fixed(rule!(require(admin_badge_address)))
            )
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
                    withdrawer => rule!(deny_all);
                    withdrawer_updater => rule!(require(fund_manager_badge_address));
                ))
                .recall_roles(recall_roles!(
                    recaller => rule!(require(fund_manager_badge_address));
                    recaller_updater => rule!(require(fund_manager_badge_address));
                ))
                .create_with_no_initial_supply();

            // Instantiate an AccountLocker
            // Both the admins and this component can use it for distribution
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

            // Instantiate the component and globalize it
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

        // This method can be called just once, immediately after component instantiation, to mint
        // admin badges and the initial supply of fund units
        pub fn init(
            &mut self,
            number_of_admin_badges: u8, // number of admin badges to mint
            min_authorizers: u8, // number of authorizers for multisig operation
            fund_units_initial_supply: Decimal,
        ) -> (
            NonFungibleBucket, // Admin badges
            FungibleBucket, // Fund units initial supply
        ) {
            // Make sure this method hasn't been invoked before
            assert!(
                self.number_of_admins == 0,
                "Component already initialised",
            );

            // Make sure the numbers make sense
            assert!(
                number_of_admin_badges > 0,
                "Create at least one admin badge",
            );
            assert!(
                number_of_admin_badges > min_authorizers,
                "The minimum number of authorizers must be smaller than the number of badges",
            );

            self.min_authorizers = min_authorizers;

            // Mint the admin badges numbering them from 1 to number_of_admin_badges
            let mut admin_badges_bucket = NonFungibleBucket::new(
                self.admin_badge_resource_manager.address()
            );
            for n in 1..=number_of_admin_badges {
                admin_badges_bucket.put(
                    self.admin_badge_resource_manager.mint_non_fungible(
                        &NonFungibleLocalId::integer(n.into()),
                        Admin {},
                    )
                );
            }
            self.number_of_admins = number_of_admin_badges;

            // Return all of the admin badges and the fund units initial supply
            (
                admin_badges_bucket,
                self.fund_unit_resource_manager.mint(fund_units_initial_supply)
            )
        }

        // This method mints a bot badge and sends it to the specified account.
        // Admins must authorize this operation and agree on the account that will receive the
        // badge.
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

        // This method deposits back the Validator owner badge in the component in case it has
        // been previously withdrawn.
        // A single admin can perform this operation, authorization not needed.
        pub fn deposit_validator_badge(
            &mut self,
            validator_badge: NonFungibleBucket,
        ) {
            // It's not possible to deposit more than one Validator badge
            assert!(
                self.validator_badge_vault.is_empty(),
                "There's already a validator badge",
            );

            self.validator_badge_vault.put(validator_badge);
        }

        // This method deposits back the fund manager badge in the component in case it has
        // been previously withdrawn.
        // A single admin can perform this operation, authorization not needed.
        pub fn deposit_fund_manager_badge(
            &mut self,
            fund_manager_badge: FungibleBucket,
        ) {
            self.fund_manager_badge_vault.put(fund_manager_badge);
        }

        // Private method to validate an admin proof and get the numeric id of the used admin
        // badge.
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

        // Private method to remove expired entries from the authorization_vector
        fn purge_authorization_vector(&mut self) {
            let now = Clock::current_time_rounded_to_seconds().seconds_since_unix_epoch;

            self.authorization_vector.retain(|authorization| {
                authorization.timestamp + AUTHORIZATION_TIMEOUT > now
            });

            // TODO: save state space by creating a new vector if len == 0 and capacity is big?
        }

        // An admin can invoke this method to authorize another admin to perform a multisignature
        // operation.
        pub fn authorize_admin_operation(
            &mut self,
            admin_proof: Proof,
            allowed_admin_id: u8, // The id of badge of the admin to authorize
            authorized_operation: u8, // AuthorizedOperation
            protocol_name: Option<String>,
            percentage: Option<u8>,
            account_address: Option<Global<Account>>,
        ) {
            // Verify the proof and get the id out of it
            let allower_admin_id = self.get_admin_id(admin_proof);

            // Make sure the admin isn't cheating
            assert!(
                allower_admin_id != allowed_admin_id,
                "You can't authorize yourself",
            );

            // Remove expired entries from the authorization_vector
            self.purge_authorization_vector();

            // Avoid state explosion
            assert!(
                self.authorization_vector.len() < MAX_VECTOR_SIZE,
                "Authorization vector is getting too big",
            );

            // Make sure the admin isn't trying to authorize the same operation more than once
            assert!(
                self.authorization_vector
                    .iter()
                    .filter(|&authorization| {
                        authorization.allower_admin_id == allower_admin_id &&
                        authorization.allowed_admin_id == allowed_admin_id &&
                        authorization.authorized_operation == authorized_operation.into() &&
                        authorization.protocol_name == protocol_name &&
                        authorization.percentage == percentage &&
                        authorization.account_address == account_address
                    })
                    .next()
                    .is_none(),
                "Can't authorize the same operation more than once"
            );

            // Add the new authorization to the authorization_vector
            self.authorization_vector.push(
                Authorization {
                    timestamp: Clock::current_time_rounded_to_seconds().seconds_since_unix_epoch,
                    allower_admin_id: allower_admin_id,
                    allowed_admin_id: allowed_admin_id,
                    authorized_operation: authorized_operation.into(),
                    protocol_name: protocol_name,
                    percentage: percentage,
                    account_address: account_address,
                }
            );
        }

        // Private method to verify that a multisig operation has been authorized
        fn check_operation_authorization(
            &mut self,
            admin_id: u8,
            authorized_operation: AuthorizedOperation,
            protocol_name: Option<String>,
            percentage: Option<u8>,
            account_address: Option<Global<Account>>,
        ) {
            // Remove expired entries from the authorization_vector
            self.purge_authorization_vector();

            // Count the received authorizations for this operation
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

            // Make sure there are enough authorizers
            assert!(
                authorizers_number >= self.min_authorizers.into(),
                "Operation not authorized",
            );

            // Remove authorizations for this operation
            self.authorization_vector.retain(|authorization| {
                authorization.allowed_admin_id != admin_id ||
                authorization.authorized_operation != authorized_operation ||
                authorization.protocol_name != protocol_name ||
                authorization.account_address != account_address
            });
        }

        // Get the net and gross (withdrawal fee included) USD value of a fund unit
        pub fn fund_unit_value(&self) -> (Decimal, Decimal) {
            let gross_value = self.total_value / self.fund_unit_resource_manager.total_supply().unwrap();

            (
                (gross_value * (100 - self.withdrawal_fee)) / 100, // net value
                gross_value
            )
        }

        // This method returns the list of DeFi protocol positions and their value
        pub fn fund_details(&self) -> HashMap<String, Decimal> {
            let mut protocols_value = HashMap::new();

            for name in self.defi_protocols_list.iter() {
                let defi_protocol = self.defi_protocols.get(&name).unwrap();

                protocols_value.insert(name.clone(), defi_protocol.value);
            }

            protocols_value
        }

        // An admin can call this method to withdraw the Validator owner badge.
        // This operation must be authorized by other admins.
        pub fn withdraw_validator_badge(
            &mut self,
            admin_proof: Proof,
        ) -> NonFungibleBucket {

            // Check the admin proof and that there are enough authorizations for this operation
            self.check_operation_authorization(
                self.get_admin_id(admin_proof),
                AuthorizedOperation::WithdrawValidatorBadge,
                None,
                None,
                None,
            );

            self.validator_badge_vault.take_all()
        }

        // An admin can call this method to withdraw the fund manager badge.
        // This operation must be authorized by other admins.
        // Warning: the admin who performs this operation will be God!
        pub fn withdraw_fund_manager_badge(
            &mut self,
            admin_proof: Proof,
        ) -> FungibleBucket {

            // Check the admin proof and that there are enough authorizations for this operation
            self.check_operation_authorization(
                self.get_admin_id(admin_proof),
                AuthorizedOperation::WithdrawFundManagerBadge,
                None,
                None,
                None,
            );

            self.fund_manager_badge_vault.take_all()
        }

        // Increase by 1 the number of authorizations needed for a multisig operation (the increase
        // does not apply to this operation).
        // This operation must be authorized by other admins.
        pub fn increase_min_authorizers(
            &mut self,
            admin_proof: Proof,
        ) {

            // Check the admin proof and that there are enough authorizations for this operation
            self.check_operation_authorization(
                self.get_admin_id(admin_proof),
                AuthorizedOperation::IncreaseMinAuthorizers,
                None,
                None,
                None,
            );

            self.min_authorizers += 1;

            // Verify that enough admins exist or the multisig operations will be impossible!
            assert!(
                self.min_authorizers < self.number_of_admins,
                "The minimum number of authorizers must be smaller than the number of badges",
            );
        }

        // Decrease by 1 the number of authorizations needed for a multisig operation (the decrease
        // does not apply to this operation).
        // This operation must be authorized by other admins.
        // Warning: if min_authorizers goes to zero, the authorization system will be no longer
        // effective.
        pub fn decrease_min_authorizers(
            &mut self,
            admin_proof: Proof,
        ) {

            // Check the admin proof and that there are enough authorizations for this operation
            self.check_operation_authorization(
                self.get_admin_id(admin_proof),
                AuthorizedOperation::DecreaseMinAuthorizers,
                None,
                None,
                None,
            );

            self.min_authorizers -= 1;
        }

        // Mint a new admin badge and send it to the specified account.
        // Admins must authorize this operation ad agree on the account address to send the badge
        // to.
        pub fn mint_admin_badge(
            &mut self,
            admin_proof: Proof,
            mut new_admin_account: Global<Account>,
        ) {

            // Check the admin proof and that there are enough authorizations for this operation
            self.check_operation_authorization(
                self.get_admin_id(admin_proof),
                AuthorizedOperation::MintAdminBadge,
                None,
                None,
                Some(new_admin_account),
            );

            // Mint the new admin badge
            self.number_of_admins += 1;
            let admin_badge = self.admin_badge_resource_manager.mint_non_fungible(
                &NonFungibleLocalId::integer(self.number_of_admins.into()),
                Admin {},
            );

            // and send it to the specified account
            new_admin_account.try_deposit_or_abort(
                admin_badge.into(),
                None
            );
        }

        // The bot can invoke this method to start the unlock of the Validator's owner locked LSUs
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

        // The bot can invoke this method to complete the unlock of the Validator's owner LSUs and
        // start their unstake
        pub fn start_unstake(&mut self) {

            // Complete LSU unlock
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

            // Start LSU unstake and get the claim NFT
            let claim_nft_bucket = self.validator.unstake(lsu_bucket);

            // Emit the LsuUnstakeStartedEvent event
            Runtime::emit_event(
                LsuUnstakeStartedEvent {
                    lsu_amount: lsu_amount,
                    claim_nft_id: claim_nft_bucket.non_fungible_local_id(),
                }
            );
            
            // Store the received claim NFT
            self.claim_nft_vault.put(claim_nft_bucket);
        }

        // Private method to find the name of the DeFi protocol position to invest in
        fn find_where_to_deposit_to(&self) -> 
            String // The name of the DeFi protocol position
        {
            let mut smallest_percentage_diff: Decimal = dec!(101);
            let mut smallest_percentage_diff_name: Option<String> = None;

            // Go through the list of DeFi protocols positions to find the one whose currently
            // invested percentage of the fund is much below the desired percentage
            for name in self.defi_protocols_list.iter() {
                let defi_protocol = self.defi_protocols.get(&name).unwrap();

                let percentage = 100 * defi_protocol.value / self.total_value;
                let percentage_diff: Decimal = percentage - defi_protocol.desired_percentage;

                if percentage_diff < smallest_percentage_diff {
                    smallest_percentage_diff = percentage_diff;
                    smallest_percentage_diff_name = Some(name.to_string());
                }
            }

            // Return the name of the DeFi protocol position, fail if there are no DeFi protocol
            // positions
            smallest_percentage_diff_name.unwrap()
        }

        // The bot can invoke this method to complete the unstake of a batch of LSUs and invest the
        // resulting XRD in one of the existing DeFi protocol positions.
        // The method will also mint new fund units to distribute.
        // Some DeFi protocols may require data from the morpher oracle upon investment. Since the
        // bot doesn't know which protocol the fund will invest in, it is advisable to always send
        // all of the morpher oracle data to this method.
        pub fn finish_unstake(
            &mut self,
            claim_nft_id: String, // String representation of the claim NFT id to unstake
            morpher_data: HashMap<ResourceAddress, (String, String)>, 
        ) {
            // The bot must complete previous distributions before invoking this method
            assert!(
                self.fund_units_vault.amount() == Decimal::ZERO,
                "Previous distribution was not finished",
            );

            // Take the specified claim NFT out of the Vault
            let claim_nft_bucket = self.claim_nft_vault.take_non_fungible(
                &NonFungibleLocalId::String(StringNonFungibleLocalId::try_from(claim_nft_id).unwrap())
            );

            // Get the XRD out of it
            let mut bucket = self.validator.claim_xrd(claim_nft_bucket);

            // Send a percentage of the XRD to the buyback fund account
            let buyback_fund_bucket = bucket.take(
                (bucket.amount() * self.buyback_fund_percentage) / 100
            );
            self.buyback_fund_account.try_deposit_or_abort(
                buyback_fund_bucket.into(),
                None
            );

            // Compute the amount of new fund units to mint to keep their value constant and mint
            // them
            let xrd_amount = bucket.amount();
            let xrd_price = self.oracle_component.unwrap().get_price(
                XRD,
                morpher_data.clone()
            );
            let (_, fund_unit_gross_value) = self.fund_unit_value();
            self.fund_units_to_distribute = xrd_amount * xrd_price / fund_unit_gross_value;
            self.fund_units_vault.put(
                self.fund_unit_resource_manager.mint(self.fund_units_to_distribute + Decimal::ONE)
            );

            // Find th DeFi protocol position to invest in
            let defi_protocol_name = self.find_where_to_deposit_to();
            let mut defi_protocol = self.defi_protocols.get_mut(&defi_protocol_name).unwrap();

            // Extracts the eventual morpher oracle data that this protocol needs
            let (message, signature) = match defi_protocol.needed_morpher_data {
                Some(resource_address) => {
                    let morpher_data_needed_by_protocol = morpher_data.get(&resource_address).expect("Missing needed morpher data").clone();
                    
                    (Some(morpher_data_needed_by_protocol.0), Some(morpher_data_needed_by_protocol.1))
                },
                None => (None, None),
            };

            let coin_amount: Decimal;
            let other_coin_amount: Option<Decimal>;

            // If the DeFi protocol position accepts XRD send them an get the new total amount of
            // coin invested in it
            if defi_protocol.coin == XRD {
                (coin_amount, other_coin_amount) = self.fund_manager_badge_vault.authorize_with_amount(
                    1,
                    || defi_protocol.wrapper.deposit_coin(
                        bucket,
                        None,
                        message,
                        signature,
                    )
                );

            } else if defi_protocol.other_coin == Some(XRD) {
                let defi_protocol_coin = defi_protocol.coin;

                (coin_amount, other_coin_amount) = self.fund_manager_badge_vault.authorize_with_amount(
                    1,
                    || defi_protocol.wrapper.deposit_coin(
                        FungibleBucket::new(defi_protocol_coin),
                        Some(bucket),
                        message,
                        signature,
                    )
                );

            // If the DeFi protocol position needs a different coin, swap the XRD for that coin and
            // get the new total amount of coins invested in it
            } else {
                bucket = self.fund_manager_badge_vault.authorize_with_amount(
                    1,
                    || FungibleBucket(
                        self.dex.unwrap().swap(
                            bucket.into(),
                            defi_protocol.coin,
                            true
                        )
                    )
                );

                (coin_amount, other_coin_amount) = self.fund_manager_badge_vault.authorize_with_amount(
                    1,
                    || defi_protocol.wrapper.deposit_coin(
                        bucket,
                        None,
                        message,
                        signature,
                    )
                );
            }

            // Compute the new value of the DeFi protocol position
            let mut new_protocol_value = match defi_protocol.coin {
                XRD => coin_amount * xrd_price,
                _ => coin_amount * self.oracle_component.unwrap().get_price(
                    defi_protocol.coin,
                    morpher_data.clone()
                ),
            };
            if other_coin_amount.is_some() {
                if defi_protocol.other_coin.unwrap() == XRD {
                    new_protocol_value += other_coin_amount.unwrap() * xrd_price;
                } else {
                    new_protocol_value += other_coin_amount.unwrap() * self.oracle_component.unwrap().get_price(
                        defi_protocol.other_coin.unwrap(),
                        morpher_data
                    );
                }
            }

            // Update the values of the DeFi protocol position and the whole fund
            self.total_value += new_protocol_value - defi_protocol.value;
            defi_protocol.value = new_protocol_value;

            // Emit the LsuUnstakeCompletedEvent event
            Runtime::emit_event(
                LsuUnstakeCompletedEvent {
                    xrd_amount: xrd_amount,
                    defi_protocol_name: defi_protocol_name,
                    fund_units_to_distribute: self.fund_units_to_distribute,
                    protocol_value: new_protocol_value,
                    total_value: self.total_value,
                }
            );
        }

        // The bot can invoke this method to distribute the recently minted fund units.
        // The stakers IndexMap must contain the percentage of fund units to send to each account.
        // In order to avoid hitting transaction limits, this method can be called more than once
        // splitting the stakers IndexMap into multiple parts; more_stakers must always be true
        // except for the last invokation that terminates the distribution.
        pub fn fund_units_distribution (
            &mut self,
            stakers: IndexMap<Global<Account>, Decimal>,
            more_stakers: bool,
        ) {
            // Create a new IndexMap specifying the amount of fund units per recipient
            let mut distribution: IndexMap<Global<Account>, ResourceSpecifier> = IndexMap::new();
            for (account, share) in stakers.iter() {
                distribution.insert(
                    *account,
                    ResourceSpecifier::Fungible(*share * self.fund_units_to_distribute),
                );
            }

            // Send all of the fund units to the AccountLocker for the distribution and get back
            // any eventual remainings
            let remainings = self.account_locker.airdrop(
                distribution,
                self.fund_units_vault.take_all().into(),
                true,
            );

            // If the distribution is completed burn the remainings, else put them back in the
            // vault for the next distribution batch
            if more_stakers {
                self.fund_units_vault.put(FungibleBucket(remainings.unwrap()));
            } else if remainings.is_some() {
                remainings.unwrap().burn();
            }
        }

        // Register the wrapper for a DeFi protocol and assign a name to it.
        // If a wrapper with the same name is already registered, this method removes protocol
        // tokens from the old wrapper and deposits them in the new one.
        // This operation requires authorization from the other admins; they have to agree on the
        // wrapper name too.
        pub fn add_defi_protocol(
            &mut self,
            admin_proof: Proof,
            name: String, // The name to assign to the protocol wrapper
            coin: ResourceAddress, // The main coin managed by the new protocol
            protocol_token: ResourceAddress, // The token belonging to this DeFi protocol
            other_coin: Option<ResourceAddress>, // Eventual other coin managed by the protocol
            desired_percentage: u8, // The percentage of the fund value that we want to be
                                    // deposited in this protocol
            wrapper: DefiProtocolInterfaceScryptoStub, // Component address of the wrapper
            needed_morpher_data: Option<ResourceAddress>, // Whether the protocol needs data from
                                                          // the Morpher oracle
        ) {

            // Check that there are enough authorizations for this operation.
            self.check_operation_authorization(
                self.get_admin_id(admin_proof),
                AuthorizedOperation::AddDefiProtocol,
                Some(name.clone()),
                None,
                None,
            );

            let mut old_defi_protocol: Option<DefiProtocol> = None;

            // If there's no wrapper with the same name check that the list isn't getting too big
            // then add the name to the list
            if self.defi_protocols_list.iter().position(|n| *n == name).is_none() {
                assert!(
                    self.defi_protocols_list.len() < MAX_VECTOR_SIZE,
                    "Protocols list is getting too big",
                );

                self.defi_protocols_list.push(name.clone());

            // Otherwise get info about the existing wrapper position
            } else {
                old_defi_protocol= self.defi_protocols.remove(&name);
            }

            // Create the new protocol wrapper position
            let mut new_defi_protocol = DefiProtocol {
                value: Decimal::ZERO,
                desired_percentage: desired_percentage,
                wrapper: wrapper,
                coin: coin,
                protocol_token: protocol_token,
                other_coin: other_coin,
                needed_morpher_data: needed_morpher_data,
            };

            // Get liquidity from the old protocol wrapper position and deposit it in the new one
            if old_defi_protocol.is_some() {
                new_defi_protocol.value = old_defi_protocol.as_ref().unwrap().value;

                self.fund_manager_badge_vault.authorize_with_amount(
                    1,
                    || {
                        let (token_bucket, coin_bucket, other_coin_bucket) =
                            old_defi_protocol.unwrap().wrapper.withdraw_all();

                        new_defi_protocol.wrapper.deposit_all(
                            token_bucket,
                            coin_bucket,
                            other_coin_bucket,
                        );
                    }
                );
            }

            // Add the new DeFi protocol wrapper position in the KeyValueStore
            self.defi_protocols.insert(
                name,
                new_defi_protocol,
            );
        }

        // An admin can invoke this method to deposit coins in an existing DeFi protocol and
        // eventually mint new fund units corresponding to the value of the added coins.
        // There's no need for authorization; a single admin can invoke this method.
        pub fn deposit_coin(
            &mut self,
            defi_protocol_name: String, // The name of the protocol to deposit the coin in
            coin_bucket: FungibleBucket, // Bucket of coin to deposit
            other_coin_bucket: Option<FungibleBucket>, // Eventual additional bucket of coin

            // Eventual Morpher data required by the protocol or the oracle component
            morpher_data: HashMap<ResourceAddress, (String, String)>,

            mint_fund_units: bool, // Whether to mint new fund units whose value is equal to the
                                   // one of the deposited coins
        ) -> Option<FungibleBucket> // Fund units
        {

            // Compute the USD value of the first bucket of deposited coins
            let coin_price = self.oracle_component.unwrap().get_price(
                coin_bucket.resource_address(),
                morpher_data.clone(),
            );
            let mut buckets_value = coin_bucket.amount() * coin_price;

            // Get the current value of a fund unit
            let (_, fund_unit_gross_value) = self.fund_unit_value();

            // Get information about the DeFi protocol to deposit the buckets in
            let mut defi_protocol = self.defi_protocols.get_mut(&defi_protocol_name).expect("Protocol not found");

            // Extract the Morpher data needed by the DeFi protocol from the ones received
            let (message, signature) = match defi_protocol.needed_morpher_data {
                Some(resource_address) => {
                    let morpher_data_needed_by_protocol = morpher_data.get(&resource_address).expect("Missing needed morpher data").clone();
                    
                    (Some(morpher_data_needed_by_protocol.0), Some(morpher_data_needed_by_protocol.1))
                },
                None => (None, None),
            };

            // If the protocol handles two coins update buckets_value so that it contains the USD
            // value of both buckets
            let other_coin_price = match defi_protocol.other_coin {
                None => None,
                Some(other_coin) => {
                    let other_coin_price = self.oracle_component.unwrap().get_price(
                        other_coin,
                        morpher_data
                    );

                    if other_coin_bucket.is_some() {
                        buckets_value += other_coin_bucket.as_ref().unwrap().amount() * other_coin_price;
                    }

                    Some(other_coin_price)
                },
            };

            // Deposit the buckets in the DeFi protocol and get the total number of coins invested
            let (coin_amount, other_coin_amount) = self.fund_manager_badge_vault.authorize_with_amount(
                1,
                || defi_protocol.wrapper.deposit_coin(
                    coin_bucket,
                    other_coin_bucket,
                    message,
                    signature,
                )
            );

            // Compute the new value of the DeFi protocol position and update cached values 
            // information accordingly
            let mut new_protocol_value = coin_amount * coin_price;
            if other_coin_amount.is_some() {
                new_protocol_value += other_coin_amount.unwrap() * other_coin_price.unwrap();
            }
            self.total_value += new_protocol_value - defi_protocol.value;
            defi_protocol.value = new_protocol_value;

            // Emit the AdminDepositEvent event
            Runtime::emit_event(
                AdminDepositEvent {
                    defi_protocol_name: defi_protocol_name,
                    protocol_value: new_protocol_value,
                    total_value: self.total_value,
                }
            );

            // Mint the new fund units if required
            if mint_fund_units {
                Some(self.fund_unit_resource_manager.mint(buckets_value / fund_unit_gross_value))
            } else {
                None
            }
        }

        // An admin can invoke this method to deposit protocol tokens in an existing DeFi protocol
        // and eventually mint new fund units corresponding to the value of the added tokens.
        // There's no need for authorization; a single admin can invoke this method.
        pub fn deposit_protocol_token(
            &mut self,
            defi_protocol_name: String, // The name of the protocol to deposit the tokens in
            protocol_token_bucket: Bucket, // The bicket of tokens to deposit

            // Eventual Morpher data required by the protocol or the oracle component
            morpher_data: HashMap<ResourceAddress, (String, String)>,

            mint_fund_units: bool, // Whether to mint new fund units or not
        ) -> Option<FungibleBucket> // Fund units
        {
            // Get information about the DeFi protocol to deposit the bucket in
            let mut defi_protocol = self.defi_protocols.get_mut(&defi_protocol_name).unwrap();

            // Get the current amount of coins invested in this DeFi protocol position
            let (old_coin_amount, old_other_coin_amount) = defi_protocol.wrapper.get_coin_amounts();

            // Deposit the tokens and get the updated number of coins
            let (coin_amount, other_coin_amount) = self.fund_manager_badge_vault.authorize_with_amount(
                1,
                || defi_protocol.wrapper.deposit_all(protocol_token_bucket, None, None)
            );

            // Compute the protocol value immediately before and after the deposit
            let coin_price = self.oracle_component.unwrap().get_price(
                defi_protocol.coin,
                morpher_data.clone()
            );
            let mut protocol_value = coin_price * old_coin_amount;
            let mut new_protocol_value = coin_price * coin_amount;
            if defi_protocol.other_coin.is_some() {
                let other_coin_price = self.oracle_component.unwrap().get_price(
                    defi_protocol.other_coin.unwrap(),
                    morpher_data
                );

                protocol_value += other_coin_price * old_other_coin_amount.unwrap();
                new_protocol_value += other_coin_price * other_coin_amount.unwrap();
            }

            // Update cached value information
            self.total_value += new_protocol_value - defi_protocol.value;
            defi_protocol.value = new_protocol_value;

            // Emit the AdminDepositEvent event
            Runtime::emit_event(
                AdminDepositEvent {
                    defi_protocol_name: defi_protocol_name,
                    protocol_value: new_protocol_value,
                    total_value: self.total_value,
                }
            );

            // If requested, mint new fund units corresponding to the increase value because of
            // the deposit
            match mint_fund_units {
                false => None,
                true => {
                    drop(defi_protocol);

                    let (_, fund_unit_gross_value) = self.fund_unit_value();

                    Some(
                        self.fund_unit_resource_manager.mint(
                            (new_protocol_value - protocol_value) / fund_unit_gross_value
                        )
                    )
                },
            }
        }

        // Removes a DeFi protocol wrapper and returns the badge to control the Account used by the
        // wrapper
        // An admin can perform this operation only when authorized by the other admins
        pub fn remove_defi_protocol(
            &mut self,
            admin_proof: Proof,
            name: String,
        ) -> NonFungibleBucket {

            // Check other admins' authorizations
            self.check_operation_authorization(
                self.get_admin_id(admin_proof),
                AuthorizedOperation::RemoveDefiProtocol,
                Some(name.clone()),
                None,
                None,
            );

            // Remove the protocol position from the Vector and the KeyValueStore
            self.defi_protocols_list.retain(|n| { *n != name });
            let mut defi_protocol = self.defi_protocols.remove(&name)
                .expect("Protocol not found");

            // Update the total fund value and emit a RemovedProtocolEvent event containing this
            // information
            self.total_value -= defi_protocol.value;
            Runtime::emit_event(
                RemovedProtocolEvent {
                    defi_protocol_name: name,
                    total_value: self.total_value,
                }
            );

            // Get the Account badge
            self.fund_manager_badge_vault.authorize_with_amount(
                1,
                || defi_protocol.wrapper.withdraw_account_badge()
            )
        }

        // Updates the cached value of the specified DeFi protocols by asking amounts to the
        // protocols themselves and prices to the oracle component
        pub fn update_defi_protocols_value(
            &mut self,
            defi_protocols: IndexSet<String>,
            morpher_data: HashMap<ResourceAddress, (String, String)>,
        ) {
            // Variable to store the total fund value change
            let mut value_change = Decimal::ZERO;

            // Coin prices cache
            let mut prices: HashMap<ResourceAddress, Decimal> = HashMap::new();

            // For each protocol
            for name in defi_protocols.iter() {
                let mut defi_protocol = self.defi_protocols.get_mut(&name).unwrap();

                // Get the coin amounts
                let (coin_amount, other_coin_amount) = defi_protocol.wrapper.get_coin_amounts();

                // Calculate the value by multiplicating the amounts by the prices
                // The prices can be taken from the oracle component of from the cache
                let mut new_protocol_value = match prices.get(&defi_protocol.coin) {
                    Some(coin_price) => *coin_price * coin_amount,
                    None => {
                        let coin_price = self.oracle_component.unwrap().get_price(
                            defi_protocol.coin,
                            morpher_data.clone()
                        );

                        prices.insert(defi_protocol.coin, coin_price);

                        coin_price * coin_amount
                    },
                };
                if defi_protocol.other_coin.is_some() {
                    new_protocol_value += match prices.get(&defi_protocol.other_coin.unwrap()) {
                        Some(other_coin_price) => *other_coin_price * other_coin_amount.unwrap(),
                        None => {
                            let other_coin_price = self.oracle_component.unwrap().get_price(
                                defi_protocol.other_coin.unwrap(),
                                morpher_data.clone()
                            );

                            prices.insert(defi_protocol.other_coin.unwrap(), other_coin_price);

                            other_coin_price * other_coin_amount.unwrap()
                        },
                    };
                }

                // Update information about protocol value
                value_change += new_protocol_value - defi_protocol.value;
                defi_protocol.value = new_protocol_value;

                // Emit an event for each updated protocol (only the last one will report the new
                // correct total_value)
                Runtime::emit_event(
                    ProtocolValueUpdateEvent {
                        defi_protocol_name: name.clone(),
                        protocol_value: new_protocol_value,
                        total_value: self.total_value + value_change,
                    }
                );
            }

            // Update information about fund value
            self.total_value += value_change;
        }

        // Set the desired percentage for one of more DeFi protocol positions.
        // The method doesn't actually move any funds; it only influences the future deposit and
        // withdraws.
        // FundManager doesn't check that the sum of the percentages is 100; each percentage 
        // should be considered as a share of the sum of the percentages.
        pub fn set_defi_protocols_percentage(
            &mut self,
            defi_protocols_desired_percentage: HashMap<String, u8>,
        ) {

            // For each position to update
            for (name, percentage) in defi_protocols_desired_percentage.iter() {

                // Check that the percentage is acceptable
                assert!(
                    *percentage <= 100,
                    "Pergentage out of the 0-100 range"
                );

                // Update the DeFi protocol information
                let mut defi_protocol = self.defi_protocols.get_mut(&name).expect("Not found");
                defi_protocol.desired_percentage = *percentage;
            }
        }

        // Private method to select the DeFi protocol position to withdraw the given USD value from
        fn find_where_to_withdraw_from(
            &self,
            amount: Decimal,    // USD value to withdraw
        ) -> (
            String,             // Name of the DeFi protocol position
            Decimal,            // USD value that can actually be withdrawn
        ) {
            // Create a list of DeFi protocols whose value is not less than amount
            let mut defi_protocol_candidates: Vec<String> = vec![];
            for name in self.defi_protocols_list.iter() {
                let value = self.defi_protocols.get(&name).unwrap().value;

                if value >= amount {
                    defi_protocol_candidates.push(name.to_string());
                }
            }

            // If the list is empty return the DeFi protocol with the largest USD value an its USD
            // value
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

            // Search in the list of candidate protocols the one whose value percentage is bigger
            // than the desidered one
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

        // Swap fund units for any coin managed by a DeFi protocol or for a specific coin.
        // This method withdraws from a single DeFi protocol position; if the full value can't be
        // withdrawn from a single position, some fund units will be returned
        pub fn withdraw(
            &mut self,
            mut fund_units_bucket: FungibleBucket,  // Bucket of funds to swap
            swap_to: Option<ResourceAddress>,       // If set, the returned coins will be swapped
                                                    // to this coin
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


            // Get the value of a fund unit
            let (fund_unit_net_value, fund_unit_gross_value) = self.fund_unit_value();

            // Find the DeFi protocol position to withdraw from
            let fund_unit_amount = fund_units_bucket.amount();
            let (defi_protocol_name, withdrawable_value) = self.find_where_to_withdraw_from(
                fund_unit_amount * fund_unit_net_value
            );
            let mut defi_protocol = self.defi_protocols.get_mut(&defi_protocol_name).unwrap();

            // Get the price of the main coin managed from the choosen protocol
            let coin_price = self.oracle_component.unwrap().get_price(
                defi_protocol.coin,
                morpher_data.clone()
            );

            // Compute the relative price of the two coins of the protocol
            let (other_coin_to_coin_price_ratio, other_coin_price) = match defi_protocol.other_coin {
                Some(other_coin) => {
                    let other_coin_price = self.oracle_component.unwrap().get_price(
                        other_coin,
                        morpher_data
                    );

                    (Some(other_coin_price / coin_price), Some(other_coin_price))
                },
                None => (None, None),
            };

            // Withdraw coins from the protocol position
            let (mut coin_bucket, mut other_coin_bucket, coin_amount, other_coin_amount) = 
                self.fund_manager_badge_vault.authorize_with_amount(
                    1,
                    || defi_protocol.wrapper.withdraw_coin(
                        withdrawable_value / coin_price,
                        other_coin_to_coin_price_ratio,
                    )
                );

            // Compute the total value of the returned buckets
            let mut coin_bucket_value = coin_bucket.amount() * coin_price;
            if other_coin_bucket.is_some() {
                coin_bucket_value += other_coin_bucket.as_ref().unwrap().amount() * other_coin_price.unwrap();
            }

            // Update the protocol and total fund value
            let mut new_protocol_value = coin_amount * coin_price;
            if defi_protocol.other_coin.is_some() {
                new_protocol_value += other_coin_price.unwrap() * other_coin_amount.unwrap();
            }
            self.total_value += new_protocol_value - defi_protocol.value;
            defi_protocol.value = new_protocol_value;

            // If swap_to was specified, swap both buckets for the specified coin (put everithing
            // in coin_bucket, leave other_coin_bucket empty)
            if swap_to.is_some() {
                if swap_to.unwrap() != defi_protocol.coin {
                    coin_bucket = self.fund_manager_badge_vault.authorize_with_amount(
                        1,
                        || FungibleBucket(
                            self.dex.unwrap().swap(
                                coin_bucket.into(),
                                swap_to.unwrap(),
                                false
                            )
                        )
                    );
                }
                if other_coin_bucket.is_some() && swap_to.unwrap() != defi_protocol.other_coin.unwrap() {
                    coin_bucket.put(
                        self.fund_manager_badge_vault.authorize_with_amount(
                            1,
                            || FungibleBucket(
                                self.dex.unwrap().swap(
                                    other_coin_bucket.unwrap().into(),
                                    swap_to.unwrap(),
                                    false
                                )
                            )
                        )
                    );

                    other_coin_bucket = None;
                }
            }

            // Compute the amount of fund units to burn
            let mut fund_units_to_burn = coin_bucket_value / fund_unit_gross_value;
            if fund_units_to_burn > fund_unit_amount {
                assert!(
                    fund_units_to_burn < fund_unit_amount * (1 + ACCEPTABLE_VALUE_DIFFERENCE),
                    "Too much value withdrawn"
                );
                fund_units_to_burn = fund_unit_amount;
            } else if fund_units_to_burn > fund_unit_amount * (1 - ACCEPTABLE_VALUE_DIFFERENCE) {
                fund_units_to_burn = fund_unit_amount;
            }

            // Emit the WithdrawFromFundEvent event
            Runtime::emit_event(
                WithdrawFromFundEvent {
                    fund_unit_amount: fund_units_to_burn,
                    defi_protocol_name: defi_protocol_name,
                    protocol_value: new_protocol_value,
                    total_value: self.total_value,
                }
            );

            // Burn the fund units and return all of the buckets to the caller
            if fund_units_to_burn < fund_unit_amount {
                fund_units_bucket.take(fund_units_to_burn).burn();

                (coin_bucket, other_coin_bucket, Some(fund_units_bucket))
            } else {
                fund_units_bucket.burn();

                (coin_bucket, other_coin_bucket, None)
            }
        }

        // Set the dex wrapper component. The component mus implement the DexInterfaceScryptoStub
        // interface
        // An admin needs authorization from the other admins to call this method.
        pub fn set_dex_component(
            &mut self,
            admin_proof: Proof,
            dex: DexInterfaceScryptoStub,
        ) {

            // Check admins' authorization
            self.check_operation_authorization(
                self.get_admin_id(admin_proof),
                AuthorizedOperation::SetDexComponent,
                None,
                None,
                None,
            );

            // Update the dex warapper component
            self.dex = Some(dex);
        }

        // Set the withdrawal fee to pay when swapping fund units for coins.
        // Admins must agree on the percentage when performing this operation.
        pub fn set_withdrawal_fee(
            &mut self,
            admin_proof: Proof,
            percentage: u8,
        ) {

            // Check admins' authorization
            self.check_operation_authorization(
                self.get_admin_id(admin_proof),
                AuthorizedOperation::SetWithdrawalFee,
                None,
                Some(percentage),
                None,
            );

            // Make sure that percentage makes sense
            assert!(
                percentage < 100,
                "Fee must be a number from 0 to 100 (excluded)"
            );

            // Update the fee
            self.withdrawal_fee = percentage;
        }

        // Set the oracle component wrapper to use. The component must implement the
        // OracleInterfaceScryptoStub interface.
        // An admin needs authorization from the other admins to call this method.
        pub fn set_oracle_component(
            &mut self,
            admin_proof: Proof,
            component: OracleInterfaceScryptoStub,
        ) {

            // Check other admins' authorization
            self.check_operation_authorization(
                self.get_admin_id(admin_proof),
                AuthorizedOperation::SetOracleComponent,
                None,
                None,
                None,
            );

            // Update the oracle component
            self.oracle_component = Some(component);
        }

        // Set percentage and account address that manages the buyback fund.
        // Admins must agree both on the account and the percentage when performing this operation
        pub fn set_buyback_fund(
            &mut self,
            admin_proof: Proof,
            percentage: u8,
            account: Global<Account>,
        ) {
            // Verify autorization
            self.check_operation_authorization(
                self.get_admin_id(admin_proof),
                AuthorizedOperation::SetBuybackFund,
                None,
                Some(percentage),
                Some(account),
            );

            // Make sure that percentage makes sense
            assert!(
                percentage < 100,
                "Fee must be a number from 0 to 100"
            );

            // Update percentage and account
            self.buyback_fund_percentage = percentage;
            self.buyback_fund_account = account;
        }

        // Register/ungegister the Validator
        // This operation can be performed by a single admin without other admins' authorization to
        // quickly react to a node down
        pub fn register_validator(
            &mut self,
            register: bool // true -> register, false -> unregister
        ) {
            // Use the validator owner badge to register or unregister the Validator
            self.validator_badge_vault.authorize_with_non_fungibles(
                &self.validator_badge_vault.non_fungible_local_ids(1),
                || if register {
                    self.validator.register();
                } else {
                    self.validator.unregister();
                }
            )
        }
       
        // Signal Radix network protocol update readiness for the Validator; this operation is
        // sometimes required during a Validator node update
        // This operation can be performed by a single admin without other admins' authorization
        pub fn signal_protocol_update_readiness(
            &mut self,
            vote: String, // Update's name
        ) {
            // Use the validator owner badge to signal readiness
            self.validator_badge_vault.authorize_with_non_fungibles(
                &self.validator_badge_vault.non_fungible_local_ids(1),
                || self.validator.signal_protocol_update_readiness(vote)
            );
        }

        // Move the Validator from one node to another
        // This operation can be performed by a single admin without other admins' authorization to
        // quickly react to a node down or when doing a node update
        pub fn update_node_key(
            &mut self,
            key: String, // String representation of the new node public key
        ) {
            // Use the validator owner badge to set the node key
            self.validator_badge_vault.authorize_with_non_fungibles(
                &self.validator_badge_vault.non_fungible_local_ids(1),
                || self.validator.update_key(
                    Secp256k1PublicKey::from_str(&key).expect("Invalid key")
                )
            );
        }
    }
}
