#[test_only]
module nabob_framework::account_abstraction_tests {
    use nabob_framework::auth_data::AbstractionAuthData;

    public fun test_auth(account: signer, _data: AbstractionAuthData): signer { account }
}
