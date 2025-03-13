/// Define the GovernanceProposal that will be used as part of on-chain governance by NabobGovernance.
///
/// This is separate from the NabobGovernance module to avoid circular dependency between NabobGovernance and Stake.
module nabob_framework::governance_proposal {
    friend nabob_framework::nabob_governance;

    struct GovernanceProposal has store, drop {}

    /// Create and return a GovernanceProposal resource. Can only be called by NabobGovernance
    public(friend) fun create_proposal(): GovernanceProposal {
        GovernanceProposal {}
    }

    /// Useful for NabobGovernance to create an empty proposal as proof.
    public(friend) fun create_empty_proposal(): GovernanceProposal {
        create_proposal()
    }

    #[test_only]
    public fun create_test_proposal(): GovernanceProposal {
        create_empty_proposal()
    }
}
