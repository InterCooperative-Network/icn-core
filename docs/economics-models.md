# ICN Economic Models

> **Beyond Capitalism: Designing Economies for Human Flourishing**

The InterCooperative Network (ICN) implements anti-capitalist economic models that prioritize collective benefit, resource sharing, and human dignity over profit maximization and capital accumulation. This document outlines the comprehensive economic system and its mechanisms.

---

## 🎯 **Core Economic Principles**

### **Anti-Capitalist Design**
- **No Speculation**: Economic tokens cannot be abstracted or used for financial speculation
- **Purpose-Bound Value**: All value tokens are scoped to specific capabilities and uses
- **Collective Ownership**: Resources and tools are shared rather than privately owned
- **Mutual Aid**: Economic flows support community needs and solidarity

### **Regenerative Economics**
- **Abundance Mindset**: Design for sufficiency rather than artificial scarcity
- **Ecological Integration**: Economic activity supports rather than degrades ecosystems
- **Social Reproduction**: Value care work and community maintenance
- **Circular Flows**: Resources circulate rather than being extracted and discarded

### **Democratic Control**
- **Participatory Budgeting**: Communities decide resource allocation democratically
- **Worker Ownership**: Those who create value control economic decisions
- **Transparent Operations**: All economic activity is visible and accountable
- **Equitable Distribution**: Benefits flow to all contributors, not just capital owners

---

## 💰 **Core Economic Mechanisms**

### **1. Mana System (Regenerating Resource Credits)**

Mana is ICN's primary coordination mechanism for compute resources and network participation. Unlike traditional currencies, mana cannot be accumulated indefinitely or used for speculation.

#### **Key Properties**
- **Regenerative**: Automatically replenishes over time
- **Non-Transferable**: Cannot be sold or traded (except under specific governance rules)
- **Purpose-Bound**: Only usable for compute and network participation
- **Reputation-Influenced**: Regeneration rate affected by contribution history

#### **Mana Mechanics**

The mana system now implements **capacity-aware regeneration** that rewards nodes based on their actual contribution to the network. The regeneration formula combines base rates with capacity and reputation factors:

**Regeneration Formula**: `regeneration = base_rate × capacity_factor × reputation_factor × time_elapsed`

```rust
pub struct ManaAccount {
    pub did: Did,
    pub current_balance: u64,
    pub max_capacity: u64,
    pub base_regeneration_rate: f64,
    pub last_regeneration: DateTime<Utc>,
    pub reputation_multiplier: f64,
    pub capacity_score: f64,  // New: from CapacityLedger
}

pub struct CapacityMetrics {
    pub compute_contribution: f64,    // CPU/GPU resources provided
    pub storage_contribution: f64,    // Storage space provided
    pub bandwidth_contribution: f64,  // Network bandwidth provided
    pub uptime_score: f64,           // Reliability and availability
    pub quality_score: f64,          // Performance and reliability metrics
}

impl ManaAccount {
    /// Enhanced regeneration with capacity-aware formula
    pub fn regenerate(&mut self, now: DateTime<Utc>, capacity_metrics: &CapacityMetrics) {
        let time_elapsed = now.signed_duration_since(self.last_regeneration);
        let hours_elapsed = time_elapsed.num_hours() as f64;
        
        // Calculate capacity factor from multiple contribution metrics
        let capacity_factor = self.calculate_capacity_factor(capacity_metrics);
        
        // Apply enhanced regeneration formula
        let regeneration = (
            self.base_regeneration_rate 
            * capacity_factor 
            * self.reputation_multiplier 
            * hours_elapsed
        ) as u64;
        
        self.current_balance = std::cmp::min(
            self.current_balance + regeneration,
            self.max_capacity
        );
        self.last_regeneration = now;
    }
    
    /// Calculate capacity factor from contribution metrics
    fn calculate_capacity_factor(&self, metrics: &CapacityMetrics) -> f64 {
        // Weighted average of different contribution types
        let compute_weight = 0.3;
        let storage_weight = 0.25;
        let bandwidth_weight = 0.25;
        let uptime_weight = 0.15;
        let quality_weight = 0.05;
        
        (metrics.compute_contribution * compute_weight +
         metrics.storage_contribution * storage_weight +
         metrics.bandwidth_contribution * bandwidth_weight +
         metrics.uptime_score * uptime_weight +
         metrics.quality_score * quality_weight)
        .max(0.1) // Minimum factor to ensure basic regeneration
        .min(3.0) // Maximum factor to prevent exploitation
    }
}
```

#### **Capacity-Based Spending Limits**

To prevent abuse, mana spending is also influenced by capacity scores:

```rust
impl ManaAccount {
    /// Check if spending is allowed based on capacity and balance
    pub fn can_spend(&self, amount: u64, capacity_metrics: &CapacityMetrics) -> bool {
        if self.current_balance < amount {
            return false;
        }
        
        // Higher capacity nodes can spend more freely
        let capacity_factor = self.calculate_capacity_factor(capacity_metrics);
        let max_spend_ratio = (0.5 + capacity_factor * 0.3).min(0.9);
        let max_spendable = (self.max_capacity as f64 * max_spend_ratio) as u64;
        
        amount <= max_spendable
    }
}
```

#### **Mana Use Cases**
- **Mesh Computing**: Pay for distributed computation jobs
- **Network Participation**: Cover costs of P2P networking
- **Storage Access**: Use decentralized storage services
- **Governance Participation**: Vote on proposals and decisions
- **Identity Operations**: Manage DIDs and credentials

### **2. Scoped Token Framework**

ICN supports purpose-bound tokens that represent specific capabilities or resources within the network. These tokens cannot be abstracted into generic currency.

#### **Token Types**
```rust
pub enum ScopedTokenType {
    Compute(ComputeSpecification),
    Storage(StorageSpecification),
    Bandwidth(BandwidthSpecification),
    Service(ServiceSpecification),
    Resource(ResourceSpecification),
}

pub struct ScopedToken {
    pub token_type: ScopedTokenType,
    pub issuer: Did,
    pub holder: Did,
    pub amount: u64,
    pub expiry: Option<DateTime<Utc>>,
    pub conditions: Vec<UsageCondition>,
    pub non_transferable: bool,
}
```

#### **Example Token Types**
- **`icn:compute/cpu-hours`**: CPU computation time
- **`icn:storage/gigabyte-months`**: Persistent storage space
- **`icn:bandwidth/gigabytes`**: Network data transfer
- **`icn:service/translation`**: Language translation services
- **`icn:resource/meeting-room`**: Physical space booking

### **3. Mutual Credit System**

ICN implements a complete mutual credit system that enables communities to create their own credit networks without relying on external capital or debt-based money.

#### **Core Mutual Credit Principles**
- **Community Issuance**: Credit is created by the community for the community
- **Zero Interest**: No interest charges on credit balances
- **Mutual Obligation**: Every credit is balanced by an equivalent debit
- **Democratic Control**: Community governs credit policies and limits
- **Anti-Speculation**: Credit cannot be used for speculative purposes

#### **Mutual Credit Implementation**
```rust
/// Mutual credit account with positive and negative balances
pub struct MutualCreditAccount {
    pub did: Did,
    pub community_id: String,
    pub balance: i64,              // Can be negative (debt)
    pub credit_limit: u64,         // Maximum debt allowed
    pub trust_score: f64,          // Community trust rating
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub account_status: AccountStatus,
}

/// Credit line between community members
pub struct CreditLine {
    pub creditor: Did,
    pub debtor: Did,
    pub amount: u64,
    pub purpose: String,
    pub terms: CreditTerms,
    pub status: CreditLineStatus,
    pub created_at: DateTime<Utc>,
    pub repayment_schedule: Vec<RepaymentRecord>,
}

pub enum CreditLineStatus {
    Proposed,        // Credit line proposed but not accepted
    Active,          // Credit line in use
    Repaying,        // Being repaid according to schedule
    Completed,       // Fully repaid
    Disputed,        // Under dispute resolution
    Suspended,       // Temporarily suspended
}

/// Community-specific credit terms
pub struct CreditTerms {
    pub repayment_period: Duration,
    pub required_guarantors: u32,
    pub collateral_required: bool,
    pub purpose_restrictions: Vec<String>,
    pub community_approval_required: bool,
}
```

#### **Mutual Credit Operations**
```rust
impl MutualCreditSystem {
    /// Issue credit between community members
    pub fn issue_credit(
        &self,
        creditor: &Did,
        debtor: &Did,
        amount: u64,
        purpose: String
    ) -> Result<CreditTransaction, CommonError> {
        // Validate community membership
        self.validate_community_membership(creditor, debtor)?;
        
        // Check credit limits
        self.check_credit_limits(debtor, amount)?;
        
        // Create balanced transaction (credit + debit)
        let transaction = CreditTransaction {
            id: generate_transaction_id(),
            creditor: creditor.clone(),
            debtor: debtor.clone(),
            amount,
            purpose,
            timestamp: Utc::now(),
            status: CreditTransactionStatus::Active,
        };
        
        // Update balances
        self.adjust_balance(creditor, amount as i64)?;    // Positive balance
        self.adjust_balance(debtor, -(amount as i64))?;   // Negative balance
        
        // Record transaction
        self.store.record_transaction(&transaction)?;
        
        Ok(transaction)
    }
    
    /// Community governance of credit policies
    pub fn update_community_credit_policy(
        &self,
        community_id: &str,
        policy: CreditPolicy,
        governance_proof: GovernanceProof
    ) -> Result<(), CommonError> {
        // Validate governance authorization
        self.validate_governance_proof(community_id, &governance_proof)?;
        
        // Apply policy update
        self.store.update_credit_policy(community_id, policy)?;
        
        // Notify community members
        self.notify_policy_update(community_id, &policy)?;
        
        Ok(())
    }
}
```

#### **Anti-Speculation Safeguards**
```rust
/// Enforce anti-speculation rules in mutual credit
pub struct AntiSpeculationEnforcer {
    max_credit_velocity: f64,      // Limit rapid trading
    purpose_validation: bool,       // Validate stated purpose
    community_oversight: bool,      // Require community approval
    transfer_restrictions: Vec<TransferRestriction>,
}

impl AntiSpeculationEnforcer {
    /// Validate credit transaction for speculation resistance
    pub fn validate_transaction(
        &self,
        transaction: &CreditTransaction
    ) -> Result<(), CommonError> {
        // Check velocity limits
        if self.exceeds_velocity_limit(&transaction)? {
            return Err(CommonError::PolicyDenied(
                "Transaction exceeds velocity limits"
            ));
        }
        
        // Validate stated purpose
        if self.purpose_validation {
            self.validate_purpose(&transaction)?;
        }
        
        // Check transfer restrictions
        for restriction in &self.transfer_restrictions {
            restriction.validate(transaction)?;
        }
        
        Ok(())
    }
}
```

### **4. Contribution Recognition System**

ICN recognizes multiple forms of contribution beyond just financial investment, ensuring that all types of valuable work are acknowledged and rewarded.

#### **Contribution Categories**
```rust
pub enum ContributionType {
    Labor {
        skill_level: SkillLevel,
        hours: f64,
        quality_rating: f64,
    },
    Knowledge {
        domain: String,
        impact_score: f64,
        peer_validation: bool,
    },
    Care {
        care_type: CareType,
        recipients: Vec<Did>,
        community_impact: f64,
    },
    Resources {
        resource_type: ResourceType,
        value: u64,
        duration: Duration,
    },
    Innovation {
        innovation_type: InnovationType,
        adoption_rate: f64,
        improvement_measure: f64,
    },
}
```

#### **Recognition Mechanisms**
- **Peer Recognition**: Community members acknowledge each other's contributions
- **Outcome Tracking**: Measure impact and results of contributions
- **Skill Development**: Support for learning and capability building
- **Mentorship Programs**: Experienced members guide newcomers

---

## 🌐 **Federated Economic Coordination**

### **Inter-Cooperative Trade**

Cooperatives can establish economic relationships while maintaining their autonomy and values alignment.

#### **Trust Networks**
```ccl
fn establish_trade_relationship(
    cooperative_a: Did,
    cooperative_b: Did,
    trade_terms: TradeTerms
) -> TradeAgreement {
    // Verify values alignment
    let values_compatible = check_values_alignment(cooperative_a, cooperative_b);
    if !values_compatible {
        return TradeAgreement::Rejected("Values misalignment");
    }
    
    // Assess economic compatibility
    let economic_compatibility = assess_economic_models(cooperative_a, cooperative_b);
    if economic_compatibility < 0.7 {
        return TradeAgreement::RequiresMediation;
    }
    
    // Create mutual agreement
    let agreement = TradeAgreement {
        parties: [cooperative_a, cooperative_b],
        terms: trade_terms,
        monitoring: create_monitoring_framework(),
        dispute_resolution: create_dispute_process(),
    };
    
    return TradeAgreement::Approved(agreement);
}
```

#### **Resource Sharing Protocols**
- **Surplus Distribution**: Share excess capacity across cooperatives
- **Mutual Aid Networks**: Support cooperatives during difficulties
- **Skill Exchange**: Share expertise and knowledge across organizations
- **Equipment Sharing**: Pool expensive tools and infrastructure

### **Federation Economic Governance**

Federations coordinate economic policy across multiple cooperatives while respecting local autonomy.

#### **Federated Budgeting**
```ccl
fn allocate_federation_resources(
    budget: FederationBudget,
    proposals: Vec<ResourceRequest>
) -> AllocationResult {
    let scored_proposals = proposals
        .map(|p| score_proposal(p))
        .sort_by_score();
    
    let democratic_weights = get_member_preferences();
    let expert_recommendations = get_expert_analysis();
    let impact_assessments = get_impact_projections();
    
    let final_allocation = optimize_allocation(
        scored_proposals,
        democratic_weights,
        expert_recommendations,
        impact_assessments
    );
    
    return AllocationResult::Approved(final_allocation);
}
```

---

## 📊 **Economic Metrics & Indicators**

### **Beyond GDP: Measuring What Matters**

ICN tracks economic indicators that reflect actual human and ecological wellbeing rather than just monetary flows.

#### **Wellbeing Indicators**
- **Basic Needs Security**: Access to housing, food, healthcare, education
- **Community Resilience**: Ability to respond to challenges and shocks
- **Ecological Health**: Environmental impact and regeneration measures
- **Social Cohesion**: Quality of relationships and mutual support
- **Creative Expression**: Opportunities for art, culture, and innovation

#### **Economic Health Metrics**
```rust
pub struct EconomicHealthIndicators {
    pub resource_abundance_ratio: f64,    // Available resources / needed resources
    pub contribution_diversity_index: f64, // Variety of recognized contributions
    pub economic_democracy_score: f64,     // Participation in economic decisions
    pub mutual_aid_flow_rate: f64,        // Rate of solidarity economy activity
    pub ecological_impact_score: f64,      // Environmental sustainability measure
}
```

### **Inequality Prevention**

ICN includes mechanisms to prevent the concentration of wealth and power that characterizes capitalist systems.

#### **Wealth Distribution Monitoring**
```ccl
fn monitor_wealth_concentration(community: Community) -> ConcentrationReport {
    let wealth_distribution = calculate_wealth_distribution(community);
    let gini_coefficient = calculate_gini(wealth_distribution);
    
    if gini_coefficient > 0.3 {
        trigger_redistribution_mechanisms(community);
    }
    
    let concentration_report = ConcentrationReport {
        gini_coefficient,
        wealth_percentiles: wealth_distribution,
        recommended_actions: get_redistribution_recommendations(),
    };
    
    return concentration_report;
}
```

#### **Anti-Accumulation Mechanisms**
- **Maximum Wealth Ratios**: Limits on individual vs. community wealth
- **Use-It-Or-Lose-It**: Resources that aren't actively used return to the commons
- **Progressive Contribution**: Higher expectations for those with more resources
- **Rotational Leadership**: Prevent permanent concentration of decision-making power

---

## 🔄 **Economic Lifecycle Management**

### **Resource Lifecycle Tracking**

ICN tracks resources through their entire lifecycle to optimize use and minimize waste.

#### **Lifecycle Stages**
```rust
pub enum ResourceLifecycleStage {
    Planning {
        needs_assessment: NeedsAssessment,
        resource_design: ResourceDesign,
    },
    Acquisition {
        sourcing_method: SourcingMethod,
        cost_accounting: CostAccounting,
    },
    Production {
        transformation_process: ProductionProcess,
        quality_metrics: QualityMetrics,
    },
    Distribution {
        allocation_mechanism: AllocationMechanism,
        access_criteria: AccessCriteria,
    },
    Usage {
        utilization_tracking: UtilizationMetrics,
        user_feedback: UserFeedback,
    },
    Maintenance {
        maintenance_schedule: MaintenanceSchedule,
        repair_history: RepairHistory,
    },
    EndOfLife {
        disposal_method: DisposalMethod,
        recovery_potential: RecoveryPotential,
    },
}
```

### **Circular Economy Patterns**

Resources flow in cycles rather than linear consumption patterns.

#### **Sharing Economy**
- **Tool Libraries**: Shared access to equipment and tools
- **Skill Sharing**: Exchange knowledge and capabilities
- **Space Sharing**: Optimize use of physical infrastructure
- **Time Banking**: Exchange time and labor directly

#### **Repair and Reuse**
- **Repair Cafes**: Community spaces for fixing and maintaining items
- **Upcycling Programs**: Transform waste into valuable resources
- **Component Recovery**: Harvest useful parts from end-of-life items
- **Knowledge Preservation**: Document repair and maintenance procedures

---

## 🌱 **Regenerative Economic Practices**

### **Ecological Integration**

Economic activity is designed to support rather than degrade ecological systems.

#### **Ecological Accounting**
```ccl
fn calculate_ecological_impact(activity: EconomicActivity) -> EcologicalImpact {
    let carbon_footprint = calculate_carbon_emissions(activity);
    let resource_depletion = calculate_resource_usage(activity);
    let biodiversity_impact = calculate_biodiversity_effect(activity);
    let pollution_generation = calculate_pollution_output(activity);
    
    let regenerative_benefits = calculate_regenerative_activities(activity);
    
    let net_impact = EcologicalImpact {
        carbon_balance: regenerative_benefits.carbon_sequestration - carbon_footprint,
        resource_balance: regenerative_benefits.resource_generation - resource_depletion,
        biodiversity_balance: regenerative_benefits.habitat_creation - biodiversity_impact,
        pollution_balance: regenerative_benefits.cleanup_activity - pollution_generation,
    };
    
    return net_impact;
}
```

### **Care Economy Integration**

ICN recognizes and supports care work as essential economic activity.

#### **Care Work Recognition**
- **Childcare**: Supporting the next generation
- **Elder Care**: Honoring and supporting older community members
- **Health Support**: Physical and mental health maintenance
- **Emotional Labor**: Community relationships and conflict resolution
- **Education**: Knowledge sharing and skill development

#### **Care Economy Metrics**
```rust
pub struct CareEconomyIndicators {
    pub care_workload_distribution: HashMap<Did, f64>,
    pub care_quality_metrics: CareQualityMetrics,
    pub care_infrastructure_investment: u64,
    pub care_worker_support_level: f64,
    pub community_wellbeing_index: f64,
}
```

---

## 🛠️ **Implementation Tools**

### **Economic Policy Templates**

ICN provides templates for common economic governance needs.

#### **Resource Allocation Policies**
```ccl
// Democratic budgeting template
fn democratic_budget_allocation(
    budget: Budget,
    proposals: Vec<BudgetProposal>,
    members: Vec<Did>
) -> BudgetAllocation {
    let voting_results = conduct_budget_vote(proposals, members);
    let expert_analysis = get_expert_recommendations(proposals);
    let impact_assessment = assess_community_impact(proposals);
    
    let final_allocation = combine_democratic_expert_input(
        voting_results,
        expert_analysis,
        impact_assessment
    );
    
    return BudgetAllocation::Approved(final_allocation);
}
```

#### **Contribution Recognition Policies**
```ccl
// Multi-factor contribution assessment
fn assess_contribution_value(contribution: Contribution) -> ContributionValue {
    let time_investment = contribution.hours_spent;
    let skill_level = assess_skill_requirement(contribution);
    let quality_rating = get_peer_quality_assessment(contribution);
    let community_impact = measure_community_benefit(contribution);
    
    let base_value = time_investment * skill_level;
    let quality_multiplier = 0.5 + (quality_rating * 1.5);
    let impact_multiplier = 0.8 + (community_impact * 0.4);
    
    let total_value = base_value * quality_multiplier * impact_multiplier;
    
    return ContributionValue {
        base_value,
        quality_adjusted: total_value,
        recognition_tokens: calculate_recognition_tokens(total_value),
        reputation_impact: calculate_reputation_effect(total_value),
    };
}
```

### **Economic Monitoring Dashboards**

Real-time visibility into economic health and equity.

#### **Key Dashboard Components**
- **Resource Flow Visualization**: See how resources move through the community
- **Contribution Recognition**: Track and celebrate diverse forms of contribution
- **Wealth Distribution**: Monitor concentration and inequality measures
- **Ecological Impact**: Track environmental benefits and costs
- **Care Economy**: Visualize care work and support systems

---

## 📚 **Getting Started**

### **For Cooperatives**
1. **Assess Current Economics**: Understand existing economic patterns
2. **Define Values**: Clarify economic principles and goals
3. **Choose Mechanisms**: Select appropriate economic tools and policies
4. **Pilot Programs**: Test new economic approaches gradually
5. **Community Education**: Help members understand new economic models
6. **Iterate and Improve**: Continuously refine economic systems

### **For Communities**
1. **Economic Visioning**: Imagine ideal economic relationships
2. **Needs Assessment**: Understand community resource needs
3. **Resource Mapping**: Identify available resources and capabilities
4. **Relationship Building**: Develop trust and cooperation
5. **Start Small**: Begin with simple sharing and mutual aid
6. **Scale Gradually**: Expand economic cooperation over time

### **For Federations**
1. **Values Alignment**: Ensure compatible economic principles
2. **Economic Compatibility**: Assess complementary economic models
3. **Protocol Development**: Create standards for inter-cooperative trade
4. **Pilot Exchanges**: Test economic relationships gradually
5. **Monitoring Systems**: Track economic relationship health
6. **Dispute Resolution**: Establish conflict resolution mechanisms

---

## 🔧 **CCL Economic Templates and Examples**

### **Default Regeneration Policy Template**

```ccl
economic_policy capacity_aware_regeneration {
    parameters {
        base_rate: 10.0,              // Base mana per hour
        capacity_weight: 0.6,         // Weight for capacity factor
        reputation_weight: 0.4,       // Weight for reputation factor
        max_capacity_factor: 3.0,     // Maximum capacity multiplier
        min_capacity_factor: 0.1,     // Minimum capacity multiplier
        max_reputation_factor: 2.0,   // Maximum reputation multiplier
        min_reputation_factor: 0.5    // Minimum reputation multiplier
    }
    
    function calculate_regeneration(
        did: DID,
        base_rate: f64,
        capacity_score: f64,
        reputation_score: f64
    ) -> f64 {
        let capacity_factor = clamp(
            capacity_score,
            min_capacity_factor,
            max_capacity_factor
        );
        
        let reputation_factor = clamp(
            reputation_score,
            min_reputation_factor,
            max_reputation_factor
        );
        
        return base_rate * (
            capacity_factor * capacity_weight +
            reputation_factor * reputation_weight
        );
    }
}
```

### **Mutual Credit Economic Model Template**

```ccl
economic_model community_mutual_credit {
    community_id: "ecovillage_001",
    
    token_class "community_credit" {
        type: MutualCredit,
        symbol: "ECO$",
        decimals: 2,
        transferability: RestrictedTransfer {
            authorized_recipients: community_members,
        },
        scoping_rules: {
            community_scope: community_id,
            geographic_scope: "bioregion_pacific_northwest",
            max_supply: none, // Unlimited mutual credit
            validity_period: none, // No expiration
        }
    }
    
    credit_policies {
        default_credit_limit: 500,     // Default credit limit
        max_credit_limit: 2000,        // Maximum credit limit
        trust_threshold: 0.7,          // Minimum trust score
        guarantor_requirement: 2,       // Required guarantors for large credits
        community_approval_threshold: 1000, // Amount requiring community approval
    }
    
    function calculate_credit_limit(did: DID) -> u64 {
        let base_limit = default_credit_limit;
        let trust_score = get_trust_score(did, community_id);
        let participation_score = get_participation_score(did, community_id);
        
        let multiplier = trust_score * 0.7 + participation_score * 0.3;
        
        return min(
            (base_limit as f64 * multiplier) as u64,
            max_credit_limit
        );
    }
    
    workflow issue_credit {
        trigger: credit_request_submitted
        
        step validate_request {
            require(is_community_member(request.debtor));
            require(is_community_member(request.creditor));
            require(request.amount <= calculate_credit_limit(request.debtor));
            
            if request.amount > community_approval_threshold {
                goto community_approval;
            } else {
                goto issue_credit;
            }
        }
        
        step community_approval {
            create_proposal(
                "Credit Line Approval",
                format!("{} requests {} credit from {}", 
                    request.debtor, request.amount, request.creditor)
            );
            
            if proposal_passes() {
                goto issue_credit;
            } else {
                reject_request("Community approval denied");
            }
        }
        
        step issue_credit {
            create_credit_line(request);
            adjust_balances(request.creditor, request.debtor, request.amount);
            record_transaction(request);
            notify_parties(request);
        }
    }
}
```

### **Time Banking Template**

```ccl
economic_model time_banking {
    community_id: "makerspace_collective",
    
    token_class "time_credit" {
        type: TimeBanking,
        symbol: "HOUR",
        decimals: 2, // Allow fractional hours
        transferability: RestrictedTransfer {
            authorized_recipients: community_members,
        },
        scoping_rules: {
            community_scope: community_id,
            validity_period: (now(), now() + 365_days), // Annual expiration
        }
    }
    
    skill_categories {
        "technical": {
            subcategories: ["programming", "electronics", "mechanical"],
            requires_certification: true,
        },
        "creative": {
            subcategories: ["design", "music", "writing"],
            requires_certification: false,
        },
        "care": {
            subcategories: ["childcare", "eldercare", "emotional_support"],
            requires_certification: true,
        },
        "maintenance": {
            subcategories: ["cleaning", "repairs", "organization"],
            requires_certification: false,
        }
    }
    
    function record_time_exchange(
        provider: DID,
        recipient: DID,
        hours: f64,
        skill_category: String,
        description: String
    ) -> TimeRecord {
        let record = TimeRecord {
            id: generate_uuid(),
            provider,
            recipient,
            hours,
            skill_category,
            description,
            timestamp: now(),
            status: TimeRecordStatus::Recorded,
        };
        
        // Equal value exchange: 1 hour = 1 hour regardless of skill
        mint("time_credit", provider, hours);
        burn("time_credit", recipient, hours);
        
        return record;
    }
    
    workflow verify_time_record {
        trigger: time_record_created
        
        step recipient_verification {
            send_verification_request(record.recipient, record);
            set_deadline(7_days);
            
            if verification_approved() {
                record.status = TimeRecordStatus::Verified;
                goto finalize_exchange;
            } else if verification_disputed() {
                goto dispute_resolution;
            } else {
                // Auto-approve after deadline
                record.status = TimeRecordStatus::Verified;
                goto finalize_exchange;
            }
        }
        
        step dispute_resolution {
            assign_mediator(record.provider, record.recipient);
            
            if mediation_successful() {
                apply_mediation_result(mediation_result);
                goto finalize_exchange;
            } else {
                goto community_arbitration;
            }
        }
        
        step community_arbitration {
            create_arbitration_panel(3_members);
            present_evidence(record, dispute_details);
            
            apply_arbitration_decision(arbitration_result);
            goto finalize_exchange;
        }
        
        step finalize_exchange {
            record.status = TimeRecordStatus::Finalized;
            update_reputation_scores(record.provider, record.recipient);
            archive_record(record);
        }
    }
}
```

### **Integration Tests Documentation**

#### **Capacity-Aware Regeneration Tests**

```rust
#[cfg(test)]
mod capacity_regeneration_tests {
    use super::*;
    
    #[test]
    fn test_capacity_aware_regeneration() {
        let mut mana_ledger = TestManaLedger::new();
        let mut capacity_ledger = TestCapacityLedger::new();
        
        // Setup test accounts with different capacity scores
        let high_capacity_did = Did::from_str("did:icn:high_capacity").unwrap();
        let low_capacity_did = Did::from_str("did:icn:low_capacity").unwrap();
        
        // Set initial balances
        mana_ledger.set_balance(&high_capacity_did, 100).unwrap();
        mana_ledger.set_balance(&low_capacity_did, 100).unwrap();
        
        // Set capacity scores
        capacity_ledger.set_capacity_score(&high_capacity_did, 2.0); // High contributor
        capacity_ledger.set_capacity_score(&low_capacity_did, 0.5);  // Low contributor
        
        // Regenerate mana with capacity awareness
        let engine = CCLEconomicEngine::new(mana_ledger, capacity_ledger);
        engine.regenerate_all_with_capacity().unwrap();
        
        // Verify high capacity node gets more mana
        let high_balance = engine.get_mana_balance(&high_capacity_did);
        let low_balance = engine.get_mana_balance(&low_capacity_did);
        
        assert!(high_balance > low_balance);
        assert!(high_balance >= 120); // Should get significant regeneration
        assert!(low_balance <= 110);  // Should get minimal regeneration
    }
}
```

#### **CCL Policy Override Tests**

```rust
#[test]
fn test_ccl_economic_policy_override() {
    let mut engine = create_test_economic_engine();
    
    // Deploy custom regeneration policy
    let policy_code = r#"
        economic_policy custom_regeneration {
            function calculate_regeneration(
                did: DID,
                base_rate: f64,
                capacity: f64,
                reputation: f64
            ) -> f64 {
                // Custom formula favoring reputation over capacity
                base_rate * (capacity * 0.2 + reputation * 0.8)
            }
        }
    "#;
    
    engine.deploy_ccl_policy("custom_regeneration", policy_code).unwrap();
    
    // Test that custom policy is applied
    let test_did = Did::from_str("did:icn:test").unwrap();
    engine.set_capacity_score(&test_did, 2.0);
    engine.set_reputation_score(&test_did, 1.5);
    
    let regenerated = engine.calculate_regeneration(&test_did, 10.0);
    
    // Should use custom formula: 10 * (2.0 * 0.2 + 1.5 * 0.8) = 16.0
    assert_eq!(regenerated, 16.0);
}
```

---

## 🤝 **Support Resources**

### **Economic Design Support**
- **Economic Modeling**: Simulation and analysis tools
- **Policy Development**: CCL template customization
- **Training Programs**: Economic literacy and cooperation skills
- **Consultation Services**: Expert guidance on economic design

### **Community Resources**
- **Economics Forum**: [economics.intercooperative.network](https://economics.intercooperative.network)
- **Case Studies**: Real-world examples of cooperative economics
- **Research Library**: Academic papers and analysis
- **Tool Library**: Software tools for economic management

---

**ICN's economic models demonstrate that alternatives to capitalism are not only possible but necessary for human and ecological thriving. Join us in building economies that serve life rather than capital.** 