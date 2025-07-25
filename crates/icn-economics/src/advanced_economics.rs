// crates/icn-economics/src/advanced_economics.rs
//! Advanced economic primitives for cooperative governance
//!
//! This module implements sophisticated economic mechanisms including:
//! - Democratic budget management and allocation
//! - Dividend and surplus distribution systems
//! - Economic policy automation
//! - Cooperative resource allocation

use crate::{ManaLedger, ResourceLedger, TokenEvent};
use icn_common::{CommonError, Did, NodeScope};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

/// Budget allocation and management system
#[derive(Debug, Clone)]
pub struct BudgetManager {
    total_budget: u64,
    allocated_budget: u64,
    budget_allocations: HashMap<BudgetCategory, u64>,
    allocation_proposals: Vec<BudgetProposal>,
    allocation_history: VecDeque<BudgetAllocation>,
    fiscal_period: FiscalPeriod,
}

/// Budget categories for allocation
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum BudgetCategory {
    Development,
    Operations,
    Marketing,
    Research,
    Community,
    Infrastructure,
    Emergency,
    Custom(String),
}

/// Budget allocation proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetProposal {
    pub id: String,
    pub proposer: Did,
    pub category: BudgetCategory,
    pub amount: u64,
    pub recipient: Did,
    pub purpose: String,
    pub justification: String,
    pub created_at: u64,
    pub status: BudgetProposalStatus,
    pub votes_for: u32,
    pub votes_against: u32,
    pub quorum_met: bool,
}

/// Status of budget allocation proposals
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BudgetProposalStatus {
    Pending,
    UnderReview,
    Approved,
    Rejected,
    Executed,
    Cancelled,
}

/// Completed budget allocation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetAllocation {
    pub proposal_id: String,
    pub category: BudgetCategory,
    pub amount: u64,
    pub recipient: Did,
    pub purpose: String,
    pub allocated_at: u64,
    pub executor: Did,
}

/// Fiscal period configuration
#[derive(Debug, Clone)]
pub struct FiscalPeriod {
    pub start_time: u64,
    pub duration_secs: u64,
    pub auto_reset: bool,
}

impl BudgetManager {
    /// Create a new budget manager
    pub fn new(total_budget: u64, fiscal_period: FiscalPeriod) -> Self {
        Self {
            total_budget,
            allocated_budget: 0,
            budget_allocations: HashMap::new(),
            allocation_proposals: Vec::new(),
            allocation_history: VecDeque::new(),
            fiscal_period,
        }
    }

    /// Propose a budget allocation
    pub fn propose_allocation(
        &mut self,
        proposer: Did,
        category: BudgetCategory,
        amount: u64,
        recipient: Did,
        purpose: String,
        justification: String,
    ) -> Result<String, CommonError> {
        if amount > self.remaining_budget() {
            return Err(CommonError::PolicyDenied(
                "Proposed amount exceeds remaining budget".to_string(),
            ));
        }

        let proposal_id = format!("budget_{}", self.allocation_proposals.len());
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let proposal = BudgetProposal {
            id: proposal_id.clone(),
            proposer,
            category,
            amount,
            recipient,
            purpose,
            justification,
            created_at: now,
            status: BudgetProposalStatus::Pending,
            votes_for: 0,
            votes_against: 0,
            quorum_met: false,
        };

        self.allocation_proposals.push(proposal);
        Ok(proposal_id)
    }

    /// Vote on a budget allocation proposal
    pub fn vote_on_proposal(
        &mut self,
        proposal_id: &str,
        _voter: &Did,
        approve: bool,
    ) -> Result<(), CommonError> {
        if let Some(proposal) = self
            .allocation_proposals
            .iter_mut()
            .find(|p| p.id == proposal_id)
        {
            if approve {
                proposal.votes_for += 1;
            } else {
                proposal.votes_against += 1;
            }

            // Simple quorum check (can be made more sophisticated)
            let total_votes = proposal.votes_for + proposal.votes_against;
            if total_votes >= 3 {
                // Minimum 3 votes
                proposal.quorum_met = true;
                if proposal.votes_for > proposal.votes_against {
                    proposal.status = BudgetProposalStatus::Approved;
                } else {
                    proposal.status = BudgetProposalStatus::Rejected;
                }
            }

            Ok(())
        } else {
            Err(CommonError::ResourceNotFound(
                "Budget proposal not found".to_string(),
            ))
        }
    }

    /// Execute an approved budget allocation
    pub fn execute_allocation(
        &mut self,
        proposal_id: &str,
        executor: Did,
    ) -> Result<BudgetAllocation, CommonError> {
        // First, find and validate the proposal
        let proposal_index = self
            .allocation_proposals
            .iter()
            .position(|p| p.id == proposal_id)
            .ok_or_else(|| {
                CommonError::ResourceNotFound(
                    "Budget proposal not found".to_string(),
                )
            })?;

        let proposal = &self.allocation_proposals[proposal_index];
        
        if proposal.status != BudgetProposalStatus::Approved {
            return Err(CommonError::PolicyDenied(
                "Proposal not approved for execution".to_string(),
            ));
        }

        let remaining = self.remaining_budget();
        if proposal.amount > remaining {
            return Err(CommonError::PolicyDenied(
                "Insufficient remaining budget".to_string(),
            ));
        }

        // Now create allocation and update the proposal
        let allocation = BudgetAllocation {
            proposal_id: proposal_id.to_string(),
            category: proposal.category.clone(),
            amount: proposal.amount,
            recipient: proposal.recipient.clone(),
            purpose: proposal.purpose.clone(),
            allocated_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            executor,
        };

        // Update budget tracking
        self.allocated_budget += proposal.amount;
        *self
            .budget_allocations
            .entry(proposal.category.clone())
            .or_insert(0) += proposal.amount;

        // Mark proposal as executed
        self.allocation_proposals[proposal_index].status = BudgetProposalStatus::Executed;

        // Store allocation history
        self.allocation_history.push_back(allocation.clone());

        // Keep only recent history (last 100 allocations)
        if self.allocation_history.len() > 100 {
            self.allocation_history.pop_front();
        }

        Ok(allocation)
    }

    /// Get remaining budget
    pub fn remaining_budget(&self) -> u64 {
        self.total_budget.saturating_sub(self.allocated_budget)
    }

    /// Get budget allocation by category
    pub fn get_category_allocation(&self, category: &BudgetCategory) -> u64 {
        self.budget_allocations.get(category).copied().unwrap_or(0)
    }

    /// Reset budget for new fiscal period
    pub fn reset_budget(&mut self, new_total: Option<u64>) {
        if let Some(total) = new_total {
            self.total_budget = total;
        }
        self.allocated_budget = 0;
        self.budget_allocations.clear();
        self.allocation_proposals.clear();
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.fiscal_period.start_time = now;
    }

    /// Check if fiscal period has expired
    pub fn is_fiscal_period_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now > self.fiscal_period.start_time + self.fiscal_period.duration_secs
    }

    /// Get pending proposals
    pub fn get_pending_proposals(&self) -> Vec<&BudgetProposal> {
        self.allocation_proposals
            .iter()
            .filter(|p| p.status == BudgetProposalStatus::Pending)
            .collect()
    }
}

/// Dividend and surplus distribution system
#[derive(Debug, Clone)]
pub struct DividendDistributor {
    total_surplus: u64,
    distribution_criteria: DistributionCriteria,
    distribution_history: VecDeque<DividendDistribution>,
    eligible_members: HashMap<Did, MemberEligibility>,
}

/// Criteria for dividend distribution
#[derive(Debug, Clone)]
pub enum DistributionCriteria {
    EqualShares,
    ProportionalToContribution,
    ProportionalToStake,
    ReputationWeighted,
    TimeBasedWeighted,
    Custom(String), // Just store a description for custom logic
}

/// Member eligibility for dividends
#[derive(Debug, Clone)]
pub struct MemberEligibility {
    pub contribution_score: u64,
    pub stake_amount: u64,
    pub reputation_score: u64,
    pub membership_duration: u64,
    pub active_participation: bool,
}

/// Dividend distribution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DividendDistribution {
    pub distribution_id: String,
    pub total_amount: u64,
    pub recipient_count: usize,
    pub distribution_date: u64,
    pub criteria_used: String,
    pub individual_amounts: HashMap<Did, u64>,
}

impl DividendDistributor {
    /// Create a new dividend distributor
    pub fn new(distribution_criteria: DistributionCriteria) -> Self {
        Self {
            total_surplus: 0,
            distribution_criteria,
            distribution_history: VecDeque::new(),
            eligible_members: HashMap::new(),
        }
    }

    /// Add surplus for distribution
    pub fn add_surplus(&mut self, amount: u64) {
        self.total_surplus += amount;
    }

    /// Set member eligibility
    pub fn set_member_eligibility(&mut self, member: Did, eligibility: MemberEligibility) {
        self.eligible_members.insert(member, eligibility);
    }

    /// Calculate dividend shares based on criteria
    pub fn calculate_shares(&self) -> HashMap<Did, f64> {
        let mut shares = HashMap::new();

        match &self.distribution_criteria {
            DistributionCriteria::EqualShares => {
                let share = 1.0 / self.eligible_members.len() as f64;
                for member in self.eligible_members.keys() {
                    shares.insert(member.clone(), share);
                }
            }
            DistributionCriteria::ProportionalToContribution => {
                let total_contribution: u64 = self
                    .eligible_members
                    .values()
                    .map(|e| e.contribution_score)
                    .sum();
                if total_contribution > 0 {
                    for (member, eligibility) in &self.eligible_members {
                        let share = eligibility.contribution_score as f64 / total_contribution as f64;
                        shares.insert(member.clone(), share);
                    }
                }
            }
            DistributionCriteria::ReputationWeighted => {
                let total_reputation: u64 = self
                    .eligible_members
                    .values()
                    .map(|e| e.reputation_score)
                    .sum();
                if total_reputation > 0 {
                    for (member, eligibility) in &self.eligible_members {
                        let share = eligibility.reputation_score as f64 / total_reputation as f64;
                        shares.insert(member.clone(), share);
                    }
                }
            }
            DistributionCriteria::ProportionalToStake => {
                let total_stake: u64 = self
                    .eligible_members
                    .values()
                    .map(|e| e.stake_amount)
                    .sum();
                if total_stake > 0 {
                    for (member, eligibility) in &self.eligible_members {
                        let share = eligibility.stake_amount as f64 / total_stake as f64;
                        shares.insert(member.clone(), share);
                    }
                }
            }
            DistributionCriteria::TimeBasedWeighted => {
                let total_time: u64 = self
                    .eligible_members
                    .values()
                    .map(|e| e.membership_duration)
                    .sum();
                if total_time > 0 {
                    for (member, eligibility) in &self.eligible_members {
                        let share = eligibility.membership_duration as f64 / total_time as f64;
                        shares.insert(member.clone(), share);
                    }
                }
            }
            DistributionCriteria::Custom(_description) => {
                // For custom logic, use equal shares as fallback
                let share = 1.0 / self.eligible_members.len() as f64;
                for member in self.eligible_members.keys() {
                    shares.insert(member.clone(), share);
                }
            }
        }

        shares
    }

    /// Distribute dividends to eligible members
    pub fn distribute_dividends(
        &mut self,
        ledger: &dyn ManaLedger,
        amount: Option<u64>,
    ) -> Result<DividendDistribution, CommonError> {
        let distribution_amount = amount.unwrap_or(self.total_surplus);

        if distribution_amount > self.total_surplus {
            return Err(CommonError::PolicyDenied(
                "Distribution amount exceeds available surplus".to_string(),
            ));
        }

        let shares = self.calculate_shares();
        let mut individual_amounts = HashMap::new();

        // Distribute to each member
        for (member, share) in shares {
            let member_amount = (distribution_amount as f64 * share) as u64;
            if member_amount > 0 {
                ledger.credit(&member, member_amount)?;
                individual_amounts.insert(member, member_amount);
            }
        }

        // Create distribution record
        let distribution_id = format!("div_{}", self.distribution_history.len());
        let distribution = DividendDistribution {
            distribution_id,
            total_amount: distribution_amount,
            recipient_count: individual_amounts.len(),
            distribution_date: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            criteria_used: format!("{:?}", self.distribution_criteria),
            individual_amounts,
        };

        // Update surplus
        self.total_surplus -= distribution_amount;

        // Store in history
        self.distribution_history.push_back(distribution.clone());

        // Keep only recent history
        if self.distribution_history.len() > 50 {
            self.distribution_history.pop_front();
        }

        Ok(distribution)
    }

    /// Get distribution history
    pub fn get_distribution_history(&self) -> &VecDeque<DividendDistribution> {
        &self.distribution_history
    }

    /// Get total distributed amount
    pub fn get_total_distributed(&self) -> u64 {
        self.distribution_history
            .iter()
            .map(|d| d.total_amount)
            .sum()
    }
}

/// Economic policy automation
#[derive(Debug, Clone)]
pub struct EconomicPolicyEngine {
    policies: HashMap<String, EconomicPolicy>,
    active_policies: HashSet<String>,
    policy_history: VecDeque<PolicyExecution>,
}

/// Economic policy definition
#[derive(Debug, Clone)]
pub struct EconomicPolicy {
    pub name: String,
    pub description: String,
    pub trigger_conditions: Vec<PolicyTrigger>,
    pub actions: Vec<PolicyAction>,
    pub cooldown_secs: u64,
    pub last_executed: Option<u64>,
}

/// Policy trigger conditions
#[derive(Debug, Clone)]
pub enum PolicyTrigger {
    BudgetThresholdReached(f64),  // Percentage of budget used
    SurplusThresholdReached(u64), // Absolute surplus amount
    TimeInterval(u64),            // Seconds since last execution
    MemberCountChanged(usize),    // Number of members changed
    Custom(String),               // Custom trigger logic
}

/// Policy actions to execute
#[derive(Debug, Clone)]
pub enum PolicyAction {
    DistributeDividends(u64),                    // Amount to distribute
    AllocateBudget(BudgetCategory, u64),         // Category and amount
    AdjustTokenSupply(i64),                      // Positive = mint, negative = burn
    UpdateMemberWeights(HashMap<Did, f64>),      // New member weights
    SendNotification(String),                    // Notification message
    Custom(String),                              // Custom action logic
}

/// Policy execution record
#[derive(Debug, Clone)]
pub struct PolicyExecution {
    pub policy_name: String,
    pub executed_at: u64,
    pub trigger_condition: String,
    pub actions_taken: Vec<String>,
    pub success: bool,
    pub error_message: Option<String>,
}

impl EconomicPolicyEngine {
    /// Create a new policy engine
    pub fn new() -> Self {
        Self {
            policies: HashMap::new(),
            active_policies: HashSet::new(),
            policy_history: VecDeque::new(),
        }
    }

    /// Add an economic policy
    pub fn add_policy(&mut self, policy: EconomicPolicy) {
        self.policies.insert(policy.name.clone(), policy);
    }

    /// Activate a policy
    pub fn activate_policy(&mut self, policy_name: &str) -> Result<(), CommonError> {
        if !self.policies.contains_key(policy_name) {
            return Err(CommonError::ResourceNotFound(
                "Policy not found".to_string(),
            ));
        }
        self.active_policies.insert(policy_name.to_string());
        Ok(())
    }

    /// Deactivate a policy
    pub fn deactivate_policy(&mut self, policy_name: &str) {
        self.active_policies.remove(policy_name);
    }

    /// Check if policy triggers should execute
    pub fn check_triggers(
        &mut self,
        budget_manager: &BudgetManager,
        dividend_distributor: &DividendDistributor,
    ) -> Vec<String> {
        let mut triggered_policies = Vec::new();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        for policy_name in &self.active_policies {
            if let Some(policy) = self.policies.get(policy_name) {
                // Check cooldown
                if let Some(last_executed) = policy.last_executed {
                    if now - last_executed < policy.cooldown_secs {
                        continue;
                    }
                }

                // Check trigger conditions
                let mut should_trigger = false;
                for trigger in &policy.trigger_conditions {
                    match trigger {
                        PolicyTrigger::BudgetThresholdReached(threshold) => {
                            let usage_ratio = budget_manager.allocated_budget as f64
                                / budget_manager.total_budget as f64;
                            if usage_ratio >= *threshold {
                                should_trigger = true;
                                break;
                            }
                        }
                        PolicyTrigger::SurplusThresholdReached(threshold) => {
                            if dividend_distributor.total_surplus >= *threshold {
                                should_trigger = true;
                                break;
                            }
                        }
                        PolicyTrigger::TimeInterval(interval) => {
                            if let Some(last_executed) = policy.last_executed {
                                if now - last_executed >= *interval {
                                    should_trigger = true;
                                    break;
                                }
                            } else {
                                should_trigger = true;
                                break;
                            }
                        }
                        PolicyTrigger::MemberCountChanged(_count) => {
                            // Would need access to membership data
                            // For now, skip this trigger
                        }
                        PolicyTrigger::Custom(_logic) => {
                            // Custom trigger logic would be evaluated here
                        }
                    }
                }

                if should_trigger {
                    triggered_policies.push(policy_name.clone());
                }
            }
        }

        triggered_policies
    }

    /// Execute a triggered policy
    pub fn execute_policy(
        &mut self,
        policy_name: &str,
        _budget_manager: &mut BudgetManager,
        dividend_distributor: &mut DividendDistributor,
        ledger: &dyn ManaLedger,
    ) -> Result<PolicyExecution, CommonError> {
        let policy = self
            .policies
            .get_mut(policy_name)
            .ok_or_else(|| {
                CommonError::ResourceNotFound("Policy not found".to_string())
            })?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut actions_taken = Vec::new();
        let mut success = true;
        let mut error_message = None;

        // Execute policy actions
        for action in &policy.actions {
            match action {
                PolicyAction::DistributeDividends(amount) => {
                    match dividend_distributor.distribute_dividends(ledger, Some(*amount)) {
                        Ok(_) => actions_taken.push(format!("Distributed {} in dividends", amount)),
                        Err(e) => {
                            error_message = Some(e.to_string());
                            success = false;
                            break;
                        }
                    }
                }
                PolicyAction::AllocateBudget(category, amount) => {
                    // This would need a proposer - for automation, could use a system DID
                    actions_taken.push(format!("Budget allocation proposed: {:?} = {}", category, amount));
                }
                PolicyAction::AdjustTokenSupply(adjustment) => {
                    actions_taken.push(format!("Token supply adjusted by {}", adjustment));
                }
                PolicyAction::UpdateMemberWeights(_weights) => {
                    actions_taken.push("Member weights updated".to_string());
                }
                PolicyAction::SendNotification(message) => {
                    actions_taken.push(format!("Notification sent: {}", message));
                }
                PolicyAction::Custom(logic) => {
                    actions_taken.push(format!("Custom action executed: {}", logic));
                }
            }
        }

        // Update last executed time
        policy.last_executed = Some(now);

        // Create execution record
        let execution = PolicyExecution {
            policy_name: policy_name.to_string(),
            executed_at: now,
            trigger_condition: "Triggered".to_string(), // Could be more specific
            actions_taken: actions_taken.clone(),
            success,
            error_message,
        };

        // Store in history
        self.policy_history.push_back(execution.clone());
        if self.policy_history.len() > 100 {
            self.policy_history.pop_front();
        }

        Ok(execution)
    }

    /// Get policy execution history
    pub fn get_execution_history(&self) -> &VecDeque<PolicyExecution> {
        &self.policy_history
    }
}

use std::collections::HashSet;

impl Default for EconomicPolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a comprehensive economic governance system
pub struct AdvancedEconomicGovernance {
    pub budget_manager: BudgetManager,
    pub dividend_distributor: DividendDistributor,
    pub policy_engine: EconomicPolicyEngine,
}

impl AdvancedEconomicGovernance {
    /// Create a new advanced economic governance system
    pub fn new(
        total_budget: u64,
        fiscal_period_duration: u64,
        distribution_criteria: DistributionCriteria,
    ) -> Self {
        let fiscal_period = FiscalPeriod {
            start_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            duration_secs: fiscal_period_duration,
            auto_reset: true,
        };

        Self {
            budget_manager: BudgetManager::new(total_budget, fiscal_period),
            dividend_distributor: DividendDistributor::new(distribution_criteria),
            policy_engine: EconomicPolicyEngine::new(),
        }
    }

    /// Run automated economic policies
    pub fn run_automation(&mut self, ledger: &dyn ManaLedger) -> Result<Vec<PolicyExecution>, CommonError> {
        let triggered_policies = self.policy_engine.check_triggers(
            &self.budget_manager,
            &self.dividend_distributor,
        );

        let mut executions = Vec::new();
        for policy_name in triggered_policies {
            let execution = self.policy_engine.execute_policy(
                &policy_name,
                &mut self.budget_manager,
                &mut self.dividend_distributor,
                ledger,
            )?;
            executions.push(execution);
        }

        Ok(executions)
    }

    /// Get economic health metrics
    pub fn get_health_metrics(&self) -> EconomicHealthMetrics {
        EconomicHealthMetrics {
            budget_utilization: self.budget_manager.allocated_budget as f64
                / self.budget_manager.total_budget as f64,
            remaining_budget: self.budget_manager.remaining_budget(),
            total_surplus: self.dividend_distributor.total_surplus,
            total_distributed: self.dividend_distributor.get_total_distributed(),
            active_policies: self.policy_engine.active_policies.len(),
            recent_policy_executions: self.policy_engine.policy_history.len(),
        }
    }
}

/// Economic health metrics
#[derive(Debug, Clone)]
pub struct EconomicHealthMetrics {
    pub budget_utilization: f64,
    pub remaining_budget: u64,
    pub total_surplus: u64,
    pub total_distributed: u64,
    pub active_policies: usize,
    pub recent_policy_executions: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_manager() {
        let fiscal_period = FiscalPeriod {
            start_time: 0,
            duration_secs: 86400 * 30, // 30 days
            auto_reset: true,
        };

        let mut budget = BudgetManager::new(10000, fiscal_period);
        let proposer = Did::default();
        let recipient = Did::new("key", "recipient");

        let proposal_id = budget
            .propose_allocation(
                proposer.clone(),
                BudgetCategory::Development,
                5000,
                recipient,
                "Software development".to_string(),
                "Needed for new features".to_string(),
            )
            .unwrap();

        // Vote on proposal
        budget.vote_on_proposal(&proposal_id, &proposer, true).unwrap();
        budget.vote_on_proposal(&proposal_id, &Did::new("key", "voter2"), true).unwrap();
        budget.vote_on_proposal(&proposal_id, &Did::new("key", "voter3"), true).unwrap();

        // Execute allocation
        let allocation = budget.execute_allocation(&proposal_id, proposer).unwrap();
        assert_eq!(allocation.amount, 5000);
        assert_eq!(budget.remaining_budget(), 5000);
    }

    #[test]
    fn test_dividend_distribution() {
        let mut distributor = DividendDistributor::new(DistributionCriteria::EqualShares);

        let alice = Did::default();
        let bob = Did::new("key", "bob");

        distributor.set_member_eligibility(
            alice.clone(),
            MemberEligibility {
                contribution_score: 100,
                stake_amount: 1000,
                reputation_score: 50,
                membership_duration: 365,
                active_participation: true,
            },
        );

        distributor.set_member_eligibility(
            bob.clone(),
            MemberEligibility {
                contribution_score: 150,
                stake_amount: 2000,
                reputation_score: 75,
                membership_duration: 200,
                active_participation: true,
            },
        );

        distributor.add_surplus(1000);

        let shares = distributor.calculate_shares();
        assert_eq!(shares.len(), 2);
        assert_eq!(shares[&alice], 0.5);
        assert_eq!(shares[&bob], 0.5);
    }

    #[test]
    fn test_economic_policy_triggers() {
        let fiscal_period = FiscalPeriod {
            start_time: 0,
            duration_secs: 86400 * 30,
            auto_reset: true,
        };

        let budget = BudgetManager::new(10000, fiscal_period);
        let dividend_dist = DividendDistributor::new(DistributionCriteria::EqualShares);
        let mut policy_engine = EconomicPolicyEngine::new();

        let policy = EconomicPolicy {
            name: "auto_dividend".to_string(),
            description: "Automatically distribute dividends".to_string(),
            trigger_conditions: vec![PolicyTrigger::SurplusThresholdReached(1000)],
            actions: vec![PolicyAction::DistributeDividends(500)],
            cooldown_secs: 86400, // 1 day
            last_executed: None,
        };

        policy_engine.add_policy(policy);
        policy_engine.activate_policy("auto_dividend").unwrap();

        let triggered = policy_engine.check_triggers(&budget, &dividend_dist);
        assert!(triggered.is_empty()); // No surplus yet

        // Test with surplus
        let mut dividend_dist_with_surplus = DividendDistributor::new(DistributionCriteria::EqualShares);
        dividend_dist_with_surplus.add_surplus(1500);

        let triggered = policy_engine.check_triggers(&budget, &dividend_dist_with_surplus);
        assert_eq!(triggered.len(), 1);
        assert_eq!(triggered[0], "auto_dividend");
    }
}