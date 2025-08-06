// Example: Capacity-Aware Mana Regeneration Integration
// Demonstrates how to integrate capacity metrics with mana regeneration
// and how CCL policies can customize economic behavior

use icn_common::Did;
use icn_runtime::context::RuntimeContext;
use std::collections::HashMap;
use std::str::FromStr;

/// Example implementation of capacity-aware mana regeneration
/// that demonstrates the concepts documented in docs/economics-models.md
pub struct CapacityAwareManaExample {
    /// Node capacity metrics for demonstration
    capacity_database: HashMap<Did, NodeCapacityMetrics>,
    /// Base regeneration rate (mana per hour)
    base_rate: u64,
    /// CCL policy overrides for different communities
    ccl_policies: HashMap<String, ManaPolicy>,
}

/// Node capacity metrics as documented
#[derive(Debug, Clone)]
pub struct NodeCapacityMetrics {
    /// CPU and memory utilization scores (0.0 - 1.0)
    pub compute_score: f64,
    /// Storage contribution to network (0.0 - 1.0)  
    pub storage_score: f64,
    /// Network bandwidth availability (0.0 - 1.0)
    pub bandwidth_score: f64,
    /// Historical uptime percentage (0.0 - 1.0)
    pub uptime_score: f64,
    /// Service quality metrics (0.0 - 1.0)
    pub quality_score: f64,
    /// Timestamp of last metrics update
    pub last_updated: u64,
}

/// CCL-customizable mana policy
#[derive(Debug, Clone, Default)]
pub struct ManaPolicy {
    /// Custom capacity weights for different communities
    pub capacity_weights: CapacityWeights,
    /// Reputation thresholds and multipliers
    pub reputation_thresholds: ReputationThresholds,
    /// Maximum spending limits based on capacity
    pub spending_limits: SpendingLimits,
    /// Community-specific regeneration bonuses
    pub community_bonuses: CommunityBonuses,
}

#[derive(Debug, Clone)]
pub struct CapacityWeights {
    pub compute: f64,
    pub storage: f64,
    pub bandwidth: f64,
    pub uptime: f64,
    pub quality: f64,
}

#[derive(Debug, Clone)]
pub struct ReputationThresholds {
    pub high_threshold: u32,
    pub medium_threshold: u32,
    pub high_multiplier: f64,
    pub medium_multiplier: f64,
    pub low_multiplier: f64,
}

#[derive(Debug, Clone)]
pub struct SpendingLimits {
    pub base_limit: u64,
    pub capacity_multiplier: f64,
    pub reputation_multiplier: f64,
    pub max_multiplier: f64,
}

#[derive(Debug, Clone)]
pub struct CommunityBonuses {
    pub mutual_aid_bonus: f64,
    pub governance_participation_bonus: f64,
    pub infrastructure_contribution_bonus: f64,
    pub education_sharing_bonus: f64,
}

impl Default for CapacityWeights {
    fn default() -> Self {
        Self {
            compute: 0.25,
            storage: 0.20,
            bandwidth: 0.20,
            uptime: 0.20,
            quality: 0.15,
        }
    }
}

impl Default for ReputationThresholds {
    fn default() -> Self {
        Self {
            high_threshold: 80,
            medium_threshold: 50,
            high_multiplier: 1.5,
            medium_multiplier: 1.2,
            low_multiplier: 1.0,
        }
    }
}

impl Default for SpendingLimits {
    fn default() -> Self {
        Self {
            base_limit: 50,
            capacity_multiplier: 1.0,
            reputation_multiplier: 1.0,
            max_multiplier: 3.0,
        }
    }
}

impl Default for CommunityBonuses {
    fn default() -> Self {
        Self {
            mutual_aid_bonus: 1.1,
            governance_participation_bonus: 1.05,
            infrastructure_contribution_bonus: 1.2,
            education_sharing_bonus: 1.1,
        }
    }
}

impl NodeCapacityMetrics {
    /// Calculate overall capacity factor using weighted formula from documentation
    pub fn calculate_capacity_factor(&self, weights: &CapacityWeights) -> f64 {
        let total_weight = weights.compute
            + weights.storage
            + weights.bandwidth
            + weights.uptime
            + weights.quality;

        if total_weight == 0.0 {
            return 0.5; // Default fallback
        }

        (weights.compute * self.compute_score
            + weights.storage * self.storage_score
            + weights.bandwidth * self.bandwidth_score
            + weights.uptime * self.uptime_score
            + weights.quality * self.quality_score)
            / total_weight
    }

    /// Check if metrics are recent enough to be valid
    pub fn is_valid(&self, current_time: u64, max_age_seconds: u64) -> bool {
        current_time - self.last_updated <= max_age_seconds
    }
}

impl CapacityAwareManaExample {
    pub fn new(base_rate: u64) -> Self {
        Self {
            capacity_database: HashMap::new(),
            base_rate,
            ccl_policies: HashMap::new(),
        }
    }

    /// Register capacity metrics for a node (simulates monitoring system)
    pub fn register_node_capacity(&mut self, did: &Did, metrics: NodeCapacityMetrics) {
        self.capacity_database.insert(did.clone(), metrics);
    }

    /// Register a CCL policy for a community
    pub fn register_ccl_policy(&mut self, community_id: String, policy: ManaPolicy) {
        self.ccl_policies.insert(community_id, policy);
    }

    /// Get reputation multiplier based on score and thresholds
    fn get_reputation_multiplier(&self, reputation: u32, thresholds: &ReputationThresholds) -> f64 {
        if reputation >= thresholds.high_threshold {
            thresholds.high_multiplier
        } else if reputation >= thresholds.medium_threshold {
            thresholds.medium_multiplier
        } else {
            thresholds.low_multiplier
        }
    }

    /// Apply community bonuses based on recent activities
    fn apply_community_bonuses(
        &self,
        base_amount: f64,
        did: &Did,
        bonuses: &CommunityBonuses,
    ) -> f64 {
        let mut multiplier = 1.0;

        // These would check actual activity records in a real implementation
        if self.participated_in_mutual_aid(did) {
            multiplier *= bonuses.mutual_aid_bonus;
        }

        if self.participated_in_governance(did) {
            multiplier *= bonuses.governance_participation_bonus;
        }

        if self.contributed_to_infrastructure(did) {
            multiplier *= bonuses.infrastructure_contribution_bonus;
        }

        if self.shared_educational_resources(did) {
            multiplier *= bonuses.education_sharing_bonus;
        }

        base_amount * multiplier
    }

    /// Placeholder methods for activity checking (would integrate with actual tracking)
    fn participated_in_mutual_aid(&self, _did: &Did) -> bool {
        false
    }
    fn participated_in_governance(&self, _did: &Did) -> bool {
        false
    }
    fn contributed_to_infrastructure(&self, _did: &Did) -> bool {
        false
    }
    fn shared_educational_resources(&self, _did: &Did) -> bool {
        false
    }

    /// Calculate mana regeneration using the documented formula:
    /// regeneration = base_rate × capacity_factor × reputation_factor × time_elapsed
    pub fn calculate_regeneration(
        &self,
        did: &Did,
        community_id: Option<&str>,
        reputation: u32,
        time_elapsed_hours: f64,
        current_time: u64,
    ) -> u64 {
        // Get policy (community-specific or default)
        let default_policy = ManaPolicy::default();
        let policy = community_id
            .and_then(|id| self.ccl_policies.get(id))
            .unwrap_or(&default_policy);

        // Get capacity factor
        let capacity_factor = self
            .capacity_database
            .get(did)
            .filter(|metrics| metrics.is_valid(current_time, 7200)) // 2 hours max age
            .map(|metrics| metrics.calculate_capacity_factor(&policy.capacity_weights))
            .unwrap_or(0.5); // Default for new or unavailable nodes

        // Get reputation multiplier
        let reputation_factor =
            self.get_reputation_multiplier(reputation, &policy.reputation_thresholds);

        // Apply the documented regeneration formula
        let base_regeneration =
            (self.base_rate as f64) * capacity_factor * reputation_factor * time_elapsed_hours;

        // Apply community bonuses if any
        let final_regeneration =
            self.apply_community_bonuses(base_regeneration, did, &policy.community_bonuses);

        final_regeneration.round() as u64
    }

    /// Calculate capacity-based spending limits
    pub fn calculate_spending_limit(
        &self,
        did: &Did,
        community_id: Option<&str>,
        reputation: u32,
        current_time: u64,
    ) -> u64 {
        let default_policy = ManaPolicy::default();
        let policy = community_id
            .and_then(|id| self.ccl_policies.get(id))
            .unwrap_or(&default_policy);

        let capacity_factor = self
            .capacity_database
            .get(did)
            .filter(|metrics| metrics.is_valid(current_time, 7200))
            .map(|metrics| metrics.calculate_capacity_factor(&policy.capacity_weights))
            .unwrap_or(0.5);

        let reputation_multiplier =
            self.get_reputation_multiplier(reputation, &policy.reputation_thresholds);

        let capacity_adjusted_limit = (policy.spending_limits.base_limit as f64)
            * capacity_factor
            * policy.spending_limits.capacity_multiplier;

        let reputation_adjusted_limit = capacity_adjusted_limit
            * reputation_multiplier
            * policy.spending_limits.reputation_multiplier;

        let max_limit =
            (policy.spending_limits.base_limit as f64) * policy.spending_limits.max_multiplier;

        reputation_adjusted_limit.min(max_limit).round() as u64
    }

    /// Demonstrate the complete capacity-aware regeneration cycle
    pub fn demonstrate_regeneration_cycle(
        &mut self,
        ctx: &RuntimeContext,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Use a deterministic timestamp for the example
        let current_time = 1234567890;

        println!("=== Capacity-Aware Mana Regeneration Demo ===\n");

        // Create sample node capacity metrics
        let high_capacity_metrics = NodeCapacityMetrics {
            compute_score: 0.9,
            storage_score: 0.85,
            bandwidth_score: 0.9,
            uptime_score: 0.95,
            quality_score: 0.88,
            last_updated: current_time,
        };

        let low_capacity_metrics = NodeCapacityMetrics {
            compute_score: 0.3,
            storage_score: 0.25,
            bandwidth_score: 0.4,
            uptime_score: 0.65,
            quality_score: 0.45,
            last_updated: current_time,
        };

        // Register metrics
        self.register_node_capacity(&ctx.current_identity, high_capacity_metrics.clone());

        let low_capacity_did = Did::from_str("did:icn:example:low_capacity").unwrap();
        self.register_node_capacity(&low_capacity_did, low_capacity_metrics.clone());

        // Create a custom CCL policy for a cooperative community
        let cooperative_policy = ManaPolicy {
            capacity_weights: CapacityWeights {
                compute: 0.20, // Reduce compute emphasis
                storage: 0.30, // Increase storage emphasis
                bandwidth: 0.20,
                uptime: 0.20,
                quality: 0.10,
            },
            community_bonuses: CommunityBonuses {
                mutual_aid_bonus: 1.3,                  // Higher mutual aid bonus
                governance_participation_bonus: 1.2,    // Higher governance bonus
                infrastructure_contribution_bonus: 1.4, // Much higher infrastructure bonus
                education_sharing_bonus: 1.25,          // Higher education bonus
            },
            ..ManaPolicy::default()
        };

        self.register_ccl_policy("food_cooperative".to_string(), cooperative_policy.clone());

        // Get reputation scores
        let high_reputation = ctx.reputation_store.get_reputation(&ctx.current_identity) as u32;
        let low_reputation = 25u32; // Simulate low reputation

        println!("Node Capacity Analysis:");
        println!("High Capacity Node:");
        println!("  - Compute: {:.2}", high_capacity_metrics.compute_score);
        println!("  - Storage: {:.2}", high_capacity_metrics.storage_score);
        println!(
            "  - Bandwidth: {:.2}",
            high_capacity_metrics.bandwidth_score
        );
        println!("  - Uptime: {:.2}", high_capacity_metrics.uptime_score);
        println!("  - Quality: {:.2}", high_capacity_metrics.quality_score);
        println!(
            "  - Capacity Factor (default): {:.3}",
            high_capacity_metrics.calculate_capacity_factor(&CapacityWeights::default())
        );
        println!(
            "  - Capacity Factor (cooperative): {:.3}",
            high_capacity_metrics.calculate_capacity_factor(&cooperative_policy.capacity_weights)
        );

        println!("\nLow Capacity Node:");
        println!("  - Compute: {:.2}", low_capacity_metrics.compute_score);
        println!("  - Storage: {:.2}", low_capacity_metrics.storage_score);
        println!("  - Bandwidth: {:.2}", low_capacity_metrics.bandwidth_score);
        println!("  - Uptime: {:.2}", low_capacity_metrics.uptime_score);
        println!("  - Quality: {:.2}", low_capacity_metrics.quality_score);
        println!(
            "  - Capacity Factor: {:.3}",
            low_capacity_metrics.calculate_capacity_factor(&CapacityWeights::default())
        );

        // Calculate regeneration for different scenarios
        println!("\n=== Mana Regeneration Calculations (1 hour) ===");

        // High capacity node with default policy
        let high_default_regen = self.calculate_regeneration(
            &ctx.current_identity,
            None,
            high_reputation,
            1.0,
            current_time,
        );

        // High capacity node with cooperative policy
        let high_coop_regen = self.calculate_regeneration(
            &ctx.current_identity,
            Some("food_cooperative"),
            high_reputation,
            1.0,
            current_time,
        );

        // Low capacity node
        let low_regen =
            self.calculate_regeneration(&low_capacity_did, None, low_reputation, 1.0, current_time);

        println!(
            "High Capacity Node (Default Policy): {} mana",
            high_default_regen
        );
        println!(
            "High Capacity Node (Cooperative Policy): {} mana",
            high_coop_regen
        );
        println!("Low Capacity Node: {} mana", low_regen);

        // Calculate spending limits
        println!("\n=== Spending Limits ===");

        let high_spending_limit = self.calculate_spending_limit(
            &ctx.current_identity,
            None,
            high_reputation,
            current_time,
        );

        let high_coop_spending_limit = self.calculate_spending_limit(
            &ctx.current_identity,
            Some("food_cooperative"),
            high_reputation,
            current_time,
        );

        let low_spending_limit =
            self.calculate_spending_limit(&low_capacity_did, None, low_reputation, current_time);

        println!(
            "High Capacity Node (Default): {} mana/period",
            high_spending_limit
        );
        println!(
            "High Capacity Node (Cooperative): {} mana/period",
            high_coop_spending_limit
        );
        println!("Low Capacity Node: {} mana/period", low_spending_limit);

        // Apply regeneration to actual mana balances
        println!("\n=== Applying Regeneration ===");

        let initial_balance = ctx.mana_ledger.get_balance(&ctx.current_identity);
        ctx.mana_ledger
            .credit(&ctx.current_identity, high_default_regen)?;
        let final_balance = ctx.mana_ledger.get_balance(&ctx.current_identity);

        println!("Initial Balance: {} mana", initial_balance);
        println!("Regeneration Applied: {} mana", high_default_regen);
        println!("Final Balance: {} mana", final_balance);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_capacity_aware_example() {
        let ctx =
            RuntimeContext::new_with_stubs_and_mana("did:icn:test:capacity_example", 0).unwrap();
        let mut example = CapacityAwareManaExample::new(10);

        // Add reputation for testing
        ctx.reputation_store
            .record_execution(&ctx.current_identity, true, 0);
        ctx.reputation_store
            .record_execution(&ctx.current_identity, true, 0);
        ctx.reputation_store
            .record_execution(&ctx.current_identity, true, 0);

        example.demonstrate_regeneration_cycle(&ctx).unwrap();

        // Verify mana was actually credited
        assert!(ctx.mana_ledger.get_balance(&ctx.current_identity) > 0);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("ICN Capacity-Aware Mana Regeneration Example");
    println!("==============================================\n");

    let ctx = RuntimeContext::new_with_stubs_and_mana("did:icn:example:main", 0)?;
    let mut example = CapacityAwareManaExample::new(10);

    // Add some reputation for demonstration
    ctx.reputation_store
        .record_execution(&ctx.current_identity, true, 0);
    ctx.reputation_store
        .record_execution(&ctx.current_identity, true, 0);
    ctx.reputation_store
        .record_execution(&ctx.current_identity, true, 0);

    // Run the demonstration
    example.demonstrate_regeneration_cycle(&ctx)?;

    println!("\n=== Example Complete ===");
    println!("This demonstrates the capacity-aware mana regeneration system documented in:");
    println!("- docs/economics-models.md");
    println!("- docs/crates/icn-economics.md");
    println!("- docs/rfc/rfc-003-tokenomics-design.md");

    Ok(())
}
