#![doc = include_str!("../README.md")]
#![allow(clippy::new_without_default)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::to_string_in_format_args)]
#![cfg_attr(
    not(feature = "allow-nondeterminism"),
    deny(clippy::disallowed_methods)
)]

//! # ICN Governance Crate
//! This crate defines the mechanisms for network governance within the InterCooperative Network (ICN).
//! It handles proposal systems, voting procedures, quorum logic, and decision execution,
//! focusing on transparency, fairness, and flexibility.

#[allow(unused_imports)]
use icn_common::{
    Cid, CommonError, Did, FixedTimeProvider, NodeInfo, SystemTimeProvider, TimeProvider,
};
#[cfg(feature = "federation")]
#[allow(unused_imports)]
use icn_network::{MeshNetworkError, NetworkService, PeerId, StubNetworkService};
#[cfg(feature = "federation")]
use icn_protocol::FederationSyncRequestMessage;
#[cfg(feature = "federation")]
use icn_protocol::{MessagePayload, ProtocolMessage};
use std::collections::{HashMap, HashSet};
use std::fmt;
#[cfg(feature = "persist-sled")]
use std::path::PathBuf;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub mod automation;
pub mod budgeting;
pub mod crdt_proposal_state;
pub mod federation_governance;
//pub mod federation_sync;
pub mod governance_conflict_resolver;
pub mod metrics;
pub mod policy_testing;
pub mod ranked_choice;
pub mod scoped_policy;
pub mod security;
pub mod voting;
pub use security::{
    BallotValidationStats, GovernanceSecurityAudit, GovernanceSecurityConfig,
    GovernanceSecurityIssue, SecureBallotSigner, SecureBallotValidator,
};

#[cfg(test)]
mod integration_tests;

#[cfg(test)]
mod governance_extensions_tests;

pub use automation::{
    AutomationVoteWeight, AutomationVotingResult, EnforcementAction, ExecutionResult,
    GovernanceAutomationConfig, GovernanceAutomationEngine, GovernanceAutomationStats,
    GovernanceEvent as AutomationGovernanceEvent, ReminderType,
};
pub use budgeting::{apply_budget_allocation, BudgetProposal};
pub use crdt_proposal_state::{
    CRDTProposalState, CRDTProposalStateConfig, CRDTProposalStateStats, ProposalCRDT, ProposalInfo,
    ProposalMetadata, ProposalStatus as CRDTProposalStatus, Vote as CRDTVote, VoteDecision,
    VoteTally,
};
pub use governance_conflict_resolver::{
    ConflictEvidence, ConflictSeverity, GovernanceConflict, GovernanceConflictConfig,
    GovernanceConflictResolver, GovernanceConflictType, GovernanceResolution,
    GovernanceResolutionStatus,
};
pub use policy_testing::{
    ExpectedOutcome, PolicyTestResult, PolicyTestScenario, PolicyTestingFramework, TestAction,
    TestExecutionStep, TestState,
};
pub use ranked_choice::{RankedChoiceBallotValidator, RankedChoiceVotingSystem};
pub use voting::{
    BallotAnchoringService, BallotId, BallotValidator, Candidate, CandidateId, Election,
    ElectionId, EligibilityRules, RankedChoiceBallot, RankedChoiceResult, RankedChoiceRound,
    Signature, VotingError, VotingPeriod, VotingSystem,
};

/// Trait for governance execution hooks.
///
/// Hooks are invoked when a proposal is executed and can perform
/// additional actions such as minting tokens or triggering runtime
/// upgrades. Implementors may hold references to other ICN crates
/// to access the necessary APIs.
pub trait ProposalCallback: Send + Sync {
    fn on_execute(&self, proposal: &Proposal) -> Result<(), CommonError>;
}

impl<F> ProposalCallback for F
where
    F: Fn(&Proposal) -> Result<(), CommonError> + Send + Sync,
{
    fn on_execute(&self, proposal: &Proposal) -> Result<(), CommonError> {
        self(proposal)
    }
}

// --- Proposal System ---

/// Unique identifier for a governance proposal.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ProposalId(pub String); // Could be a hash of the proposal content

impl std::fmt::Display for ProposalId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for ProposalId {
    type Err = icn_common::CommonError; // Or a more specific error type if desired
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            // Or based on whatever validation rules ProposalId might have.
            // For now, just ensuring it's not empty.
            Err(icn_common::CommonError::InvalidInputError(
                "Proposal ID cannot be empty".to_string(),
            ))
        } else {
            Ok(ProposalId(s.to_string()))
        }
    }
}

/// The type of action a proposal intends to perform.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ProposalType {
    SystemParameterChange(String, String), // param_name, new_value
    NewMemberInvitation(Did),              // DID of the member to invite
    RemoveMember(Did),                     // DID of the member to remove
    SoftwareUpgrade(String),               // Version or identifier for the upgrade
    GenericText(String),                   // For general purpose proposals
    BudgetAllocation(Did, u64, String),    // recipient, amount, purpose
    Resolution(ResolutionProposal),        // Dispute or remediation actions
}

/// Specific remediation actions for dispute resolution.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ResolutionAction {
    PauseCredential(Cid),
    FreezeReputation(Did),
}

/// Proposal containing one or more resolution actions.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ResolutionProposal {
    pub actions: Vec<ResolutionAction>,
}

/// Current lifecycle state of a proposal.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ProposalStatus {
    /// Newly created and under discussion before voting opens
    Deliberation,
    /// Waiting for sufficient sponsors before voting can open
    PendingSponsorship,
    /// Actively collecting votes
    VotingOpen,
    /// Voting period ended, quorum and threshold met, waiting for time-lock
    AcceptedTimelock,
    /// Accepted and time-lock period completed, ready for execution
    Accepted,
    /// Voting period ended, quorum or threshold not met
    Rejected,
    /// Vetoed during grace period
    Vetoed,
    /// For proposals that have an on-chain/system effect
    Executed,
    /// Execution failed
    Failed,
}

/// Governance configuration for time-locks, vetoes, and sponsorship requirements
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GovernanceConfig {
    /// Minimum number of sponsors required for a proposal to proceed to voting
    pub min_sponsors: usize,
    /// Time-lock delay in seconds between acceptance and execution
    pub timelock_delay_secs: u64,
    /// Grace period in seconds during which vetoes can be applied
    pub veto_grace_period_secs: u64,
    /// Members who have veto powers (empty means no veto powers)
    pub veto_members: HashSet<Did>,
    /// Whether to require anti-spam fees for proposal submission
    pub require_anti_spam_fees: bool,
    /// Mana cost for submitting a proposal (if anti-spam fees enabled)
    pub proposal_fee: u64,
}

impl Default for GovernanceConfig {
    fn default() -> Self {
        Self {
            min_sponsors: 1,
            timelock_delay_secs: 0,
            veto_grace_period_secs: 0,
            veto_members: HashSet::new(),
            require_anti_spam_fees: false,
            proposal_fee: 0,
        }
    }
}

/// Sponsorship information for a proposal
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ProposalSponsorship {
    /// Members who have sponsored this proposal
    pub sponsors: HashSet<Did>,
    /// Timestamp when sponsorship requirement was met
    pub sponsorship_complete_at: Option<u64>,
}

impl ProposalSponsorship {
    pub fn new() -> Self {
        Self {
            sponsors: HashSet::new(),
            sponsorship_complete_at: None,
        }
    }

    pub fn add_sponsor(&mut self, sponsor: Did, timestamp: u64, min_sponsors: usize) {
        self.sponsors.insert(sponsor);
        if self.sponsors.len() >= min_sponsors && self.sponsorship_complete_at.is_none() {
            self.sponsorship_complete_at = Some(timestamp);
        }
    }

    pub fn has_sufficient_sponsors(&self, min_sponsors: usize) -> bool {
        self.sponsors.len() >= min_sponsors
    }
}

/// Veto information for a proposal
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ProposalVeto {
    /// Member who applied the veto
    pub vetoed_by: Did,
    /// Timestamp when veto was applied
    pub vetoed_at: u64,
    /// Reason for the veto
    pub reason: String,
}
/// Full proposal record stored in the governance module.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Proposal {
    pub id: ProposalId,
    pub proposer: Did, // DID of the entity that proposed this
    pub proposal_type: ProposalType,
    pub description: String,
    pub created_at: u64,      // Timestamp (e.g., Unix epoch seconds)
    pub voting_deadline: u64, // Timestamp for when voting closes
    pub status: ProposalStatus,
    pub votes: HashMap<Did, Vote>, // Voter DID to their vote
    /// Optional quorum override for this proposal
    pub quorum: Option<usize>,
    /// Optional threshold override for this proposal
    pub threshold: Option<f32>,
    /// CID of proposal body stored in the DAG
    pub content_cid: Option<Cid>,
    /// Sponsorship information
    pub sponsorship: ProposalSponsorship,
    /// Time when proposal was accepted (for time-lock calculations)
    pub accepted_at: Option<u64>,
    /// Time-lock delay in seconds (overrides global config if set)
    pub timelock_delay: Option<u64>,
    /// Veto information if proposal was vetoed
    pub veto: Option<ProposalVeto>,
}

pub fn canonical_proposal(proposals: &[Proposal]) -> Option<&Proposal> {
    proposals
        .iter()
        .min_by(|a, b| match a.created_at.cmp(&b.created_at) {
            std::cmp::Ordering::Equal => a.id.0.cmp(&b.id.0),
            other => other,
        })
}

/// Possible voting options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum VoteOption {
    Yes,
    No,
    Abstain,
}

/// A single vote on a proposal.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Vote {
    pub voter: Did,
    pub proposal_id: ProposalId,
    pub option: VoteOption,
    pub voted_at: u64, // Timestamp
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[allow(clippy::large_enum_variant)]
pub enum GovernanceEvent {
    ProposalSubmitted(Proposal),
    VoteCast(Vote),
    StatusUpdated(ProposalId, ProposalStatus),
}

// Define the Backend enum
#[derive(Debug)]
enum Backend {
    InMemory {
        proposals: HashMap<ProposalId, Proposal>,
    },
    #[cfg(feature = "persist-sled")]
    Sled {
        db: sled::Db,
        // Define names for trees (sled's equivalent of tables/collections)
        // We'll store proposals in a tree named "proposals"
        // Key: ProposalId.0 (String), Value: bincode-serialized Proposal
        proposals_tree_name: String,
    },
}

/// Manages governance proposals and voting.
pub struct GovernanceModule {
    backend: Backend,
    members: HashSet<Did>,
    delegations: HashMap<Did, Did>,
    quorum: usize,
    threshold: f32,
    config: GovernanceConfig,
    #[allow(clippy::type_complexity)]
    proposal_callbacks: Vec<Box<dyn ProposalCallback>>,
    #[allow(clippy::type_complexity)]
    event_store: Option<std::sync::Mutex<Box<dyn icn_eventstore::EventStore<GovernanceEvent>>>>,
}

/// Parameters for submitting a new proposal
#[derive(Debug, Clone)]
pub struct ProposalSubmission {
    pub proposer: Did,
    pub proposal_type: ProposalType,
    pub description: String,
    pub duration_secs: u64,
    pub quorum: Option<usize>,
    pub threshold: Option<f32>,
    pub content_cid: Option<Cid>,
    pub timelock_delay: Option<u64>,
}

impl GovernanceModule {
    /// Creates a new GovernanceModule with an in-memory backend.
    pub fn new() -> Self {
        GovernanceModule {
            backend: Backend::InMemory {
                proposals: HashMap::new(),
            },
            members: HashSet::new(),
            delegations: HashMap::new(),
            quorum: 1,
            threshold: 0.5,
            config: GovernanceConfig::default(),
            proposal_callbacks: Vec::new(),
            event_store: None,
        }
    }

    /// Creates a new GovernanceModule with specific configuration.
    pub fn with_config(config: GovernanceConfig) -> Self {
        GovernanceModule {
            backend: Backend::InMemory {
                proposals: HashMap::new(),
            },
            members: HashSet::new(),
            delegations: HashMap::new(),
            quorum: 1,
            threshold: 0.5,
            config,
            proposal_callbacks: Vec::new(),
            event_store: None,
        }
    }

    /// Creates an in-memory governance module backed by the provided event store.
    pub fn with_event_store(store: Box<dyn icn_eventstore::EventStore<GovernanceEvent>>) -> Self {
        let mut g = Self::new();
        g.event_store = Some(std::sync::Mutex::new(store));
        g
    }

    /// Rebuild module state by replaying events from the store.
    pub fn from_event_store(
        store: Box<dyn icn_eventstore::EventStore<GovernanceEvent>>,
    ) -> Result<Self, CommonError> {
        let events = store.query(None)?;
        let mut g = Self::with_event_store(store);
        for ev in events {
            g.apply_event(ev);
        }
        Ok(g)
    }

    fn apply_event(&mut self, ev: GovernanceEvent) {
        match ev {
            GovernanceEvent::ProposalSubmitted(p) => {
                if let Backend::InMemory { proposals } = &mut self.backend {
                    proposals.insert(p.id.clone(), p);
                }
                #[cfg(feature = "persist-sled")]
                if let Backend::Sled { .. } = &self.backend {
                    // not implemented for sled yet
                }
            }
            GovernanceEvent::VoteCast(v) => {
                if let Backend::InMemory { proposals } = &mut self.backend {
                    if let Some(prop) = proposals.get_mut(&v.proposal_id) {
                        prop.votes.insert(v.voter.clone(), v);
                    }
                }
            }
            GovernanceEvent::StatusUpdated(id, status) => {
                if let Backend::InMemory { proposals } = &mut self.backend {
                    if let Some(prop) = proposals.get_mut(&id) {
                        prop.status = status;
                    }
                }
            }
        }
    }

    #[cfg(feature = "persist-sled")]
    /// Creates a new GovernanceModule with a sled persistent backend.
    pub fn new_sled(db_path: PathBuf) -> Result<Self, CommonError> {
        let db = sled::open(db_path).map_err(|e| {
            CommonError::DatabaseError(format!("Failed to open sled database: {}", e))
        })?;

        let proposals_tree_name = "proposals_v1".to_string(); // versioned tree name
                                                              // sled automatically creates trees when first accessed, so no explicit creation needed here.

        Ok(GovernanceModule {
            backend: Backend::Sled {
                db,
                proposals_tree_name,
            },
            members: HashSet::new(),
            delegations: HashMap::new(),
            quorum: 1,
            threshold: 0.5,
            config: GovernanceConfig::default(),
            proposal_callbacks: Vec::new(),
            event_store: None,
        })
    }

    /// Create and store a new proposal in the governance module.
    pub fn submit_proposal(
        &mut self,
        submission: ProposalSubmission,
        time_provider: &dyn TimeProvider,
    ) -> Result<ProposalId, CommonError> {
        metrics::SUBMIT_PROPOSAL_CALLS.inc();
        let now = time_provider.unix_seconds();
        let desc_hash_part = submission.description.chars().take(10).collect::<String>();
        let proposal_id_str = format!(
            "prop:{}:{}:{}",
            submission.proposer.to_string(),
            desc_hash_part,
            now
        );
        let proposal_id = ProposalId(proposal_id_str);

        let initial_status = if self.config.min_sponsors > 1 {
            ProposalStatus::PendingSponsorship
        } else {
            ProposalStatus::Deliberation
        };

        let mut sponsorship = ProposalSponsorship::new();
        // If min_sponsors is 1, the proposer automatically satisfies sponsorship
        if self.config.min_sponsors <= 1 {
            sponsorship.add_sponsor(submission.proposer.clone(), now, self.config.min_sponsors);
        }

        let proposal = Proposal {
            id: proposal_id.clone(),
            proposer: submission.proposer,
            proposal_type: submission.proposal_type,
            description: submission.description,
            created_at: now,
            voting_deadline: now + submission.duration_secs,
            status: initial_status,
            votes: HashMap::new(),
            quorum: submission.quorum,
            threshold: submission.threshold,
            content_cid: submission.content_cid,
            sponsorship,
            accepted_at: None,
            timelock_delay: submission.timelock_delay,
            veto: None,
        };

        match &mut self.backend {
            Backend::InMemory { proposals } => {
                if proposals.contains_key(&proposal_id) {
                    return Err(CommonError::InvalidInputError(format!(
                        "Proposal with ID {} already exists",
                        proposal_id.0
                    )));
                }
                proposals.insert(proposal_id.clone(), proposal.clone());
            }
            #[cfg(feature = "persist-sled")]
            Backend::Sled {
                db,
                proposals_tree_name,
            } => {
                let tree = db.open_tree(proposals_tree_name).map_err(|e| {
                    CommonError::DatabaseError(format!("Failed to open proposals tree: {}", e))
                })?;

                let key = proposal_id.0.as_bytes();
                if tree.contains_key(key).map_err(|e| {
                    CommonError::DatabaseError(format!(
                        "Failed to check key existence in proposals tree: {}",
                        e
                    ))
                })? {
                    return Err(CommonError::InvalidInputError(format!(
                        "Proposal with ID {} already exists",
                        proposal_id.0
                    )));
                }

                // Serialize using bincode for sled
                let encoded_proposal = bincode::serialize(&proposal).map_err(|e| {
                    CommonError::SerializationError(format!(
                        "Failed to serialize proposal {}: {}",
                        proposal_id.0, e
                    ))
                })?;

                tree.insert(key, encoded_proposal).map_err(|e| {
                    CommonError::DatabaseError(format!(
                        "Failed to insert proposal {} into sled: {}",
                        proposal_id.0, e
                    ))
                })?;
                tree.flush().map_err(|e| {
                    CommonError::DatabaseError(format!(
                        "Failed to flush sled tree for proposal {}: {}",
                        proposal_id.0, e
                    ))
                })?;
            }
        }
        if let Some(store) = &self.event_store {
            store
                .lock()
                .unwrap()
                .append(&GovernanceEvent::ProposalSubmitted(proposal))?;
        }
        Ok(proposal_id)
    }

    /// Transition a proposal from `Deliberation` to `VotingOpen`.
    pub fn open_voting(&mut self, proposal_id: &ProposalId) -> Result<(), CommonError> {
        match &mut self.backend {
            Backend::InMemory { proposals } => {
                let proposal = proposals.get_mut(proposal_id).ok_or_else(|| {
                    CommonError::ResourceNotFound(format!(
                        "Proposal with ID {} not found for opening",
                        proposal_id.0
                    ))
                })?;
                if proposal.status != ProposalStatus::Deliberation {
                    return Err(CommonError::InvalidInputError(format!(
                        "Proposal {} not ready for voting, current status: {:?}",
                        proposal_id.0, proposal.status
                    )));
                }
                
                // Check sponsorship requirements
                if !proposal.sponsorship.has_sufficient_sponsors(self.config.min_sponsors) {
                    return Err(CommonError::InvalidInputError(format!(
                        "Proposal {} does not have sufficient sponsors ({} required, {} found)",
                        proposal_id.0, self.config.min_sponsors, proposal.sponsorship.sponsors.len()
                    )));
                }
                
                proposal.status = ProposalStatus::VotingOpen;
            }
            #[cfg(feature = "persist-sled")]
            Backend::Sled {
                db,
                proposals_tree_name,
            } => {
                let tree = db.open_tree(proposals_tree_name).map_err(|e| {
                    CommonError::DatabaseError(format!("Failed to open proposals tree: {}", e))
                })?;
                let key = proposal_id.0.as_bytes();
                let proposal_bytes = tree
                    .get(key)
                    .map_err(|e| {
                        CommonError::DatabaseError(format!(
                            "Failed to get proposal {} from sled: {}",
                            proposal_id.0, e
                        ))
                    })?
                    .ok_or_else(|| {
                        CommonError::ResourceNotFound(format!(
                            "Proposal with ID {} not found for opening",
                            proposal_id.0
                        ))
                    })?;
                let mut proposal: Proposal =
                    bincode::deserialize(&proposal_bytes).map_err(|e| {
                        CommonError::DeserializationError(format!(
                            "Failed to deserialize proposal {}: {}",
                            proposal_id.0, e
                        ))
                    })?;
                if proposal.status != ProposalStatus::Deliberation {
                    return Err(CommonError::InvalidInputError(format!(
                        "Proposal {} not ready for voting, current status: {:?}",
                        proposal_id.0, proposal.status
                    )));
                }
                proposal.status = ProposalStatus::VotingOpen;
                let encoded = bincode::serialize(&proposal).map_err(|e| {
                    CommonError::SerializationError(format!(
                        "Failed to serialize updated proposal {}: {}",
                        proposal_id.0, e
                    ))
                })?;
                tree.insert(key, encoded).map_err(|e| {
                    CommonError::DatabaseError(format!(
                        "Failed to persist proposal {}: {}",
                        proposal_id.0, e
                    ))
                })?;
                tree.flush().map_err(|e| {
                    CommonError::DatabaseError(format!(
                        "Failed to flush sled tree for proposal {} open: {}",
                        proposal_id.0, e
                    ))
                })?;
            }
        }
        if let Some(store) = &self.event_store {
            store
                .lock()
                .unwrap()
                .append(&GovernanceEvent::StatusUpdated(
                    proposal_id.clone(),
                    ProposalStatus::VotingOpen,
                ))?;
        }
        Ok(())
    }

    /// Record a vote for the specified proposal.
    pub fn cast_vote(
        &mut self,
        voter: Did,
        proposal_id: &ProposalId,
        option: VoteOption,
        time_provider: &dyn TimeProvider,
    ) -> Result<(), CommonError> {
        metrics::CAST_VOTE_CALLS.inc();
        let now = time_provider.unix_seconds();

        // expire outdated proposals before attempting to cast a vote
        self.expire_proposals(now)?;

        match &mut self.backend {
            Backend::InMemory { proposals } => {
                let proposal = proposals.get_mut(proposal_id).ok_or_else(|| {
                    CommonError::ResourceNotFound(format!(
                        "Proposal with ID {} not found for casting vote",
                        proposal_id.0
                    ))
                })?;

                if now > proposal.voting_deadline {
                    return Err(CommonError::InvalidInputError(format!(
                        "Voting period for proposal {} has closed.",
                        proposal_id.0
                    )));
                }
                if proposal.status != ProposalStatus::VotingOpen {
                    return Err(CommonError::InvalidInputError(format!(
                        "Proposal {} not open for voting, current status: {:?}",
                        proposal_id.0, proposal.status
                    )));
                }

                let vote = Vote {
                    voter: voter.clone(),
                    proposal_id: proposal_id.clone(),
                    option,
                    voted_at: now,
                };
                proposal.votes.insert(voter.clone(), vote.clone());
            }
            #[cfg(feature = "persist-sled")]
            Backend::Sled {
                db,
                proposals_tree_name,
            } => {
                let tree = db.open_tree(proposals_tree_name).map_err(|e| {
                    CommonError::DatabaseError(format!("Failed to open proposals tree: {}", e))
                })?;

                let key = proposal_id.0.as_bytes();
                let proposal_bytes_ivec = tree
                    .get(key)
                    .map_err(|e| {
                        CommonError::DatabaseError(format!(
                            "Failed to get proposal {} from sled: {}",
                            proposal_id.0, e
                        ))
                    })?
                    .ok_or_else(|| {
                        CommonError::ResourceNotFound(format!(
                            "Proposal with ID {} not found for casting vote",
                            proposal_id.0
                        ))
                    })?;

                let mut proposal: Proposal =
                    bincode::deserialize(&proposal_bytes_ivec).map_err(|e| {
                        CommonError::DeserializationError(format!(
                            "Failed to deserialize proposal {}: {}",
                            proposal_id.0, e
                        ))
                    })?;

                if now > proposal.voting_deadline {
                    return Err(CommonError::InvalidInputError(format!(
                        "Voting period for proposal {} has closed.",
                        proposal_id.0
                    )));
                }
                if proposal.status != ProposalStatus::VotingOpen {
                    return Err(CommonError::InvalidInputError(format!(
                        "Proposal {} not open for voting, current status: {:?}",
                        proposal_id.0, proposal.status
                    )));
                }

                let vote = Vote {
                    voter: voter.clone(),
                    proposal_id: proposal_id.clone(),
                    option,
                    voted_at: now,
                };
                proposal.votes.insert(voter.clone(), vote.clone());

                let encoded_proposal = bincode::serialize(&proposal).map_err(|e| {
                    CommonError::SerializationError(format!(
                        "Failed to serialize updated proposal {}: {}",
                        proposal_id.0, e
                    ))
                })?;

                tree.insert(key, encoded_proposal).map_err(|e| {
                    CommonError::DatabaseError(format!(
                        "Failed to insert updated proposal {} into sled: {}",
                        proposal_id.0, e
                    ))
                })?;
                tree.flush().map_err(|e| {
                    CommonError::DatabaseError(format!(
                        "Failed to flush sled tree for vote on {}: {}",
                        proposal_id.0, e
                    ))
                })?;
            }
        }
        if let Some(store) = &self.event_store {
            store
                .lock()
                .unwrap()
                .append(&GovernanceEvent::VoteCast(Vote {
                    voter,
                    proposal_id: proposal_id.clone(),
                    option,
                    voted_at: now,
                }))?;
        }
        Ok(())
    }

    /// Fetch a proposal by ID if it exists.
    pub fn get_proposal(&self, proposal_id: &ProposalId) -> Result<Option<Proposal>, CommonError> {
        match &self.backend {
            Backend::InMemory { proposals } => Ok(proposals.get(proposal_id).cloned()),
            #[cfg(feature = "persist-sled")]
            Backend::Sled {
                db,
                proposals_tree_name,
            } => {
                let tree = db.open_tree(proposals_tree_name).map_err(|e| {
                    CommonError::DatabaseError(format!(
                        "Failed to open proposals tree for get_proposal: {}",
                        e
                    ))
                })?;
                let key = proposal_id.0.as_bytes();
                match tree.get(key) {
                    Ok(Some(proposal_bytes_ivec)) => {
                        let proposal: Proposal = bincode::deserialize(&proposal_bytes_ivec)
                            .map_err(|e| {
                                CommonError::DeserializationError(format!(
                                    "Failed to deserialize proposal {} from sled: {}",
                                    proposal_id.0, e
                                ))
                            })?;
                        Ok(Some(proposal))
                    }
                    Ok(None) => Ok(None),
                    Err(e) => Err(CommonError::DatabaseError(format!(
                        "Failed to get proposal {} from sled: {}",
                        proposal_id.0, e
                    ))),
                }
            }
        }
    }

    /// Return all currently stored proposals.
    pub fn list_proposals(&self) -> Result<Vec<Proposal>, CommonError> {
        match &self.backend {
            Backend::InMemory { proposals } => Ok(proposals.values().cloned().collect()),
            #[cfg(feature = "persist-sled")]
            Backend::Sled {
                db,
                proposals_tree_name,
            } => {
                let tree = db.open_tree(proposals_tree_name).map_err(|e| {
                    CommonError::DatabaseError(format!(
                        "Failed to open proposals tree for list_proposals: {}",
                        e
                    ))
                })?;

                let mut proposals_vec = Vec::new();
                for item in tree.iter() {
                    let (_id_bytes, proposal_bytes_ivec) = item.map_err(|e| {
                        CommonError::DatabaseError(format!(
                            "Failed to iterate over proposals in sled: {}",
                            e
                        ))
                    })?;
                    let proposal: Proposal =
                        bincode::deserialize(&proposal_bytes_ivec).map_err(|e| {
                            CommonError::DeserializationError(format!(
                                "Failed to deserialize proposal from sled iteration: {}",
                                e
                            ))
                        })?;
                    proposals_vec.push(proposal);
                }
                Ok(proposals_vec)
            }
        }
    }

    /// Adds a new member eligible to vote.
    pub fn add_member(&mut self, member: Did) {
        self.members.insert(member);
    }

    /// Removes an existing member, preventing them from voting.
    pub fn remove_member(&mut self, did: &Did) {
        self.members.remove(did);
    }

    /// Delegate `from` member's vote to `to` member.
    pub fn delegate_vote(&mut self, from: Did, to: Did) -> Result<(), CommonError> {
        if !self.members.contains(&from) || !self.members.contains(&to) {
            return Err(CommonError::InvalidInputError(
                "Both delegator and delegatee must be members".to_string(),
            ));
        }
        self.delegations.insert(from, to);
        Ok(())
    }

    /// Revoke any delegation for `from`.
    pub fn revoke_delegation(&mut self, from: Did) {
        self.delegations.remove(&from);
    }

    /// Returns a reference to the current member set.
    pub fn members(&self) -> &HashSet<Did> {
        &self.members
    }

    /// Sets the minimum number of votes required for a proposal to be valid.
    pub fn set_quorum(&mut self, quorum: usize) {
        self.quorum = quorum;
    }

    /// Sets the fraction of `Yes` votes required for acceptance.
    pub fn set_threshold(&mut self, threshold: f32) {
        self.threshold = threshold;
    }

    /// Register a callback executed when proposals are run via [`GovernanceModule::execute_proposal`].
    pub fn set_callback<F>(&mut self, cb: F)
    where
        F: ProposalCallback + 'static,
    {
        self.proposal_callbacks.push(Box::new(cb));
    }

    /// Update governance configuration
    pub fn set_config(&mut self, config: GovernanceConfig) {
        self.config = config;
    }

    /// Get current governance configuration
    pub fn config(&self) -> &GovernanceConfig {
        &self.config
    }

    /// Sponsor a proposal (add sponsor support for advancement to voting)
    pub fn sponsor_proposal(
        &mut self,
        proposal_id: &ProposalId,
        sponsor: Did,
        time_provider: &dyn TimeProvider,
    ) -> Result<(), CommonError> {
        let now = time_provider.unix_seconds();

        // Check if sponsor is a member
        if !self.members.contains(&sponsor) {
            return Err(CommonError::InvalidInputError(
                "Sponsor must be a member".to_string(),
            ));
        }

        match &mut self.backend {
            Backend::InMemory { proposals } => {
                let proposal = proposals.get_mut(proposal_id).ok_or_else(|| {
                    CommonError::ResourceNotFound(format!(
                        "Proposal with ID {} not found",
                        proposal_id.0
                    ))
                })?;

                if proposal.status != ProposalStatus::PendingSponsorship {
                    return Err(CommonError::InvalidInputError(format!(
                        "Proposal {} is not pending sponsorship",
                        proposal_id.0
                    )));
                }

                proposal.sponsorship.add_sponsor(sponsor, now, self.config.min_sponsors);

                // If sponsorship requirement is met, advance to Deliberation
                if proposal.sponsorship.has_sufficient_sponsors(self.config.min_sponsors) {
                    proposal.status = ProposalStatus::Deliberation;
                    if let Some(store) = &self.event_store {
                        store
                            .lock()
                            .unwrap()
                            .append(&GovernanceEvent::StatusUpdated(
                                proposal_id.clone(),
                                ProposalStatus::Deliberation,
                            ))?;
                    }
                }

                Ok(())
            }
            #[cfg(feature = "persist-sled")]
            Backend::Sled { .. } => {
                // TODO: Implement sled version
                Err(CommonError::NotImplemented(
                    "Sled backend not implemented for sponsor_proposal".to_string(),
                ))
            }
        }
    }

    /// Apply a veto to an accepted proposal during grace period
    pub fn veto_proposal(
        &mut self,
        proposal_id: &ProposalId,
        vetoed_by: Did,
        reason: String,
        time_provider: &dyn TimeProvider,
    ) -> Result<(), CommonError> {
        let now = time_provider.unix_seconds();

        // Check if vetoed_by has veto powers
        if !self.config.veto_members.contains(&vetoed_by) {
            return Err(CommonError::InvalidInputError(
                "Member does not have veto powers".to_string(),
            ));
        }

        match &mut self.backend {
            Backend::InMemory { proposals } => {
                let proposal = proposals.get_mut(proposal_id).ok_or_else(|| {
                    CommonError::ResourceNotFound(format!(
                        "Proposal with ID {} not found",
                        proposal_id.0
                    ))
                })?;

                // Can only veto accepted proposals during grace period
                if proposal.status != ProposalStatus::AcceptedTimelock {
                    return Err(CommonError::InvalidInputError(format!(
                        "Proposal {} cannot be vetoed in current status: {:?}",
                        proposal_id.0, proposal.status
                    )));
                }

                // Check if still within grace period
                if let Some(accepted_at) = proposal.accepted_at {
                    if now > accepted_at + self.config.veto_grace_period_secs {
                        return Err(CommonError::InvalidInputError(
                            "Veto grace period has expired".to_string(),
                        ));
                    }
                } else {
                    return Err(CommonError::InvalidInputError(
                        "Proposal accepted time not recorded".to_string(),
                    ));
                }

                proposal.veto = Some(ProposalVeto {
                    vetoed_by,
                    vetoed_at: now,
                    reason,
                });
                proposal.status = ProposalStatus::Vetoed;

                if let Some(store) = &self.event_store {
                    store
                        .lock()
                        .unwrap()
                        .append(&GovernanceEvent::StatusUpdated(
                            proposal_id.clone(),
                            ProposalStatus::Vetoed,
                        ))?;
                }

                Ok(())
            }
            #[cfg(feature = "persist-sled")]
            Backend::Sled { .. } => {
                // TODO: Implement sled version
                Err(CommonError::NotImplemented(
                    "Sled backend not implemented for veto_proposal".to_string(),
                ))
            }
        }
    }

    /// Process time-locked proposals that are ready for execution
    pub fn process_timelocked_proposals(
        &mut self,
        time_provider: &dyn TimeProvider,
    ) -> Result<Vec<ProposalId>, CommonError> {
        let now = time_provider.unix_seconds();
        let mut ready_proposals = Vec::new();

        match &mut self.backend {
            Backend::InMemory { proposals } => {
                for (id, proposal) in proposals.iter_mut() {
                    if proposal.status == ProposalStatus::AcceptedTimelock {
                        if let Some(accepted_at) = proposal.accepted_at {
                            let timelock_delay = proposal.timelock_delay.unwrap_or(self.config.timelock_delay_secs);
                            
                            // Check if time-lock period has elapsed and grace period for vetos has passed
                            let timelock_complete = now >= accepted_at + timelock_delay;
                            let grace_complete = now >= accepted_at + self.config.veto_grace_period_secs;
                            
                            if timelock_complete && grace_complete {
                                proposal.status = ProposalStatus::Accepted;
                                ready_proposals.push(id.clone());

                                if let Some(store) = &self.event_store {
                                    let _ = store
                                        .lock()
                                        .unwrap()
                                        .append(&GovernanceEvent::StatusUpdated(
                                            id.clone(),
                                            ProposalStatus::Accepted,
                                        ));
                                }
                            }
                        }
                    }
                }
                Ok(ready_proposals)
            }
            #[cfg(feature = "persist-sled")]
            Backend::Sled { .. } => {
                // TODO: Implement sled version
                Err(CommonError::NotImplemented(
                    "Sled backend not implemented for process_timelocked_proposals".to_string(),
                ))
            }
        }
    }

    /// Inserts a proposal that originated from another node into the governance module.
    pub fn insert_external_proposal(&mut self, proposal: Proposal) -> Result<(), CommonError> {
        match &mut self.backend {
            Backend::InMemory { proposals } => {
                if proposals.contains_key(&proposal.id) {
                    return Err(CommonError::InvalidInputError(format!(
                        "Proposal with ID {} already exists",
                        proposal.id.0
                    )));
                }
                proposals.insert(proposal.id.clone(), proposal);
                Ok(())
            }
            #[cfg(feature = "persist-sled")]
            Backend::Sled {
                db,
                proposals_tree_name,
            } => {
                let tree = db.open_tree(proposals_tree_name).map_err(|e| {
                    CommonError::DatabaseError(format!("Failed to open proposals tree: {}", e))
                })?;
                let key = proposal.id.0.as_bytes();
                if tree.contains_key(key).map_err(|e| {
                    CommonError::DatabaseError(format!(
                        "Failed to check key existence in proposals tree: {}",
                        e
                    ))
                })? {
                    return Err(CommonError::InvalidInputError(format!(
                        "Proposal with ID {} already exists",
                        proposal.id.0
                    )));
                }
                let encoded_proposal = bincode::serialize(&proposal).map_err(|e| {
                    CommonError::SerializationError(format!(
                        "Failed to serialize proposal {}: {}",
                        proposal.id.0, e
                    ))
                })?;
                tree.insert(key, encoded_proposal).map_err(|e| {
                    CommonError::DatabaseError(format!(
                        "Failed to insert proposal {} into sled: {}",
                        proposal.id.0, e
                    ))
                })?;
                tree.flush().map_err(|e| {
                    CommonError::DatabaseError(format!(
                        "Failed to flush sled tree for external proposal {}: {}",
                        proposal.id.0, e
                    ))
                })?;
                Ok(())
            }
        }
    }

    /// Inserts a vote that originated from another node.
    pub fn insert_external_vote(&mut self, vote: Vote) -> Result<(), CommonError> {
        match &mut self.backend {
            Backend::InMemory { proposals } => {
                if let Some(prop) = proposals.get_mut(&vote.proposal_id) {
                    prop.votes.insert(vote.voter.clone(), vote);
                    Ok(())
                } else {
                    Err(CommonError::ResourceNotFound(format!(
                        "Proposal with ID {} not found for external vote",
                        vote.proposal_id.0
                    )))
                }
            }
            #[cfg(feature = "persist-sled")]
            Backend::Sled {
                db,
                proposals_tree_name,
            } => {
                let tree = db.open_tree(proposals_tree_name).map_err(|e| {
                    CommonError::DatabaseError(format!("Failed to open proposals tree: {}", e))
                })?;
                let key = vote.proposal_id.0.as_bytes();
                let proposal_bytes = tree
                    .get(key)
                    .map_err(|e| {
                        CommonError::DatabaseError(format!(
                            "Failed to get proposal {}: {}",
                            vote.proposal_id.0, e
                        ))
                    })?
                    .ok_or_else(|| {
                        CommonError::ResourceNotFound(format!(
                            "Proposal with ID {} not found for external vote",
                            vote.proposal_id.0
                        ))
                    })?;
                let mut proposal: Proposal =
                    bincode::deserialize(&proposal_bytes).map_err(|e| {
                        CommonError::DeserializationError(format!(
                            "Failed to deserialize proposal {}: {}",
                            vote.proposal_id.0, e
                        ))
                    })?;
                proposal.votes.insert(vote.voter.clone(), vote.clone());
                let encoded = bincode::serialize(&proposal).map_err(|e| {
                    CommonError::SerializationError(format!(
                        "Failed to serialize proposal {}: {}",
                        proposal.id.0, e
                    ))
                })?;
                tree.insert(key, encoded).map_err(|e| {
                    CommonError::DatabaseError(format!(
                        "Failed to persist proposal {}: {}",
                        proposal.id.0, e
                    ))
                })?;
                tree.flush().map_err(|e| {
                    CommonError::DatabaseError(format!(
                        "Failed to flush sled tree for external vote {}: {}",
                        proposal.id.0, e
                    ))
                })?;
                Ok(())
            }
        }
    }

    /// Counts yes/no/abstain votes for a proposal, considering only current members.
    pub fn tally_votes(&self, proposal: &Proposal) -> (usize, usize, usize) {
        Self::tally_votes_static(&self.members, &self.delegations, proposal)
    }

    fn tally_votes_static(
        members: &HashSet<Did>,
        delegations: &HashMap<Did, Did>,
        proposal: &Proposal,
    ) -> (usize, usize, usize) {
        let mut yes = 0;
        let mut no = 0;
        let mut abstain = 0;
        for member in members {
            let mut option = proposal.votes.get(member).map(|v| v.option);
            if option.is_none() {
                if let Some(delegate) = delegations.get(member) {
                    option = proposal.votes.get(delegate).map(|v| v.option);
                }
            }
            match option {
                Some(VoteOption::Yes) => yes += 1,
                Some(VoteOption::No) => no += 1,
                Some(VoteOption::Abstain) => abstain += 1,
                None => {}
            }
        }
        (yes, no, abstain)
    }

    /// Mark any proposals past their deadline as `Rejected` without tallying votes.
    pub fn expire_proposals(&mut self, now: u64) -> Result<(), CommonError> {
        match &mut self.backend {
            Backend::InMemory { proposals } => {
                for proposal in proposals.values_mut() {
                    if (proposal.status == ProposalStatus::VotingOpen
                        || proposal.status == ProposalStatus::Deliberation)
                        && proposal.voting_deadline <= now
                    {
                        proposal.status = ProposalStatus::Rejected;
                    }
                }
                Ok(())
            }
            #[cfg(feature = "persist-sled")]
            Backend::Sled {
                db,
                proposals_tree_name,
            } => {
                let tree = db.open_tree(proposals_tree_name).map_err(|e| {
                    CommonError::DatabaseError(format!(
                        "Failed to open proposals tree for expire_proposals: {}",
                        e
                    ))
                })?;
                let mut updates = Vec::new();
                for item in tree.iter() {
                    let (key, val) = item.map_err(|e| {
                        CommonError::DatabaseError(format!(
                            "Failed to iterate proposals tree: {}",
                            e
                        ))
                    })?;
                    let mut prop: Proposal = bincode::deserialize(&val).map_err(|e| {
                        CommonError::DeserializationError(format!(
                            "Failed to deserialize proposal: {}",
                            e
                        ))
                    })?;
                    if (prop.status == ProposalStatus::VotingOpen
                        || prop.status == ProposalStatus::Deliberation)
                        && prop.voting_deadline <= now
                    {
                        prop.status = ProposalStatus::Rejected;
                        updates.push((key, prop));
                    }
                }
                for (key, prop) in updates {
                    let encoded = bincode::serialize(&prop).map_err(|e| {
                        CommonError::SerializationError(format!(
                            "Failed to serialize expired proposal {}: {}",
                            prop.id.0, e
                        ))
                    })?;
                    tree.insert(key, encoded).map_err(|e| {
                        CommonError::DatabaseError(format!(
                            "Failed to persist expired proposal: {}",
                            e
                        ))
                    })?;
                }
                tree.flush().map_err(|e| {
                    CommonError::DatabaseError(format!(
                        "Failed to flush sled tree for expire_proposals: {}",
                        e
                    ))
                })?;
                Ok(())
            }
        }
    }

    /// Automatically close all proposals whose voting deadlines have passed.
    pub fn close_expired_proposals(
        &mut self,
        time_provider: &dyn TimeProvider,
    ) -> Result<(), CommonError> {
        let now = time_provider.unix_seconds();
        match &mut self.backend {
            Backend::InMemory { proposals } => {
                let expired_voting: Vec<ProposalId> = proposals
                    .values()
                    .filter(|p| p.voting_deadline <= now && p.status == ProposalStatus::VotingOpen)
                    .map(|p| p.id.clone())
                    .collect();
                let expired_deliberation: Vec<ProposalId> = proposals
                    .values()
                    .filter(|p| {
                        p.voting_deadline <= now && p.status == ProposalStatus::Deliberation
                    })
                    .map(|p| p.id.clone())
                    .collect();

                // For deliberation proposals, directly mark as rejected
                for id in expired_deliberation {
                    if let Some(proposal) = proposals.get_mut(&id) {
                        proposal.status = ProposalStatus::Rejected;
                        if let Some(store) = &self.event_store {
                            let _ = store
                                .lock()
                                .unwrap()
                                .append(&GovernanceEvent::StatusUpdated(
                                    id.clone(),
                                    ProposalStatus::Rejected,
                                ));
                        }
                    }
                }

                // For voting proposals, properly close with vote tallying
                for id in expired_voting {
                    let _ = self.close_voting_period(&id, time_provider)?;
                }
                Ok(())
            }
            #[cfg(feature = "persist-sled")]
            Backend::Sled {
                db,
                proposals_tree_name,
            } => {
                let tree = db.open_tree(proposals_tree_name).map_err(|e| {
                    CommonError::DatabaseError(format!(
                        "Failed to open proposals tree for close_expired_proposals: {}",
                        e
                    ))
                })?;
                let mut expired_voting = Vec::new();
                let mut expired_deliberation = Vec::new();

                for item in tree.iter() {
                    let (key, val) = item.map_err(|e| {
                        CommonError::DatabaseError(format!(
                            "Failed to iterate proposals tree: {}",
                            e
                        ))
                    })?;
                    let prop: Proposal = bincode::deserialize(&val).map_err(|e| {
                        CommonError::DeserializationError(format!(
                            "Failed to deserialize proposal: {}",
                            e
                        ))
                    })?;
                    if prop.voting_deadline <= now {
                        let id_str = String::from_utf8(key.to_vec()).map_err(|e| {
                            CommonError::DeserializationError(format!(
                                "Invalid UTF-8 in proposal key: {}",
                                e
                            ))
                        })?;
                        let proposal_id = ProposalId(id_str);

                        if prop.status == ProposalStatus::VotingOpen {
                            expired_voting.push(proposal_id);
                        } else if prop.status == ProposalStatus::Deliberation {
                            expired_deliberation.push(proposal_id);
                        }
                    }
                }

                // Handle deliberation proposals - mark as rejected directly
                for id in expired_deliberation {
                    let key = id.0.as_bytes();
                    let proposal_bytes = tree
                        .get(key)
                        .map_err(|e| {
                            CommonError::DatabaseError(format!(
                                "Failed to get proposal {} from sled: {}",
                                id.0, e
                            ))
                        })?
                        .ok_or_else(|| {
                            CommonError::ResourceNotFound(format!(
                                "Proposal with ID {} not found for expiration",
                                id.0
                            ))
                        })?;
                    let mut proposal: Proposal =
                        bincode::deserialize(&proposal_bytes).map_err(|e| {
                            CommonError::DeserializationError(format!(
                                "Failed to deserialize proposal {}: {}",
                                id.0, e
                            ))
                        })?;

                    proposal.status = ProposalStatus::Rejected;
                    let encoded = bincode::serialize(&proposal).map_err(|e| {
                        CommonError::SerializationError(format!(
                            "Failed to serialize expired proposal {}: {}",
                            id.0, e
                        ))
                    })?;
                    tree.insert(key, encoded).map_err(|e| {
                        CommonError::DatabaseError(format!(
                            "Failed to persist expired proposal {}: {}",
                            id.0, e
                        ))
                    })?;

                    if let Some(store) = &self.event_store {
                        let _ = store
                            .lock()
                            .unwrap()
                            .append(&GovernanceEvent::StatusUpdated(
                                id.clone(),
                                ProposalStatus::Rejected,
                            ));
                    }
                }

                tree.flush().map_err(|e| {
                    CommonError::DatabaseError(format!(
                        "Failed to flush sled tree for expired deliberation proposals: {}",
                        e
                    ))
                })?;
                drop(tree);

                // Handle voting proposals - properly close with vote tallying
                for id in expired_voting {
                    let _ = self.close_voting_period(&id, time_provider)?;
                }
                Ok(())
            }
        }
    }

    /// Finalizes voting on a proposal and updates its status based on quorum and threshold.
    pub fn close_voting_period(
        &mut self,
        proposal_id: &ProposalId,
        time_provider: &dyn TimeProvider,
    ) -> Result<(ProposalStatus, (usize, usize, usize)), CommonError> {
        let now = time_provider.unix_seconds();

        // expire any proposals that have passed their deadline before closing
        self.expire_proposals(now)?;

        match &mut self.backend {
            Backend::InMemory { proposals } => {
                let proposal = proposals.get_mut(proposal_id).ok_or_else(|| {
                    CommonError::ResourceNotFound(format!(
                        "Proposal with ID {} not found for closing",
                        proposal_id.0
                    ))
                })?;
                // Allow early closing of the voting period
                if proposal.status != ProposalStatus::VotingOpen {
                    return Ok((proposal.status.clone(), (0, 0, 0)));
                }
                let members = self.members.clone();
                let delegations = self.delegations.clone();
                let (yes, no, abstain) = Self::tally_votes_static(&members, &delegations, proposal);
                let total = yes + no + abstain;
                let quorum = proposal.quorum.unwrap_or(self.quorum);
                let threshold = proposal.threshold.unwrap_or(self.threshold);
                if total < quorum {
                    proposal.status = ProposalStatus::Rejected;
                } else if (yes as f32) >= (total as f32 * threshold) {
                    // Handle time-lock logic
                    let timelock_delay = proposal.timelock_delay.unwrap_or(self.config.timelock_delay_secs);
                    if timelock_delay > 0 || self.config.veto_grace_period_secs > 0 {
                        proposal.status = ProposalStatus::AcceptedTimelock;
                        proposal.accepted_at = Some(now);
                    } else {
                        proposal.status = ProposalStatus::Accepted;
                    }
                } else {
                    proposal.status = ProposalStatus::Rejected;
                }
                if let Some(store) = &self.event_store {
                    store
                        .lock()
                        .unwrap()
                        .append(&GovernanceEvent::StatusUpdated(
                            proposal_id.clone(),
                            proposal.status.clone(),
                        ))?;
                }
                Ok((proposal.status.clone(), (yes, no, abstain)))
            }
            #[cfg(feature = "persist-sled")]
            Backend::Sled {
                db,
                proposals_tree_name,
            } => {
                let tree = db.open_tree(proposals_tree_name).map_err(|e| {
                    CommonError::DatabaseError(format!(
                        "Failed to open proposals tree for close_voting_period: {}",
                        e
                    ))
                })?;
                let key = proposal_id.0.as_bytes();
                let proposal_bytes = tree
                    .get(key)
                    .map_err(|e| {
                        CommonError::DatabaseError(format!(
                            "Failed to get proposal {} from sled: {}",
                            proposal_id.0, e
                        ))
                    })?
                    .ok_or_else(|| {
                        CommonError::ResourceNotFound(format!(
                            "Proposal with ID {} not found for closing",
                            proposal_id.0
                        ))
                    })?;
                let mut proposal: Proposal =
                    bincode::deserialize(&proposal_bytes).map_err(|e| {
                        CommonError::DeserializationError(format!(
                            "Failed to deserialize proposal {}: {}",
                            proposal_id.0, e
                        ))
                    })?;
                // Allow early closing of the voting period
                if proposal.status != ProposalStatus::VotingOpen {
                    return Ok((proposal.status.clone(), (0, 0, 0)));
                }
                let members = self.members.clone();
                let delegations = self.delegations.clone();
                let (yes, no, abstain) =
                    Self::tally_votes_static(&members, &delegations, &proposal);
                let total = yes + no + abstain;
                let quorum = proposal.quorum.unwrap_or(self.quorum);
                let threshold = proposal.threshold.unwrap_or(self.threshold);
                if total < quorum {
                    proposal.status = ProposalStatus::Rejected;
                } else if (yes as f32) >= (total as f32 * threshold) {
                    // Handle time-lock logic
                    let timelock_delay = proposal.timelock_delay.unwrap_or(self.config.timelock_delay_secs);
                    if timelock_delay > 0 || self.config.veto_grace_period_secs > 0 {
                        proposal.status = ProposalStatus::AcceptedTimelock;
                        proposal.accepted_at = Some(now);
                    } else {
                        proposal.status = ProposalStatus::Accepted;
                    }
                } else {
                    proposal.status = ProposalStatus::Rejected;
                }
                let encoded = bincode::serialize(&proposal).map_err(|e| {
                    CommonError::SerializationError(format!(
                        "Failed to serialize updated proposal {}: {}",
                        proposal_id.0, e
                    ))
                })?;
                tree.insert(key, encoded).map_err(|e| {
                    CommonError::DatabaseError(format!(
                        "Failed to persist updated proposal {}: {}",
                        proposal_id.0, e
                    ))
                })?;
                tree.flush().map_err(|e| {
                    CommonError::DatabaseError(format!(
                        "Failed to flush sled tree for proposal {} close: {}",
                        proposal_id.0, e
                    ))
                })?;
                if let Some(store) = &self.event_store {
                    store
                        .lock()
                        .unwrap()
                        .append(&GovernanceEvent::StatusUpdated(
                            proposal_id.clone(),
                            proposal.status.clone(),
                        ))?;
                }
                Ok((proposal.status, (yes, no, abstain)))
            }
        }
    }

    /// Executes an accepted proposal. New members are added when executed.
    pub fn execute_proposal(&mut self, proposal_id: &ProposalId) -> Result<(), CommonError> {
        metrics::EXECUTE_PROPOSAL_CALLS.inc();
        match &mut self.backend {
            Backend::InMemory { proposals } => {
                let proposal = proposals.get_mut(proposal_id).ok_or_else(|| {
                    CommonError::ResourceNotFound(format!(
                        "Proposal with ID {} not found for execution",
                        proposal_id.0
                    ))
                })?;
                if proposal.status != ProposalStatus::Accepted {
                    return Err(CommonError::InvalidInputError(format!(
                        "Proposal {} not accepted",
                        proposal_id.0
                    )));
                }
                match &proposal.proposal_type {
                    ProposalType::NewMemberInvitation(did) => {
                        self.members.insert(did.clone());
                    }
                    ProposalType::RemoveMember(did) => {
                        self.members.remove(did);
                    }
                    _ => {}
                }
                for cb in &self.proposal_callbacks {
                    if let Err(e) = cb.on_execute(proposal) {
                        proposal.status = ProposalStatus::Failed;
                        if let Some(store) = &self.event_store {
                            let _ = store
                                .lock()
                                .unwrap()
                                .append(&GovernanceEvent::StatusUpdated(
                                    proposal_id.clone(),
                                    ProposalStatus::Failed,
                                ));
                        }
                        return Err(e);
                    }
                }
                proposal.status = ProposalStatus::Executed;
                if let Some(store) = &self.event_store {
                    store
                        .lock()
                        .unwrap()
                        .append(&GovernanceEvent::StatusUpdated(
                            proposal_id.clone(),
                            ProposalStatus::Executed,
                        ))?;
                }
                Ok(())
            }
            #[cfg(feature = "persist-sled")]
            Backend::Sled {
                db,
                proposals_tree_name,
            } => {
                let tree = db.open_tree(proposals_tree_name).map_err(|e| {
                    CommonError::DatabaseError(format!(
                        "Failed to open proposals tree for execute_proposal: {}",
                        e
                    ))
                })?;
                let key = proposal_id.0.as_bytes();
                let proposal_bytes = tree
                    .get(key)
                    .map_err(|e| {
                        CommonError::DatabaseError(format!(
                            "Failed to get proposal {} from sled: {}",
                            proposal_id.0, e
                        ))
                    })?
                    .ok_or_else(|| {
                        CommonError::ResourceNotFound(format!(
                            "Proposal with ID {} not found for execution",
                            proposal_id.0
                        ))
                    })?;
                let mut proposal: Proposal =
                    bincode::deserialize(&proposal_bytes).map_err(|e| {
                        CommonError::DeserializationError(format!(
                            "Failed to deserialize proposal {}: {}",
                            proposal_id.0, e
                        ))
                    })?;
                if proposal.status != ProposalStatus::Accepted {
                    return Err(CommonError::InvalidInputError(format!(
                        "Proposal {} not accepted",
                        proposal_id.0
                    )));
                }
                match &proposal.proposal_type {
                    ProposalType::NewMemberInvitation(did) => {
                        self.members.insert(did.clone());
                    }
                    ProposalType::RemoveMember(did) => {
                        self.members.remove(did);
                    }
                    _ => {}
                }
                for cb in &self.proposal_callbacks {
                    if let Err(e) = cb.on_execute(&proposal) {
                        proposal.status = ProposalStatus::Failed;
                        let encoded = bincode::serialize(&proposal).map_err(|e| {
                            CommonError::SerializationError(format!(
                                "Failed to serialize failed proposal {}: {}",
                                proposal_id.0, e
                            ))
                        })?;
                        tree.insert(key, encoded).map_err(|e| {
                            CommonError::DatabaseError(format!(
                                "Failed to persist failed proposal {}: {}",
                                proposal_id.0, e
                            ))
                        })?;
                        tree.flush().map_err(|e| {
                            CommonError::DatabaseError(format!(
                                "Failed to flush sled tree for failed proposal {}: {}",
                                proposal_id.0, e
                            ))
                        })?;
                        if let Some(store) = &self.event_store {
                            let _ = store
                                .lock()
                                .unwrap()
                                .append(&GovernanceEvent::StatusUpdated(
                                    proposal_id.clone(),
                                    ProposalStatus::Failed,
                                ));
                        }
                        return Err(e);
                    }
                }
                proposal.status = ProposalStatus::Executed;
                let encoded = bincode::serialize(&proposal).map_err(|e| {
                    CommonError::SerializationError(format!(
                        "Failed to serialize executed proposal {}: {}",
                        proposal_id.0, e
                    ))
                })?;
                tree.insert(key, encoded).map_err(|e| {
                    CommonError::DatabaseError(format!(
                        "Failed to persist executed proposal {}: {}",
                        proposal_id.0, e
                    ))
                })?;
                tree.flush().map_err(|e| {
                    CommonError::DatabaseError(format!(
                        "Failed to flush sled tree for executed proposal {}: {}",
                        proposal_id.0, e
                    ))
                })?;
                if let Some(store) = &self.event_store {
                    store
                        .lock()
                        .unwrap()
                        .append(&GovernanceEvent::StatusUpdated(
                            proposal_id.clone(),
                            ProposalStatus::Executed,
                        ))?;
                }
                Ok(())
            }
        }
    }

    pub fn event_store(
        &self,
    ) -> Option<&std::sync::Mutex<Box<dyn icn_eventstore::EventStore<GovernanceEvent>>>> {
        self.event_store.as_ref()
    }
}

impl fmt::Debug for GovernanceModule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GovernanceModule")
            .field("backend", &self.backend)
            .field("members", &self.members)
            .field("delegations", &self.delegations)
            .field("quorum", &self.quorum)
            .field("threshold", &self.threshold)
            .finish()
    }
}

/// Request federation data synchronization from a peer.
///
/// This uses the provided [`NetworkService`] to send a
/// [`MessagePayload::FederationSyncRequest`] to `target_peer`.
#[cfg(feature = "federation")]
pub async fn request_federation_sync(
    service: &dyn NetworkService,
    target_peer: &PeerId,
    since_timestamp: Option<u64>,
) -> Result<(), CommonError> {
    let payload = FederationSyncRequestMessage {
        federation_id: "default".to_string(),
        since_timestamp,
        sync_types: vec![],
    };

    let msg = ProtocolMessage::new(
        MessagePayload::FederationSyncRequest(payload),
        Did::default(),
        None,
    );
    service
        .send_message(target_peer, msg)
        .await
        .map_err(map_mesh_err)
}

#[cfg(feature = "federation")]
fn map_mesh_err(err: MeshNetworkError) -> CommonError {
    match err {
        MeshNetworkError::PeerNotFound(e) => CommonError::PeerNotFound(e),
        MeshNetworkError::SendFailure(e) => CommonError::MessageSendError(e),
        MeshNetworkError::Timeout(e) => CommonError::TimeoutError(e),
        MeshNetworkError::InvalidInput(e) => CommonError::InvalidInputError(e),
        MeshNetworkError::Common(e) => e,
        other => CommonError::NetworkError(other.to_string()),
    }
}

/// Placeholder function demonstrating use of common types for governance operations.
pub fn submit_governance_proposal(
    info: &NodeInfo,
    proposal_id: u32,
) -> Result<String, CommonError> {
    Ok(format!(
        "Node {} submitted governance proposal {}",
        info.name, proposal_id
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_submit_governance_proposal() {
        let node_info = NodeInfo {
            name: "TestNode".to_string(),
            version: "0.2.0-beta".to_string(),
            status_message: "Testing".to_string(),
        };
        let result = submit_governance_proposal(&node_info, 123);
        assert!(result.is_ok());
    }

    #[test]
    fn test_expire_deliberation_proposals() {
        let mut gov = GovernanceModule::new();
        let proposer = Did::default();

        // Submit a proposal that will expire while in Deliberation status
        let now = 1000u64; // Use fixed timestamp for deterministic testing
        let time_provider = FixedTimeProvider::new(now);

        let submission = ProposalSubmission {
            proposer: proposer.clone(),
            proposal_type: ProposalType::GenericText("Test proposal".to_string()),
            description: "A test proposal".to_string(),
            duration_secs: 10, // Short duration for testing
            quorum: None,
            threshold: None,
            content_cid: None,
            timelock_delay: None,
        };

        let proposal_id = gov.submit_proposal(submission, &time_provider).unwrap();

        // Verify proposal is in Deliberation status
        let proposal = gov.get_proposal(&proposal_id).unwrap().unwrap();
        assert_eq!(proposal.status, ProposalStatus::Deliberation);

        // Simulate time passing beyond the deadline
        let future_time = now + 20; // Past the 10-second deadline

        // Call expire_proposals - this should mark the Deliberation proposal as Rejected
        gov.expire_proposals(future_time).unwrap();

        // Verify the proposal is now Rejected
        let proposal = gov.get_proposal(&proposal_id).unwrap().unwrap();
        assert_eq!(proposal.status, ProposalStatus::Rejected);
    }

    #[test]
    fn test_close_expired_deliberation_proposals() {
        let mut gov = GovernanceModule::new();
        let proposer = Did::default();

        // Submit a proposal that will expire while in Deliberation status
        let now = 1000u64; // Use fixed timestamp for deterministic testing
        let time_provider = FixedTimeProvider::new(now);

        let submission = ProposalSubmission {
            proposer: proposer.clone(),
            proposal_type: ProposalType::GenericText("Test proposal".to_string()),
            description: "A test proposal".to_string(),
            duration_secs: 10, // Short duration for testing
            quorum: None,
            threshold: None,
            content_cid: None,
            timelock_delay: None,
        };

        let proposal_id = gov.submit_proposal(submission, &time_provider).unwrap();

        // Verify proposal is in Deliberation status
        let proposal = gov.get_proposal(&proposal_id).unwrap().unwrap();
        assert_eq!(proposal.status, ProposalStatus::Deliberation);

        // Simulate time passing beyond the deadline by manipulating the proposal's deadline
        // We need to update the proposal's voting_deadline to be in the past
        if let Backend::InMemory { proposals } = &mut gov.backend {
            if let Some(prop) = proposals.get_mut(&proposal_id) {
                prop.voting_deadline = now - 10; // Set deadline to past
            }
        }

        // Call close_expired_proposals - this should handle the Deliberation proposal
        gov.close_expired_proposals(&time_provider).unwrap();

        // Verify the proposal is now Rejected
        let proposal = gov.get_proposal(&proposal_id).unwrap().unwrap();
        assert_eq!(proposal.status, ProposalStatus::Rejected);
    }
}
