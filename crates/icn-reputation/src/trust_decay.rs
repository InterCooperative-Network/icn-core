//! Trust Decay Models
//!
//! This module implements various models for trust degradation over time and distance,
//! allowing for realistic trust relationship dynamics in the cooperative network.

use crate::trust_graph::{TrustEdge, TrustGraph};
use icn_common::TimeProvider;
use serde::{Deserialize, Serialize};

/// Different types of decay models for trust relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecayModel {
    /// Exponential decay with configurable half-life
    Exponential { half_life_seconds: u64 },
    /// Linear decay over a specified period
    Linear { decay_period_seconds: u64 },
    /// Step decay with discrete levels
    Step { intervals: Vec<DecayInterval> },
    /// Sigmoid decay with configurable steepness
    Sigmoid {
        midpoint_seconds: u64,
        steepness: f64,
    },
    /// Custom decay function combining multiple factors
    Composite {
        models: Vec<DecayModel>,
        weights: Vec<f64>,
    },
}

/// Represents a step interval for step decay model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecayInterval {
    /// Duration of this interval in seconds
    pub duration_seconds: u64,
    /// Multiplier to apply during this interval
    pub multiplier: f64,
}

/// Configuration for distance-based trust decay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistanceDecayConfig {
    /// Decay factor per hop (0.0 to 1.0)
    pub decay_per_hop: f64,
    /// Minimum trust score regardless of distance
    pub min_trust_floor: f64,
    /// Maximum effective distance before trust goes to minimum
    pub max_effective_distance: usize,
    /// Whether to use cumulative or per-hop decay
    pub use_cumulative_decay: bool,
}

impl Default for DistanceDecayConfig {
    fn default() -> Self {
        Self {
            decay_per_hop: 0.1,
            min_trust_floor: 0.01,
            max_effective_distance: 6,
            use_cumulative_decay: true,
        }
    }
}

/// Configuration for interaction-based trust decay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionDecayConfig {
    /// Time period after which lack of interaction causes decay
    pub interaction_timeout_seconds: u64,
    /// Rate of decay when no interactions occur
    pub inactivity_decay_rate: f64,
    /// Boost factor for recent positive interactions
    pub interaction_boost_factor: f64,
    /// Maximum number of interactions to consider for boost
    pub max_interaction_count: u64,
}

impl Default for InteractionDecayConfig {
    fn default() -> Self {
        Self {
            interaction_timeout_seconds: 30 * 24 * 3600, // 30 days
            inactivity_decay_rate: 0.1,
            interaction_boost_factor: 0.05,
            max_interaction_count: 100,
        }
    }
}

/// Engine for calculating various types of trust decay
pub struct TrustDecayCalculator {
    time_decay_model: DecayModel,
    distance_config: DistanceDecayConfig,
    interaction_config: InteractionDecayConfig,
}

impl TrustDecayCalculator {
    /// Create a new decay calculator with default exponential time decay
    pub fn new() -> Self {
        Self {
            time_decay_model: DecayModel::Exponential {
                half_life_seconds: 90 * 24 * 3600, // 90 days
            },
            distance_config: DistanceDecayConfig::default(),
            interaction_config: InteractionDecayConfig::default(),
        }
    }

    /// Create a decay calculator with custom time decay model
    pub fn with_time_decay(decay_model: DecayModel) -> Self {
        Self {
            time_decay_model: decay_model,
            distance_config: DistanceDecayConfig::default(),
            interaction_config: InteractionDecayConfig::default(),
        }
    }

    /// Create a decay calculator with custom configurations
    pub fn with_configs(
        time_decay: DecayModel,
        distance_config: DistanceDecayConfig,
        interaction_config: InteractionDecayConfig,
    ) -> Self {
        Self {
            time_decay_model: time_decay,
            distance_config,
            interaction_config,
        }
    }

    /// Calculate time-based decay factor for a trust edge
    pub fn calculate_time_decay(&self, edge: &TrustEdge, current_time: u64) -> f64 {
        let age_seconds = current_time.saturating_sub(edge.updated_at);
        self.apply_decay_model(&self.time_decay_model, age_seconds)
    }

    /// Calculate distance-based decay factor for a trust path
    pub fn calculate_distance_decay(&self, path_length: usize) -> f64 {
        if path_length == 0 {
            return 1.0;
        }

        let effective_distance = path_length.min(self.distance_config.max_effective_distance);

        let decay_factor = if self.distance_config.use_cumulative_decay {
            // Cumulative decay: decay compounds with each hop
            (1.0 - self.distance_config.decay_per_hop).powi(effective_distance as i32)
        } else {
            // Per-hop decay: linear reduction per hop
            1.0 - (self.distance_config.decay_per_hop * effective_distance as f64)
        };

        decay_factor.max(self.distance_config.min_trust_floor)
    }

    /// Calculate interaction-based decay/boost factor for a trust edge
    pub fn calculate_interaction_decay(&self, edge: &TrustEdge, current_time: u64) -> f64 {
        let time_since_update = current_time.saturating_sub(edge.updated_at);

        // Base decay for inactivity
        let inactivity_factor =
            if time_since_update > self.interaction_config.interaction_timeout_seconds {
                let excess_time =
                    time_since_update - self.interaction_config.interaction_timeout_seconds;
                let decay_periods =
                    excess_time as f64 / self.interaction_config.interaction_timeout_seconds as f64;
                (1.0 - self.interaction_config.inactivity_decay_rate).powf(decay_periods)
            } else {
                1.0
            };

        // Boost for interaction count (diminishing returns)
        let interaction_boost = if edge.interaction_count > 0 {
            let effective_count = edge
                .interaction_count
                .min(self.interaction_config.max_interaction_count);
            let boost =
                self.interaction_config.interaction_boost_factor * (effective_count as f64).ln();
            1.0 + boost
        } else {
            1.0
        };

        inactivity_factor * interaction_boost
    }

    /// Calculate combined decay factor considering all decay types
    pub fn calculate_combined_decay(
        &self,
        edge: &TrustEdge,
        path_length: usize,
        current_time: u64,
    ) -> f64 {
        let time_decay = self.calculate_time_decay(edge, current_time);
        let distance_decay = self.calculate_distance_decay(path_length);
        let interaction_decay = self.calculate_interaction_decay(edge, current_time);

        // Combine all decay factors multiplicatively
        time_decay * distance_decay * interaction_decay
    }

    /// Apply a specific decay model to calculate decay factor
    #[allow(clippy::only_used_in_recursion)]
    fn apply_decay_model(&self, model: &DecayModel, age_seconds: u64) -> f64 {
        match model {
            DecayModel::Exponential { half_life_seconds } => {
                let half_lives = age_seconds as f64 / *half_life_seconds as f64;
                0.5_f64.powf(half_lives)
            }
            DecayModel::Linear {
                decay_period_seconds,
            } => {
                if age_seconds >= *decay_period_seconds {
                    0.0
                } else {
                    1.0 - (age_seconds as f64 / *decay_period_seconds as f64)
                }
            }
            DecayModel::Step { intervals } => {
                let mut accumulated_time = 0u64;
                for interval in intervals {
                    accumulated_time += interval.duration_seconds;
                    if age_seconds <= accumulated_time {
                        return interval.multiplier;
                    }
                }
                // If age exceeds all intervals, return the last multiplier
                intervals.last().map(|i| i.multiplier).unwrap_or(0.0)
            }
            DecayModel::Sigmoid {
                midpoint_seconds,
                steepness,
            } => {
                let x = age_seconds as f64 / *midpoint_seconds as f64;
                1.0 / (1.0 + (steepness * x).exp())
            }
            DecayModel::Composite { models, weights } => {
                let mut weighted_sum = 0.0;
                let mut total_weight = 0.0;

                for (model, weight) in models.iter().zip(weights.iter()) {
                    weighted_sum += self.apply_decay_model(model, age_seconds) * weight;
                    total_weight += weight;
                }

                if total_weight > 0.0 {
                    weighted_sum / total_weight
                } else {
                    1.0
                }
            }
        }
    }

    /// Update a trust graph by applying decay to all edges
    pub fn apply_decay_to_graph(
        &self,
        graph: &mut TrustGraph,
        time_provider: &dyn TimeProvider,
        remove_threshold: f64,
    ) -> usize {
        let current_time = time_provider.unix_seconds();
        let mut edges_to_remove = Vec::new();
        let mut edges_to_update = Vec::new();

        // Collect edges that need updating or removal
        for node in graph.get_all_nodes() {
            if let Some(outgoing_edges) = graph.get_outgoing_edges(&node) {
                for (target, edge) in outgoing_edges {
                    let decay_factor = self.calculate_time_decay(edge, current_time);
                    let new_weight = edge.weight * decay_factor;

                    if new_weight < remove_threshold {
                        edges_to_remove.push((node.clone(), target.clone()));
                    } else if decay_factor < 1.0 {
                        let mut updated_edge = edge.clone();
                        updated_edge.weight = new_weight;
                        edges_to_update.push(updated_edge);
                    }
                }
            }
        }

        // Remove edges below threshold
        let removed_count = edges_to_remove.len();
        for (from, to) in edges_to_remove {
            graph.remove_edge(&from, &to);
        }

        // Update remaining edges with decay
        for updated_edge in edges_to_update {
            graph.add_edge(updated_edge);
        }

        removed_count
    }

    /// Calculate trust score with decay for a specific relationship
    pub fn get_decayed_trust_score(
        &self,
        edge: &TrustEdge,
        path_length: usize,
        time_provider: &dyn TimeProvider,
    ) -> f64 {
        let current_time = time_provider.unix_seconds();
        let decay_factor = self.calculate_combined_decay(edge, path_length, current_time);
        edge.weight * decay_factor
    }

    /// Get statistics about decay effects on the graph
    pub fn calculate_decay_statistics(
        &self,
        graph: &TrustGraph,
        time_provider: &dyn TimeProvider,
    ) -> DecayStatistics {
        let current_time = time_provider.unix_seconds();
        let mut stats = DecayStatistics::default();
        let mut age_sum = 0u64;
        let mut decay_factor_sum = 0.0;

        for node in graph.get_all_nodes() {
            if let Some(outgoing_edges) = graph.get_outgoing_edges(&node) {
                for edge in outgoing_edges.values() {
                    stats.total_edges += 1;

                    let age = current_time.saturating_sub(edge.updated_at);
                    age_sum += age;

                    let decay_factor = self.calculate_time_decay(edge, current_time);
                    decay_factor_sum += decay_factor;

                    if decay_factor < 0.5 {
                        stats.significantly_decayed += 1;
                    }
                    if decay_factor < 0.1 {
                        stats.critically_decayed += 1;
                    }
                }
            }
        }

        if stats.total_edges > 0 {
            stats.average_age_seconds = age_sum / stats.total_edges as u64;
            stats.average_decay_factor = decay_factor_sum / stats.total_edges as f64;
        }

        stats
    }
}

/// Statistics about decay effects on the trust graph
#[derive(Debug, Default, Clone)]
pub struct DecayStatistics {
    /// Total number of edges analyzed
    pub total_edges: usize,
    /// Number of edges with decay factor < 0.5
    pub significantly_decayed: usize,
    /// Number of edges with decay factor < 0.1
    pub critically_decayed: usize,
    /// Average age of all edges in seconds
    pub average_age_seconds: u64,
    /// Average decay factor across all edges
    pub average_decay_factor: f64,
}

impl DecayStatistics {
    /// Get the percentage of significantly decayed edges
    pub fn significant_decay_percentage(&self) -> f64 {
        if self.total_edges == 0 {
            0.0
        } else {
            (self.significantly_decayed as f64 / self.total_edges as f64) * 100.0
        }
    }

    /// Get the percentage of critically decayed edges
    pub fn critical_decay_percentage(&self) -> f64 {
        if self.total_edges == 0 {
            0.0
        } else {
            (self.critically_decayed as f64 / self.total_edges as f64) * 100.0
        }
    }
}

impl Default for TrustDecayCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trust_graph::TrustGraph;
    use icn_common::{Did, FixedTimeProvider};

    fn create_test_did(id: &str) -> Did {
        Did::from_str(&format!("did:test:{}", id)).unwrap()
    }

    #[test]
    fn test_exponential_decay() {
        let calculator = TrustDecayCalculator::with_time_decay(DecayModel::Exponential {
            half_life_seconds: 86400, // 1 day
        });

        let alice = create_test_did("alice");
        let bob = create_test_did("bob");
        let edge = TrustEdge::new(alice, bob, 1.0, 1000);

        // After 1 day (1 half-life), should be ~0.5
        let decay_1_day = calculator.calculate_time_decay(&edge, 1000 + 86400);
        assert!((decay_1_day - 0.5).abs() < 0.01);

        // After 2 days (2 half-lives), should be ~0.25
        let decay_2_days = calculator.calculate_time_decay(&edge, 1000 + 2 * 86400);
        assert!((decay_2_days - 0.25).abs() < 0.01);
    }

    #[test]
    fn test_linear_decay() {
        let calculator = TrustDecayCalculator::with_time_decay(DecayModel::Linear {
            decay_period_seconds: 86400, // 1 day
        });

        let alice = create_test_did("alice");
        let bob = create_test_did("bob");
        let edge = TrustEdge::new(alice, bob, 1.0, 1000);

        // After half day, should be 0.5
        let decay_half_day = calculator.calculate_time_decay(&edge, 1000 + 43200);
        assert!((decay_half_day - 0.5).abs() < 0.01);

        // After full day, should be 0.0
        let decay_full_day = calculator.calculate_time_decay(&edge, 1000 + 86400);
        assert!(decay_full_day < 0.01);

        // After more than full day, should still be 0.0
        let decay_overtime = calculator.calculate_time_decay(&edge, 1000 + 2 * 86400);
        assert!(decay_overtime < 0.01);
    }

    #[test]
    fn test_step_decay() {
        let intervals = vec![
            DecayInterval {
                duration_seconds: 3600,
                multiplier: 1.0,
            }, // First hour: no decay
            DecayInterval {
                duration_seconds: 3600,
                multiplier: 0.8,
            }, // Second hour: 80%
            DecayInterval {
                duration_seconds: 3600,
                multiplier: 0.5,
            }, // Third hour: 50%
        ];

        let calculator = TrustDecayCalculator::with_time_decay(DecayModel::Step { intervals });

        let alice = create_test_did("alice");
        let bob = create_test_did("bob");
        let edge = TrustEdge::new(alice, bob, 1.0, 1000);

        // After 30 minutes (within first hour)
        let decay_30min = calculator.calculate_time_decay(&edge, 1000 + 1800);
        assert_eq!(decay_30min, 1.0);

        // After 90 minutes (within second hour)
        let decay_90min = calculator.calculate_time_decay(&edge, 1000 + 5400);
        assert_eq!(decay_90min, 0.8);

        // After 150 minutes (within third hour)
        let decay_150min = calculator.calculate_time_decay(&edge, 1000 + 9000);
        assert_eq!(decay_150min, 0.5);

        // After 4 hours (beyond all intervals)
        let decay_4hours = calculator.calculate_time_decay(&edge, 1000 + 14400);
        assert_eq!(decay_4hours, 0.5); // Should use last interval's multiplier
    }

    #[test]
    fn test_distance_decay() {
        let calculator = TrustDecayCalculator::new();

        // Path length 0 (self) should have no decay
        assert_eq!(calculator.calculate_distance_decay(0), 1.0);

        // Path length 1 should have some decay
        let decay_1 = calculator.calculate_distance_decay(1);
        assert!(decay_1 < 1.0 && decay_1 > 0.0);

        // Longer paths should have more decay
        let decay_2 = calculator.calculate_distance_decay(2);
        let decay_3 = calculator.calculate_distance_decay(3);
        assert!(decay_1 > decay_2);
        assert!(decay_2 > decay_3);

        // Should respect minimum floor
        let decay_long = calculator.calculate_distance_decay(100);
        assert!(decay_long >= calculator.distance_config.min_trust_floor);
    }

    #[test]
    fn test_interaction_decay() {
        let calculator = TrustDecayCalculator::new();
        let alice = create_test_did("alice");
        let bob = create_test_did("bob");

        // Edge with no interactions, recently updated
        let recent_edge = TrustEdge::new(alice.clone(), bob.clone(), 1.0, 1000);
        let recent_decay = calculator.calculate_interaction_decay(&recent_edge, 1000 + 3600); // 1 hour later
        assert_eq!(recent_decay, 1.0); // No decay for recent activity

        // Edge with no interactions, old update
        let old_edge = TrustEdge::new(alice.clone(), bob.clone(), 1.0, 1000);
        let old_decay = calculator.calculate_interaction_decay(&old_edge, 1000 + 60 * 24 * 3600); // 60 days later
        assert!(old_decay < 1.0); // Should have decay for inactivity

        // Edge with many interactions
        let mut interactive_edge = TrustEdge::new(alice, bob, 1.0, 1000);
        interactive_edge.interaction_count = 50;
        let interactive_decay =
            calculator.calculate_interaction_decay(&interactive_edge, 1000 + 3600);
        assert!(interactive_decay > 1.0); // Should have boost for interactions
    }

    #[test]
    fn test_combined_decay() {
        let calculator = TrustDecayCalculator::new();
        let alice = create_test_did("alice");
        let bob = create_test_did("bob");
        let edge = TrustEdge::new(alice, bob, 1.0, 1000);

        let current_time = 1000 + 30 * 24 * 3600; // 30 days later
        let path_length = 2;

        let combined_decay = calculator.calculate_combined_decay(&edge, path_length, current_time);

        // Should be less than 1.0 due to time and distance decay
        assert!(combined_decay < 1.0);
        assert!(combined_decay > 0.0);

        // Should be the product of individual decay factors
        let time_decay = calculator.calculate_time_decay(&edge, current_time);
        let distance_decay = calculator.calculate_distance_decay(path_length);
        let interaction_decay = calculator.calculate_interaction_decay(&edge, current_time);
        let expected = time_decay * distance_decay * interaction_decay;

        assert!((combined_decay - expected).abs() < 1e-6);
    }

    #[test]
    fn test_apply_decay_to_graph() {
        let mut calculator = TrustDecayCalculator::with_time_decay(DecayModel::Linear {
            decay_period_seconds: 86400, // 1 day
        });
        let mut graph = TrustGraph::new();
        let time_provider = FixedTimeProvider::new(1000 + 2 * 86400); // 2 days later

        let alice = create_test_did("alice");
        let bob = create_test_did("bob");
        let charlie = create_test_did("charlie");

        // Add edges with different ages
        graph.add_edge(TrustEdge::new(alice.clone(), bob.clone(), 1.0, 1000)); // 2 days old - should be removed
        graph.add_edge(TrustEdge::new(
            bob.clone(),
            charlie.clone(),
            1.0,
            1000 + 43200,
        )); // 1.5 days old - should be removed

        assert_eq!(graph.edge_count(), 2);

        let removed_count = calculator.apply_decay_to_graph(&mut graph, &time_provider, 0.1);

        // Both edges should be removed due to linear decay over 1 day
        assert_eq!(removed_count, 2);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_decay_statistics() {
        let calculator = TrustDecayCalculator::new();
        let mut graph = TrustGraph::new();
        let time_provider = FixedTimeProvider::new(1000 + 90 * 24 * 3600); // 90 days later

        let alice = create_test_did("alice");
        let bob = create_test_did("bob");
        let charlie = create_test_did("charlie");
        let david = create_test_did("david");

        // Add edges with different ages
        graph.add_edge(TrustEdge::new(alice.clone(), bob.clone(), 1.0, 1000)); // 90 days old
        graph.add_edge(TrustEdge::new(
            bob.clone(),
            charlie.clone(),
            1.0,
            1000 + 45 * 24 * 3600,
        )); // 45 days old
        graph.add_edge(TrustEdge::new(
            charlie.clone(),
            david.clone(),
            1.0,
            1000 + 80 * 24 * 3600,
        )); // 10 days old

        let stats = calculator.calculate_decay_statistics(&graph, &time_provider);

        assert_eq!(stats.total_edges, 3);
        assert!(stats.average_age_seconds > 0);
        assert!(stats.average_decay_factor < 1.0);
        assert!(stats.significant_decay_percentage() >= 0.0);
    }

    #[test]
    fn test_composite_decay_model() {
        let models = vec![
            DecayModel::Exponential {
                half_life_seconds: 86400,
            },
            DecayModel::Linear {
                decay_period_seconds: 172800,
            },
        ];
        let weights = vec![0.7, 0.3];

        let calculator =
            TrustDecayCalculator::with_time_decay(DecayModel::Composite { models, weights });

        let alice = create_test_did("alice");
        let bob = create_test_did("bob");
        let edge = TrustEdge::new(alice, bob, 1.0, 1000);

        let decay = calculator.calculate_time_decay(&edge, 1000 + 86400);

        // Should be weighted combination of exponential (0.5) and linear (0.5)
        let expected = 0.7 * 0.5 + 0.3 * 0.5;
        assert!((decay - expected).abs() < 0.01);
    }
}
