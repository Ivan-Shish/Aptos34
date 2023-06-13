/// This module allows publishing to a resource account and retaining control its signer for future upgrades or
/// for other purposes such as creating an NFT collection.
///
/// The deployment flow is as follows:
/// 1. Deploy the package, including this package_manager module, using the Aptos CLI command create-resource-and-publish-package
/// with an appropriate seed. This will create a resource account and deploy the module. The deployer address also needs
/// to be specified in Move.toml.
/// 2. Make sure the created resource address is persisted in the Move.toml for future deployments and upgrades as the
/// CLI doesn't do so by default.
/// 3. During deployment, package_manager::init_module will be called and extract the SignerCapability from the newly
/// created resource account.
/// 4. All other modules from the same package that are friends can call package_manager to obtain the resource account
/// signer when needed. If cross-package access is needed, authorization can be granted via an address-based whitelist
/// instead of friendship, which is limited to the same package.
/// 5. If new modules need to be deployed or existing modules in this package need to be updated, an assigned admin
/// account (defaults to the deployer account) can call package_manager::publish_package to with the new code.
module resource_account_package::package_manager {
    use aptos_framework::account::{Self, SignerCapability};
    use aptos_framework::code;
    use aptos_framework::resource_account;
    use std::error;
    use std::signer;

    // Modules from the same package can be added here as friends to gain access to the resource account signer.
    // friend resource_account_package::custom_module;

    /// Stores permission config such as SignerCapability for controlling the resource account.
    struct PermissionConfig has key {
        /// Required to obtain the resource account signer.
        signer_cap: SignerCapability,
        /// The admin account who can publish new modules or upgrade existing ones in this package.
        /// This admin account can also be set to point to a multisig account or another resource account for more
        /// sophisticated governance setups.
        upgrade_admin: address,
    }

    /// The caller is not an admin and cannot perform upgrades for this package.
    const ENOT_ADMIN: u64 = 0;

    /// Initialize PermissionConfig to establish control over the resource account.
    /// This function is invoked only when this package is deployed the first time.
    fun init_module(resource_acc: &signer) {
        let signer_cap = resource_account::retrieve_resource_account_cap(resource_acc, @deployer);
        move_to(resource_acc, PermissionConfig { signer_cap, upgrade_admin: @deployer });
    }

    /// Can be called by friended modules to obtain the resource account signer.
    public(friend) fun get_signer(): signer acquires PermissionConfig {
        let signer_cap = &borrow_global<PermissionConfig>(@resource_account_package).signer_cap;
        account::create_signer_with_capability(signer_cap)
    }

    /// Can be called by the admin to publish new modules or upgrade existing modules in this package.
    public entry fun publish(
        admin: &signer,
        package_metadata: vector<u8>,
        code: vector<vector<u8>>,
    ) acquires PermissionConfig {
        let package_signer = authorized_create_signer(admin);
        code::publish_package_txn(package_signer, package_metadata, code);
    }

    inline fun authorized_create_signer(admin: &signer): &signer {
        assert!(
            signer::address_of(admin) == borrow_global<PermissionConfig>(@resource_account_package).upgrade_admin,
            error::permission_denied(ENOT_ADMIN),
        );
        &get_signer()
    }

    #[test(deployer = @0x123)]
    public fun test_can_get_signer(deployer: &signer) acquires PermissionConfig {
        let deployer_addr = signer::address_of(deployer);
        account::create_account_for_test(deployer_addr);
        let resource_addr = account::create_resource_address(&deployer_addr, b"");
        resource_account::create_resource_account(deployer, b"", b"");
        init_module(&account::create_signer_for_test(resource_addr));
        // We need to replicate get_signer()'s logic here since the resource account there is not available when running
        // aptos move test.
        let signer_cap = &borrow_global<PermissionConfig>(resource_addr).signer_cap;
        assert!(signer::address_of(&account::create_signer_with_capability(signer_cap)) == resource_addr, 0);
    }
}
