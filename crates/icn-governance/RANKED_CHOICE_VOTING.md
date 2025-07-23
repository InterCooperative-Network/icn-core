# ICN Governance Voting Primitives

This document demonstrates how to use the new ranked choice voting primitives in the icn-governance crate.

## Basic Usage

### Creating a Ranked Choice Election

```rust
use icn_governance::{
    Election, ElectionId, Candidate, CandidateId, VotingPeriod, EligibilityRules
};
use icn_common::Did;
use std::time::SystemTime;

// Create candidates
let candidates = vec![
    Candidate {
        id: CandidateId("alice".to_string()),
        name: "Alice Smith".to_string(),
        description: "Experienced leader".to_string(),
        metadata: None,
    },
    Candidate {
        id: CandidateId("bob".to_string()),
        name: "Bob Jones".to_string(),
        description: "Community organizer".to_string(),
        metadata: None,
    },
];

// Set voting period
let now = SystemTime::now();
let voting_period = VotingPeriod {
    start_time: now,
    end_time: now + std::time::Duration::from_secs(86400), // 24 hours
};

// Create election
let election = Election {
    election_id: ElectionId("election-2023".to_string()),
    title: "Board Election".to_string(),
    description: "Annual board election".to_string(),
    candidates,
    voting_period,
    eligibility_rules: EligibilityRules {
        required_credentials: vec![],
        min_reputation: None,
        required_federation: None,
        custom_rules: None,
    },
    content_cid: None,
    creator: Did::default(),
    created_at: now,
};
```

### Creating and Validating Ballots

```rust
use icn_governance::{
    RankedChoiceBallot, BallotId, Signature, VotingSystem, BallotValidator,
    RankedChoiceVotingSystem, RankedChoiceBallotValidator
};
use icn_identity::KeyDidResolver;
use std::sync::Arc;

// Create a ranked choice ballot
let ballot = RankedChoiceBallot {
    ballot_id: BallotId("ballot-001".to_string()),
    voter_did: Did::default(),
    election_id: ElectionId("election-2023".to_string()),
    preferences: vec![
        CandidateId("alice".to_string()),  // 1st choice
        CandidateId("bob".to_string()),    // 2nd choice
    ],
    timestamp: SystemTime::now(),
    signature: Signature {
        algorithm: "ed25519".to_string(),
        value: vec![0u8; 64], // Real signature would go here
    },
};

// Validate ballot preferences
assert!(ballot.validate_preferences().is_ok());

// Create validator and validate ballot
let did_resolver = Arc::new(KeyDidResolver);
let validator = RankedChoiceBallotValidator::new(did_resolver.clone());

validator.validate_format(&ballot).unwrap();
validator.validate_signature(&ballot).unwrap();
validator.check_duplicate(&ballot).unwrap();
```

### Running Ranked Choice Voting

```rust
// Create voting system
let voting_system = RankedChoiceVotingSystem::new(did_resolver, 1);

// Collect ballots from voters
let ballots = vec![
    ballot, // ... more ballots
];

// Validate all ballots
for ballot in &ballots {
    voting_system.validate_ballot(ballot).unwrap();
}

// Count votes using ranked choice algorithm
let result = voting_system.count_votes(ballots).unwrap();

// Check results
println!("Winner: {:?}", result.winner);
println!("Total ballots: {}", result.total_ballots);
println!("Rounds: {}", result.rounds.len());

for round in &result.rounds {
    println!("Round {}: {:?}", round.round_number, round.vote_counts);
}
```

## Voting System Trait

The `VotingSystem` trait provides a generic interface for different voting methods:

```rust
pub trait VotingSystem {
    type Ballot;
    type Result;
    type Error;
    
    fn validate_ballot(&self, ballot: &Self::Ballot) -> Result<(), Self::Error>;
    fn count_votes(&self, ballots: Vec<Self::Ballot>) -> Result<Self::Result, Self::Error>;
    fn is_eligible_voter(&self, voter_id: &str) -> Result<bool, Self::Error>;
}
```

This allows for easy extension to other voting methods like approval voting, STAR voting, etc.

## Ballot Validator Trait

The `BallotValidator` trait provides validation capabilities:

```rust
pub trait BallotValidator {
    fn validate_format(&self, ballot: &dyn Any) -> Result<(), VotingError>;
    fn validate_signature(&self, ballot: &dyn Any) -> Result<(), VotingError>;
    fn check_duplicate(&self, ballot: &dyn Any) -> Result<(), VotingError>;
}
```

## Integration with ICN Components

### Identity Integration
- Uses `icn-identity::DidResolver` for voter verification
- Integrates with DID documents for signature validation
- Supports federation-based eligibility rules

### DAG Integration  
- Election metadata can be stored in the DAG via `content_cid`
- Candidate information can reference DAG content
- Ballot signatures ensure integrity

### Economics Integration
- Can integrate with mana-based voting weights
- Supports reputation-based eligibility
- Compatible with economic policy enforcement

## Error Handling

The voting system provides comprehensive error handling:

```rust
pub enum VotingError {
    InvalidBallot(String),
    IneligibleVoter(String),
    DuplicateVote,
    ElectionInactive,
    InvalidSignature,
    InvalidCandidate(String),
    DuplicatePreferences,
    ElectionNotFound(String),
    VotingPeriodEnded,
    VotingPeriodNotStarted,
    InvalidRanking(String),
}
```

## Backward Compatibility

The new voting primitives are fully backward compatible with the existing governance system. The original `Vote` and `VoteOption` types remain unchanged, and existing governance proposals continue to work as before.