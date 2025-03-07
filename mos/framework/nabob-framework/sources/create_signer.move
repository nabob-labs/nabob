/// Provides a common place for exporting `create_signer` across the Nabob Framework.
///
/// To use create_signer, add the module below, such that:
/// `friend nabob_framework::friend_wants_create_signer`
/// where `friend_wants_create_signer` is the module that needs `create_signer`.
///
/// Note, that this is only available within the Nabob Framework.
///
/// This exists to make auditing straight forward and to limit the need to depend
/// on account to have access to this.
module nabob_framework::create_signer {
    friend nabob_framework::account;
    friend nabob_framework::nabob_account;
    friend nabob_framework::coin;
    friend nabob_framework::fungible_asset;
    friend nabob_framework::genesis;
    friend nabob_framework::account_abstraction;
    friend nabob_framework::multisig_account;
    friend nabob_framework::object;
    friend nabob_framework::permissioned_signer;
    friend nabob_framework::transaction_validation;

    public(friend) native fun create_signer(addr: address): signer;
}
