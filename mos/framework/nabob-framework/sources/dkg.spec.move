spec nabob_framework::dkg {

    spec module {
        use nabob_framework::chain_status;
        invariant [suspendable] chain_status::is_operating() ==> exists<DKGState>(@nabob_framework);
    }

    spec initialize(nabob_framework: &signer) {
        use std::signer;
        let nabob_framework_addr = signer::address_of(nabob_framework);
        aborts_if nabob_framework_addr != @nabob_framework;
    }

    spec start(
        dealer_epoch: u64,
        randomness_config: RandomnessConfig,
        dealer_validator_set: vector<ValidatorConsensusInfo>,
        target_validator_set: vector<ValidatorConsensusInfo>,
    ) {
        aborts_if !exists<DKGState>(@nabob_framework);
        aborts_if !exists<timestamp::CurrentTimeMicroseconds>(@nabob_framework);
    }

    spec finish(transcript: vector<u8>) {
        use std::option;
        requires exists<DKGState>(@nabob_framework);
        requires option::is_some(global<DKGState>(@nabob_framework).in_progress);
        aborts_if false;
    }

    spec fun has_incomplete_session(): bool {
        if (exists<DKGState>(@nabob_framework)) {
            option::spec_is_some(global<DKGState>(@nabob_framework).in_progress)
        } else {
            false
        }
    }

    spec try_clear_incomplete_session(fx: &signer) {
        use std::signer;
        let addr = signer::address_of(fx);
        aborts_if addr != @nabob_framework;
    }

    spec incomplete_session(): Option<DKGSessionState> {
        aborts_if false;
    }
}
