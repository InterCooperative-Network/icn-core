//! Computational Mana System - Connecting Mana to Real Computational Resources
//!
//! This module implements the core ICN vision where mana regeneration is directly tied
//! to the computational resources contributed by nodes in the federation.

use icn_common::{CommonError, Did, SystemInfoProvider, TimeProvider};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Computational resource capacity that affects mana generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputationalCapacity {
    /// CPU cores available for ICN work
    pub cpu_cores: u32,
    /// Memory in MB available for ICN work
    pub memory_mb: u64,
    /// Storage in MB available for ICN work
    pub storage_mb: u64,
    /// Network bandwidth in Mbps available for ICN work
    pub network_mbps: u32,
    /// GPU compute units available (if any)
    pub gpu_compute_units: Option<u32>,
}

impl ComputationalCapacity {
    /// Calculate a computational power score from the capacity
    pub fn compute_power_score(&self) -> u64 {
        // Base score from CPU and memory (primary factors)
        let cpu_score = self.cpu_cores as u64 * 100;
        let memory_score = self.memory_mb / 1024; // Convert to GB for scoring
        let storage_score = self.storage_mb / 10240; // Convert to GB/10 for scoring
        let network_score = self.network_mbps as u64 * 2;

        // GPU provides significant bonus if available
        let gpu_bonus = self.gpu_compute_units.unwrap_or(0) as u64 * 200;

        cpu_score + memory_score + storage_score + network_score + gpu_bonus
    }
}

/// Resource contribution metrics for a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContribution {
    /// Node's DID
    pub node_did: Did,
    /// Current computational capacity
    pub capacity: ComputationalCapacity,
    /// Historical uptime percentage (0.0 to 1.0)
    pub uptime_factor: f64,
    /// Jobs successfully completed
    pub jobs_completed: u64,
    /// Jobs failed due to resource issues
    pub jobs_failed: u64,
    /// Total compute hours contributed
    pub compute_hours_contributed: f64,
    /// Last time metrics were updated
    pub last_updated: u64,
}

impl ResourceContribution {
    /// Calculate the contribution score affecting mana generation
    pub fn contribution_score(&self) -> f64 {
        let base_power = self.capacity.compute_power_score() as f64;
        let reliability_factor = self.uptime_factor * self.success_rate();
        let contribution_factor = (self.compute_hours_contributed / 1000.0).min(2.0); // Cap at 2x

        base_power * reliability_factor * (1.0 + contribution_factor)
    }

    /// Calculate success rate from job completion statistics
    fn success_rate(&self) -> f64 {
        if self.jobs_completed + self.jobs_failed == 0 {
            1.0 // New nodes start with benefit of doubt
        } else {
            self.jobs_completed as f64 / (self.jobs_completed + self.jobs_failed) as f64
        }
    }
}

/// Mana regeneration configuration based on computational resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputationalManaConfig {
    /// Base mana regeneration per hour for minimal contribution
    pub base_regeneration_per_hour: u64,
    /// Maximum mana capacity multiplier based on contribution
    pub max_capacity_multiplier: f64,
    /// How often to recalculate mana regeneration rates (seconds)
    pub recalculation_interval: u64,
    /// Minimum contribution score for any mana regeneration
    pub minimum_contribution_threshold: f64,
    /// Federation-wide resource pool affects individual rates
    pub federation_pool_factor: f64,
}

impl Default for ComputationalManaConfig {
    fn default() -> Self {
        Self {
            base_regeneration_per_hour: 10,
            max_capacity_multiplier: 5.0,
            recalculation_interval: 3600, // 1 hour
            minimum_contribution_threshold: 100.0,
            federation_pool_factor: 1.0,
        }
    }
}

/// Federation-wide computational resource pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationResourcePool {
    /// Total computational capacity across all nodes
    pub total_capacity: ComputationalCapacity,
    /// Number of active contributing nodes
    pub active_nodes: u32,
    /// Average contribution score across federation
    pub average_contribution_score: f64,
    /// Total compute hours contributed by federation
    pub total_compute_hours: f64,
    /// Resource demand vs supply ratio
    pub demand_supply_ratio: f64,
}

impl FederationResourcePool {
    /// Calculate federation health factor affecting mana regeneration
    pub fn federation_health_factor(&self) -> f64 {
        // Higher factor when supply exceeds demand
        let supply_factor = (2.0 - self.demand_supply_ratio).max(0.5).min(2.0);

        // Higher factor for larger, more active federations
        let scale_factor = (self.active_nodes as f64 / 10.0).min(1.5);

        supply_factor * scale_factor
    }
}

/// Service managing computational mana generation and allocation
pub struct ComputationalManaService {
    config: ComputationalManaConfig,
    contributions: Arc<Mutex<HashMap<Did, ResourceContribution>>>,
    federation_pool: Arc<Mutex<FederationResourcePool>>,
    system_info: Arc<dyn SystemInfoProvider>,
    time_provider: Arc<dyn TimeProvider>,
}

impl ComputationalManaService {
    /// Create new computational mana service
    pub fn new(
        config: ComputationalManaConfig,
        system_info: Arc<dyn SystemInfoProvider>,
        time_provider: Arc<dyn TimeProvider>,
    ) -> Self {
        Self {
            config,
            contributions: Arc::new(Mutex::new(HashMap::new())),
            federation_pool: Arc::new(Mutex::new(FederationResourcePool {
                total_capacity: ComputationalCapacity {
                    cpu_cores: 0,
                    memory_mb: 0,
                    storage_mb: 0,
                    network_mbps: 0,
                    gpu_compute_units: None,
                },
                active_nodes: 0,
                average_contribution_score: 0.0,
                total_compute_hours: 0.0,
                demand_supply_ratio: 1.0,
            })),
            system_info,
            time_provider,
        }
    }

    /// Register or update a node's computational contribution
    pub async fn update_node_contribution(
        &self,
        node_did: Did,
        capacity: ComputationalCapacity,
        uptime_factor: f64,
        jobs_completed: u64,
        jobs_failed: u64,
        compute_hours_contributed: f64,
    ) -> Result<(), CommonError> {
        let mut contributions = self.contributions.lock().await;
        let now = self.time_provider.unix_seconds();

        contributions.insert(
            node_did.clone(),
            ResourceContribution {
                node_did,
                capacity,
                uptime_factor,
                jobs_completed,
                jobs_failed,
                compute_hours_contributed,
                last_updated: now,
            },
        );

        // Update federation pool
        self.recalculate_federation_pool().await?;

        Ok(())
    }

    /// Calculate mana regeneration rate for a specific node
    pub async fn calculate_mana_regeneration_rate(
        &self,
        node_did: &Did,
    ) -> Result<u64, CommonError> {
        let contributions = self.contributions.lock().await;
        let federation_pool = self.federation_pool.lock().await;

        if let Some(contribution) = contributions.get(node_did) {
            let contribution_score = contribution.contribution_score();

            if contribution_score < self.config.minimum_contribution_threshold {
                return Ok(0); // No mana regeneration for insufficient contribution
            }

            // Base regeneration scaled by contribution
            let base_rate = self.config.base_regeneration_per_hour;
            let contribution_factor =
                contribution_score / self.config.minimum_contribution_threshold;
            let federation_factor = federation_pool.federation_health_factor();

            let regeneration_rate =
                (base_rate as f64 * contribution_factor * federation_factor) as u64;

            Ok(regeneration_rate.min(base_rate * 10)) // Cap at 10x base rate
        } else {
            Ok(0) // Unknown nodes get no mana regeneration
        }
    }

    /// Calculate maximum mana capacity for a node based on contribution
    pub async fn calculate_max_mana_capacity(&self, node_did: &Did) -> Result<u64, CommonError> {
        let contributions = self.contributions.lock().await;

        if let Some(contribution) = contributions.get(node_did) {
            let contribution_score = contribution.contribution_score();
            let base_capacity = 1000u64; // Base capacity for minimum contributors

            let capacity_multiplier = (contribution_score
                / self.config.minimum_contribution_threshold)
                .min(self.config.max_capacity_multiplier);

            Ok((base_capacity as f64 * capacity_multiplier) as u64)
        } else {
            Ok(100) // Very limited capacity for unknown nodes
        }
    }

    /// Get current computational capacity of the local node
    pub async fn get_local_computational_capacity(
        &self,
    ) -> Result<ComputationalCapacity, CommonError> {
        let cpu_cores = self.system_info.cpu_cores();
        let memory_mb = self.system_info.memory_mb() as u64;

        Ok(ComputationalCapacity {
            cpu_cores,
            memory_mb,
            storage_mb: 1024 * 1024, // Default 1TB - could be measured dynamically
            network_mbps: 100,       // Default - could be measured dynamically
            gpu_compute_units: None, // Could be detected via GPU libraries
        })
    }

    /// Update demand/supply ratio based on job queue and resource availability
    pub async fn update_demand_supply_ratio(&self, demand_ratio: f64) -> Result<(), CommonError> {
        let mut federation_pool = self.federation_pool.lock().await;
        federation_pool.demand_supply_ratio = demand_ratio;
        Ok(())
    }

    /// Get federation resource statistics
    pub async fn get_federation_stats(&self) -> Result<FederationResourcePool, CommonError> {
        let federation_pool = self.federation_pool.lock().await;
        Ok(federation_pool.clone())
    }

    /// Recalculate federation-wide resource pool statistics
    async fn recalculate_federation_pool(&self) -> Result<(), CommonError> {
        let contributions = self.contributions.lock().await;
        let mut federation_pool = self.federation_pool.lock().await;

        let mut total_capacity = ComputationalCapacity {
            cpu_cores: 0,
            memory_mb: 0,
            storage_mb: 0,
            network_mbps: 0,
            gpu_compute_units: Some(0),
        };

        let mut total_score = 0.0;
        let mut total_compute_hours = 0.0;
        let active_nodes = contributions.len() as u32;

        for contribution in contributions.values() {
            total_capacity.cpu_cores += contribution.capacity.cpu_cores;
            total_capacity.memory_mb += contribution.capacity.memory_mb;
            total_capacity.storage_mb += contribution.capacity.storage_mb;
            total_capacity.network_mbps += contribution.capacity.network_mbps;

            if let Some(gpu_units) = contribution.capacity.gpu_compute_units {
                total_capacity.gpu_compute_units =
                    Some(total_capacity.gpu_compute_units.unwrap_or(0) + gpu_units);
            }

            total_score += contribution.contribution_score();
            total_compute_hours += contribution.compute_hours_contributed;
        }

        federation_pool.total_capacity = total_capacity;
        federation_pool.active_nodes = active_nodes;
        federation_pool.average_contribution_score = if active_nodes > 0 {
            total_score / active_nodes as f64
        } else {
            0.0
        };
        federation_pool.total_compute_hours = total_compute_hours;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::{SysinfoSystemInfoProvider, SystemTimeProvider};
    use std::str::FromStr;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_computational_capacity_scoring() {
        let capacity = ComputationalCapacity {
            cpu_cores: 8,
            memory_mb: 16384,    // 16GB
            storage_mb: 1024000, // 1TB
            network_mbps: 1000,
            gpu_compute_units: Some(4),
        };

        let score = capacity.compute_power_score();
        assert!(score > 0);

        // GPU should provide significant bonus
        let capacity_no_gpu = ComputationalCapacity {
            gpu_compute_units: None,
            ..capacity
        };
        assert!(capacity.compute_power_score() > capacity_no_gpu.compute_power_score());
    }

    #[tokio::test]
    async fn test_contribution_score_calculation() {
        let contribution = ResourceContribution {
            node_did: Did::from_str("did:key:test").unwrap(),
            capacity: ComputationalCapacity {
                cpu_cores: 4,
                memory_mb: 8192,
                storage_mb: 512000,
                network_mbps: 100,
                gpu_compute_units: None,
            },
            uptime_factor: 0.95,
            jobs_completed: 100,
            jobs_failed: 5,
            compute_hours_contributed: 500.0,
            last_updated: 0,
        };

        let score = contribution.contribution_score();
        assert!(score > 0.0);

        // Higher uptime should increase score
        let high_uptime_contribution = ResourceContribution {
            uptime_factor: 0.99,
            ..contribution.clone()
        };
        assert!(high_uptime_contribution.contribution_score() > contribution.contribution_score());
    }

    #[tokio::test]
    async fn test_mana_regeneration_rate_calculation() {
        let system_info = Arc::new(SysinfoSystemInfoProvider);
        let time_provider = Arc::new(SystemTimeProvider);
        let service = ComputationalManaService::new(
            ComputationalManaConfig::default(),
            system_info,
            time_provider,
        );

        let node_did = Did::from_str("did:key:test").unwrap();
        let capacity = ComputationalCapacity {
            cpu_cores: 8,
            memory_mb: 16384,
            storage_mb: 1024000,
            network_mbps: 1000,
            gpu_compute_units: Some(2),
        };

        // Register node contribution
        service
            .update_node_contribution(node_did.clone(), capacity, 0.95, 100, 5, 1000.0)
            .await
            .unwrap();

        // Calculate regeneration rate
        let rate = service
            .calculate_mana_regeneration_rate(&node_did)
            .await
            .unwrap();
        assert!(rate > 0);

        // Calculate max capacity
        let max_capacity = service
            .calculate_max_mana_capacity(&node_did)
            .await
            .unwrap();
        assert!(max_capacity > 1000); // Should be higher than base capacity
    }

    #[tokio::test]
    async fn test_federation_pool_health_factor() {
        let pool = FederationResourcePool {
            total_capacity: ComputationalCapacity {
                cpu_cores: 100,
                memory_mb: 1000000,
                storage_mb: 10000000,
                network_mbps: 10000,
                gpu_compute_units: Some(20),
            },
            active_nodes: 25,
            average_contribution_score: 1500.0,
            total_compute_hours: 50000.0,
            demand_supply_ratio: 0.8, // Supply exceeds demand
        };

        let health_factor = pool.federation_health_factor();
        assert!(health_factor > 1.0); // Should boost regeneration when supply > demand

        // Test high demand scenario
        let high_demand_pool = FederationResourcePool {
            demand_supply_ratio: 1.5, // Demand exceeds supply
            ..pool.clone()
        };
        assert!(high_demand_pool.federation_health_factor() < pool.federation_health_factor());
    }
}
