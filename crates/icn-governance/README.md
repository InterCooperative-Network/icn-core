# ICN Governance Crate

> **⚠️ Development Status**: Governance mechanisms contain significant stub implementations. Voting procedures, quorum logic, and decision execution need substantial development work.

This crate defines the mechanisms for network governance within the InterCooperative Network (ICN).

See [CONTEXT.md](../../CONTEXT.md) for ICN Core design philosophy and crate roles.
See [docs/ASYNC_OVERVIEW.md](../../docs/ASYNC_OVERVIEW.md) for async API guidelines.

## Purpose

The `icn-governance` crate is responsible for:

*   **Proposal Systems:** Defining how proposals for network changes (e.g., protocol upgrades, parameter adjustments, funding) are submitted and managed.
*   **Voting Procedures:** Implementing the logic for how stakeholders vote on proposals, including vote counting and threshold determination.
*   **Quorum Logic:** Defining the requirements for a vote to be considered valid (e.g., minimum participation).
*   **Decision Execution:** Potentially interfacing with other crates to enact decisions once they are approved through governance.
*   **Role Management:** Managing roles and permissions related to governance participation.
*   **Ranked Choice Voting:** Advanced voting mechanisms including ranked choice voting with comprehensive ballot validation.
*   **Ballot Anchoring:** Permanent storage and verification of ballots using the DAG infrastructure.

This crate is essential for the decentralized control and evolution of the ICN.

## Enhanced Voting Primitives

The crate now includes sophisticated voting mechanisms built on core primitives:

### Core Traits

- **`VotingSystem`**: Generic trait for implementing different voting algorithms
- **`BallotValidator`**: Comprehensive ballot validation including format, signatures, and duplicate detection
- **`Signable`**: Integration with ICN's cryptographic infrastructure for ballot integrity

### Ranked Choice Voting

- **`RankedChoiceVotingSystem`**: Full implementation of ranked choice voting algorithm
- **`RankedChoiceBallot`**: Cryptographically-signed ballots with ordered candidate preferences
- **`RankedChoiceResult`**: Detailed round-by-round election results

### DAG Integration

- **`BallotAnchoringService`**: Permanent ballot storage using content-addressed DAG
- Ballot retrieval and verification from distributed storage
- Election result linking and aggregation

### Voter Eligibility

- **`EligibilityRules`**: Configurable voter eligibility with federation, reputation, and credential requirements
- Integration points for reputation scoring and federation membership
- Custom rule evaluation from DAG-stored criteria

## Usage Examples

```rust
use icn_governance::{
    RankedChoiceVotingSystem, RankedChoiceBallot, BallotAnchoringService,
    EligibilityRules, Election, BallotId, CandidateId
};
use icn_identity::KeyDidResolver;
use icn_dag::InMemoryDagStore;
use std::sync::Arc;

// Create voting system with DID resolver
let did_resolver = Arc::new(KeyDidResolver);
let voting_system = RankedChoiceVotingSystem::new(did_resolver, 1);

// Create ballot anchoring service with DAG storage
let dag_storage = InMemoryDagStore::new();
let mut anchoring_service = BallotAnchoringService::new(dag_storage);

// Create election with configurable eligibility
let eligibility = EligibilityRules::federation_members_only("my-federation".to_string());
let election = Election { /* election details */ };

// Create and validate ballot
let ballot = RankedChoiceBallot::new(
    BallotId("ballot-001".to_string()),
    voter_did_document,
    election.election_id.clone(),
    vec![CandidateId("alice".to_string()), CandidateId("bob".to_string())],
    signature,
);

// Anchor ballot in DAG for permanent storage
let ballot_cid = anchoring_service.anchor_ballot(&ballot)?;

// Execute ranked choice voting
let result = voting_system.count_votes(ballots)?;
```

## Public API Style

The API style emphasizes:

*   **Transparency:** Clear and auditable governance processes.
*   **Fairness:** Ensuring that voting and proposal mechanisms are equitable.
*   **Flexibility:** Allowing for different governance models or parameters to be configured.
*   **Interoperability:** Providing clear interfaces for other crates (e.g., `icn-cli`, `icn-node`) to interact with governance functions.
*   **Persistence:** With the `persist-sled` feature, proposals and votes are stored using `Sled`, enabling recovery across restarts.

## Federation Sync

When built with the `federation` feature, this crate exposes
`request_federation_sync`, an async helper that uses a provided
`NetworkService` to request state from another peer. It sends a
`FederationSyncRequest` message via the network layer and returns a
`CommonError` if the underlying send fails.
## DAO Reward Templates

Reusable CCL snippets for rewarding contributors live in `templates/`. Examples:
- `reward_member.ccl` — grant scoped tokens to a specific member.
- `community_bonus.ccl` — distribute a bonus to all members.
- `dao_reward_issuance.ccl` — issue DAO reward tokens to members.
- `contributor_recognition.ccl` — mint a recognition badge for a contributor.


## Contributing

Contributions are welcome! Please see the main [CONTRIBUTING.md](../../CONTRIBUTING.md) in the root of the `icn-core` repository for guidelines.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](../../LICENSE). 