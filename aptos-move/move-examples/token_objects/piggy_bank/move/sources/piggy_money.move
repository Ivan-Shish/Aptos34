module token_objects::piggy_money {
    use std::error;
    use std::option;
    use std::string::{Self, String};
    use std::signer;

    use aptos_std::string_utils;

    use aptos_framework::object::{Self, Object};
    use aptos_token_objects::collection;
    use aptos_token_objects::token;
    use aptos_token_objects::property_map;
    use aptos_framework::fungible_asset::{Self, Metadata};
    use aptos_framework::primary_fungible_store;

    /// The token does not exist
    const ETOKEN_DOES_NOT_EXIST: u64 = 1;
    /// The provided signer is not the creator
    const ENOT_CREATOR: u64 = 2;
    /// Attempted to mutate an immutable field
    const EFIELD_NOT_MUTABLE: u64 = 3;
    /// Attempted to burn a non-burnable token
    const ETOKEN_NOT_BURNABLE: u64 = 4;
    /// Attempted to mutate a property map that is not mutable
    const EPROPERTIES_NOT_MUTABLE: u64 = 5;
    // The collection does not exist
    const ECOLLECTION_DOES_NOT_EXIST: u64 = 6;

    /// The piggy money collection name
    const COLLECTION_NAME: vector<u8> = b"Piggy Money Collection Name";
    /// The piggy money collection description
    const COLLECTION_DESCRIPTION: vector<u8> = b"Piggy Money Collection Description";
    /// The piggy money collection URI
    const COLLECTION_URI: vector<u8> = b"Piggy Money Collection URI";

    /// The piggy money cent token name
    const CENT_TOKEN_NAME: vector<u8> = b"Piggy Cent Token";
    /// The piggy money dime token name
    const DIME_TOKEN_NAME: vector<u8> = b"Piggy Dime Token";

    // Piggy Money Token
    struct MoneyToken has key {
        /// Used to burn.
        burn_ref: token::BurnRef,
        /// Used to mutate properties
        property_mutator_ref: property_map::MutatorRef,
        /// Used to mint fungible assets.
        fungible_asset_mint_ref: fungible_asset::MintRef,
        /// Used to transfer fungible assets.
        fungible_asset_transfer_ref: fungible_asset::TransferRef,
        /// Used to burn fungible assets.
        fungible_asset_burn_ref: fungible_asset::BurnRef,
    }

    /// Value of the token in cent
    struct ValueInCent has key {
        value_in_cent: u64,
    }

    /// Initializes the module, creating a collection and creating two fungible assets such as CENT, and DIME.
    fun init_module(sender: &signer) {
        create_money_collection(sender);
        create_money_token_as_fungible_asset(
            sender,
            string::utf8(b"Piggy Money Cent Token Description"),
            string::utf8(CENT_TOKEN_NAME),
            string::utf8(b"https://raw.githubusercontent.com/junkil-park/metadata/main/piggy_money/Cent"),
            string::utf8(b"Piggy Cent"),
            string::utf8(b"CENT"),
            string::utf8(b"https://raw.githubusercontent.com/junkil-park/metadata/main/piggy_money/Cent.png"),
            1,
        );
        create_money_token_as_fungible_asset(
            sender,
            string::utf8(b"Piggy Money Dime Token Description"),
            string::utf8(DIME_TOKEN_NAME),
            string::utf8(b"https://raw.githubusercontent.com/junkil-park/metadata/main/piggy_money/Dime"),
            string::utf8(b"Piggy Dime"),
            string::utf8(b"DIME"),
            string::utf8(b"https://raw.githubusercontent.com/junkil-park/metadata/main/piggy_money/Dime.png"),
            1,
        );
    }

    /// Creates the piggy money collection.
    fun create_money_collection(creator: &signer) {
        // Constructs the strings from the bytes.
        let description = string::utf8(COLLECTION_DESCRIPTION);
        let name = string::utf8(COLLECTION_NAME);
        let uri = string::utf8(COLLECTION_URI);

        // Creates the collection with unlimited supply and without establishing any royalty configuration.
        collection::create_unlimited_collection(
            creator,
            description,
            name,
            option::none(),
            uri,
        );
    }

    /// Creates the piggy money token as fungible asset.
    fun create_money_token_as_fungible_asset(
        creator: &signer,
        description: String,
        name: String,
        uri: String,
        fungible_asset_name: String,
        fungible_asset_symbol: String,
        icon_uri: String,
        value_in_cent: u64,
    ) {
        // The collection name is used to locate the collection object and to create a new token object.
        let collection = string::utf8(COLLECTION_NAME);
        // Creates the piggy money token, and get the constructor ref of the token. The constructor ref
        // is used to generate the refs of the token.
        let constructor_ref = token::create_named_token(
            creator,
            collection,
            description,
            name,
            option::none(),
            uri,
        );

        // Generates the object signer and the refs. The object signer is used to publish a resource
        // (e.g., ValueInCent) under the token object address. The refs are used to manage the token.
        let object_signer = object::generate_signer(&constructor_ref);
        let burn_ref = token::generate_burn_ref(&constructor_ref);
        let property_mutator_ref = property_map::generate_mutator_ref(&constructor_ref);

        // Initializes the value with the given value in cent.
        move_to(&object_signer, ValueInCent { value_in_cent });

        // Initialize the property map.
        let properties = property_map::prepare_input(vector[], vector[], vector[]);
        property_map::init(&constructor_ref, properties);
        property_map::add_typed(
            &property_mutator_ref,
            string::utf8(b"value_in_cent"),
            string_utils::to_string(&value_in_cent)
        );

        /// Creates the fungible asset.
        primary_fungible_store::create_primary_store_enabled_fungible_asset(
            &constructor_ref,
            option::none(),
            fungible_asset_name,
            fungible_asset_symbol,
            0,
            icon_uri,
        );
        let fungible_asset_mint_ref = fungible_asset::generate_mint_ref(&constructor_ref);
        let fungible_asset_burn_ref = fungible_asset::generate_burn_ref(&constructor_ref);
        let fungible_asset_transfer_ref = fungible_asset::generate_transfer_ref(&constructor_ref);

        // Publishes the MoneyToken resource with the refs.
        let money_token = MoneyToken {
            burn_ref,
            property_mutator_ref,
            fungible_asset_mint_ref,
            fungible_asset_transfer_ref,
            fungible_asset_burn_ref,
        };
        move_to(&object_signer, money_token);
    }

    /// Mints the given amount of the CENT token to the given receiver.
    public entry fun mint_cent(creator: &signer, receiver: address, amount: u64) acquires MoneyToken {
        let cent_token_address = token::create_token_address(&signer::address_of(creator), &string::utf8(COLLECTION_NAME), &string::utf8(b"Piggy Cent Token"));
        let cent_token = object::address_to_object<MoneyToken>(cent_token_address);
        mint_internal(creator, cent_token, receiver, amount);
    }

    /// Mints the given amount of the DIME token to the given receiver.
    public entry fun mint_dime(creator: &signer, receiver: address, amount: u64) acquires MoneyToken {
        let dime_token_address = token::create_token_address(&signer::address_of(creator), &string::utf8(COLLECTION_NAME), &string::utf8(b"Piggy Dime Token"));
        let dime_token = object::address_to_object<MoneyToken>(dime_token_address);
        mint_internal(creator, dime_token, receiver, amount);
    }

    /// The internal mint function.
    fun mint_internal(creator: &signer, token: Object<MoneyToken>, receiver: address, amount: u64) acquires MoneyToken {
        let money_token = authorized_borrow<MoneyToken>(creator, &token);
        let fungible_asset_mint_ref = &money_token.fungible_asset_mint_ref;
        let fa = fungible_asset::mint(fungible_asset_mint_ref, amount);
        primary_fungible_store::deposit(receiver, fa);
    }

    /// Transfers the given amount of the CENT token from the given sender to the given receiver.
    public entry fun transfer_cent(from: &signer, to: address, amount: u64) {
        let cent_token_address = token::create_token_address(&signer::address_of(from), &string::utf8(COLLECTION_NAME), &string::utf8(b"Piggy Cent Token"));
        let metadata = object::address_to_object<Metadata>(cent_token_address);
        primary_fungible_store::transfer(from, metadata, to, amount);
    }

    /// Transfers the given amount of the DIME token from the given sender to the given receiver.
    public entry fun transfer_dime(from: &signer, to: address, amount: u64) {
        let dime_token_address = token::create_token_address(&signer::address_of(from), &string::utf8(COLLECTION_NAME), &string::utf8(b"Piggy Dime Token"));
        let metadata = object::address_to_object<Metadata>(dime_token_address);
        primary_fungible_store::transfer(from, metadata, to, amount);
    }

    inline fun authorized_borrow<T: key>(creator: &signer, token: &Object<T>): &MoneyToken {
        let token_address = object::object_address(token);
        assert!(
            exists<MoneyToken>(token_address),
            error::not_found(ETOKEN_DOES_NOT_EXIST),
        );

        assert!(
            token::creator(*token) == signer::address_of(creator),
            error::permission_denied(ENOT_CREATOR),
        );
        borrow_global<MoneyToken>(token_address)
    }

    #[test(creator = @token_objects, user1 = @0x456, user2_addr = @0x789)]
    fun test_create_and_transfer(creator: &signer, user1: &signer, user2_addr: address) acquires MoneyToken {
        // ------------------------------------------------------------------------
        // Creator creates the collection, and mints CENT and DIME tokens in it.
        // ------------------------------------------------------------------------
        init_module(creator);

        // ----------------------------------------------
        // Creator mints and sends 100 CENTs to User1.
        // ----------------------------------------------
        let user1_addr = signer::address_of(user1);
        mint_cent(creator, user1_addr, 100);

        let cent_token_address = token::create_token_address(&signer::address_of(creator), &string::utf8(COLLECTION_NAME), &string::utf8(b"Piggy Cent Token"));
        let cent_metadata = object::address_to_object<Metadata>(cent_token_address);
        // Assert that the user1 has 100 CENTs.
        assert!(primary_fungible_store::balance(user1_addr, cent_metadata) == 100, 0);

        // ----------------------------------------------
        // Creator mints and sends 200 DIMEs to User2.
        // ----------------------------------------------
        mint_dime(creator, user2_addr, 200);
        let dime_token_address = token::create_token_address(&signer::address_of(creator), &string::utf8(COLLECTION_NAME), &string::utf8(b"Piggy Dime Token"));
        let dime_metadata = object::address_to_object<Metadata>(dime_token_address);
        // Assert that the user2 has 200 DIMEs.
        assert!(primary_fungible_store::balance(user2_addr, dime_metadata) == 200, 0);

        // ---------------------------------
        // User1 sends 10 CENTs to User2.
        // ---------------------------------
        primary_fungible_store::transfer(user1, cent_metadata, user2_addr, 10);
        // Assert that the user1 has 90 CENTs.
        assert!(primary_fungible_store::balance(user1_addr, cent_metadata) == 90, 0);
        // Assert that the user2 has 10 CENTs.
        assert!(primary_fungible_store::balance(user2_addr, cent_metadata) == 10, 0);

        // ----------------------------------------------
        // Creator sends 20 DIMEs from User2 to User1.
        // ----------------------------------------------
        let dime_token: Object<MoneyToken> = object::convert(dime_metadata);
        let piggy_money_ref = authorized_borrow<MoneyToken>(creator, &dime_token);
        fungible_asset::transfer_with_ref(
            &piggy_money_ref.fungible_asset_transfer_ref,
            primary_fungible_store::ensure_primary_store_exists(user2_addr, dime_metadata),
            primary_fungible_store::ensure_primary_store_exists(user1_addr, dime_metadata),
            20
        );
        // Assert that the user1 has 20 DIMEs.
        assert!(primary_fungible_store::balance(user1_addr, dime_metadata) == 20, 0);
        // Assert that the user2 has 180 DIMEs.
        assert!(primary_fungible_store::balance(user2_addr, dime_metadata) == 180, 0);
    }
}
