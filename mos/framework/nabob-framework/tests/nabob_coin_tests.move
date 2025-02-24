#[test_only]
module nabob_framework::nabob_coin_tests {
    use nabob_framework::nabob_coin;
    use nabob_framework::coin;
    use nabob_framework::fungible_asset::{Self, FungibleStore, Metadata};
    use nabob_framework::primary_fungible_store;
    use nabob_framework::object::{Self, Object};

    public fun mint_bob_fa_to_for_test<T: key>(store: Object<T>, amount: u64) {
        fungible_asset::deposit(store, nabob_coin::mint_bob_fa_for_test(amount));
    }

    public fun mint_bob_fa_to_primary_fungible_store_for_test(
        owner: address,
        amount: u64,
    ) {
        primary_fungible_store::deposit(owner, nabob_coin::mint_bob_fa_for_test(amount));
    }

    #[test(nabob_framework = @nabob_framework)]
    fun test_bob_setup_and_mint(nabob_framework: &signer) {
        let (burn_cap, mint_cap) = nabob_coin::initialize_for_test(nabob_framework);
        let coin = coin::mint(100, &mint_cap);
        let fa = coin::coin_to_fungible_asset(coin);
        primary_fungible_store::deposit(@nabob_framework, fa);
        assert!(
            primary_fungible_store::balance(
                @nabob_framework,
                object::address_to_object<Metadata>(@nabob_fungible_asset)
            ) == 100,
            0
        );
        coin::destroy_mint_cap(mint_cap);
        coin::destroy_burn_cap(burn_cap);
    }

    #[test]
    fun test_fa_helpers_for_test() {
        assert!(!object::object_exists<Metadata>(@nabob_fungible_asset), 0);
        nabob_coin::ensure_initialized_with_bob_fa_metadata_for_test();
        assert!(object::object_exists<Metadata>(@nabob_fungible_asset), 0);
        mint_bob_fa_to_primary_fungible_store_for_test(@nabob_framework, 100);
        let metadata = object::address_to_object<Metadata>(@nabob_fungible_asset);
        assert!(primary_fungible_store::balance(@nabob_framework, metadata) == 100, 0);
        let store_addr = primary_fungible_store::primary_store_address(@nabob_framework, metadata);
        mint_bob_fa_to_for_test(object::address_to_object<FungibleStore>(store_addr), 100);
        assert!(primary_fungible_store::balance(@nabob_framework, metadata) == 200, 0);
    }
}
