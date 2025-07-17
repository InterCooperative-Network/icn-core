//! Trust Aggregation
//!
//! This module implements algorithms for combining multiple trust signals
//! into composite scores, enabling comprehensive trust evaluation in the cooperative network.

use crate::trust_graph::{TrustGraph, TrustEdge};
use crate::trust_calculation::TrustCalculationEngine;
use crate::trust_pathfinding::TrustPathfinder;
use crate::trust_decay::TrustDecayCalculator;
use icn_common::{Did, TimeProvider};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Different types of trust signals that can be aggregated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrustSignal {
    /// Direct trust relationship between two entities
    DirectTrust { from: Did, to: Did, weight: f64 },
    /// Indirect trust through intermediaries
    IndirectTrust { from: Did, to: Did, path_length: usize, weight: f64 },
    /// Reputation-based trust (from reputation system)
    ReputationTrust { entity: Did, score: f64 },
    /// Activity-based trust (based on interaction frequency)
    ActivityTrust { entity: Did, interaction_count: u64, recency_score: f64 },
    /// Endorsement from other trusted entities
    EndorsementTrust { endorser: Did, target: Did, weight: f64 },
    /// Historical performance metrics
    PerformanceTrust { entity: Did, success_rate: f64, sample_size: u64 },
    /// Network position-based trust (centrality measures)
    NetworkTrust { entity: Did, centrality_score: f64 },
}

/// Configuration for trust aggregation algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationConfig {
    /// Weights for different types of trust signals
    pub signal_weights: HashMap<String, f64>,
    /// Method for combining multiple signals
    pub combination_method: CombinationMethod,
    /// Minimum number of signals required for reliable aggregation
    pub min_signals: usize,
    /// Confidence decay factor when fewer signals are available
    pub confidence_decay: f64,
    /// Whether to normalize final scores to [0, 1] range
    pub normalize_scores: bool,
    /// Outlier detection threshold (standard deviations)
    pub outlier_threshold: f64,
}

/// Different methods for combining trust signals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CombinationMethod {
    /// Weighted arithmetic mean
    WeightedMean,
    /// Weighted geometric mean (emphasizes consistency)
    WeightedGeometricMean,
    /// Weighted harmonic mean (emphasizes weakest signals)
    WeightedHarmonicMean,
    /// Maximum likelihood estimation
    MaximumLikelihood,
    /// Bayesian combination with prior beliefs
    Bayesian { prior_mean: f64, prior_variance: f64 },
    /// Fuzzy logic combination
    FuzzyLogic,
    /// Custom weighted combination with non-linear transforms
    Custom { alpha: f64, beta: f64 },
}

impl Default for AggregationConfig {
    fn default() -> Self {
        let mut signal_weights = HashMap::new();
        signal_weights.insert("direct".to_string(), 0.4);
        signal_weights.insert("indirect".to_string(), 0.2);
        signal_weights.insert("reputation".to_string(), 0.15);
        signal_weights.insert("activity".to_string(), 0.1);
        signal_weights.insert("endorsement".to_string(), 0.1);
        signal_weights.insert("performance".to_string(), 0.15);
        signal_weights.insert("network".to_string(), 0.1);

        Self {
            signal_weights,
            combination_method: CombinationMethod::WeightedMean,
            min_signals: 2,
            confidence_decay: 0.1,
            normalize_scores: true,
            outlier_threshold: 2.0,
        }
    }
}

/// Result of trust aggregation for an entity
#[derive(Debug, Clone)]
pub struct AggregatedTrust {
    /// The target entity
    pub entity: Did,
    /// Final aggregated trust score
    pub trust_score: f64,
    /// Confidence in the aggregated score (0.0 to 1.0)
    pub confidence: f64,
    /// Number of signals used in aggregation
    pub signal_count: usize,
    /// Breakdown of scores by signal type
    pub signal_breakdown: HashMap<String, f64>,
    /// Variance in the input signals
    pub signal_variance: f64,
    /// Timestamp when aggregation was performed
    pub timestamp: u64,
}

/// Engine for aggregating multiple trust signals into composite scores
pub struct TrustAggregator {
    config: AggregationConfig,
    calculation_engine: TrustCalculationEngine,
    pathfinder: TrustPathfinder,
    decay_calculator: TrustDecayCalculator,
}

impl TrustAggregator {
    /// Create a new trust aggregator with default configuration
    pub fn new() -> Self {
        Self {
            config: AggregationConfig::default(),
            calculation_engine: TrustCalculationEngine::new(),
            pathfinder: TrustPathfinder::new(),
            decay_calculator: TrustDecayCalculator::new(),
        }
    }

    /// Create a new trust aggregator with custom configuration
    pub fn with_config(config: AggregationConfig) -> Self {
        Self {
            config,
            calculation_engine: TrustCalculationEngine::new(),
            pathfinder: TrustPathfinder::new(),
            decay_calculator: TrustDecayCalculator::new(),
        }
    }

    /// Aggregate trust signals for a specific entity
    pub fn aggregate_trust_for_entity(
        &self,
        entity: &Did,
        signals: Vec<TrustSignal>,
        time_provider: &dyn TimeProvider,
    ) -> AggregatedTrust {
        let timestamp = time_provider.unix_seconds();
        
        // Filter and categorize signals by type
        let signal_scores = self.extract_signal_scores(&signals, entity);
        
        // Remove outliers if configured
        let filtered_scores = if self.config.outlier_threshold > 0.0 {
            self.remove_outliers(signal_scores)
        } else {
            signal_scores
        };

        // Calculate weighted combination
        let (trust_score, confidence) = self.calculate_weighted_score(&filtered_scores);
        
        // Calculate variance for confidence assessment
        let signal_variance = self.calculate_signal_variance(&filtered_scores);
        
        // Apply confidence decay if insufficient signals
        let adjusted_confidence = self.adjust_confidence_for_signal_count(
            confidence,
            filtered_scores.len(),
        );

        // Normalize score if configured
        let final_score = if self.config.normalize_scores {
            trust_score.clamp(0.0, 1.0)
        } else {
            trust_score
        };

        AggregatedTrust {
            entity: entity.clone(),
            trust_score: final_score,
            confidence: adjusted_confidence,
            signal_count: filtered_scores.len(),
            signal_breakdown: filtered_scores,
            signal_variance,
            timestamp,
        }
    }

    /// Aggregate trust for multiple entities and return ranked results
    pub fn aggregate_trust_for_entities(
        &self,
        entities: &[Did],
        graph: &TrustGraph,
        observer: &Did,
        time_provider: &dyn TimeProvider,
    ) -> Vec<AggregatedTrust> {
        let mut results = Vec::new();

        for entity in entities {
            if entity == observer {
                continue; // Skip self-trust evaluation
            }

            let signals = self.collect_trust_signals(graph, observer, entity, time_provider);
            let aggregated = self.aggregate_trust_for_entity(entity, signals, time_provider);
            results.push(aggregated);
        }

        // Sort by trust score (highest first)
        results.sort_by(|a, b| {
            b.trust_score.partial_cmp(&a.trust_score).unwrap_or(std::cmp::Ordering::Equal)
        });

        results
    }

    /// Collect various trust signals for an entity from the trust graph
    fn collect_trust_signals(
        &self,
        graph: &TrustGraph,
        observer: &Did,
        target: &Did,
        time_provider: &dyn TimeProvider,
    ) -> Vec<TrustSignal> {
        let mut signals = Vec::new();

        // Direct trust signal
        if let Some(edge) = graph.get_edge(observer, target) {
            let decayed_weight = self.decay_calculator.calculate_time_decay(edge, time_provider.unix_seconds());
            signals.push(TrustSignal::DirectTrust {
                from: observer.clone(),
                to: target.clone(),
                weight: decayed_weight,
            });
        }

        // Indirect trust signals (through paths)
        let paths = self.pathfinder.find_multiple_paths(graph, observer, target, time_provider);
        for path in paths.iter().take(3) { // Limit to top 3 paths to avoid noise
            if path.length > 1 {
                signals.push(TrustSignal::IndirectTrust {
                    from: observer.clone(),
                    to: target.clone(),
                    path_length: path.length,
                    weight: path.trust_score,
                });
            }
        }

        // Reputation signal (based on incoming edges)
        if let Some(incoming_edges) = graph.get_incoming_edges(target) {
            let mut total_weight = 0.0;
            let mut count = 0;
            
            for edge in incoming_edges.values() {
                let decayed_weight = self.decay_calculator.calculate_time_decay(edge, time_provider.unix_seconds());
                total_weight += decayed_weight;
                count += 1;
            }
            
            if count > 0 {
                signals.push(TrustSignal::ReputationTrust {
                    entity: target.clone(),
                    score: total_weight / count as f64,
                });
            }
        }

        // Activity signal (based on interaction patterns)
        if let Some(edge) = graph.get_edge(observer, target) {
            let current_time = time_provider.unix_seconds();
            let recency_score = if edge.updated_at > 0 {
                let age_days = (current_time.saturating_sub(edge.updated_at)) as f64 / 86400.0;
                (-age_days / 30.0).exp() // Exponential decay over 30 days
            } else {
                0.0
            };

            signals.push(TrustSignal::ActivityTrust {
                entity: target.clone(),
                interaction_count: edge.interaction_count,
                recency_score,
            });
        }

        // Network position signal (simplified centrality measure)
        let centrality_score = self.calculate_simple_centrality(graph, target);
        signals.push(TrustSignal::NetworkTrust {
            entity: target.clone(),
            centrality_score,
        });

        signals
    }

    /// Extract numerical scores from signals by type
    fn extract_signal_scores(&self, signals: &[TrustSignal], _entity: &Did) -> HashMap<String, f64> {
        let mut scores = HashMap::new();

        for signal in signals {
            let (signal_type, score) = match signal {
                TrustSignal::DirectTrust { weight, .. } => ("direct", *weight),
                TrustSignal::IndirectTrust { weight, path_length, .. } => {
                    // Apply distance penalty to indirect trust
                    let distance_penalty = 0.8_f64.powi(*path_length as i32 - 1);
                    ("indirect", weight * distance_penalty)
                }
                TrustSignal::ReputationTrust { score, .. } => ("reputation", *score),
                TrustSignal::ActivityTrust { interaction_count, recency_score, .. } => {
                    // Combine interaction count and recency
                    let activity_score = ((*interaction_count as f64).ln() / 10.0).min(1.0) * recency_score;
                    ("activity", activity_score)
                }
                TrustSignal::EndorsementTrust { weight, .. } => ("endorsement", *weight),
                TrustSignal::PerformanceTrust { success_rate, sample_size, .. } => {
                    // Weight by sample size
                    let weight = (*sample_size as f64 / 100.0).min(1.0);
                    ("performance", success_rate * weight)
                }
                TrustSignal::NetworkTrust { centrality_score, .. } => ("network", *centrality_score),
            };

            // If multiple signals of same type, take the maximum
            let current_score = scores.get(signal_type).copied().unwrap_or(0.0);
            scores.insert(signal_type.to_string(), current_score.max(score));
        }

        scores
    }

    /// Remove outlier signals based on statistical threshold
    fn remove_outliers(&self, mut scores: HashMap<String, f64>) -> HashMap<String, f64> {
        if scores.len() < 3 {
            return scores; // Can't identify outliers with too few samples
        }

        let values: Vec<f64> = scores.values().copied().collect();
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        if std_dev == 0.0 {
            return scores; // All values are the same
        }

        let threshold = self.config.outlier_threshold * std_dev;
        
        scores.retain(|_, score| (*score - mean).abs() <= threshold);
        scores
    }

    /// Calculate weighted score using the configured combination method
    fn calculate_weighted_score(&self, signal_scores: &HashMap<String, f64>) -> (f64, f64) {
        if signal_scores.is_empty() {
            return (0.0, 0.0);
        }

        let mut weighted_sum = 0.0;
        let mut total_weight = 0.0;
        let mut values = Vec::new();

        for (signal_type, score) in signal_scores {
            let weight = self.config.signal_weights.get(signal_type).copied().unwrap_or(1.0);
            weighted_sum += score * weight;
            total_weight += weight;
            values.push(*score);
        }

        if total_weight == 0.0 {
            return (0.0, 0.0);
        }

        let combined_score = match &self.config.combination_method {
            CombinationMethod::WeightedMean => weighted_sum / total_weight,
            
            CombinationMethod::WeightedGeometricMean => {
                let mut product = 1.0;
                let mut weight_sum = 0.0;
                
                for (signal_type, score) in signal_scores {
                    let weight = self.config.signal_weights.get(signal_type).copied().unwrap_or(1.0);
                    if *score > 0.0 {
                        product *= score.powf(weight);
                        weight_sum += weight;
                    }
                }
                
                if weight_sum > 0.0 {
                    product.powf(1.0 / weight_sum)
                } else {
                    0.0
                }
            }
            
            CombinationMethod::WeightedHarmonicMean => {
                let mut harmonic_sum = 0.0;
                let mut weight_sum = 0.0;
                
                for (signal_type, score) in signal_scores {
                    let weight = self.config.signal_weights.get(signal_type).copied().unwrap_or(1.0);
                    if *score > 0.0 {
                        harmonic_sum += weight / score;
                        weight_sum += weight;
                    }
                }
                
                if harmonic_sum > 0.0 {
                    weight_sum / harmonic_sum
                } else {
                    0.0
                }
            }
            
            CombinationMethod::Bayesian { prior_mean, prior_variance } => {
                let sample_mean = values.iter().sum::<f64>() / values.len() as f64;
                let sample_variance = values.iter().map(|v| (v - sample_mean).powi(2)).sum::<f64>() / values.len() as f64;
                
                // Bayesian updating
                let posterior_precision = 1.0 / prior_variance + values.len() as f64 / sample_variance;
                let posterior_mean = (prior_mean / prior_variance + sample_mean * values.len() as f64 / sample_variance) / posterior_precision;
                
                posterior_mean
            }
            
            CombinationMethod::Custom { alpha, beta } => {
                let base_score = weighted_sum / total_weight;
                // Apply non-linear transform: score^alpha * (1 - score)^beta normalized
                let transformed = base_score.powf(*alpha) * (1.0 - base_score).powf(*beta);
                transformed / (alpha / (alpha + beta)).powf(*alpha) * (beta / (alpha + beta)).powf(*beta)
            }
            
            _ => weighted_sum / total_weight, // Default to weighted mean
        };

        // Calculate confidence based on signal consistency
        let confidence = self.calculate_signal_confidence(&values);
        
        (combined_score, confidence)
    }

    /// Calculate confidence in the aggregated score based on signal consistency
    fn calculate_signal_confidence(&self, values: &[f64]) -> f64 {
        if values.len() < 2 {
            return 0.5; // Low confidence with single signal
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        // Confidence inversely related to variance (more consistent = higher confidence)
        let consistency_factor = (-std_dev).exp();
        
        // Scale by number of signals (more signals = higher confidence, with diminishing returns)
        let signal_factor = 1.0 - (-values.len() as f64 / 5.0).exp();
        
        (consistency_factor * signal_factor).clamp(0.0, 1.0)
    }

    /// Adjust confidence based on the number of available signals
    fn adjust_confidence_for_signal_count(&self, base_confidence: f64, signal_count: usize) -> f64 {
        if signal_count >= self.config.min_signals {
            base_confidence
        } else {
            let shortage = (self.config.min_signals - signal_count) as f64;
            let penalty = shortage * self.config.confidence_decay;
            (base_confidence - penalty).max(0.0)
        }
    }

    /// Calculate signal variance for the aggregated trust result
    fn calculate_signal_variance(&self, signal_scores: &HashMap<String, f64>) -> f64 {
        if signal_scores.len() < 2 {
            return 0.0;
        }

        let values: Vec<f64> = signal_scores.values().copied().collect();
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64
    }

    /// Calculate a simple centrality measure for network position
    fn calculate_simple_centrality(&self, graph: &TrustGraph, entity: &Did) -> f64 {
        let incoming_count = graph.get_incoming_edges(entity).map(|edges| edges.len()).unwrap_or(0);
        let outgoing_count = graph.get_outgoing_edges(entity).map(|edges| edges.len()).unwrap_or(0);
        let total_nodes = graph.node_count();

        if total_nodes <= 1 {
            return 0.0;
        }

        // Simple degree centrality normalized by possible connections
        let degree = (incoming_count + outgoing_count) as f64;
        let max_possible = (total_nodes - 1) as f64; // Exclude self
        degree / max_possible
    }
}

impl Default for TrustAggregator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trust_graph::TrustGraph;
    use icn_common::FixedTimeProvider;
    use std::str::FromStr;

    fn create_test_did(id: &str) -> Did {
        Did::from_str(&format!("did:test:{}", id)).unwrap()
    }

    #[test]
    fn test_signal_extraction() {
        let aggregator = TrustAggregator::new();
        let alice = create_test_did("alice");
        let bob = create_test_did("bob");

        let signals = vec![
            TrustSignal::DirectTrust { from: alice.clone(), to: bob.clone(), weight: 0.8 },
            TrustSignal::ReputationTrust { entity: bob.clone(), score: 0.9 },
            TrustSignal::ActivityTrust { entity: bob.clone(), interaction_count: 10, recency_score: 0.7 },
        ];

        let scores = aggregator.extract_signal_scores(&signals, &bob);

        assert!(scores.contains_key("direct"));
        assert!(scores.contains_key("reputation"));
        assert!(scores.contains_key("activity"));
        assert_eq!(scores.get("direct"), Some(&0.8));
        assert_eq!(scores.get("reputation"), Some(&0.9));
    }

    #[test]
    fn test_weighted_mean_combination() {
        let mut config = AggregationConfig::default();
        config.combination_method = CombinationMethod::WeightedMean;
        let aggregator = TrustAggregator::with_config(config);

        let mut signal_scores = HashMap::new();
        signal_scores.insert("direct".to_string(), 0.8);
        signal_scores.insert("reputation".to_string(), 0.9);

        let (combined_score, confidence) = aggregator.calculate_weighted_score(&signal_scores);

        // Should be weighted average based on configured weights
        assert!(combined_score > 0.0 && combined_score <= 1.0);
        assert!(confidence > 0.0 && confidence <= 1.0);
    }

    #[test]
    fn test_geometric_mean_combination() {
        let mut config = AggregationConfig::default();
        config.combination_method = CombinationMethod::WeightedGeometricMean;
        let aggregator = TrustAggregator::with_config(config);

        let mut signal_scores = HashMap::new();
        signal_scores.insert("direct".to_string(), 0.8);
        signal_scores.insert("reputation".to_string(), 0.9);

        let (combined_score, confidence) = aggregator.calculate_weighted_score(&signal_scores);

        // Geometric mean should be less than arithmetic mean for these values
        assert!(combined_score > 0.0 && combined_score < 0.85);
        assert!(confidence > 0.0 && confidence <= 1.0);
    }

    #[test]
    fn test_outlier_removal() {
        let aggregator = TrustAggregator::new();

        let mut signal_scores = HashMap::new();
        signal_scores.insert("signal1".to_string(), 0.8);
        signal_scores.insert("signal2".to_string(), 0.85);
        signal_scores.insert("signal3".to_string(), 0.9);
        signal_scores.insert("outlier".to_string(), 0.1); // Clear outlier

        let filtered = aggregator.remove_outliers(signal_scores);

        // Outlier should be removed
        assert!(!filtered.contains_key("outlier"));
        assert_eq!(filtered.len(), 3);
    }

    #[test]
    fn test_confidence_calculation() {
        let aggregator = TrustAggregator::new();

        // High consistency signals
        let consistent_values = vec![0.8, 0.82, 0.78, 0.81];
        let high_confidence = aggregator.calculate_signal_confidence(&consistent_values);

        // Low consistency signals
        let inconsistent_values = vec![0.2, 0.9, 0.1, 0.8];
        let low_confidence = aggregator.calculate_signal_confidence(&inconsistent_values);

        assert!(high_confidence > low_confidence);
        assert!(high_confidence > 0.5);
        assert!(low_confidence < 0.5);
    }

    #[test]
    fn test_aggregate_trust_for_entity() {
        let aggregator = TrustAggregator::new();
        let time_provider = FixedTimeProvider::new(1000);
        let alice = create_test_did("alice");
        let bob = create_test_did("bob");

        let signals = vec![
            TrustSignal::DirectTrust { from: alice.clone(), to: bob.clone(), weight: 0.8 },
            TrustSignal::ReputationTrust { entity: bob.clone(), score: 0.9 },
            TrustSignal::NetworkTrust { entity: bob.clone(), centrality_score: 0.6 },
        ];

        let result = aggregator.aggregate_trust_for_entity(&bob, signals, &time_provider);

        assert_eq!(result.entity, bob);
        assert!(result.trust_score > 0.0);
        assert!(result.confidence > 0.0);
        assert_eq!(result.signal_count, 3);
        assert!(!result.signal_breakdown.is_empty());
        assert_eq!(result.timestamp, 1000);
    }

    #[test]
    fn test_collect_trust_signals() {
        let aggregator = TrustAggregator::new();
        let time_provider = FixedTimeProvider::new(1000);
        let mut graph = TrustGraph::new();

        let alice = create_test_did("alice");
        let bob = create_test_did("bob");
        let charlie = create_test_did("charlie");

        // Create some trust relationships
        graph.add_edge(TrustEdge::new(alice.clone(), bob.clone(), 0.8, 950));
        graph.add_edge(TrustEdge::new(charlie.clone(), bob.clone(), 0.9, 960));
        graph.add_edge(TrustEdge::new(alice.clone(), charlie.clone(), 0.7, 970));

        let signals = aggregator.collect_trust_signals(&graph, &alice, &bob, &time_provider);

        // Should collect various types of signals
        assert!(!signals.is_empty());
        
        // Should include direct trust signal
        assert!(signals.iter().any(|s| matches!(s, TrustSignal::DirectTrust { .. })));
        
        // Should include network trust signal
        assert!(signals.iter().any(|s| matches!(s, TrustSignal::NetworkTrust { .. })));
    }

    #[test]
    fn test_simple_centrality_calculation() {
        let aggregator = TrustAggregator::new();
        let mut graph = TrustGraph::new();

        let alice = create_test_did("alice");
        let bob = create_test_did("bob");
        let charlie = create_test_did("charlie");

        // Bob is more central (connected to both Alice and Charlie)
        graph.add_edge(TrustEdge::new(alice.clone(), bob.clone(), 0.8, 1000));
        graph.add_edge(TrustEdge::new(bob.clone(), charlie.clone(), 0.9, 1000));

        let bob_centrality = aggregator.calculate_simple_centrality(&graph, &bob);
        let alice_centrality = aggregator.calculate_simple_centrality(&graph, &alice);

        // Bob should have higher centrality (more connections)
        assert!(bob_centrality > alice_centrality);
    }

    #[test]
    fn test_confidence_decay_for_insufficient_signals() {
        let mut config = AggregationConfig::default();
        config.min_signals = 3;
        config.confidence_decay = 0.2;
        let aggregator = TrustAggregator::with_config(config);

        let base_confidence = 0.8;
        let signal_count = 1; // Below minimum

        let adjusted = aggregator.adjust_confidence_for_signal_count(base_confidence, signal_count);

        // Should have penalty for insufficient signals
        assert!(adjusted < base_confidence);
        
        // With minimum signals, should not be penalized
        let sufficient = aggregator.adjust_confidence_for_signal_count(base_confidence, 3);
        assert_eq!(sufficient, base_confidence);
    }

    #[test]
    fn test_aggregation_with_empty_signals() {
        let aggregator = TrustAggregator::new();
        let time_provider = FixedTimeProvider::new(1000);
        let alice = create_test_did("alice");

        let result = aggregator.aggregate_trust_for_entity(&alice, vec![], &time_provider);

        assert_eq!(result.trust_score, 0.0);
        assert_eq!(result.confidence, 0.0);
        assert_eq!(result.signal_count, 0);
    }
}