# Mana, Reputation, and DID Integration in ICN/CCL

> **Purpose:** This document explains how mana (regenerative participation credits), reputation (trust scores), and DIDs (identity) work together to create a merit-based, anti-spam, polycentric governance system.

---

## 1 · Conceptual Framework

### The Trinity of Participation
```
DID (Identity) ←→ Mana (Capacity) ←→ Reputation (Trust)
       ↘                ↙              ↗
         Governance Actions & Economic Transactions
```

**DID**: Who you are (cryptographically verified identity)
**Mana**: What you can do (regenerating action capacity)  
**Reputation**: How much the community trusts you (earned social credit)

### Key Principles
- **Non-transferable**: Mana cannot be sold, bought, or transferred between DIDs
- **Regenerative**: Mana replenishes over time based on reputation and policies
- **Scoped**: Different mana pools for different federations/communities
- **Merit-based**: Higher reputation → faster regeneration and higher caps
- **Anti-Sybil**: New DIDs start with low mana and must earn reputation

---

## 2 · Mana Mechanics

### 2.1 Basic Mana Operations

```ccl
// Core mana state per DID per scope
state mana_accounts: map<(did, scope), ManaAccount>;

struct ManaAccount {
    current_balance: token<Mana>,
    max_capacity: token<Mana>,
    regeneration_rate: float,         // mana per hour
    last_regeneration: timestamp,
    total_earned: token<Mana>,        // lifetime total
    total_spent: token<Mana>          // lifetime spending
}

// Basic mana operations
fn charge_mana(actor: did, amount: token<Mana>, action: string) {
    let scope = get_current_scope();
    let account = mana_accounts[(actor, scope)];
    
    require(account.current_balance >= amount);
    
    account.current_balance -= amount;
    account.total_spent += amount;
    mana_accounts[(actor, scope)] = account;
    
    emit ManaCharged {
        actor: actor,
        amount: amount,
        action: action,
        remaining: account.current_balance,
        timestamp: now()
    };
}

fn regenerate_mana(actor: did) {
    let scope = get_current_scope();
    let mut account = mana_accounts[(actor, scope)];
    
    let time_passed = now() - account.last_regeneration;
    let hours_passed = time_passed.as_hours();
    let regenerated = account.regeneration_rate * hours_passed;
    
    // Cap at maximum capacity
    account.current_balance = min(
        account.current_balance + regenerated,
        account.max_capacity
    );
    
    account.last_regeneration = now();
    account.total_earned += regenerated;
    mana_accounts[(actor, scope)] = account;
    
    emit ManaRegenerated {
        actor: actor,
        amount: regenerated,
        new_balance: account.current_balance,
        timestamp: now()
    };
}
```

### 2.2 Reputation-Influenced Mana Parameters

```ccl
fn calculate_mana_parameters(actor: did, base_config: ManaConfig) -> ManaAccount {
    let reputation = get_reputation_score(actor);
    let credentials = get_verified_credentials(actor);
    
    // Base parameters
    let mut max_capacity = base_config.base_capacity;
    let mut regen_rate = base_config.base_regeneration_rate;
    
    // Reputation multipliers
    if reputation >= 90.0 {
        max_capacity *= 2.0;      // Highly trusted actors get 2x capacity
        regen_rate *= 1.5;        // 50% faster regeneration
    } else if reputation >= 75.0 {
        max_capacity *= 1.5;
        regen_rate *= 1.25;
    } else if reputation >= 50.0 {
        max_capacity *= 1.2;
        regen_rate *= 1.1;
    } else if reputation < 25.0 {
        max_capacity *= 0.5;      // Low reputation = reduced capacity
        regen_rate *= 0.7;        // Slower regeneration
    }
    
    // Credential bonuses
    for credential in credentials {
        match credential.credential_type {
            "founding_member" => {
                max_capacity *= 1.3;
                regen_rate *= 1.2;
            },
            "elected_representative" => {
                max_capacity *= 1.4;
                regen_rate *= 1.15;
            },
            "technical_contributor" => {
                max_capacity *= 1.2;
                regen_rate *= 1.1;
            },
            "community_mediator" => {
                max_capacity *= 1.25;
                regen_rate *= 1.1;
            }
        }
    }
    
    ManaAccount {
        current_balance: max_capacity,  // Start at full capacity
        max_capacity: max_capacity,
        regeneration_rate: regen_rate,
        last_regeneration: now(),
        total_earned: token<Mana>(0),
        total_spent: token<Mana>(0)
    }
}
```

---

## 3 · Reputation System Integration

### 3.1 Reputation Calculation

```ccl
state reputation_history: map<did, [ReputationEvent]>;
state reputation_scores: map<did, ReputationScore>;

struct ReputationScore {
    current_score: float,           // 0.0 to 100.0
    historical_average: float,      // Long-term average
    last_updated: timestamp,
    positive_interactions: int,
    negative_interactions: int,
    peer_endorsements: int,
    violations: int
}

struct ReputationEvent {
    event_type: ReputationEventType,
    weight: float,
    source: did,                    // Who reported/observed this
    timestamp: timestamp,
    context: string,
    verified: bool
}

enum ReputationEventType {
    ProposalApproved,              // +2.0
    ProposalRejected,              // -0.5
    VotedWithMajority,             // +0.2
    VotedAgainstMajority,          // -0.1
    SuccessfulJobExecution,        // +1.5
    FailedJobExecution,            // -1.0
    PeerEndorsement,               // +0.5
    CommunityViolation,            // -5.0
    MediationSuccess,              // +3.0
    ContributedCode,               // +1.0
    DocumentationContribution,     // +0.8
    Mentorship,                    // +1.2
    ConflictResolution,            // +2.5
    SpamReported,                  // -10.0
    SybilDetected                  // -50.0
}

fn update_reputation(
    subject: did, 
    event_type: ReputationEventType, 
    reporter: did,
    context: string
) {
    require(is_trusted_reporter(reporter) || reporter == subject);
    
    let weight = get_event_weight(event_type);
    let event = ReputationEvent {
        event_type: event_type,
        weight: weight,
        source: reporter,
        timestamp: now(),
        context: context,
        verified: verify_reputation_event(event_type, subject, context)
    };
    
    reputation_history[subject].push(event);
    
    // Recalculate reputation score
    let mut score = reputation_scores[subject];
    
    // Apply immediate impact
    score.current_score += weight;
    score.current_score = max(0.0, min(100.0, score.current_score));
    
    if weight > 0.0 {
        score.positive_interactions += 1;
    } else {
        score.negative_interactions += 1;
    }
    
    // Update historical average (exponential moving average)
    let alpha = 0.1;  // Learning rate
    score.historical_average = 
        alpha * score.current_score + (1.0 - alpha) * score.historical_average;
    
    score.last_updated = now();
    reputation_scores[subject] = score;
    
    // Trigger mana parameter recalculation
    update_mana_parameters(subject);
    
    emit ReputationUpdated {
        subject: subject,
        event_type: event_type,
        new_score: score.current_score,
        reporter: reporter,
        timestamp: now()
    };
}

fn get_reputation_score(actor: did) -> float {
    if let Some(score) = reputation_scores.get(actor) {
        // Time-weighted reputation (slight decay over inactivity)
        let days_since_update = (now() - score.last_updated).as_days();
        let decay_factor = max(0.95, 1.0 - (days_since_update / 365.0) * 0.05); // 5% decay per year
        score.current_score * decay_factor
    } else {
        50.0  // Default starting reputation
    }
}
```

### 3.2 Reputation-Mana Feedback Loops

```ccl
fn apply_reputation_to_mana(actor: did) {
    let reputation = get_reputation_score(actor);
    let scope = get_current_scope();
    let mut account = mana_accounts[(actor, scope)];
    let base_config = get_mana_config(scope);
    
    // Recalculate mana parameters based on current reputation
    let new_params = calculate_mana_parameters(actor, base_config);
    
    // Smoothly transition to new parameters (prevent sudden drops)
    if new_params.max_capacity < account.max_capacity {
        // Gradual reduction over 30 days
        let reduction_rate = (account.max_capacity - new_params.max_capacity) / 30.0;
        account.max_capacity = max(
            new_params.max_capacity,
            account.max_capacity - reduction_rate
        );
    } else {
        // Immediate increases
        account.max_capacity = new_params.max_capacity;
    }
    
    account.regeneration_rate = new_params.regeneration_rate;
    mana_accounts[(actor, scope)] = account;
    
    emit ManaParametersUpdated {
        actor: actor,
        new_capacity: account.max_capacity,
        new_regen_rate: account.regeneration_rate,
        reputation: reputation,
        timestamp: now()
    };
}
```

---

## 4 · Scoped Mana Across Federations

### 4.1 Multi-Federation Mana Management

```ccl
// Each DID can have mana in multiple scopes
fn get_mana_balance(actor: did, scope: string) -> token<Mana> {
    if let Some(account) = mana_accounts.get((actor, scope)) {
        regenerate_mana_for_scope(actor, scope);  // Auto-regenerate on query
        mana_accounts[(actor, scope)].current_balance
    } else {
        // First time in this scope - initialize account
        initialize_mana_account(actor, scope)
    }
}

fn initialize_mana_account(actor: did, scope: string) -> token<Mana> {
    let base_config = get_mana_config(scope);
    let reputation = get_reputation_score(actor);
    
    // New members get reduced initial mana
    let initial_multiplier = match reputation {
        r if r >= 75.0 => 1.0,      // Trusted actors get full initial mana
        r if r >= 50.0 => 0.8,      // Good reputation gets 80%
        r if r >= 25.0 => 0.5,      // Medium reputation gets 50%
        _ => 0.2                    // Low/new reputation gets 20%
    };
    
    let account = ManaAccount {
        current_balance: base_config.base_capacity * initial_multiplier,
        max_capacity: base_config.base_capacity * initial_multiplier,
        regeneration_rate: base_config.base_regeneration_rate,
        last_regeneration: now(),
        total_earned: token<Mana>(0),
        total_spent: token<Mana>(0)
    };
    
    mana_accounts[(actor, scope)] = account;
    
    emit ManaAccountInitialized {
        actor: actor,
        scope: scope,
        initial_balance: account.current_balance,
        reputation: reputation,
        timestamp: now()
    };
    
    account.current_balance
}

// Cross-scope reputation portability
fn calculate_cross_scope_reputation(actor: did, target_scope: string) -> float {
    let base_reputation = get_reputation_score(actor);
    
    // Check for cross-scope endorsements or credentials
    let endorsements = get_cross_scope_endorsements(actor, target_scope);
    let portable_credentials = get_portable_credentials(actor);
    
    let mut adjusted_reputation = base_reputation * 0.7;  // 30% penalty for new scope
    
    // Add endorsement bonuses
    for endorsement in endorsements {
        if verify_endorsement(endorsement) {
            adjusted_reputation += 5.0;  // +5 points per valid endorsement
        }
    }
    
    // Add credential bonuses
    for credential in portable_credentials {
        match credential.credential_type {
            "icn_core_contributor" => adjusted_reputation += 10.0,
            "federation_founder" => adjusted_reputation += 8.0,
            "verified_identity" => adjusted_reputation += 3.0,
            _ => {}
        }
    }
    
    min(100.0, adjusted_reputation)
}
```

---

## 5 · Anti-Sybil and Gaming Resistance

### 5.1 Sybil Attack Prevention

```ccl
fn detect_sybil_patterns(suspect: did) -> SybilRisk {
    let account_age = now() - get_did_creation_time(suspect);
    let reputation = get_reputation_score(suspect);
    let mana_usage = get_mana_usage_pattern(suspect);
    let social_connections = get_social_graph_connections(suspect);
    
    let mut risk_score = 0.0;
    
    // New account with high activity
    if account_age < 7.days && mana_usage.total_spent > token<Mana>(100) {
        risk_score += 20.0;
    }
    
    // Low reputation with high mana usage
    if reputation < 30.0 && mana_usage.actions_per_day > 50 {
        risk_score += 25.0;
    }
    
    // Lack of social connections
    if social_connections.len() < 3 && account_age > 30.days {
        risk_score += 15.0;
    }
    
    // Repetitive behavior patterns
    if detect_bot_like_behavior(mana_usage) {
        risk_score += 30.0;
    }
    
    match risk_score {
        r if r >= 50.0 => SybilRisk::High,
        r if r >= 30.0 => SybilRisk::Medium,
        r if r >= 15.0 => SybilRisk::Low,
        _ => SybilRisk::None
    }
}

fn apply_sybil_penalties(actor: did, risk: SybilRisk) {
    match risk {
        SybilRisk::High => {
            // Severe restrictions
            let mut account = mana_accounts[(actor, get_current_scope())];
            account.max_capacity *= 0.1;      // 90% reduction
            account.regeneration_rate *= 0.2; // 80% slower regen
            flag_for_community_review(actor, "High Sybil Risk");
        },
        SybilRisk::Medium => {
            // Moderate restrictions
            let mut account = mana_accounts[(actor, get_current_scope())];
            account.max_capacity *= 0.5;      // 50% reduction
            account.regeneration_rate *= 0.7; // 30% slower regen
        },
        SybilRisk::Low => {
            // Minor restrictions
            let mut account = mana_accounts[(actor, get_current_scope())];
            account.regeneration_rate *= 0.9; // 10% slower regen
        },
        SybilRisk::None => {
            // No penalties
        }
    }
}
```

### 5.2 Mana Cost Scaling

```ccl
// Dynamic mana costs based on system load and actor behavior
fn calculate_action_cost(actor: did, action: ActionType) -> token<Mana> {
    let base_cost = get_base_action_cost(action);
    let reputation = get_reputation_score(actor);
    let system_load = get_current_system_load();
    let recent_activity = get_recent_activity(actor, 1.hour);
    
    let mut cost = base_cost;
    
    // Reputation-based discounts
    if reputation >= 90.0 {
        cost *= 0.7;  // 30% discount for highly trusted actors
    } else if reputation >= 75.0 {
        cost *= 0.85; // 15% discount
    } else if reputation < 25.0 {
        cost *= 1.5;  // 50% penalty for low reputation
    }
    
    // System load-based scaling
    if system_load > 0.8 {
        cost *= 1.5;  // Higher costs during peak usage
    }
    
    // Rate limiting through exponential cost increase
    let actions_in_hour = recent_activity.len();
    if actions_in_hour > 10 {
        let penalty = 1.0 + (actions_in_hour - 10) as f64 * 0.1;
        cost *= penalty;
    }
    
    cost
}

enum ActionType {
    SubmitProposal,     // Base: 50 mana
    Vote,               // Base: 2 mana
    TransferTokens,     // Base: 5 mana
    CreateContract,     // Base: 200 mana
    AmendContract,      // Base: 100 mana
    JoinFederation,     // Base: 20 mana
    UpdateReputation,   // Base: 10 mana
    SubmitEvidence,     // Base: 15 mana
}

fn get_base_action_cost(action: ActionType) -> token<Mana> {
    match action {
        ActionType::SubmitProposal => token<Mana>(50),
        ActionType::Vote => token<Mana>(2),
        ActionType::TransferTokens => token<Mana>(5),
        ActionType::CreateContract => token<Mana>(200),
        ActionType::AmendContract => token<Mana>(100),
        ActionType::JoinFederation => token<Mana>(20),
        ActionType::UpdateReputation => token<Mana>(10),
        ActionType::SubmitEvidence => token<Mana>(15),
    }
}
```

---

## 6 · Integration with Privacy and ZKP

### 6.1 Anonymous Mana Usage

```ccl
// Use ZKPs to prove mana sufficiency without revealing balance
fn prove_mana_sufficiency(
    required_amount: token<Mana>,
    zkp_proof: ZKProof
) -> bool {
    // Prove actor has >= required_amount without revealing actual balance
    zkp::verify_range_proof(
        zkp_proof,
        "mana_balance",
        min: required_amount.value(),
        max: None
    )
}

fn anonymous_vote_with_mana(
    proposal_id: int,
    vote: Vote,
    mana_proof: ZKProof,
    nullifier: Nullifier
) {
    // Verify voter has enough mana without revealing identity
    require(prove_mana_sufficiency(get_voting_cost(), mana_proof));
    require(!zkp::nullifier_used(nullifier));
    
    // Charge mana anonymously (tracked by nullifier)
    anonymous_mana_charge(nullifier, get_voting_cost());
    
    // Count vote
    proposal_votes[proposal_id].push(AnonymousVote {
        vote: vote,
        nullifier: nullifier,
        timestamp: now()
    });
    
    emit AnonymousVoteCast {
        proposal_id: proposal_id,
        nullifier: nullifier,
        timestamp: now()
    };
}
```

---

## 7 · Example: Complete Mana-Reputation Flow

```ccl
contract HousingCollectiveManaDemo {
    scope: "local:brooklyn:housing_coop_123";
    
    // Mana costs for different actions
    const PROPOSAL_COST: token<Mana> = token<Mana>(25);
    const VOTE_COST: token<Mana> = token<Mana>(2);
    const AMENDMENT_COST: token<Mana> = token<Mana>(50);
    
    fn submit_maintenance_proposal(
        description: string,
        amount: token<USD>,
        contractor: string
    ) -> ProposalId {
        let caller_did = caller();
        let mana_cost = calculate_action_cost(caller_did, ActionType::SubmitProposal);
        
        // Check and charge mana
        charge_mana(caller_did, mana_cost, "submit_proposal");
        
        // Create proposal
        let proposal_id = create_proposal(ProposalType::MaintenanceRequest {
            description,
            amount,
            contractor,
            submitter: caller_did
        });
        
        // Track submission for reputation
        update_reputation(
            caller_did,
            ReputationEventType::ProposalSubmitted,
            caller_did,
            format!("Submitted proposal {}", proposal_id)
        );
        
        proposal_id
    }
    
    fn vote_on_proposal(proposal_id: ProposalId, vote: VoteChoice) {
        let caller_did = caller();
        let mana_cost = calculate_action_cost(caller_did, ActionType::Vote);
        
        // Check and charge mana
        charge_mana(caller_did, mana_cost, "vote");
        
        // Submit vote
        cast_vote(proposal_id, vote, caller_did);
        
        // Track voting for reputation (small positive)
        update_reputation(
            caller_did,
            ReputationEventType::ParticipatedInVote,
            caller_did,
            format!("Voted on proposal {}", proposal_id)
        );
    }
    
    fn execute_proposal(proposal_id: ProposalId) {
        let proposal = get_proposal(proposal_id);
        require(proposal.status == ProposalStatus::Approved);
        
        // Execute the proposal (transfer funds, etc.)
        execute_proposal_actions(proposal);
        
        // Reward submitter with reputation boost if successful
        update_reputation(
            proposal.submitter,
            ReputationEventType::SuccessfulProposal,
            contract_address(),
            format!("Proposal {} executed successfully", proposal_id)
        );
        
        // Trigger mana parameter updates for all affected parties
        update_mana_parameters(proposal.submitter);
    }
}
```

---

## 8 · System Benefits and Emergent Properties

### 8.1 Positive Feedback Loops
- **High reputation** → **Higher mana capacity** → **More participation** → **More opportunities to earn reputation**
- **Good behavior** → **Community trust** → **Easier access to governance** → **More influence to do good**

### 8.2 Natural Rate Limiting
- **Spam prevention**: Low reputation actors can't flood the system
- **Sybil resistance**: New identities start with limited capacity
- **Quality incentives**: Better proposals/votes → better reputation → more capacity

### 8.3 Polycentric Governance
- **Local expertise**: High mana where you contribute most
- **Portable reputation**: Core credentials transfer across scopes
- **Graduated participation**: Earn trust and capacity over time

### 8.4 Economic Justice
- **Merit-based**: Capacity based on contribution, not wealth
- **Non-extractive**: Mana regenerates, isn't consumed permanently
- **Democratic**: Community controls reputation and mana policies

---

This system creates a **self-regulating, merit-based participation economy** where trust, identity, and capacity reinforce each other to enable effective cooperative governance at any scale. 