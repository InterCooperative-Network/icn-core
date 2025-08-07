# InterCooperative Network Governance Protocol
## Definitive Specification

---

## Executive Summary

The ICN Governance Protocol implements **true democratic decision-making** where voting power derives from membership, not wealth. Unlike token-weighted DAOs that enable plutocratic capture, ICN ensures one member equals one vote, regardless of economic holdings. This protocol defines how proposals are created, debated, voted upon, and executed across different organizational scopes—from local cooperatives to global federations.

Every governance action creates an immutable DAG record, ensuring transparency and auditability. The system supports various decision-making models (consensus, majority, supermajority) while maintaining protection against Sybil attacks through membership verification and nominal mana fees for anti-spam protection.

---

## 0 · Scope and Implementation Alignment (Normative)

### 0.1 Scopes & Models
- Local org proposals and voting; federation-level proposals in basic form
- Voting models implemented: simple majority; ranked choice is experimental

### 0.2 Operations
- Create proposal, open/close voting, cast vote, tally, expire
- Quorum/thresholds configurable per proposal type

### 0.3 Persistence & Audit
- Proposals and votes recorded to DAG; basic indexing available

### 0.4 Pending Extensions
- Liquid democracy/delegation at scale
- Working groups with scoped permissions and nested scopes
- Privacy-preserving voting as default (ZK tally)
- Cross-federation proposal routing with conflict resolution

### 0.5 Mappings
- Crates: `icn-governance`, `icn-api::governance_trait`, DAG write via `icn-dag`

---

## 1. Core Design Principles

### 1.1 Democratic Equality
- **One member, one vote** - no token weighting
- Membership is earned through participation, not purchased
- Economic inequality never translates to political inequality

### 1.2 Subsidiarity
- Decisions made at the most local level possible
- Federation-level governance only for cross-org matters
- Local autonomy preserved within global coherence

### 1.3 Transparency & Accountability
- All proposals, votes, and outcomes recorded in DAG
- Voting records are public (with optional privacy)
- Clear audit trail for all governance actions

### 1.4 Inclusive Participation
- Mana fees waivable for members below threshold
- Multiple voting methods (direct, delegated, liquid)
- Accessibility features for diverse participation

---

## 2. Organizational Governance Structures

### 2.1 Governance Scopes

```rust
pub enum GovernanceScope {
    // Local organization governance
    Local {
        org_id: OrganizationId,
        org_type: OrganizationType,
        members: HashSet<DID>,
    },
    
    // Federation-level governance
    Federation {
        fed_id: FederationId,
        member_orgs: Vec<OrganizationId>,
        delegates: HashMap<OrganizationId, Vec<DID>>,
    },
    
    // Global network governance
    Global {
        all_federations: Vec<FederationId>,
        global_delegates: Vec<DID>,
    },
    
    // Special-purpose governance
    WorkingGroup {
        group_id: WorkingGroupId,
        mandate: String,
        members: HashSet<DID>,
        parent_scope: Box<GovernanceScope>,
    },
}

pub enum OrganizationType {
    Cooperative {
        worker_members: HashSet<DID>,
        consumer_members: Option<HashSet<DID>>,
        governance_model: CoopGovernanceModel,
    },
    
    Community {
        residents: HashSet<DID>,
        governance_model: CommunityGovernanceModel,
    },
    
    Federation {
        member_organizations: Vec<OrganizationId>,
        charter: FederationCharter,
    },
}

pub enum GovernanceModel {
    // Different decision-making models
    SimpleMajority,           // >50%
    SuperMajority(f64),       // Custom threshold (e.g., 67%)
    Consensus,                // 100% agreement
    Sociocracy,              // Consent-based (no objections)
    LiquidDemocracy,         // Delegative democracy
    Futarchy,                // Prediction market-based
    Random,                  // Sortition/lottery
}
```

### 2.2 Membership & Voting Rights

```rust
pub struct Membership {
    member: DID,
    organization: OrganizationId,
    
    // Membership credentials
    credential: MembershipCredential,
    issued_at: Timestamp,
    issued_by: DID,
    
    // Voting rights
    voting_status: VotingStatus,
    voting_weight: f64,  // Always 1.0 for equal voting
    
    // Delegation (optional)
    delegation: Option<Delegation>,
    
    // Standing
    good_standing: bool,
    suspension: Option<Suspension>,
}

pub enum VotingStatus {
    Active,                    // Can vote
    Probationary,             // New member, limited voting
    Suspended,                // Temporarily cannot vote
    Emeritus,                 // Honorary, advisory only
}

pub struct MembershipCredential {
    // Cryptographic proof of membership
    credential_type: CredentialType,
    claims: HashMap<String, Value>,
    signature: Signature,
    revocation_id: Option<RevocationId>,
    
    // Soul-bound - cannot be transferred
    non_transferable: bool,  // Always true
}

impl MembershipVerification {
    pub fn verify_voting_rights(&self, did: &DID, org: &OrganizationId) -> Result<bool> {
        // 1. Check membership credential
        let credential = get_membership_credential(did, org)?;
        if !verify_credential(&credential)? {
            return Ok(false);
        }
        
        // 2. Check not revoked
        if is_revoked(&credential)? {
            return Ok(false);
        }
        
        // 3. Check good standing
        let membership = get_membership(did, org)?;
        if !membership.good_standing {
            return Ok(false);
        }
        
        // 4. Check voting status
        Ok(matches!(membership.voting_status, VotingStatus::Active))
    }
}
```

---

## 3. Proposal System

### 3.1 Proposal Structure

```rust
pub struct Proposal {
    // Identity
    id: ProposalId,
    proposer: DID,
    co_sponsors: Vec<DID>,        // Optional co-sponsors
    
    // Content
    title: String,
    description: String,
    rationale: String,
    impact_assessment: Option<ImpactAssessment>,
    
    // Classification
    category: ProposalCategory,
    scope: GovernanceScope,
    urgency: UrgencyLevel,
    
    // Actions to execute if passed
    actions: Vec<ProposalAction>,
    
    // Voting parameters
    voting_method: VotingMethod,
    quorum: QuorumRequirement,
    threshold: PassThreshold,
    
    // Timeline
    created_at: Timestamp,
    discussion_ends: Timestamp,
    voting_starts: Timestamp,
    voting_ends: Timestamp,
    grace_period_ends: Option<Timestamp>,  // For time-locks
    
    // State
    state: ProposalState,
    result: Option<ProposalResult>,
}

pub enum ProposalCategory {
    // Economic decisions
    Economic {
        budget_impact: Option<Mana>,
        token_changes: Option<TokenChanges>,
        resource_allocation: Option<ResourceAllocation>,
    },
    
    // Governance changes
    Constitutional {
        articles_affected: Vec<ArticleId>,
        requires_supermajority: bool,
    },
    
    // Membership decisions
    Membership {
        action: MembershipAction,
        affected_member: DID,
    },
    
    // Operational decisions
    Operational {
        department: String,
        operational_change: String,
    },
    
    // Emergency actions
    Emergency {
        threat_type: ThreatType,
        emergency_powers: Vec<EmergencyPower>,
        expiry: Timestamp,
    },
    
    // Protocol upgrades
    Technical {
        upgrade_type: UpgradeType,
        code_changes: Vec<CID>,
        migration_plan: Option<MigrationPlan>,
    },
}

pub enum ProposalAction {
    // Economic actions
    TransferFunds { from: AccountId, to: AccountId, amount: Mana },
    MintTokens { token_class: TokenClass, amount: u64, recipient: DID },
    BurnTokens { token_class: TokenClass, amount: u64 },
    
    // Governance actions
    UpdateParameter { param: String, value: Value },
    GrantRole { did: DID, role: Role },
    RevokeRole { did: DID, role: Role },
    
    // Smart contract actions
    DeployContract { code: CID, init_params: Bytes },
    CallContract { address: ContractAddress, function: String, args: Bytes },
    UpgradeContract { address: ContractAddress, new_code: CID },
    
    // Membership actions
    IssueMembership { to: DID, membership_type: MembershipType },
    RevokeMembership { from: DID, reason: String },
    SuspendMember { did: DID, duration: Duration, reason: String },
    
    // Federation actions
    JoinFederation { federation: FederationId },
    LeaveFederation { federation: FederationId },
    ProposeToFederation { federation: FederationId, proposal: Box<Proposal> },
}
```

### 3.2 Proposal Lifecycle

```rust
pub enum ProposalState {
    Draft,                     // Being prepared
    Submitted,                 // Awaiting sponsorship
    Sponsored,                 // Has required sponsors
    Discussion,                // Open for debate
    Voting,                    // Voting period active
    Passed,                    // Vote succeeded
    Failed,                    // Vote failed
    Vetoed,                    // Vetoed by safety mechanism
    Queued,                    // Awaiting time-lock
    Executed,                  // Actions completed
    Cancelled,                 // Withdrawn or cancelled
    Expired,                   // Timed out
}

pub struct ProposalLifecycle {
    pub fn submit_proposal(proposal: Proposal) -> Result<ProposalId> {
        // 1. Validate proposer is member
        require(is_member(&proposal.proposer, &proposal.scope)?);
        
        // 2. Check proposal is well-formed
        validate_proposal(&proposal)?;
        
        // 3. Charge anti-spam fee (refundable)
        let fee = calculate_proposal_fee(&proposal);
        if fee > 0 && !is_fee_waived(&proposal.proposer)? {
            lock_mana(&proposal.proposer, fee, proposal.voting_ends)?;
        }
        
        // 4. Check if sponsorship required
        if requires_sponsorship(&proposal)? {
            proposal.state = ProposalState::Submitted;
        } else {
            proposal.state = ProposalState::Discussion;
        }
        
        // 5. Record in DAG
        let proposal_cid = put_dag(&proposal)?;
        
        emit ProposalSubmitted(proposal.id, proposal_cid);
        Ok(proposal.id)
    }
    
    pub fn sponsor_proposal(proposal_id: ProposalId, sponsor: DID) -> Result<()> {
        let mut proposal = get_proposal(&proposal_id)?;
        
        require(proposal.state == ProposalState::Submitted);
        require(is_member(&sponsor, &proposal.scope)?);
        require(sponsor != proposal.proposer);  // Can't self-sponsor
        
        proposal.co_sponsors.push(sponsor);
        
        // Check if enough sponsors
        let required_sponsors = get_required_sponsors(&proposal.category);
        if proposal.co_sponsors.len() >= required_sponsors {
            proposal.state = ProposalState::Discussion;
            emit ProposalSponsored(proposal_id);
        }
        
        update_proposal(&proposal)?;
        Ok(())
    }
    
    pub fn transition_to_voting(proposal_id: ProposalId) -> Result<()> {
        let mut proposal = get_proposal(&proposal_id)?;
        
        require(proposal.state == ProposalState::Discussion);
        require(current_time() >= proposal.voting_starts);
        
        proposal.state = ProposalState::Voting;
        update_proposal(&proposal)?;
        
        // Notify all members
        notify_voting_started(&proposal)?;
        
        emit VotingStarted(proposal_id);
        Ok(())
    }
}
```

---

## 4. Voting Mechanisms

### 4.1 Voting Methods

```rust
pub enum VotingMethod {
    // Simple yes/no/abstain
    SimpleBallot {
        options: Vec<VoteOption>,  // [Yes, No, Abstain]
    },
    
    // Ranked choice voting
    RankedChoice {
        candidates: Vec<Choice>,
        elimination_rounds: bool,
    },
    
    // Score voting
    ScoreVoting {
        min_score: i32,
        max_score: i32,
    },
    
    // Quadratic voting (but equal budgets!)
    QuadraticVoting {
        vote_credits: u32,  // Same for all members
    },
    
    // Conviction voting
    ConvictionVoting {
        conviction_levels: Vec<f64>,
        time_weighted: bool,
    },
    
    // Consensus seeking
    Consensus {
        rounds: u32,
        modification_allowed: bool,
    },
}

pub struct Vote {
    proposal_id: ProposalId,
    voter: DID,
    
    // Vote content
    choice: VoteChoice,
    conviction: Option<f64>,      // Strength of preference
    rationale: Option<String>,     // Optional explanation
    
    // Metadata
    timestamp: Timestamp,
    signature: Signature,
    
    // Privacy
    privacy_level: VotePrivacy,
    encryption: Option<Encryption>,
}

pub enum VoteChoice {
    Simple(VoteOption),            // Yes/No/Abstain
    Ranked(Vec<ChoiceId>),         // Ordered preferences
    Score(HashMap<ChoiceId, i32>), // Scores per option
    Quadratic(HashMap<ChoiceId, u32>), // Vote credits distribution
}

pub enum VoteOption {
    Yes,
    No,
    Abstain,
    NoWithVeto,  // Blocks even if majority passes
}

pub enum VotePrivacy {
    Public,                        // Vote visible to all
    PseudonymousPublic,           // Vote visible, voter hidden
    PrivateUntilEnd,              // Hidden until voting ends
    FullyPrivate,                 // Zero-knowledge proof only
}
```

### 4.2 Voting Process

```rust
pub struct VotingProcess {
    pub fn cast_vote(vote: Vote) -> Result<()> {
        let proposal = get_proposal(&vote.proposal_id)?;
        
        // 1. Verify voting period
        require(proposal.state == ProposalState::Voting);
        require(current_time() >= proposal.voting_starts);
        require(current_time() <= proposal.voting_ends);
        
        // 2. Verify voter eligibility
        require(can_vote(&vote.voter, &proposal.scope)?);
        
        // 3. Check not already voted (unless changing)
        if has_voted(&vote.voter, &vote.proposal_id)? {
            require(allows_vote_changing(&proposal)?);
        }
        
        // 4. Validate vote format
        validate_vote(&vote, &proposal.voting_method)?;
        
        // 5. Charge nominal anti-spam fee (waivable)
        let fee = get_vote_fee();
        if !is_fee_waived(&vote.voter)? && fee > 0 {
            charge_mana(&vote.voter, fee)?;
        }
        
        // 6. Record vote
        let vote_cid = match vote.privacy_level {
            VotePrivacy::Public => {
                put_dag(&vote)?
            },
            VotePrivacy::PrivateUntilEnd => {
                store_encrypted_vote(&vote, &proposal.voting_ends)?
            },
            VotePrivacy::FullyPrivate => {
                let proof = generate_vote_proof(&vote)?;
                put_dag(&proof)?
            },
            _ => put_dag(&vote)?,
        };
        
        emit VoteCast(vote.proposal_id, vote.voter, vote_cid);
        Ok(())
    }
    
    pub fn delegate_vote(
        delegator: DID,
        delegate: DID,
        scope: DelegationScope
    ) -> Result<()> {
        // Verify both are members
        require(is_member(&delegator, &scope.organization)?);
        require(is_member(&delegate, &scope.organization)?);
        
        // Create delegation
        let delegation = Delegation {
            from: delegator,
            to: delegate,
            scope,
            valid_until: current_time() + DEFAULT_DELEGATION_PERIOD,
            revocable: true,
        };
        
        // Charge delegation fee (prevents gaming)
        charge_mana(&delegator, DELEGATION_FEE)?;
        
        // Record delegation
        store_delegation(&delegation)?;
        
        emit VoteDelegated(delegator, delegate, scope);
        Ok(())
    }
}
```

### 4.3 Vote Tallying

```rust
pub struct VoteTallying {
    pub fn tally_votes(proposal_id: ProposalId) -> Result<ProposalResult> {
        let proposal = get_proposal(&proposal_id)?;
        
        require(current_time() > proposal.voting_ends);
        
        // Get all votes (including delegated)
        let votes = get_all_votes(&proposal_id)?;
        let expanded_votes = expand_delegations(&votes)?;
        
        // Check quorum
        let total_members = count_eligible_voters(&proposal.scope)?;
        let votes_cast = expanded_votes.len();
        
        if !meets_quorum(&proposal.quorum, votes_cast, total_members)? {
            return Ok(ProposalResult::Failed(FailureReason::QuorumNotMet));
        }
        
        // Count based on voting method
        let result = match &proposal.voting_method {
            VotingMethod::SimpleBallot { .. } => {
                count_simple_majority(&expanded_votes, &proposal.threshold)?
            },
            
            VotingMethod::RankedChoice { .. } => {
                instant_runoff_voting(&expanded_votes)?
            },
            
            VotingMethod::ScoreVoting { .. } => {
                calculate_score_winner(&expanded_votes)?
            },
            
            VotingMethod::QuadraticVoting { .. } => {
                calculate_quadratic_result(&expanded_votes)?
            },
            
            VotingMethod::ConvictionVoting { .. } => {
                calculate_conviction_weighted(&expanded_votes)?
            },
            
            VotingMethod::Consensus { .. } => {
                check_consensus(&expanded_votes)?
            },
        };
        
        // Check for vetos
        if has_valid_veto(&expanded_votes)? {
            return Ok(ProposalResult::Vetoed);
        }
        
        Ok(result)
    }
    
    fn count_simple_majority(
        votes: &[Vote],
        threshold: &PassThreshold
    ) -> Result<ProposalResult> {
        let mut yes = 0;
        let mut no = 0;
        let mut abstain = 0;
        let mut no_with_veto = 0;
        
        for vote in votes {
            match vote.choice {
                VoteChoice::Simple(VoteOption::Yes) => yes += 1,
                VoteChoice::Simple(VoteOption::No) => no += 1,
                VoteChoice::Simple(VoteOption::Abstain) => abstain += 1,
                VoteChoice::Simple(VoteOption::NoWithVeto) => {
                    no += 1;
                    no_with_veto += 1;
                },
                _ => {},
            }
        }
        
        let total_voting = yes + no;  // Abstentions don't count
        
        // Check if veto threshold met (usually 33%)
        if no_with_veto as f64 / total_voting as f64 > VETO_THRESHOLD {
            return Ok(ProposalResult::Vetoed);
        }
        
        // Check if pass threshold met
        let pass_ratio = yes as f64 / total_voting as f64;
        
        match threshold {
            PassThreshold::SimpleMajority => {
                if pass_ratio > 0.5 {
                    Ok(ProposalResult::Passed)
                } else {
                    Ok(ProposalResult::Failed(FailureReason::InsufficientSupport))
                }
            },
            PassThreshold::SuperMajority(threshold) => {
                if pass_ratio >= *threshold {
                    Ok(ProposalResult::Passed)
                } else {
                    Ok(ProposalResult::Failed(FailureReason::InsufficientSupport))
                }
            },
            PassThreshold::Unanimity => {
                if no == 0 && no_with_veto == 0 {
                    Ok(ProposalResult::Passed)
                } else {
                    Ok(ProposalResult::Failed(FailureReason::NotUnanimous))
                }
            },
        }
    }
}
```

---

## 5. Quorum & Thresholds

### 5.1 Quorum Requirements

```rust
pub enum QuorumRequirement {
    // Fixed number
    Fixed(u32),
    
    // Percentage of members
    Percentage(f64),
    
    // Dynamic based on importance
    Dynamic {
        base: f64,
        importance_multiplier: f64,
    },
    
    // No quorum required
    None,
}

pub struct QuorumCalculator {
    pub fn calculate_quorum(
        requirement: &QuorumRequirement,
        total_members: usize,
        proposal: &Proposal
    ) -> usize {
        match requirement {
            QuorumRequirement::Fixed(n) => *n as usize,
            
            QuorumRequirement::Percentage(p) => {
                (total_members as f64 * p).ceil() as usize
            },
            
            QuorumRequirement::Dynamic { base, importance_multiplier } => {
                let importance = calculate_importance(&proposal);
                let dynamic_percentage = base + (importance * importance_multiplier);
                (total_members as f64 * dynamic_percentage.min(1.0)).ceil() as usize
            },
            
            QuorumRequirement::None => 0,
        }
    }
    
    fn calculate_importance(proposal: &Proposal) -> f64 {
        let mut importance = 0.0;
        
        // Constitutional changes are most important
        if matches!(proposal.category, ProposalCategory::Constitutional { .. }) {
            importance += 0.5;
        }
        
        // Emergency proposals are important
        if matches!(proposal.category, ProposalCategory::Emergency { .. }) {
            importance += 0.4;
        }
        
        // Large economic impact
        if let ProposalCategory::Economic { budget_impact, .. } = &proposal.category {
            if let Some(impact) = budget_impact {
                if *impact > 10000 {
                    importance += 0.3;
                }
            }
        }
        
        importance.min(1.0)
    }
}
```

### 5.2 Pass Thresholds

```rust
pub enum PassThreshold {
    SimpleMajority,            // >50%
    SuperMajority(f64),        // Custom threshold
    Unanimity,                 // 100% of votes cast
    UnanimityMinusOne,         // All but one
    UnanimityMinusTwo,         // All but two
}

pub struct ThresholdConfiguration {
    pub fn get_threshold_for_proposal(proposal: &Proposal) -> PassThreshold {
        match &proposal.category {
            ProposalCategory::Constitutional { .. } => {
                PassThreshold::SuperMajority(0.75)  // 75% for constitutional changes
            },
            
            ProposalCategory::Emergency { .. } => {
                PassThreshold::SuperMajority(0.80)  // 80% for emergency powers
            },
            
            ProposalCategory::Membership { action, .. } => {
                match action {
                    MembershipAction::Expel => PassThreshold::SuperMajority(0.67),
                    MembershipAction::Suspend => PassThreshold::SimpleMajority,
                    _ => PassThreshold::SimpleMajority,
                }
            },
            
            ProposalCategory::Economic { budget_impact, .. } => {
                if let Some(impact) = budget_impact {
                    if *impact > 100000 {
                        PassThreshold::SuperMajority(0.67)  // Large budgets need 67%
                    } else {
                        PassThreshold::SimpleMajority
                    }
                } else {
                    PassThreshold::SimpleMajority
                }
            },
            
            _ => PassThreshold::SimpleMajority,
        }
    }
}
```

---

## 6. Time-locks & Safety Mechanisms

### 6.1 Time-lock Implementation

```rust
pub struct TimeLock {
    proposal_id: ProposalId,
    
    // Delay before execution
    delay: Duration,
    
    // When the time-lock started
    queued_at: Timestamp,
    
    // When execution becomes possible
    executable_at: Timestamp,
    
    // Expiry (if not executed)
    expires_at: Timestamp,
}

impl TimeLockProtocol {
    pub fn queue_proposal(proposal_id: ProposalId) -> Result<()> {
        let proposal = get_proposal(&proposal_id)?;
        
        require(proposal.state == ProposalState::Passed);
        
        // Calculate delay based on proposal type
        let delay = calculate_timelock_delay(&proposal);
        
        let timelock = TimeLock {
            proposal_id,
            delay,
            queued_at: current_time(),
            executable_at: current_time() + delay,
            expires_at: current_time() + delay + EXECUTION_WINDOW,
        };
        
        store_timelock(&timelock)?;
        proposal.state = ProposalState::Queued;
        update_proposal(&proposal)?;
        
        emit ProposalQueued(proposal_id, timelock.executable_at);
        Ok(())
    }
    
    fn calculate_timelock_delay(proposal: &Proposal) -> Duration {
        match &proposal.category {
            ProposalCategory::Emergency { .. } => Duration::from_secs(0),  // No delay
            ProposalCategory::Constitutional { .. } => Duration::from_secs(7 * 24 * 3600),  // 7 days
            ProposalCategory::Economic { budget_impact, .. } => {
                if let Some(impact) = budget_impact {
                    if *impact > 50000 {
                        Duration::from_secs(3 * 24 * 3600)  // 3 days for large amounts
                    } else {
                        Duration::from_secs(24 * 3600)  // 1 day
                    }
                } else {
                    Duration::from_secs(24 * 3600)
                }
            },
            _ => Duration::from_secs(24 * 3600),  // Default 1 day
        }
    }
}
```

### 6.2 Veto Mechanisms

```rust
pub struct VetoMechanism {
    pub fn check_veto_power(did: &DID, proposal: &Proposal) -> bool {
        // Different veto powers for different roles
        match get_role(did) {
            Some(Role::SafetyCommittee) => {
                // Safety committee can veto dangerous proposals
                matches!(proposal.category, 
                    ProposalCategory::Technical { .. } |
                    ProposalCategory::Emergency { .. })
            },
            
            Some(Role::ConstitutionalGuardian) => {
                // Guardians can veto unconstitutional proposals
                matches!(proposal.category, ProposalCategory::Constitutional { .. })
            },
            
            _ => false,
        }
    }
    
    pub fn exercise_veto(vetoer: DID, proposal_id: ProposalId, reason: String) -> Result<()> {
        let proposal = get_proposal(&proposal_id)?;
        
        require(check_veto_power(&vetoer, &proposal));
        require(proposal.state == ProposalState::Queued);
        
        // Record veto
        let veto = Veto {
            proposal_id,
            vetoer,
            reason,
            timestamp: current_time(),
        };
        
        store_veto(&veto)?;
        proposal.state = ProposalState::Vetoed;
        update_proposal(&proposal)?;
        
        // Refund proposal fee
        if let Some(fee) = get_locked_proposal_fee(&proposal_id) {
            refund_mana(&proposal.proposer, fee)?;
        }
        
        emit ProposalVetoed(proposal_id, vetoer, reason);
        Ok(())
    }
}
```

---

## 7. Emergency Governance

### 7.1 Emergency Proposals

```rust
pub struct EmergencyGovernance {
    pub fn submit_emergency_proposal(
        proposal: Proposal,
        evidence: EmergencyEvidence
    ) -> Result<ProposalId> {
        // Verify emergency conditions
        require(verify_emergency_conditions(&evidence)?);
        
        // Check proposer has emergency rights
        require(can_propose_emergency(&proposal.proposer)?);
        
        // Fast-track the proposal
        let mut emergency_proposal = proposal;
        emergency_proposal.category = ProposalCategory::Emergency {
            threat_type: identify_threat(&evidence),
            emergency_powers: determine_powers(&evidence),
            expiry: current_time() + EMERGENCY_DURATION,
        };
        
        // Shortened timelines
        emergency_proposal.discussion_ends = current_time() + Duration::from_secs(3600);  // 1 hour
        emergency_proposal.voting_starts = emergency_proposal.discussion_ends;
        emergency_proposal.voting_ends = emergency_proposal.voting_starts + Duration::from_secs(3600);
        
        // Lower quorum for emergencies
        emergency_proposal.quorum = QuorumRequirement::Percentage(0.1);  // 10% in emergency
        
        // Immediate notification to all members
        broadcast_emergency_proposal(&emergency_proposal)?;
        
        // Record with high priority
        let proposal_cid = put_dag_priority(&emergency_proposal)?;
        
        emit EmergencyProposalSubmitted(emergency_proposal.id, proposal_cid);
        Ok(emergency_proposal.id)
    }
    
    pub fn activate_emergency_powers(proposal_id: ProposalId) -> Result<()> {
        let proposal = get_proposal(&proposal_id)?;
        
        require(matches!(proposal.category, ProposalCategory::Emergency { .. }));
        require(proposal.state == ProposalState::Passed);
        
        if let ProposalCategory::Emergency { emergency_powers, expiry, .. } = &proposal.category {
            for power in emergency_powers {
                grant_emergency_power(power, *expiry)?;
            }
        }
        
        emit EmergencyPowersActivated(proposal_id);
        Ok(())
    }
}

pub enum EmergencyPower {
    FreezeAllTransactions,
    BypassNormalGovernance,
    EmergencyMinting(Mana),
    NetworkPartitionRecovery,
    ValidatorRotation,
    ContractPause,
}
```

### 7.2 Crisis Response

```rust
pub struct CrisisResponse {
    pub fn initiate_crisis_mode(threat: ThreatType) -> Result<()> {
        // Only certain entities can trigger
        require(is_validator(&msg.sender) || is_safety_committee(&msg.sender));
        
        // Create crisis state
        let crisis = CrisisState {
            threat,
            initiated_by: msg.sender,
            started_at: current_time(),
            emergency_committee: select_emergency_committee(),
            powers_granted: determine_crisis_powers(&threat),
        };
        
        // Immediate actions
        match threat {
            ThreatType::NetworkAttack => {
                enable_defensive_mode()?;
                increase_validation_threshold(0.80)?;
            },
            
            ThreatType::EconomicAttack => {
                freeze_large_transactions()?;
                enable_enhanced_monitoring()?;
            },
            
            ThreatType::GovernanceCapture => {
                pause_proposal_execution()?;
                require_multi_sig_for_actions()?;
            },
            
            ThreatType::SystemFailure => {
                activate_backup_validators()?;
                enable_recovery_mode()?;
            },
        }
        
        store_crisis_state(&crisis)?;
        emit CrisisModeActivated(threat);
        Ok(())
    }
}
```

---

## 8. Cross-Federation Governance

### 8.1 Federation Proposals

```rust
pub struct FederationGovernance {
    pub fn propose_to_federation(
        local_proposal: ProposalId,
        target_federation: FederationId
    ) -> Result<ProposalId> {
        // Get local proposal that passed
        let proposal = get_proposal(&local_proposal)?;
        require(proposal.state == ProposalState::Passed);
        
        // Check if affects federation
        require(affects_federation(&proposal, &target_federation)?);
        
        // Create federation proposal
        let fed_proposal = FederationProposal {
            originating_org: get_current_org(),
            original_proposal: local_proposal,
            federation: target_federation,
            
            // Federation uses different voting
            voting_method: VotingMethod::WeightedByOrgSize,
            quorum: QuorumRequirement::Percentage(0.67),  // 67% of orgs
            
            // Longer timeline for federation
            discussion_period: Duration::from_secs(7 * 24 * 3600),  // 7 days
            voting_period: Duration::from_secs(7 * 24 * 3600),      // 7 days
        };
        
        // Submit to federation
        let fed_proposal_id = submit_to_federation(&fed_proposal)?;
        
        emit FederationProposalSubmitted(fed_proposal_id);
        Ok(fed_proposal_id)
    }
    
    pub fn federation_vote(
        org_id: OrganizationId,
        proposal_id: ProposalId,
        org_vote: OrganizationVote
    ) -> Result<()> {
        // Verify org is federation member
        let federation = get_proposal_federation(&proposal_id)?;
        require(is_member_org(&org_id, &federation)?);
        
        // Verify vote is authorized by org
        require(org_vote.authorized_by_members);
        require(verify_org_vote_authorization(&org_vote)?);
        
        // Record weighted vote
        let weight = calculate_org_weight(&org_id, &federation)?;
        
        store_federation_vote(FederationVote {
            proposal_id,
            org_id,
            vote: org_vote.decision,
            weight,
            timestamp: current_time(),
        })?;
        
        emit FederationVoteCast(proposal_id, org_id);
        Ok(())
    }
}
```

---

## 9. Execution & Enforcement

### 9.1 Proposal Execution

```rust
pub struct ProposalExecution {
    pub fn execute_proposal(proposal_id: ProposalId) -> Result<()> {
        let proposal = get_proposal(&proposal_id)?;
        
        // Check can execute
        require(proposal.state == ProposalState::Queued);
        
        let timelock = get_timelock(&proposal_id)?;
        require(current_time() >= timelock.executable_at);
        require(current_time() <= timelock.expires_at);
        
        // Execute each action
        for action in &proposal.actions {
            execute_action(action)?;
        }
        
        // Update state
        proposal.state = ProposalState::Executed;
        update_proposal(&proposal)?;
        
        // Release proposer's stake
        if let Some(stake) = get_proposal_stake(&proposal_id) {
            unlock_mana(&proposal.proposer, stake)?;
        }
        
        emit ProposalExecuted(proposal_id);
        Ok(())
    }
    
    fn execute_action(action: &ProposalAction) -> Result<()> {
        match action {
            ProposalAction::TransferFunds { from, to, amount } => {
                transfer_funds(from, to, *amount)
            },
            
            ProposalAction::MintTokens { token_class, amount, recipient } => {
                mint_tokens(token_class, *amount, recipient)
            },
            
            ProposalAction::UpdateParameter { param, value } => {
                update_system_parameter(param, value)
            },
            
            ProposalAction::DeployContract { code, init_params } => {
                deploy_contract(code, init_params)
            },
            
            ProposalAction::IssueMembership { to, membership_type } => {
                issue_membership_credential(to, membership_type)
            },
            
            // ... other actions
            _ => Ok(()),
        }
    }
}
```

---

## 10. Monitoring & Analytics

### 10.1 Governance Metrics

```rust
pub struct GovernanceMetrics {
    // Participation metrics
    voter_turnout: Gauge,
    average_quorum: Gauge,
    delegation_rate: Gauge,
    
    // Proposal metrics
    proposals_submitted: Counter,
    proposals_passed: Counter,
    proposals_failed: Counter,
    proposals_vetoed: Counter,
    
    // Time metrics
    average_discussion_time: Histogram,
    average_voting_time: Histogram,
    average_execution_delay: Histogram,
    
    // Health metrics
    member_engagement: Gauge,
    proposal_diversity: Gauge,  // Gini coefficient of proposers
    vote_consistency: Gauge,     // How often members vote together
}

pub struct GovernanceAnalytics {
    pub fn analyze_voting_patterns(org: &OrganizationId) -> VotingAnalysis {
        let recent_proposals = get_recent_proposals(org, 30)?;  // Last 30 days
        
        VotingAnalysis {
            average_turnout: calculate_average_turnout(&recent_proposals),
            common_voting_blocks: identify_voting_blocks(&recent_proposals),
            swing_voters: identify_swing_voters(&recent_proposals),
            proposal_success_predictors: analyze_success_factors(&recent_proposals),
        }
    }
    
    pub fn calculate_democratic_health(org: &OrganizationId) -> DemocraticHealth {
        DemocraticHealth {
            participation_score: calculate_participation_score(org),
            diversity_score: calculate_proposer_diversity(org),
            deliberation_score: calculate_deliberation_quality(org),
            execution_score: calculate_execution_efficiency(org),
            overall_health: weighted_average(&scores),
        }
    }
}
```

---

## 11. Implementation Roadmap

### 11.1 Phase 1: Core Governance (Months 1-2)
- [ ] Basic proposal system
- [ ] Simple majority voting
- [ ] Membership verification
- [ ] DAG integration

### 11.2 Phase 2: Advanced Voting (Months 3-4)
- [ ] Multiple voting methods
- [ ] Delegation system
- [ ] Vote privacy options
- [ ] Tallying algorithms

### 11.3 Phase 3: Safety & Federation (Months 5-6)
- [ ] Time-lock implementation
- [ ] Veto mechanisms
- [ ] Emergency governance
- [ ] Cross-federation proposals

### 11.4 Phase 4: Analytics & Optimization (Months 7-8)
- [ ] Governance metrics
- [ ] Pattern analysis
- [ ] Performance optimization
- [ ] Comprehensive testing

---

## Appendix A: Configuration

```yaml
governance:
  # Membership
  membership:
    probation_period: 2592000  # 30 days in seconds
    credential_type: "VerifiableCredential"
    voting_weight: 1.0  # Always 1.0 for equality
    
  # Proposals
  proposals:
    min_description_length: 100
    max_description_length: 10000
    default_discussion_period: 259200  # 3 days
    default_voting_period: 432000      # 5 days
    
  # Voting
  voting:
    default_method: "SimpleBallot"
    allow_vote_changing: true
    vote_privacy_default: "Public"
    anti_spam_fee: 0.1  # Mana
    fee_waiver_threshold: 10  # Mana balance
    
  # Quorum
  quorum:
    default_requirement: 0.25  # 25%
    constitutional_requirement: 0.5  # 50%
    emergency_requirement: 0.1  # 10%
    
  # Thresholds
  thresholds:
    simple_majority: 0.5
    super_majority: 0.67
    constitutional: 0.75
    veto_threshold: 0.33
    
  # Time-locks
  timelocks:
    default_delay: 86400  # 1 day
    constitutional_delay: 604800  # 7 days
    emergency_delay: 0  # No delay
    execution_window: 604800  # 7 days to execute
    
  # Emergency
  emergency:
    duration: 604800  # 7 days
    committee_size: 5
    response_time: 3600  # 1 hour
```

---

## Appendix B: Error Codes

| Code | Error | Description |
|------|-------|-------------|
| G001 | NotAMember | Caller is not a member |
| G002 | AlreadyVoted | Member has already voted |
| G003 | VotingNotActive | Not in voting period |
| G004 | QuorumNotMet | Insufficient participation |
| G005 | ProposalExpired | Proposal has expired |
| G006 | InvalidDelegation | Delegation not valid |
| G007 | NotAuthorized | Lacks required permission |
| G008 | ProposalVetoed | Proposal was vetoed |
| G009 | TimeLockActive | Still in time-lock period |
| G010 | EmergencyOnly | Only in emergency mode |

---

*This completes the Governance Protocol specification. The system implements true democratic decision-making where membership, not wealth, determines voting power.*

**Protocol Status**: DEFINITIVE  
**Dependencies**: DAG Protocol, Economic Protocol, Identity Protocol  
**Implementation Complexity**: High (multiple voting methods, delegation, federation coordination)  
**Estimated Development**: 8 months for full implementation