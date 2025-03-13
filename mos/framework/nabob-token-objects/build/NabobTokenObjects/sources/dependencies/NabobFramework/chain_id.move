/// The chain id distinguishes between different chains (e.g., testnet and the main network).
/// One important role is to prevent transactions intended for one chain from being executed on another.
/// This code provides a container for storing a chain id and functions to initialize and get it.
module nabob_framework::chain_id {
    use nabob_framework::system_addresses;

    friend nabob_framework::genesis;

    struct ChainId has key {
        id: u8
    }

    /// Only called during genesis.
    /// Publish the chain ID `id` of this instance under the SystemAddresses address
    public(friend) fun initialize(nabob_framework: &signer, id: u8) {
        system_addresses::assert_nabob_framework(nabob_framework);
        move_to(nabob_framework, ChainId { id })
    }

    #[view]
    /// Return the chain ID of this instance.
    public fun get(): u8 acquires ChainId {
        borrow_global<ChainId>(@nabob_framework).id
    }

    #[test_only]
    use std::signer;

    #[test_only]
    public fun initialize_for_test(nabob_framework: &signer, id: u8) {
        if (!exists<ChainId>(signer::address_of(nabob_framework))) {
            initialize(nabob_framework, id);
        }
    }

    #[test(nabob_framework = @0x1)]
    fun test_get(nabob_framework: &signer) acquires ChainId {
        initialize_for_test(nabob_framework, 1u8);
        assert!(get() == 1u8, 1);
    }
}
