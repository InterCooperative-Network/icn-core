# ICN Governance (`icn-governance`) - Democratic Decision Engine

> **A comprehensive democratic governance platform that enables transparent, fair, and flexible collective decision-making within cooperative networks**

## Overview

The `icn-governance` crate implements ICN's democratic governance system, providing the infrastructure for proposal creation, voting procedures, quorum management, and decision execution. It supports both basic governance for single communities and advanced federation governance with trust-aware policies and cross-federation coordination.

**Key Principle**: All governance is transparent, auditable, and democratically controlled by community members, with flexible policies that adapt to community needs.

## Core Architecture

### 🏛️ Proposal System

The governance system centers around a sophisticated proposal lifecycle that ensures democratic deliberation and decision-making:

#### ProposalId - Unique Identification
```rust
/// Unique identifier for governance proposals
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProposalId(pub String);

// Generated format: "prop:{proposer}:{description_hash}:{timestamp}"
```

#### Proposal Types - Supported Actions
```rust
/// The type of action a proposal intends to perform
pub enum ProposalType {
    SystemParameterChange(String, String), // param_name, new_value
    NewMemberInvitation(Did),              // DID of member to invite
    RemoveMember(Did),                     // DID of member to remove
    SoftwareUpgrade(String),               // Version identifier
    GenericText(String),                   // General purpose proposals
    BudgetAllocation(Did, u64, String),    // recipient, amount, purpose
    Resolution(ResolutionProposal),        // Dispute resolution actions
}
```

#### Proposal Lifecycle States
```rust
/// Current lifecycle state of a proposal
pub enum ProposalStatus {
    Deliberation,  // Under discussion before voting opens
    VotingOpen,    // Actively collecting votes
    Accepted,      // Voting ended, quorum and threshold met
    Rejected,      // Voting ended, requirements not met
    Executed,      // Proposal actions have been implemented
    Failed,        // Execution failed
}
```

### 🗳️ Voting System

#### Vote Options
```rust
/// Possible voting options for proposals
pub enum VoteOption {
    Yes,     // Support the proposal
    No,      // Oppose the proposal
    Abstain, // Neutral/no preference
}
```

#### Vote Recording
```rust
/// A single vote on a proposal
pub struct Vote {
    pub voter: Did,              // DID of the voter
    pub proposal_id: ProposalId, // Proposal being voted on
    pub option: VoteOption,      // Vote choice
    pub voted_at: u64,          // Timestamp of vote
}
```

### 🏗️ GovernanceModule - Core Engine

The `GovernanceModule` is the central component that manages the entire governance process:

#### Backend Storage Options
```rust
/// Storage backend for governance data
enum Backend {
    InMemory {
        proposals: HashMap<ProposalId, Proposal>,
    },
    Sled {
        db: sled::Db,
        proposals_tree_name: String,
    },
}
```

#### Core Capabilities
```rust
impl GovernanceModule {
    /// Create new in-memory governance module
    pub fn new() -> Self;
    
    /// Create persistent governance module with Sled backend
    pub fn new_sled(db_path: PathBuf) -> Result<Self, CommonError>;
    
    /// Submit a new proposal for consideration
    pub fn submit_proposal(&mut self, submission: ProposalSubmission) -> Result<ProposalId, CommonError>;
    
    /// Transition proposal from deliberation to voting
    pub fn open_voting(&mut self, proposal_id: &ProposalId) -> Result<(), CommonError>;
    
    /// Cast a vote on an active proposal
    pub fn cast_vote(&mut self, voter: Did, proposal_id: &ProposalId, option: VoteOption) -> Result<(), CommonError>;
    
    /// Close voting and determine outcome
    pub fn close_voting_period(&mut self, proposal_id: &ProposalId) -> Result<(ProposalStatus, (usize, usize, usize)), CommonError>;
    
    /// Execute an accepted proposal
    pub fn execute_proposal(&mut self, proposal_id: &ProposalId) -> Result<(), CommonError>;
}
```

## Advanced Features

### 🤝 Member Management

#### Membership Control
```rust
impl GovernanceModule {
    /// Add new voting member
    pub fn add_member(&mut self, member: Did);
    
    /// Remove voting member
    pub fn remove_member(&mut self, did: &Did);
    
    /// Get current member set
    pub fn members(&self) -> &HashSet<Did>;
}
```

#### Vote Delegation
```rust
impl GovernanceModule {
    /// Delegate voting power to another member
    pub fn delegate_vote(&mut self, from: Did, to: Did) -> Result<(), CommonError>;
    
    /// Revoke existing delegation
    pub fn revoke_delegation(&mut self, from: Did);
}
```

### ⚖️ Governance Parameters

#### Quorum and Threshold Management
```rust
impl GovernanceModule {
    /// Set minimum votes required for valid proposal
    pub fn set_quorum(&mut self, quorum: usize);
    
    /// Set fraction of Yes votes required for acceptance
    pub fn set_threshold(&mut self, threshold: f32);
}
```

### 🔗 Federation Governance

The advanced federation governance system provides trust-aware policies and cross-federation coordination:

#### Trust-Aware Governance Policies
```rust
/// Governance policy with trust requirements
pub struct TrustAwareGovernancePolicy {
    pub action: GovernanceAction,                    // Action this policy applies to
    pub required_context: TrustContext,             // Required trust context
    pub min_trust_level: TrustLevel,                // Minimum trust level
    pub require_federation_membership: bool,        // Federation membership requirement
    pub voting_threshold: f64,                      // Voting threshold (0.0-1.0)
    pub quorum_requirement: f64,                    // Quorum requirement (0.0-1.0)
    pub allow_cross_federation: bool,               // Allow cross-federation participation
}
```

#### Federation Governance Engine
```rust
/// Federation governance engine with trust validation
pub struct FederationGovernanceEngine {
    trust_engine: TrustPolicyEngine,                          // Trust validation engine
    policies: HashMap<String, TrustAwareGovernancePolicy>,    // Governance policies
    proposals: HashMap<ProposalId, FederationProposal>,       // Active proposals
    federation_id: Option<FederationId>,                      // Federation this serves
    trust_committees: HashMap<String, TrustCommittee>,        // Trust oversight committees
    active_sanctions: HashMap<Did, Vec<TrustSanction>>,       // Active trust sanctions
}
```

#### Governance Actions
```rust
/// Actions that require trust validation
pub enum GovernanceAction {
    SubmitProposal { proposal_id: ProposalId },
    Vote { proposal_id: ProposalId, vote: bool },
    ExecuteProposal { proposal_id: ProposalId },
    ModifyMembership { target: Did, action: MembershipAction },
    UpdateTrust { target: Did, new_level: TrustLevel },
    CreateBridge { target_federation: FederationId },
}
```

### 🏛️ Trust Committees

#### Committee Structure
```rust
/// Trust committee for governance oversight
pub struct TrustCommittee {
    pub id: String,                                           // Committee identifier
    pub federation: FederationId,                           // Federation this serves
    pub members: HashMap<Did, TrustCommitteeMember>,         // Committee members
    pub managed_contexts: HashSet<TrustContext>,             // Trust contexts handled
    pub policies: HashMap<String, TrustThresholdPolicy>,     // Committee policies
    pub status: CommitteeStatus,                             // Committee status
}
```

#### Committee Roles
```rust
/// Roles within trust committees
pub enum TrustCommitteeRole {
    Chair,    // Committee chair with enhanced privileges
    Member,   // Standard voting member
    Advisor,  // Advisory member with limited voting
    Observer, // Observer without voting rights
}
```

## Practical Usage Examples

### Basic Community Governance
```rust
use icn_governance::{GovernanceModule, ProposalSubmission, ProposalType, VoteOption};
use icn_common::Did;

// Setup community governance
let mut governance = GovernanceModule::new();

// Add community members
governance.add_member(Did::from_str("did:example:alice")?);
governance.add_member(Did::from_str("did:example:bob")?);
governance.add_member(Did::from_str("did:example:charlie")?);

// Configure governance parameters
governance.set_quorum(2);     // Minimum 2 votes required
governance.set_threshold(0.6); // 60% Yes votes required

// Submit proposal for new member
let proposal_id = governance.submit_proposal(ProposalSubmission {
    proposer: Did::from_str("did:example:alice")?,
    proposal_type: ProposalType::NewMemberInvitation(
        Did::from_str("did:example:dave")?
    ),
    description: "Invite Dave as new community member".to_string(),
    duration_secs: 86400, // 24 hours voting period
    quorum: None,         // Use default quorum
    threshold: None,      // Use default threshold
    content_cid: None,    // No additional content
})?;

// Open voting after deliberation period
governance.open_voting(&proposal_id)?;

// Members cast votes
governance.cast_vote(
    Did::from_str("did:example:bob")?,
    &proposal_id,
    VoteOption::Yes
)?;
governance.cast_vote(
    Did::from_str("did:example:charlie")?,
    &proposal_id,
    VoteOption::Yes
)?;

// Close voting and check results
let (status, (yes, no, abstain)) = governance.close_voting_period(&proposal_id)?;
assert_eq!(status, ProposalStatus::Accepted);

// Execute accepted proposal
governance.execute_proposal(&proposal_id)?;

// Dave is now a member
assert!(governance.members().contains(&Did::from_str("did:example:dave")?));
```

### Federation Governance with Trust
```rust
use icn_governance::federation_governance::{FederationGovernanceEngine, TrustAwareGovernancePolicy};
use icn_identity::{TrustPolicyEngine, TrustContext, TrustLevel, FederationId};

// Setup trust-aware federation governance
let trust_engine = TrustPolicyEngine::new();
let federation_id = FederationId::new("housing_coop_federation".to_string());
let mut fed_governance = FederationGovernanceEngine::new(trust_engine, Some(federation_id.clone()));

// Define voting policy
let voting_policy = TrustAwareGovernancePolicy {
    action: GovernanceAction::Vote {
        proposal_id: ProposalId("dummy".to_string()),
        vote: true,
    },
    required_context: TrustContext::Governance,
    min_trust_level: TrustLevel::Basic,
    require_federation_membership: true,
    voting_threshold: 0.66,      // 2/3 majority required
    quorum_requirement: 0.4,     // 40% participation required
    allow_cross_federation: false,
};

fed_governance.add_policy("vote".to_string(), voting_policy);

// Submit federation proposal
let proposal_id = fed_governance.submit_proposal(
    &Did::from_str("did:example:alice")?,
    federation_id,
    TrustContext::Governance,
    "Allocate $50,000 for affordable housing development".to_string(),
    1209600, // 2 weeks voting period
)?;

// Vote with trust validation
fed_governance.vote(
    &Did::from_str("did:example:bob")?,
    &proposal_id,
    true
)?;

// Finalize proposal after voting period
let result = fed_governance.finalize_proposal(&proposal_id)?;
```

### Persistent Governance with Sled
```rust
use icn_governance::GovernanceModule;
use std::path::PathBuf;

// Create persistent governance with Sled backend
let db_path = PathBuf::from("./governance_data");
let mut governance = GovernanceModule::new_sled(db_path)?;

// All proposals and votes are automatically persisted
// System can recover state after restart
```

## Event Sourcing Integration

### Governance Events
```rust
/// Events emitted by governance operations
pub enum GovernanceEvent {
    ProposalSubmitted(Proposal),                    // New proposal created
    VoteCast(Vote),                                // Vote recorded
    StatusUpdated(ProposalId, ProposalStatus),     // Proposal status changed
}
```

### Event Store Integration
```rust
// Create governance with event sourcing
let event_store = Box::new(InMemoryEventStore::new());
let mut governance = GovernanceModule::with_event_store(event_store);

// Rebuild state from events
let governance = GovernanceModule::from_event_store(event_store)?;
```

## Trust and Security Features

### 🔒 Dispute Resolution
```rust
/// Resolution actions for disputes
pub enum ResolutionAction {
    PauseCredential(Cid),  // Pause specific credential
    FreezeReputation(Did), // Freeze reputation score
}

/// Dispute resolution proposal
pub struct ResolutionProposal {
    pub actions: Vec<ResolutionAction>,
}
```

### 🛡️ Trust Violations
```rust
/// Types of trust violations
pub enum ViolationType {
    Malicious,                    // Malicious behavior
    ResourceAbuse,               // Resource abuse
    GovernanceManipulation,      // Governance manipulation
    IdentityMisrepresentation,   // Identity misrepresentation
    EconomicMisconduct,          // Economic misconduct
    DataPrivacyViolation,        // Data privacy violation
    InfrastructureAbuse,         // Infrastructure abuse
    Custom(String),              // Custom violation type
}
```

### ⚖️ Trust Sanctions
```rust
/// Trust sanctions for violations
pub enum TrustSanction {
    TemporaryRestriction {
        duration_hours: u64,
        restricted_contexts: HashSet<TrustContext>,
    },
    ReputationPenalty {
        penalty_amount: f64,
        decay_period_hours: u64,
    },
    CapabilityRevocation {
        revoked_capabilities: HashSet<String>,
        review_required: bool,
    },
    FederationSuspension {
        federation: FederationId,
        suspension_duration_hours: u64,
    },
}
```

## Budgeting and Resource Allocation

### Budget Proposals
```rust
/// Budget allocation proposal
pub struct BudgetProposal {
    pub recipient: Did,       // Who receives the allocation
    pub amount: u64,         // Amount to allocate
    pub purpose: String,     // Purpose/justification
    pub duration: u64,       // Duration of allocation
    pub conditions: Vec<String>, // Conditions for release
}

/// Apply budget allocation after approval
pub fn apply_budget_allocation(proposal: &BudgetProposal) -> Result<(), CommonError>;
```

## Integration Points

### 🔌 Runtime Integration
The governance system integrates with the ICN runtime through:
- **Host ABI Functions**: Access governance from WASM modules
- **Proposal Callbacks**: Execute system changes when proposals pass
- **Event Notifications**: Real-time governance events

### 🌐 Network Integration
Federation governance coordinates across nodes through:
- **Proposal Synchronization**: Share proposals across federation
- **Vote Aggregation**: Collect votes from distributed members
- **Trust Validation**: Verify cross-federation trust relationships

### 📊 Economics Integration
Governance interfaces with the economics system for:
- **Budget Management**: Democratic resource allocation
- **Mana-based Voting**: Weight votes by economic contribution
- **Cost-benefit Analysis**: Economic impact assessment

## Metrics and Monitoring

### Governance Metrics
```rust
// Prometheus metrics for governance operations
static SUBMIT_PROPOSAL_CALLS: Counter;  // Proposals submitted
static CAST_VOTE_CALLS: Counter;        // Votes cast
static EXECUTE_PROPOSAL_CALLS: Counter; // Proposals executed

// Usage tracking
governance.submit_proposal(submission)?; // Increments SUBMIT_PROPOSAL_CALLS
governance.cast_vote(voter, &id, vote)?; // Increments CAST_VOTE_CALLS
governance.execute_proposal(&id)?;       // Increments EXECUTE_PROPOSAL_CALLS
```

## Testing and Validation

### Unit Testing
```rust
#[test]
fn test_proposal_lifecycle() {
    let mut gov = GovernanceModule::new();
    
    // Test complete proposal workflow
    let proposal_id = gov.submit_proposal(submission).unwrap();
    gov.open_voting(&proposal_id).unwrap();
    gov.cast_vote(voter, &proposal_id, VoteOption::Yes).unwrap();
    let (status, _) = gov.close_voting_period(&proposal_id).unwrap();
    assert_eq!(status, ProposalStatus::Accepted);
    
    gov.execute_proposal(&proposal_id).unwrap();
}
```

### Integration Testing
```rust
#[test]
fn test_federation_governance_integration() {
    // Test trust-aware governance across federations
    let mut engine = setup_federation_governance();
    
    // Test proposal submission with trust validation
    let proposal_id = engine.submit_proposal(
        &proposer,
        federation_id,
        TrustContext::Governance,
        content,
        deadline
    ).unwrap();
    
    // Test voting with trust checks
    engine.vote(&voter, &proposal_id, true).unwrap();
    
    // Test finalization with policy enforcement
    let result = engine.finalize_proposal(&proposal_id).unwrap();
}
```

---

**Key Insight**: ICN's governance system provides both simple community governance and sophisticated federation governance with trust-aware policies, enabling democratic decision-making at any scale from small communities to large federations. 