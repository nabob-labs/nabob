module aa::test_functions {
    use nabob_framework::nabob_account;

    /// test function for multi-agent aa.
    public entry fun transfer_to_the_last(a: &signer, b: &signer, c: &signer, d: address) {
        nabob_account::transfer(a, d, 1);
        nabob_account::transfer(b, d, 1);
        nabob_account::transfer(c, d, 1);
    }
}
