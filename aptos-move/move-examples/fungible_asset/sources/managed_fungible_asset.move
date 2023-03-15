module fungible_asset::managed_fungible_asset {
    use std::option;
    use aptos_framework::object::{Object, object_address, ExtendRef, is_owner};
    use fungible_asset::fungible_source::{MintCap, FreezeCap, BurnCap, unfreeze_with_cap, burn_with_cap, mint_with_cap, init_fungible_source, freeze_with_cap};
    use std::option::Option;
    use aptos_framework::object;
    use std::error;
    use std::signer::address_of;
    use fungible_asset::fungible_source;
    #[test_only]
    use fungible_asset::fungible_asset::{create_test_token, balance_of, is_frozen};
    #[test_only]
    use std::signer;
    #[test_only]
    use aptos_framework::object::generate_extend_ref;

    /// Mint capability exists or does not exist.
    const EMINT_CAP: u64 = 1;
    /// Freeze capability exists does not exist.
    const EFREEZE_CAP: u64 = 2;
    /// Burn capability exists or does not exist.
    const EBURN_CAP: u64 = 3;
    /// Not the owner.
    const ENOT_OWNER: u64 = 4;
    /// Caps existence errors.
    const EMANAGED_FUNGIBLE_ASSET_CAPS: u64 = 5;

    struct Caps has key {
        mint: Option<MintCap>,
        freeze: Option<FreezeCap>,
        burn: Option<BurnCap>,
    }

    public fun initialize_managing_capabilities(
        extend_ref: &ExtendRef,
        maximum_supply: u64,
    ) {
        let (mint_cap, freeze_cap, burn_cap) = init_fungible_source(extend_ref, maximum_supply);
        let asset_object_signer = object::generate_signer_for_extending(extend_ref);
        move_to(
            &asset_object_signer,
            Caps { mint: option::some(mint_cap), freeze: option::some(freeze_cap), burn: option::some(burn_cap) }
        )
    }

    /// Mint fungible tokens as the owner of the base asset.
    public fun mint_by_asset_owner<T: key>(
        asset_owner: &signer,
        asset: &Object<T>,
        amount: u64,
        to: address
    ) acquires Caps {
        assert_owner(asset_owner, asset);
        let mint_cap = borrow_mint_from_caps(asset);
        mint_with_cap(mint_cap, asset, amount, to);
    }

    /// Burn fungible tokens as the owner of the base asset.
    public fun burn_by_asset_owner<T: key>(
        asset_owner: &signer,
        asset: &Object<T>,
        amount: u64,
        from: address
    ) acquires Caps {
        assert_owner(asset_owner, asset);
        let burn_cap = borrow_burn_from_caps(asset);
        burn_with_cap(burn_cap, asset, amount, from);
    }

    /// Freeze an owner of fungible asset.
    public fun freeze_by_asset_owner<T: key>(
        asset_owner: &signer,
        asset: &Object<T>,
        account: address,
    ) acquires Caps {
        assert_owner(asset_owner, asset);
        let freeze_cap = borrow_freeze_from_caps(asset);
        freeze_with_cap(freeze_cap, account, asset);
    }

    /// Unfreeze an owner of fungible asset.
    public fun unfreeze_by_asset_owner<T: key>(
        asset_owner: &signer,
        asset: &Object<T>,
        fungible_asset_owner: address
    ) acquires Caps {
        assert_owner(asset_owner, asset);
        let freeze_cap = borrow_freeze_from_caps(asset);
        unfreeze_with_cap(freeze_cap, fungible_asset_owner, asset);
    }

    public fun owner_can_mint<T: key>(asset: &Object<T>): bool acquires Caps {
        option::is_some(&borrow_caps(asset).mint)
    }

    public fun owner_can_freeze<T: key>(asset: &Object<T>): bool acquires Caps {
        option::is_some(&borrow_caps(asset).freeze)
    }

    public fun owner_can_burn<T: key>(asset: &Object<T>): bool acquires Caps {
        option::is_some(&borrow_caps(asset).burn)
    }

    public fun destroy_mint_cap<T: key>(asset_owner: &signer, asset: &Object<T>) acquires Caps {
        let mint_cap = &mut borrow_caps_mut(asset_owner, asset).mint;
        assert!(option::is_some(mint_cap), error::not_found(EMINT_CAP));
        fungible_source::destory_mint_cap(option::extract(mint_cap));
    }

    public fun destroy_freeze_cap<T: key>(asset_owner: &signer, asset: &Object<T>) acquires Caps {
        let freeze_cap = &mut borrow_caps_mut(asset_owner, asset).freeze;
        assert!(option::is_some(freeze_cap), error::not_found(EFREEZE_CAP));
        fungible_source::destory_freeze_cap(option::extract(freeze_cap));
    }

    public fun destroy_burn_cap<T: key>(asset_owner: &signer, asset: &Object<T>) acquires Caps {
        let burn_cap = &mut borrow_caps_mut(asset_owner, asset).burn;
        assert!(option::is_some(burn_cap), error::not_found(EFREEZE_CAP));
        fungible_source::destory_burn_cap(option::extract(burn_cap));
    }

    inline fun borrow_mint_from_caps<T: key>(
        asset: &Object<T>
    ): &MintCap acquires Caps {
        let mint_cap = &borrow_caps(asset).mint;
        assert!(option::is_some(mint_cap), error::not_found(EMINT_CAP));
        option::borrow(mint_cap)
    }

    inline fun borrow_freeze_from_caps<T: key>(
        asset: &Object<T>
    ): &FreezeCap acquires Caps {
        let freeze_cap = &borrow_caps(asset).freeze;
        assert!(option::is_some(freeze_cap), error::not_found(EFREEZE_CAP));
        option::borrow(freeze_cap)
    }

    inline fun borrow_burn_from_caps<T: key>(
        asset: &Object<T>
    ): &BurnCap acquires Caps {
        let burn_cap = &borrow_caps(asset).burn;
        assert!(option::is_some(burn_cap), error::not_found(EBURN_CAP));
        option::borrow(burn_cap)
    }

    inline fun borrow_caps<T: key>(
        asset: &Object<T>
    ): &Caps acquires Caps {
        verify(asset);
        borrow_global<Caps>(object_address(asset))
    }

    inline fun borrow_caps_mut<T: key>(
        owner: &signer,
        asset: &Object<T>
    ): &mut Caps acquires Caps {
        assert_owner(owner, asset);
        verify(asset);
        borrow_global_mut<Caps>(object_address(asset))
    }

    inline fun assert_owner<T: key>(owner: &signer, asset: &Object<T>) {
        assert!(is_owner(*asset, address_of(owner)), error::permission_denied(ENOT_OWNER));
    }

    inline fun verify<T: key>(caps: &Object<T>): address {
        let caps_address = object::object_address(caps);
        assert!(
            exists<Caps>(caps_address),
            error::not_found(EMANAGED_FUNGIBLE_ASSET_CAPS),
        );
        caps_address
    }

    #[test(creator = @0xcafe)]
    fun test_basic_flow(
        creator: &signer,
    ) acquires Caps {
        let (creator_ref, asset) = create_test_token(creator);
        initialize_managing_capabilities(&generate_extend_ref(&creator_ref), 100 /* max supply */);
        let creator_address = signer::address_of(creator);

        assert!(owner_can_mint(&asset), 1);
        assert!(owner_can_freeze(&asset), 2);
        assert!(owner_can_burn(&asset), 3);

        mint_by_asset_owner(creator, &asset, 100, creator_address);
        assert!(balance_of(creator_address, &asset) == 100, 4);
        freeze_by_asset_owner(creator, &asset, creator_address);
        assert!(is_frozen(creator_address, &asset), 5);
        unfreeze_by_asset_owner(creator, &asset, creator_address);
        assert!(!is_frozen(creator_address, &asset), 6);
        burn_by_asset_owner(creator, &asset, 90, creator_address);

        destroy_mint_cap(creator, &asset);
        destroy_freeze_cap(creator, &asset);
        destroy_burn_cap(creator, &asset);

        assert!(!owner_can_mint(&asset), 7);
        assert!(!owner_can_freeze(&asset), 8);
        assert!(!owner_can_burn(&asset), 9);
    }

    #[test(creator = @0xcafe, aaron = @0xface)]
    #[expected_failure(abort_code = 0x50004, location = Self)]
    fun test_permission_denied(
        creator: &signer,
        aaron: &signer
    ) acquires Caps {
        let (creator_ref, asset) = create_test_token(creator);
        initialize_managing_capabilities(&generate_extend_ref(&creator_ref), 100 /* max supply */);
        let creator_address = signer::address_of(creator);
        assert!(owner_can_mint(&asset), 1);
        mint_by_asset_owner(aaron, &asset, 100, creator_address);
    }
}
