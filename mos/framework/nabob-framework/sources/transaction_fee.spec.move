spec nabob_framework::transaction_fee {
    /// <high-level-req>
    /// No.: 1
    /// Requirement: Given the blockchain is in an operating state, it guarantees that the Nabob framework signer may burn
    /// Nabob coins.
    /// Criticality: Critical
    /// Implementation: The NabobCoinCapabilities structure is defined in this module and it stores burn capability to
    /// burn the gas fees.
    /// Enforcement: Formally Verified via [high-level-req-1](module).
    ///
    /// No.: 2
    /// Requirement: The initialization function may only be called once.
    /// Criticality: Medium
    /// Implementation: The initialize_fee_collection_and_distribution function ensures CollectedFeesPerBlock does not
    /// already exist.
    /// Enforcement: Formally verified via [high-level-req-2](initialize_fee_collection_and_distribution).
    ///
    /// No.: 3
    /// Requirement: Only the admin address is authorized to call the initialization function.
    /// Criticality: Critical
    /// Implementation: The initialize_fee_collection_and_distribution function ensures only the Nabob framework address
    /// calls it.
    /// Enforcement: Formally verified via [high-level-req-3](initialize_fee_collection_and_distribution).
    ///
    /// No.: 4
    /// Requirement: The percentage of the burnt collected fee is always a value from 0 to 100.
    /// Criticality: Medium
    /// Implementation: During the initialization of CollectedFeesPerBlock in
    /// Initialize_fee_collection_and_distribution, and while upgrading burn percentage, it asserts that burn_percentage
    /// is within the specified limits.
    /// Enforcement: Formally verified via [high-level-req-4](CollectedFeesPerBlock).
    ///
    /// No.: 5
    /// Requirement: Prior to upgrading the burn percentage, it must process all the fees collected up to that point.
    /// Criticality: Critical
    /// Implementation: The upgrade_burn_percentage function ensures process_collected_fees function is called before
    /// updating the burn percentage.
    /// Enforcement: Formally verified in [high-level-req-5](ProcessCollectedFeesRequiresAndEnsures).
    ///
    /// No.: 6
    /// Requirement: The presence of the resource, indicating collected fees per block under the Nabob framework account,
    /// is a prerequisite for the successful execution of the following functionalities: Upgrading burn percentage.
    /// Registering a block proposer. Processing collected fees.
    /// Criticality: Low
    /// Implementation: The functions: upgrade_burn_percentage, register_proposer_for_fee_collection, and
    /// process_collected_fees all ensure that the CollectedFeesPerBlock resource exists under nabob_framework by
    /// calling the is_fees_collection_enabled method, which returns a boolean value confirming if the resource exists
    /// or not.
    /// Enforcement: Formally verified via [high-level-req-6.1](register_proposer_for_fee_collection), [high-level-req-6.2](process_collected_fees), and [high-level-req-6.3](upgrade_burn_percentage).
    /// </high-level-req>
    ///
    spec module {
        use nabob_framework::chain_status;

        // TODO(fa_migration)
        pragma verify = false;

        pragma aborts_if_is_strict;
        // property 1: Given the blockchain is in an operating state, it guarantees that the Nabob framework signer may burn Nabob coins.
        /// [high-level-req-1]
        invariant [suspendable] chain_status::is_operating() ==> exists<NabobCoinCapabilities>(@nabob_framework) || exists<NabobFABurnCapabilities>(@nabob_framework);
    }

    spec CollectedFeesPerBlock {
        // property 4: The percentage of the burnt collected fee is always a value from 0 to 100.
        /// [high-level-req-4]
        invariant burn_percentage <= 100;
    }

    spec initialize_fee_collection_and_distribution(_nabob_framework: &signer, _burn_percentage: u8) {
    }

    /// `NabobCoinCapabilities` should be exists.
    spec burn_fee(account: address, fee: u64) {
        use nabob_std::type_info;
        use nabob_framework::optional_aggregator;
        use nabob_framework::coin;
        use nabob_framework::coin::{CoinInfo, CoinStore};
        // TODO(fa_migration)
        pragma verify = false;

        aborts_if !exists<NabobCoinCapabilities>(@nabob_framework);

        // This function essentially calls `coin::burn_coin`, monophormized for `NabobCoin`.
        let account_addr = account;
        let amount = fee;

        let nabob_addr = type_info::type_of<NabobCoin>().account_address;
        let coin_store = global<CoinStore<NabobCoin>>(account_addr);
        let post post_coin_store = global<CoinStore<NabobCoin>>(account_addr);

        // modifies global<CoinStore<NabobCoin>>(account_addr);

        aborts_if amount != 0 && !(exists<CoinInfo<NabobCoin>>(nabob_addr)
            && exists<CoinStore<NabobCoin>>(account_addr));
        aborts_if coin_store.coin.value < amount;

        let maybe_supply = global<CoinInfo<NabobCoin>>(nabob_addr).supply;
        let supply_aggr = option::spec_borrow(maybe_supply);
        let value = optional_aggregator::optional_aggregator_value(supply_aggr);

        let post post_maybe_supply = global<CoinInfo<NabobCoin>>(nabob_addr).supply;
        let post post_supply = option::spec_borrow(post_maybe_supply);
        let post post_value = optional_aggregator::optional_aggregator_value(post_supply);

        aborts_if option::spec_is_some(maybe_supply) && value < amount;

        ensures post_coin_store.coin.value == coin_store.coin.value - amount;
        ensures if (option::spec_is_some(maybe_supply)) {
            post_value == value - amount
        } else {
            option::spec_is_none(post_maybe_supply)
        };
        ensures coin::supply<NabobCoin> == old(coin::supply<NabobCoin>) - amount;
    }

    spec mint_and_refund(account: address, refund: u64) {
        use nabob_std::type_info;
        use nabob_framework::nabob_coin::NabobCoin;
        use nabob_framework::coin::{CoinInfo, CoinStore};
        use nabob_framework::coin;
        // TODO(fa_migration)
        pragma verify = false;
        // pragma opaque;

        let nabob_addr = type_info::type_of<NabobCoin>().account_address;

        aborts_if (refund != 0) && !exists<CoinInfo<NabobCoin>>(nabob_addr);
        include coin::CoinAddAbortsIf<NabobCoin> { amount: refund };

        aborts_if !exists<CoinStore<NabobCoin>>(account);
        // modifies global<CoinStore<NabobCoin>>(account);

        aborts_if !exists<NabobCoinMintCapability>(@nabob_framework);

        let supply = coin::supply<NabobCoin>;
        let post post_supply = coin::supply<NabobCoin>;
        aborts_if [abstract] supply + refund > MAX_U128;
        ensures post_supply == supply + refund;
    }

    /// Ensure caller is admin.
    /// Aborts if `NabobCoinCapabilities` already exists.
    spec store_nabob_coin_burn_cap(nabob_framework: &signer, burn_cap: BurnCapability<NabobCoin>) {
        use std::signer;

        // TODO(fa_migration)
        pragma verify = false;

        let addr = signer::address_of(nabob_framework);
        aborts_if !system_addresses::is_nabob_framework_address(addr);

        aborts_if exists<NabobFABurnCapabilities>(addr);
        aborts_if exists<NabobCoinCapabilities>(addr);

        ensures exists<NabobFABurnCapabilities>(addr) || exists<NabobCoinCapabilities>(addr);
    }

    /// Ensure caller is admin.
    /// Aborts if `NabobCoinMintCapability` already exists.
    spec store_nabob_coin_mint_cap(nabob_framework: &signer, mint_cap: MintCapability<NabobCoin>) {
        use std::signer;
        let addr = signer::address_of(nabob_framework);
        aborts_if !system_addresses::is_nabob_framework_address(addr);
        aborts_if exists<NabobCoinMintCapability>(addr);
        ensures exists<NabobCoinMintCapability>(addr);
    }

    /// Historical. Aborts.
    spec initialize_storage_refund(_: &signer) {
        aborts_if true;
    }

    /// Aborts if module event feature is not enabled.
    spec emit_fee_statement {}
}
