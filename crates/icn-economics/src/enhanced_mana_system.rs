//! Enhanced Mana System with Contribution-Weighted Regeneration
//!
//! This module extends the basic mana system with advanced regeneration mechanics
//! based on contribution levels, capacity scores, and organizational participation.

use crate::{ManaLedger, OrganizationType};
use icn_common::{CommonError, Did, SystemTimeProvider, TimeProvider};
use icn_reputation::ReputationStore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Enhanced mana account with contribution tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedManaAccount {
    pub did: Did,
    pub current_balance: u64,
    pub max_capacity: u64,
    pub base_regeneration_rate: f64,
    pub last_regeneration: u64,
    pub contribution_metrics: ContributionMetrics,
    pub capacity_metrics: CapacityMetrics,
    pub organizational_bonuses: OrganizationalBonuses,
    pub cooperation_bonuses: CooperationBonuses,
    pub anti_accumulation_state: AntiAccumulationState,
}

/// Metrics for tracking various types of contributions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributionMetrics {
    pub compute_contribution: f64,    // CPU/GPU resources provided
    pub storage_contribution: f64,    // Storage space provided
    pub bandwidth_contribution: f64,  // Network bandwidth provided
    pub governance_participation: f64, // Participation in governance
    pub mutual_aid_provided: f64,     // Resources given to others
    pub knowledge_sharing: f64,       // Documentation, teaching, etc.
    pub community_building: f64,      // Social coordination activities
    pub innovation_contribution: f64, // New ideas, improvements
    pub care_work: f64,              // Emotional labor, support
    pub last_updated: u64,
}

/// Capacity metrics for network resource provision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityMetrics {
    pub total_compute_capacity: u64,
    pub available_compute_capacity: u64,
    pub total_storage_capacity: u64,
    pub available_storage_capacity: u64,
    pub network_bandwidth: u64,
    pub uptime_percentage: f64,
    pub reliability_score: f64,
    pub quality_metrics: QualityMetrics,
}

/// Quality metrics for resource provision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub job_completion_rate: f64,
    pub average_response_time: u64,
    pub error_rate: f64,
    pub user_satisfaction_score: f64,
}

/// Bonuses from organizational membership
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationalBonuses {
    pub organization_type_bonus: f64,
    pub leadership_bonus: f64,
    pub inter_org_coordination_bonus: f64,
    pub solidarity_bonus: f64,
}

/// Bonuses from cooperation and mutual aid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CooperationBonuses {
    pub trust_score: f64,
    pub mutual_aid_multiplier: f64,
    pub collective_action_bonus: f64,
    pub conflict_resolution_bonus: f64,
}

/// State for anti-accumulation mechanisms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiAccumulationState {
    pub accumulation_penalty: f64,        // Penalty for excessive accumulation
    pub last_significant_spend: u64,      // Last time significant mana was spent
    pub hoarding_detection_score: f64,    // Score indicating hoarding behavior
    pub redistribution_contributions: u64, // Contributions to redistribution
}

/// Regeneration policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegenerationPolicy {
    pub base_rate: f64,
    pub contribution_weights: ContributionWeights,
    pub capacity_weights: CapacityWeights,
    pub organizational_multipliers: OrganizationalMultipliers,
    pub cooperation_multipliers: CooperationMultipliers,
    pub anti_accumulation_config: AntiAccumulationConfig,
    pub emergency_adjustments: EmergencyAdjustments,
}

/// Weights for different contribution types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributionWeights {
    pub compute: f64,
    pub storage: f64,
    pub bandwidth: f64,
    pub governance: f64,
    pub mutual_aid: f64,
    pub knowledge_sharing: f64,
    pub community_building: f64,
    pub innovation: f64,
    pub care_work: f64,
}

/// Weights for capacity metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityWeights {
    pub compute_capacity: f64,
    pub storage_capacity: f64,
    pub bandwidth_capacity: f64,
    pub uptime: f64,
    pub reliability: f64,
    pub quality: f64,
}

/// Multipliers for organizational participation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationalMultipliers {
    pub coop_member: f64,
    pub community_member: f64,
    pub federation_member: f64,
    pub leadership_role: f64,
    pub bridge_builder: f64,
}

/// Multipliers for cooperation activities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CooperationMultipliers {
    pub high_trust: f64,
    pub mutual_aid_provider: f64,
    pub collective_action: f64,
    pub conflict_resolver: f64,
    pub solidarity_network: f64,
}

/// Configuration for anti-accumulation mechanisms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiAccumulationConfig {
    pub max_accumulation_ratio: f64,      // Max mana as ratio of average
    pub hoarding_threshold: f64,          // Threshold for hoarding detection
    pub penalty_escalation_rate: f64,     // How quickly penalties increase
    pub redistribution_threshold: u64,    // Mana amount triggering redistribution
    pub use_it_or_lose_it_period: u64,   // Time before unused mana decays
}

/// Emergency adjustments to regeneration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyAdjustments {
    pub crisis_response_bonus: f64,
    pub mutual_aid_emergency_multiplier: f64,
    pub basic_needs_guarantee: u64,
    pub solidarity_crisis_factor: f64,
}

/// Enhanced mana ledger with contribution-weighted regeneration
pub struct ContributionWeightedManaLedger<L: ManaLedger, R: ReputationStore> {
    base_ledger: L,
    reputation_store: R,
    enhanced_accounts: HashMap<Did, EnhancedManaAccount>,
    regeneration_policy: RegenerationPolicy,
    organization_registry: HashMap<Did, OrganizationType>,
    trust_network: TrustNetwork,
    collective_pool: CollectiveResourcePool,
}

/// Trust network for cooperation bonuses
#[derive(Debug, Clone)]
pub struct TrustNetwork {
    trust_scores: HashMap<(Did, Did), f64>,
    mutual_aid_history: HashMap<Did, Vec<MutualAidRecord>>,
    collective_actions: Vec<CollectiveAction>,
}

/// Record of mutual aid activity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutualAidRecord {
    pub provider: Did,
    pub recipient: Did,
    pub amount: u64,
    pub aid_type: String,
    pub timestamp: u64,
    pub reciprocated: bool,
}

/// Record of collective action participation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectiveAction {
    pub action_id: String,
    pub participants: Vec<Did>,
    pub action_type: String,
    pub impact_score: f64,
    pub completion_timestamp: u64,
}

/// Collective resource pool for mutual aid
#[derive(Debug, Clone)]
pub struct CollectiveResourcePool {
    pub total_pool: u64,
    pub contributions: HashMap<Did, u64>,
    pub distributions: HashMap<Did, u64>,
    emergency_reserves: u64,
}

impl Default for ContributionWeights {
    fn default() -> Self {
        Self {
            compute: 0.20,
            storage: 0.15,
            bandwidth: 0.15,
            governance: 0.15,
            mutual_aid: 0.12,
            knowledge_sharing: 0.08,
            community_building: 0.05,
            innovation: 0.05,
            care_work: 0.05,
        }
    }
}

impl Default for RegenerationPolicy {
    fn default() -> Self {
        Self {
            base_rate: 10.0, // Base mana per hour
            contribution_weights: ContributionWeights::default(),
            capacity_weights: CapacityWeights {
                compute_capacity: 0.3,
                storage_capacity: 0.25,
                bandwidth_capacity: 0.2,
                uptime: 0.15,
                reliability: 0.05,
                quality: 0.05,
            },
            organizational_multipliers: OrganizationalMultipliers {
                coop_member: 1.2,
                community_member: 1.15,
                federation_member: 1.25,
                leadership_role: 1.5,
                bridge_builder: 1.3,
            },
            cooperation_multipliers: CooperationMultipliers {
                high_trust: 1.2,
                mutual_aid_provider: 1.4,
                collective_action: 1.3,
                conflict_resolver: 1.25,
                solidarity_network: 1.1,
            },
            anti_accumulation_config: AntiAccumulationConfig {
                max_accumulation_ratio: 5.0,
                hoarding_threshold: 0.8,
                penalty_escalation_rate: 0.1,
                redistribution_threshold: 10000,
                use_it_or_lose_it_period: 30 * 24 * 3600, // 30 days
            },
            emergency_adjustments: EmergencyAdjustments {
                crisis_response_bonus: 2.0,
                mutual_aid_emergency_multiplier: 3.0,
                basic_needs_guarantee: 100,
                solidarity_crisis_factor: 1.5,
            },
        }
    }
}

impl<L: ManaLedger, R: ReputationStore> ContributionWeightedManaLedger<L, R> {
    /// Create a new contribution-weighted mana ledger
    pub fn new(base_ledger: L, reputation_store: R, policy: RegenerationPolicy) -> Self {
        Self {
            base_ledger,
            reputation_store,
            enhanced_accounts: HashMap::new(),
            regeneration_policy: policy,
            organization_registry: HashMap::new(),
            trust_network: TrustNetwork {
                trust_scores: HashMap::new(),
                mutual_aid_history: HashMap::new(),
                collective_actions: Vec::new(),
            },
            collective_pool: CollectiveResourcePool {
                total_pool: 0,
                contributions: HashMap::new(),
                distributions: HashMap::new(),
                emergency_reserves: 0,
            },
        }
    }

    /// Register an organization for a DID
    pub fn register_organization(&mut self, did: Did, org_type: OrganizationType) {
        self.organization_registry.insert(did, org_type);
    }

    /// Update contribution metrics for an account
    pub fn update_contribution_metrics(
        &mut self,
        did: &Did,
        metrics: ContributionMetrics,
    ) -> Result<(), CommonError> {
        let account = self.get_or_create_enhanced_account(did);
        account.contribution_metrics = metrics;
        Ok(())
    }

    /// Update capacity metrics for an account
    pub fn update_capacity_metrics(
        &mut self,
        did: &Did,
        metrics: CapacityMetrics,
    ) -> Result<(), CommonError> {
        let account = self.get_or_create_enhanced_account(did);
        account.capacity_metrics = metrics;
        Ok(())
    }

    /// Record mutual aid activity
    pub fn record_mutual_aid(
        &mut self,
        provider: &Did,
        recipient: &Did,
        amount: u64,
        aid_type: String,
    ) -> Result<(), CommonError> {
        let record = MutualAidRecord {
            provider: provider.clone(),
            recipient: recipient.clone(),
            amount,
            aid_type,
            timestamp: SystemTimeProvider.unix_seconds(),
            reciprocated: false,
        };

        self.trust_network
            .mutual_aid_history
            .entry(provider.clone())
            .or_default()
            .push(record.clone());

        // Update contribution metrics
        let mutual_aid_multiplier = self.calculate_mutual_aid_multiplier(provider);
        let provider_account = self.get_or_create_enhanced_account(provider);
        provider_account.contribution_metrics.mutual_aid_provided += amount as f64;

        // Update cooperation bonuses
        provider_account.cooperation_bonuses.mutual_aid_multiplier = mutual_aid_multiplier;

        Ok(())
    }

    /// Record collective action participation
    pub fn record_collective_action(
        &mut self,
        action_id: String,
        participants: Vec<Did>,
        action_type: String,
        impact_score: f64,
    ) -> Result<(), CommonError> {
        let action = CollectiveAction {
            action_id,
            participants: participants.clone(),
            action_type,
            impact_score,
            completion_timestamp: SystemTimeProvider.unix_seconds(),
        };

        self.trust_network.collective_actions.push(action);

        // Update collective action bonuses for participants
        for participant in participants {
            let bonus = self.calculate_collective_action_bonus(&participant);
            let account = self.get_or_create_enhanced_account(&participant);
            account.cooperation_bonuses.collective_action_bonus = bonus;
        }

        Ok(())
    }

    /// Perform contribution-weighted regeneration for all accounts
    pub fn regenerate_all_accounts(&mut self) -> Result<(), CommonError> {
        let current_time = SystemTimeProvider.unix_seconds();
        let all_accounts = self.base_ledger.all_accounts();

        for did in &all_accounts {
            self.regenerate_account_mana(did, current_time)?;
        }

        Ok(())
    }

    /// Regenerate mana for a specific account
    pub fn regenerate_account_mana(&mut self, did: &Did, current_time: u64) -> Result<(), CommonError> {
        // Extract necessary data first to avoid borrowing conflicts
        let base_rate = self.regeneration_policy.base_rate;
        let current_balance = self.base_ledger.get_balance(did);
        
        let (last_regeneration, max_capacity, contribution_metrics, capacity_metrics, cooperation_bonuses) = {
            let account = self.get_or_create_enhanced_account(did);
            (
                account.last_regeneration,
                account.max_capacity,
                account.contribution_metrics.clone(),
                account.capacity_metrics.clone(),
                account.cooperation_bonuses.clone(),
            )
        };
        
        let time_elapsed = current_time - last_regeneration;
        
        if time_elapsed == 0 {
            return Ok(());
        }

        let hours_elapsed = time_elapsed as f64 / 3600.0;
        
        // Calculate contribution score
        let contribution_score = self.calculate_contribution_score(&contribution_metrics);
        
        // Calculate capacity score
        let capacity_score = self.calculate_capacity_score(&capacity_metrics);
        
        // Calculate organizational bonuses
        let org_bonus = self.calculate_organizational_bonus(did);
        
        // Calculate cooperation bonuses
        let coop_bonus = self.calculate_cooperation_bonus(&cooperation_bonuses);
        
        // Apply anti-accumulation penalties
        let anti_accumulation_penalty = self.calculate_anti_accumulation_penalty(did);
        
        // Calculate final regeneration amount
        let base_regeneration = base_rate * hours_elapsed;
        let total_regeneration = (base_regeneration 
            * contribution_score 
            * capacity_score 
            * org_bonus 
            * coop_bonus 
            * (1.0 - anti_accumulation_penalty)) as u64;

        // Apply regeneration with capacity limits
        let new_balance = (current_balance + total_regeneration).min(max_capacity);
        
        // Update balance in base ledger
        if new_balance > current_balance {
            let actual_regeneration = new_balance - current_balance;
            self.base_ledger.credit(did, actual_regeneration)?;
        }

        // Update last regeneration time
        let enhanced_account = self.enhanced_accounts.get_mut(did).unwrap();
        enhanced_account.last_regeneration = current_time;

        Ok(())
    }

    /// Calculate contribution score from metrics
    fn calculate_contribution_score(&self, metrics: &ContributionMetrics) -> f64 {
        let weights = &self.regeneration_policy.contribution_weights;
        
        (metrics.compute_contribution * weights.compute +
         metrics.storage_contribution * weights.storage +
         metrics.bandwidth_contribution * weights.bandwidth +
         metrics.governance_participation * weights.governance +
         metrics.mutual_aid_provided * weights.mutual_aid +
         metrics.knowledge_sharing * weights.knowledge_sharing +
         metrics.community_building * weights.community_building +
         metrics.innovation_contribution * weights.innovation +
         metrics.care_work * weights.care_work)
        .max(0.1) // Minimum score
        .min(3.0) // Maximum score
    }

    /// Calculate capacity score from metrics
    fn calculate_capacity_score(&self, metrics: &CapacityMetrics) -> f64 {
        let weights = &self.regeneration_policy.capacity_weights;
        
        let compute_ratio = if metrics.total_compute_capacity > 0 {
            metrics.available_compute_capacity as f64 / metrics.total_compute_capacity as f64
        } else { 0.0 };
        
        let storage_ratio = if metrics.total_storage_capacity > 0 {
            metrics.available_storage_capacity as f64 / metrics.total_storage_capacity as f64
        } else { 0.0 };
        
        (compute_ratio * weights.compute_capacity +
         storage_ratio * weights.storage_capacity +
         (metrics.network_bandwidth as f64 / 1000.0) * weights.bandwidth_capacity +
         metrics.uptime_percentage * weights.uptime +
         metrics.reliability_score * weights.reliability +
         (metrics.quality_metrics.job_completion_rate * 
          (1.0 - metrics.quality_metrics.error_rate) * 
          metrics.quality_metrics.user_satisfaction_score) * weights.quality)
        .max(0.1)
        .min(3.0)
    }

    /// Calculate organizational bonus
    fn calculate_organizational_bonus(&self, did: &Did) -> f64 {
        if let Some(org_type) = self.organization_registry.get(did) {
            let multipliers = &self.regeneration_policy.organizational_multipliers;
            
            match org_type {
                OrganizationType::Coop { .. } => multipliers.coop_member,
                OrganizationType::Community { .. } => multipliers.community_member,
                OrganizationType::Federation { .. } => multipliers.federation_member,
            }
        } else {
            1.0 // No organizational bonus
        }
    }

    /// Calculate cooperation bonus
    fn calculate_cooperation_bonus(&self, bonuses: &CooperationBonuses) -> f64 {
        let multipliers = &self.regeneration_policy.cooperation_multipliers;
        
        let mut total_bonus = 1.0;
        
        if bonuses.trust_score > 0.8 {
            total_bonus *= multipliers.high_trust;
        }
        
        if bonuses.mutual_aid_multiplier > 1.0 {
            total_bonus *= multipliers.mutual_aid_provider;
        }
        
        if bonuses.collective_action_bonus > 1.0 {
            total_bonus *= multipliers.collective_action;
        }
        
        if bonuses.conflict_resolution_bonus > 1.0 {
            total_bonus *= multipliers.conflict_resolver;
        }
        
        total_bonus.min(2.0) // Cap total cooperation bonus
    }

    /// Calculate mutual aid multiplier based on history
    fn calculate_mutual_aid_multiplier(&self, did: &Did) -> f64 {
        if let Some(history) = self.trust_network.mutual_aid_history.get(did) {
            let recent_aid = history.iter()
                .filter(|record| {
                    let age = SystemTimeProvider.unix_seconds() - record.timestamp;
                    age < 30 * 24 * 3600 // Last 30 days
                })
                .map(|record| record.amount as f64)
                .sum::<f64>();
            
            1.0 + (recent_aid / 1000.0).min(0.5) // Up to 50% bonus
        } else {
            1.0
        }
    }

    /// Calculate collective action bonus
    fn calculate_collective_action_bonus(&self, did: &Did) -> f64 {
        let recent_actions = self.trust_network.collective_actions.iter()
            .filter(|action| {
                action.participants.contains(did) &&
                (SystemTimeProvider.unix_seconds() - action.completion_timestamp) < 90 * 24 * 3600 // Last 90 days
            })
            .map(|action| action.impact_score)
            .sum::<f64>();
        
        1.0 + (recent_actions / 10.0).min(0.4) // Up to 40% bonus
    }

    /// Calculate anti-accumulation penalty
    fn calculate_anti_accumulation_penalty(&self, did: &Did) -> f64 {
        let config = &self.regeneration_policy.anti_accumulation_config;
        let current_balance = self.base_ledger.get_balance(did);
        
        // Calculate average balance across all accounts
        let all_accounts = self.base_ledger.all_accounts();
        let total_balance: u64 = all_accounts.iter()
            .map(|account| self.base_ledger.get_balance(account))
            .sum();
        let average_balance = if all_accounts.is_empty() {
            0.0
        } else {
            total_balance as f64 / all_accounts.len() as f64
        };
        
        // Calculate accumulation ratio
        let accumulation_ratio = if average_balance > 0.0 {
            current_balance as f64 / average_balance
        } else {
            1.0
        };
        
        // Apply penalty if accumulation ratio exceeds threshold
        if accumulation_ratio > config.max_accumulation_ratio {
            let excess_ratio = accumulation_ratio - config.max_accumulation_ratio;
            (excess_ratio * config.penalty_escalation_rate).min(0.8) // Max 80% penalty
        } else {
            0.0
        }
    }

    /// Get or create enhanced account
    fn get_or_create_enhanced_account(&mut self, did: &Did) -> &mut EnhancedManaAccount {
        self.enhanced_accounts.entry(did.clone()).or_insert_with(|| {
            EnhancedManaAccount {
                did: did.clone(),
                current_balance: 0,
                max_capacity: 10000, // Default capacity
                base_regeneration_rate: self.regeneration_policy.base_rate,
                last_regeneration: SystemTimeProvider.unix_seconds(),
                contribution_metrics: ContributionMetrics {
                    compute_contribution: 0.0,
                    storage_contribution: 0.0,
                    bandwidth_contribution: 0.0,
                    governance_participation: 0.0,
                    mutual_aid_provided: 0.0,
                    knowledge_sharing: 0.0,
                    community_building: 0.0,
                    innovation_contribution: 0.0,
                    care_work: 0.0,
                    last_updated: SystemTimeProvider.unix_seconds(),
                },
                capacity_metrics: CapacityMetrics {
                    total_compute_capacity: 0,
                    available_compute_capacity: 0,
                    total_storage_capacity: 0,
                    available_storage_capacity: 0,
                    network_bandwidth: 0,
                    uptime_percentage: 0.0,
                    reliability_score: 0.0,
                    quality_metrics: QualityMetrics {
                        job_completion_rate: 0.0,
                        average_response_time: 0,
                        error_rate: 0.0,
                        user_satisfaction_score: 0.0,
                    },
                },
                organizational_bonuses: OrganizationalBonuses {
                    organization_type_bonus: 0.0,
                    leadership_bonus: 0.0,
                    inter_org_coordination_bonus: 0.0,
                    solidarity_bonus: 0.0,
                },
                cooperation_bonuses: CooperationBonuses {
                    trust_score: 0.0,
                    mutual_aid_multiplier: 1.0,
                    collective_action_bonus: 1.0,
                    conflict_resolution_bonus: 1.0,
                },
                anti_accumulation_state: AntiAccumulationState {
                    accumulation_penalty: 0.0,
                    last_significant_spend: SystemTimeProvider.unix_seconds(),
                    hoarding_detection_score: 0.0,
                    redistribution_contributions: 0,
                },
            }
        })
    }

    /// Contribute to collective resource pool
    pub fn contribute_to_collective_pool(&mut self, did: &Did, amount: u64) -> Result<(), CommonError> {
        // Deduct from account
        self.base_ledger.spend(did, amount)?;
        
        // Add to collective pool
        self.collective_pool.total_pool += amount;
        *self.collective_pool.contributions.entry(did.clone()).or_insert(0) += amount;
        
        // Update mutual aid contribution metrics
        let account = self.get_or_create_enhanced_account(did);
        account.contribution_metrics.mutual_aid_provided += amount as f64;
        
        Ok(())
    }

    /// Distribute from collective resource pool based on need
    pub fn distribute_from_collective_pool(
        &mut self,
        recipients: &[(Did, u64)], // (recipient, amount_needed)
    ) -> Result<(), CommonError> {
        let total_needed: u64 = recipients.iter().map(|(_, amount)| amount).sum();
        
        if total_needed > self.collective_pool.total_pool {
            return Err(CommonError::PolicyDenied(
                "Insufficient funds in collective pool".into()
            ));
        }
        
        for (recipient, amount) in recipients {
            // Credit recipient
            self.base_ledger.credit(recipient, *amount)?;
            
            // Update collective pool
            self.collective_pool.total_pool -= amount;
            *self.collective_pool.distributions.entry(recipient.clone()).or_insert(0) += amount;
        }
        
        Ok(())
    }

    /// Get enhanced account information
    pub fn get_enhanced_account(&self, did: &Did) -> Option<&EnhancedManaAccount> {
        self.enhanced_accounts.get(did)
    }

    /// Get collective pool status
    pub fn get_collective_pool_status(&self) -> &CollectiveResourcePool {
        &self.collective_pool
    }
}

// Delegate ManaLedger methods to base_ledger
impl<L: ManaLedger, R: ReputationStore> ManaLedger for ContributionWeightedManaLedger<L, R> {
    fn get_balance(&self, did: &Did) -> u64 {
        self.base_ledger.get_balance(did)
    }

    fn set_balance(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
        self.base_ledger.set_balance(did, amount)
    }

    fn spend(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
        self.base_ledger.spend(did, amount)
    }

    fn credit(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
        self.base_ledger.credit(did, amount)
    }

    fn credit_all(&self, amount: u64) -> Result<(), CommonError> {
        self.base_ledger.credit_all(amount)
    }

    fn all_accounts(&self) -> Vec<Did> {
        self.base_ledger.all_accounts()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_reputation::InMemoryReputationStore;
    use std::collections::HashMap;
    use std::str::FromStr;

    #[derive(Default)]
    struct TestLedger {
        balances: std::sync::Mutex<HashMap<Did, u64>>,
    }

    impl ManaLedger for TestLedger {
        fn get_balance(&self, did: &Did) -> u64 {
            *self.balances.lock().unwrap().get(did).unwrap_or(&0)
        }

        fn set_balance(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
            self.balances.lock().unwrap().insert(did.clone(), amount);
            Ok(())
        }

        fn spend(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
            let mut balances = self.balances.lock().unwrap();
            let balance = balances.get_mut(did).ok_or_else(|| {
                CommonError::InvalidInputError("Account not found".into())
            })?;
            if *balance < amount {
                return Err(CommonError::PolicyDenied("Insufficient balance".into()));
            }
            *balance -= amount;
            Ok(())
        }

        fn credit(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
            let mut balances = self.balances.lock().unwrap();
            let balance = balances.entry(did.clone()).or_insert(0);
            *balance += amount;
            Ok(())
        }

        fn credit_all(&self, amount: u64) -> Result<(), CommonError> {
            let mut balances = self.balances.lock().unwrap();
            for balance in balances.values_mut() {
                *balance += amount;
            }
            Ok(())
        }

        fn all_accounts(&self) -> Vec<Did> {
            self.balances.lock().unwrap().keys().cloned().collect()
        }
    }

    #[test]
    fn test_contribution_weighted_regeneration() {
        let base_ledger = TestLedger::default();
        let reputation_store = InMemoryReputationStore::new();
        let policy = RegenerationPolicy::default();
        let mut enhanced_ledger = ContributionWeightedManaLedger::new(
            base_ledger, 
            reputation_store, 
            policy
        );

        let did = Did::from_str("did:test:contributor").unwrap();
        
        // Set up initial balance
        enhanced_ledger.set_balance(&did, 100).unwrap();
        
        // Update contribution metrics for high contributor
        let high_contributions = ContributionMetrics {
            compute_contribution: 2.0,
            storage_contribution: 1.5,
            bandwidth_contribution: 1.8,
            governance_participation: 2.2,
            mutual_aid_provided: 1.0,
            knowledge_sharing: 1.5,
            community_building: 1.2,
            innovation_contribution: 0.8,
            care_work: 1.1,
            last_updated: SystemTimeProvider.unix_seconds(),
        };
        
        enhanced_ledger.update_contribution_metrics(&did, high_contributions).unwrap();
        
        // Set up capacity metrics
        let capacity_metrics = CapacityMetrics {
            total_compute_capacity: 1000,
            available_compute_capacity: 800,
            total_storage_capacity: 10000,
            available_storage_capacity: 8000,
            network_bandwidth: 1000,
            uptime_percentage: 0.95,
            reliability_score: 0.9,
            quality_metrics: QualityMetrics {
                job_completion_rate: 0.95,
                average_response_time: 100,
                error_rate: 0.02,
                user_satisfaction_score: 0.9,
            },
        };
        
        enhanced_ledger.update_capacity_metrics(&did, capacity_metrics).unwrap();
        
        // Register organization for bonus
        enhanced_ledger.register_organization(
            did.clone(),
            OrganizationType::Coop {
                economic_focus: crate::organizational_structures::EconomicFocus::Production {
                    sectors: vec!["technology".to_string()],
                },
                production_capacity: crate::organizational_structures::ProductionCapacity {
                    total_capacity: 1000,
                    current_utilization: 0.8,
                    capacity_by_resource: HashMap::new(),
                    seasonal_variations: None,
                },
            }
        );
        
        let initial_balance = enhanced_ledger.get_balance(&did);
        
        // Simulate regeneration after 1 hour
        let current_time = SystemTimeProvider.unix_seconds();
        enhanced_ledger.regenerate_account_mana(&did, current_time + 3600).unwrap();
        
        let final_balance = enhanced_ledger.get_balance(&did);
        
        // Should have regenerated more than base rate due to high contributions
        assert!(final_balance > initial_balance);
        assert!(final_balance > initial_balance + 10); // Should be more than just base rate
    }

    #[test]
    fn test_mutual_aid_tracking() {
        let base_ledger = TestLedger::default();
        let reputation_store = InMemoryReputationStore::new();
        let policy = RegenerationPolicy::default();
        let mut enhanced_ledger = ContributionWeightedManaLedger::new(
            base_ledger, 
            reputation_store, 
            policy
        );

        let provider = Did::from_str("did:test:provider").unwrap();
        let recipient = Did::from_str("did:test:recipient").unwrap();
        
        // Set up balances
        enhanced_ledger.set_balance(&provider, 1000).unwrap();
        enhanced_ledger.set_balance(&recipient, 50).unwrap();
        
        // Record mutual aid
        enhanced_ledger.record_mutual_aid(
            &provider,
            &recipient,
            100,
            "emergency_assistance".to_string()
        ).unwrap();
        
        // Check that mutual aid was recorded
        let provider_account = enhanced_ledger.get_enhanced_account(&provider).unwrap();
        assert!(provider_account.contribution_metrics.mutual_aid_provided > 0.0);
        assert!(provider_account.cooperation_bonuses.mutual_aid_multiplier >= 1.0);
    }

    #[test]
    fn test_collective_resource_pool() {
        let base_ledger = TestLedger::default();
        let reputation_store = InMemoryReputationStore::new();
        let policy = RegenerationPolicy::default();
        let mut enhanced_ledger = ContributionWeightedManaLedger::new(
            base_ledger, 
            reputation_store, 
            policy
        );

        let contributor = Did::from_str("did:test:contributor").unwrap();
        let recipient1 = Did::from_str("did:test:recipient1").unwrap();
        let recipient2 = Did::from_str("did:test:recipient2").unwrap();
        
        // Set up balances
        enhanced_ledger.set_balance(&contributor, 1000).unwrap();
        enhanced_ledger.set_balance(&recipient1, 10).unwrap();
        enhanced_ledger.set_balance(&recipient2, 20).unwrap();
        
        // Contribute to collective pool
        enhanced_ledger.contribute_to_collective_pool(&contributor, 500).unwrap();
        
        // Check pool status
        let pool_status = enhanced_ledger.get_collective_pool_status();
        assert_eq!(pool_status.total_pool, 500);
        
        // Distribute from pool
        let recipients = vec![
            (recipient1.clone(), 200),
            (recipient2.clone(), 150),
        ];
        enhanced_ledger.distribute_from_collective_pool(&recipients).unwrap();
        
        // Check balances after distribution
        assert_eq!(enhanced_ledger.get_balance(&recipient1), 210);
        assert_eq!(enhanced_ledger.get_balance(&recipient2), 170);
        
        // Check pool status after distribution
        let pool_status = enhanced_ledger.get_collective_pool_status();
        assert_eq!(pool_status.total_pool, 150); // 500 - 200 - 150
    }

    #[test]
    fn test_anti_accumulation_penalty() {
        let base_ledger = TestLedger::default();
        let reputation_store = InMemoryReputationStore::new();
        let policy = RegenerationPolicy::default();
        let mut enhanced_ledger = ContributionWeightedManaLedger::new(
            base_ledger, 
            reputation_store, 
            policy
        );

        // Create accounts with different balance levels
        let high_balance_did = Did::from_str("did:test:high_balance").unwrap();
        let normal_balance_did = Did::from_str("did:test:normal_balance").unwrap();
        
        enhanced_ledger.set_balance(&high_balance_did, 50000).unwrap(); // Very high balance
        enhanced_ledger.set_balance(&normal_balance_did, 100).unwrap(); // Normal balance
        
        // Calculate penalty for high balance account
        let penalty = enhanced_ledger.calculate_anti_accumulation_penalty(&high_balance_did);
        
        // With balances of 50,000 and 100, average is 25,050
        // High balance ratio = 50,000 / 25,050 ≈ 2.0, which is less than default max of 5.0
        // Let's create a more extreme case by adding more low-balance accounts
        let low1 = Did::from_str("did:test:low1").unwrap();
        let low2 = Did::from_str("did:test:low2").unwrap();
        let low3 = Did::from_str("did:test:low3").unwrap();
        
        enhanced_ledger.set_balance(&low1, 50).unwrap();
        enhanced_ledger.set_balance(&low2, 50).unwrap();
        enhanced_ledger.set_balance(&low3, 50).unwrap();
        
        // Now calculate penalty with more skewed distribution
        let penalty = enhanced_ledger.calculate_anti_accumulation_penalty(&high_balance_did);
        
        // Debug: let's check the actual values
        // Average of [50000, 100, 50, 50, 50] = 50250 / 5 = 10050
        // Ratio = 50000 / 10050 = ~4.97, which is still less than 5.0
        // Let's use even more extreme values
        enhanced_ledger.set_balance(&high_balance_did, 100000).unwrap(); // Very high balance
        
        let penalty = enhanced_ledger.calculate_anti_accumulation_penalty(&high_balance_did);
        // Average of [100000, 100, 50, 50, 50] = 100250 / 5 = 20050
        // Ratio = 100000 / 20050 = ~4.99, still less than 5.0!
        
        // Let's try with even lower other balances
        enhanced_ledger.set_balance(&normal_balance_did, 10).unwrap();
        enhanced_ledger.set_balance(&low1, 10).unwrap();
        enhanced_ledger.set_balance(&low2, 10).unwrap();
        enhanced_ledger.set_balance(&low3, 10).unwrap();
        
        let penalty = enhanced_ledger.calculate_anti_accumulation_penalty(&high_balance_did);
        // Average of [100000, 10, 10, 10, 10] = 100040 / 5 = 20008
        // Ratio = 100000 / 20008 = ~4.998, still less than 5.0
        
        // Let's use 1 instead
        enhanced_ledger.set_balance(&normal_balance_did, 1).unwrap();
        enhanced_ledger.set_balance(&low1, 1).unwrap();
        enhanced_ledger.set_balance(&low2, 1).unwrap();
        enhanced_ledger.set_balance(&low3, 1).unwrap();
        
        let penalty = enhanced_ledger.calculate_anti_accumulation_penalty(&high_balance_did);
        // Average of [100000, 1, 1, 1, 1] = 100004 / 5 = 20000.8
        // Ratio = 100000 / 20000.8 = ~4.9998, still less than 5.0!
        
        // The ratio is always close to 5 because the high balance dominates the average
        // Let's use an extreme case with 20 low accounts
        for i in 0..20 {
            let did = Did::from_str(&format!("did:test:low{}", i)).unwrap();
            enhanced_ledger.set_balance(&did, 1).unwrap();
        }
        
        let penalty = enhanced_ledger.calculate_anti_accumulation_penalty(&high_balance_did);
        // Now we have 100000 and 24 accounts with 1 each
        // Average = (100000 + 24) / 25 = 4001.6
        // Ratio = 100000 / 4001.6 = ~24.99, which is definitely > 5.0
        
        // Should have some penalty for excessive accumulation
        assert!(penalty > 0.0);
        
        // Normal balance account should have no penalty
        let normal_penalty = enhanced_ledger.calculate_anti_accumulation_penalty(&normal_balance_did);
        assert_eq!(normal_penalty, 0.0);
    }
}