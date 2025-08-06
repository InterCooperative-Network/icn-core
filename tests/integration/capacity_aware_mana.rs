//! Integration tests for capacity-aware mana regeneration
//! 
//! Tests the enhanced mana system documented in economics-models.md that uses
//! capacity factors to determine regeneration rates based on actual node contribution.

use icn_runtime::context::RuntimeContext;
use icn_common::Did;
use std::collections::HashMap;

/// Capacity metrics used to calculate capacity factor
#[derive(Debug, Clone)]
pub struct CapacityMetrics {
    pub compute_score: f64,      // 0.0 - 1.0 based on CPU/memory availability
    pub storage_score: f64,      // 0.0 - 1.0 based on storage contribution
    pub bandwidth_score: f64,    // 0.0 - 1.0 based on network bandwidth
    pub uptime_score: f64,       // 0.0 - 1.0 based on node uptime history
    pub quality_score: f64,      // 0.0 - 1.0 based on service quality metrics
}

impl Default for CapacityMetrics {
    fn default() -> Self {
        Self {
            compute_score: 0.5,
            storage_score: 0.5,
            bandwidth_score: 0.5,
            uptime_score: 0.5,
            quality_score: 0.5,
        }
    }
}

impl CapacityMetrics {
    /// Calculate capacity factor using weighted average
    /// Formula from docs: capacity_factor = (compute_weight × compute_score + 
    ///                                      storage_weight × storage_score + 
    ///                                      bandwidth_weight × bandwidth_score + 
    ///                                      uptime_weight × uptime_score + 
    ///                                      quality_weight × quality_score) / total_weights
    pub fn calculate_capacity_factor(&self) -> f64 {
        let compute_weight = 0.25;
        let storage_weight = 0.20;
        let bandwidth_weight = 0.20;
        let uptime_weight = 0.20;
        let quality_weight = 0.15;
        
        let total_weights = compute_weight + storage_weight + bandwidth_weight + uptime_weight + quality_weight;
        
        (compute_weight * self.compute_score +
         storage_weight * self.storage_score +
         bandwidth_weight * self.bandwidth_score +
         uptime_weight * self.uptime_score +
         quality_weight * self.quality_score) / total_weights
    }
}

/// Enhanced mana regenerator that implements capacity-aware regeneration
pub struct CapacityAwareManaRegenerator {
    capacity_metrics: HashMap<Did, CapacityMetrics>,
}

impl CapacityAwareManaRegenerator {
    pub fn new() -> Self {
        Self {
            capacity_metrics: HashMap::new(),
        }
    }
    
    pub fn set_capacity_metrics(&mut self, did: &Did, metrics: CapacityMetrics) {
        self.capacity_metrics.insert(did.clone(), metrics);
    }
    
    /// Implement the capacity-aware regeneration formula from documentation:
    /// regeneration = base_rate × capacity_factor × reputation_factor × time_elapsed
    pub fn calculate_regeneration(
        &self, 
        did: &Did, 
        base_rate: u64, 
        reputation_factor: f64, 
        time_elapsed_hours: f64
    ) -> u64 {
        let capacity_factor = self.capacity_metrics
            .get(did)
            .map(|m| m.calculate_capacity_factor())
            .unwrap_or(0.5); // Default capacity factor
            
        let regeneration = (base_rate as f64) * capacity_factor * reputation_factor * time_elapsed_hours;
        regeneration.round() as u64
    }
}

#[tokio::test]
async fn test_capacity_aware_regeneration_basic() {
    let ctx = RuntimeContext::new_with_stubs_and_mana("did:icn:test:capacity", 0).unwrap();
    let mut regenerator = CapacityAwareManaRegenerator::new();
    
    // Set up a high-capacity node
    let high_capacity_metrics = CapacityMetrics {
        compute_score: 0.9,
        storage_score: 0.8,
        bandwidth_score: 0.9,
        uptime_score: 0.95,
        quality_score: 0.85,
    };
    
    regenerator.set_capacity_metrics(&ctx.current_identity, high_capacity_metrics);
    
    // Record some reputation
    ctx.reputation_store.record_execution(&ctx.current_identity, true, 0);
    ctx.reputation_store.record_execution(&ctx.current_identity, true, 0);
    ctx.reputation_store.record_execution(&ctx.current_identity, true, 0);
    
    let reputation = ctx.reputation_store.get_reputation(&ctx.current_identity);
    let reputation_factor = (reputation as f64 / 100.0).clamp(0.1, 2.0);
    
    // Calculate regeneration for 1 hour
    let base_rate = 10;
    let regeneration = regenerator.calculate_regeneration(
        &ctx.current_identity,
        base_rate,
        reputation_factor,
        1.0
    );
    
    // High capacity node should get more mana than base rate
    assert!(regeneration > base_rate);
    assert!(regeneration > 15); // Should be significantly higher due to high capacity
    
    // Apply the regeneration
    let initial_balance = ctx.mana_ledger.get_balance(&ctx.current_identity);
    ctx.mana_ledger.credit(&ctx.current_identity, regeneration).unwrap();
    let final_balance = ctx.mana_ledger.get_balance(&ctx.current_identity);
    
    assert_eq!(final_balance, initial_balance + regeneration);
}

#[tokio::test]
async fn test_capacity_aware_regeneration_comparison() {
    let ctx = RuntimeContext::new_with_stubs_and_mana("did:icn:test:comparison", 0).unwrap();
    let mut regenerator = CapacityAwareManaRegenerator::new();
    
    // Create two test identities
    let high_capacity_did = Did::from("did:icn:test:high_capacity");
    let low_capacity_did = Did::from("did:icn:test:low_capacity");
    
    // High capacity node metrics
    let high_capacity_metrics = CapacityMetrics {
        compute_score: 0.9,
        storage_score: 0.85,
        bandwidth_score: 0.9,
        uptime_score: 0.95,
        quality_score: 0.88,
    };
    
    // Low capacity node metrics
    let low_capacity_metrics = CapacityMetrics {
        compute_score: 0.3,
        storage_score: 0.2,
        bandwidth_score: 0.4,
        uptime_score: 0.6,
        quality_score: 0.4,
    };
    
    regenerator.set_capacity_metrics(&high_capacity_did, high_capacity_metrics);
    regenerator.set_capacity_metrics(&low_capacity_did, low_capacity_metrics);
    
    // Same reputation for both to isolate capacity factor effect
    let reputation_factor = 1.0;
    let base_rate = 10;
    let time_elapsed = 1.0;
    
    let high_regen = regenerator.calculate_regeneration(
        &high_capacity_did,
        base_rate,
        reputation_factor,
        time_elapsed
    );
    
    let low_regen = regenerator.calculate_regeneration(
        &low_capacity_did,
        base_rate,
        reputation_factor,
        time_elapsed
    );
    
    // High capacity node should regenerate significantly more mana
    assert!(high_regen > low_regen);
    
    // The ratio should reflect the capacity difference
    let high_factor = regenerator.capacity_metrics[&high_capacity_did].calculate_capacity_factor();
    let low_factor = regenerator.capacity_metrics[&low_capacity_did].calculate_capacity_factor();
    
    assert!(high_factor > low_factor);
    assert!(high_factor > 0.8); // High capacity node should have high factor
    assert!(low_factor < 0.5);  // Low capacity node should have low factor
}

#[tokio::test]
async fn test_capacity_spending_limits() {
    let ctx = RuntimeContext::new_with_stubs_and_mana("did:icn:test:spending", 100).unwrap();
    let mut regenerator = CapacityAwareManaRegenerator::new();
    
    // Set up capacity metrics
    let capacity_metrics = CapacityMetrics {
        compute_score: 0.7,
        storage_score: 0.6,
        bandwidth_score: 0.8,
        uptime_score: 0.9,
        quality_score: 0.75,
    };
    
    regenerator.set_capacity_metrics(&ctx.current_identity, capacity_metrics);
    
    let capacity_factor = regenerator.capacity_metrics[&ctx.current_identity].calculate_capacity_factor();
    
    // Calculate capacity-based spending limit
    // From docs: spending_limit = base_limit × capacity_factor × reputation_multiplier
    let base_limit = 50u64;
    let reputation = ctx.reputation_store.get_reputation(&ctx.current_identity);
    let reputation_multiplier = if reputation >= 80 { 1.5 } 
                               else if reputation >= 50 { 1.2 }
                               else { 1.0 };
    
    let spending_limit = (base_limit as f64 * capacity_factor * reputation_multiplier) as u64;
    
    // High capacity nodes should have higher spending limits
    assert!(spending_limit >= base_limit);
    
    // Should be able to spend up to the limit
    let spend_amount = spending_limit.min(ctx.mana_ledger.get_balance(&ctx.current_identity));
    assert!(ctx.mana_ledger.spend(&ctx.current_identity, spend_amount).is_ok());
}

#[tokio::test]
async fn test_capacity_factor_calculation() {
    let metrics = CapacityMetrics {
        compute_score: 1.0,
        storage_score: 1.0,
        bandwidth_score: 1.0,
        uptime_score: 1.0,
        quality_score: 1.0,
    };
    
    let perfect_factor = metrics.calculate_capacity_factor();
    assert!((perfect_factor - 1.0).abs() < 0.001); // Should be very close to 1.0
    
    let zero_metrics = CapacityMetrics {
        compute_score: 0.0,
        storage_score: 0.0,
        bandwidth_score: 0.0,
        uptime_score: 0.0,
        quality_score: 0.0,
    };
    
    let zero_factor = zero_metrics.calculate_capacity_factor();
    assert!((zero_factor - 0.0).abs() < 0.001); // Should be very close to 0.0
    
    let default_metrics = CapacityMetrics::default();
    let default_factor = default_metrics.calculate_capacity_factor();
    assert!((default_factor - 0.5).abs() < 0.001); // Should be very close to 0.5
}

#[tokio::test]
async fn test_time_based_regeneration() {
    let ctx = RuntimeContext::new_with_stubs_and_mana("did:icn:test:time", 0).unwrap();
    let mut regenerator = CapacityAwareManaRegenerator::new();
    
    let metrics = CapacityMetrics::default();
    regenerator.set_capacity_metrics(&ctx.current_identity, metrics);
    
    let base_rate = 10;
    let reputation_factor = 1.0;
    
    // Test different time periods
    let one_hour = regenerator.calculate_regeneration(&ctx.current_identity, base_rate, reputation_factor, 1.0);
    let two_hours = regenerator.calculate_regeneration(&ctx.current_identity, base_rate, reputation_factor, 2.0);
    let half_hour = regenerator.calculate_regeneration(&ctx.current_identity, base_rate, reputation_factor, 0.5);
    
    // Regeneration should scale linearly with time
    assert_eq!(two_hours, one_hour * 2);
    assert_eq!(half_hour * 2, one_hour);
}