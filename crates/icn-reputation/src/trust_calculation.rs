//! Trust Score Calculation Engine
//!
//! This module implements various algorithms for calculating trust scores
//! including PageRank-style algorithms and weighted trust propagation.

use crate::trust_graph::{TrustEdge, TrustGraph};
use icn_common::{Did, TimeProvider};
use std::collections::HashMap;

/// Configuration parameters for trust score calculation algorithms
#[derive(Debug, Clone)]
pub struct TrustCalculationConfig {
    /// Damping factor for PageRank-style calculations (typically 0.85)
    pub damping_factor: f64,
    /// Maximum number of iterations for iterative algorithms
    pub max_iterations: usize,
    /// Convergence threshold for iterative algorithms
    pub convergence_threshold: f64,
    /// Minimum trust score (prevents scores from going to zero)
    pub min_score: f64,
    /// Maximum trust score (caps extremely high scores)
    pub max_score: f64,
    /// Weight for direct vs. indirect trust relationships
    pub direct_trust_weight: f64,
}

impl Default for TrustCalculationConfig {
    fn default() -> Self {
        Self {
            damping_factor: 0.85,
            max_iterations: 100,
            convergence_threshold: 1e-6,
            min_score: 0.01,
            max_score: 1.0,
            direct_trust_weight: 0.7,
        }
    }
}

/// Engine for calculating trust scores using various algorithms
pub struct TrustCalculationEngine {
    config: TrustCalculationConfig,
}

impl TrustCalculationEngine {
    /// Create a new trust calculation engine with default configuration
    pub fn new() -> Self {
        Self {
            config: TrustCalculationConfig::default(),
        }
    }

    /// Create a new trust calculation engine with custom configuration
    pub fn with_config(config: TrustCalculationConfig) -> Self {
        Self { config }
    }

    /// Calculate PageRank-style trust scores for all nodes in the graph
    ///
    /// This algorithm treats trust relationships as directed edges and calculates
    /// a stationary distribution representing the long-term trust flow in the network.
    pub fn calculate_pagerank_scores(
        &self,
        graph: &TrustGraph,
        time_provider: &dyn TimeProvider,
    ) -> HashMap<Did, f64> {
        let nodes = graph.get_all_nodes();
        if nodes.is_empty() {
            return HashMap::new();
        }

        let n = nodes.len();
        let mut scores: HashMap<Did, f64> = nodes
            .iter()
            .map(|did| (did.clone(), 1.0 / n as f64))
            .collect();

        let current_time = time_provider.unix_seconds();

        for _iteration in 0..self.config.max_iterations {
            let mut new_scores = HashMap::new();
            let mut max_change: f64 = 0.0;

            for node in &nodes {
                let mut score = (1.0 - self.config.damping_factor) / n as f64;

                // Sum contributions from incoming edges
                if let Some(incoming_edges) = graph.get_incoming_edges(node) {
                    for (source, edge) in incoming_edges {
                        if let Some(outgoing_edges) = graph.get_outgoing_edges(source) {
                            let out_degree = outgoing_edges.len() as f64;
                            if out_degree > 0.0 {
                                // Apply time-based decay to edge weight
                                let decayed_weight = self.apply_time_decay(edge, current_time);
                                let source_score = scores.get(source).unwrap_or(&0.0);
                                score += self.config.damping_factor * source_score * decayed_weight
                                    / out_degree;
                            }
                        }
                    }
                }

                score = score.clamp(self.config.min_score, self.config.max_score);
                let old_score = scores.get(node).unwrap_or(&0.0);
                max_change = max_change.max((score - old_score).abs());
                new_scores.insert(node.clone(), score);
            }

            scores = new_scores;

            // Check for convergence
            if max_change < self.config.convergence_threshold {
                break;
            }
        }

        scores
    }

    /// Calculate weighted trust scores that combine direct and indirect trust
    ///
    /// This algorithm considers both direct trust relationships and trust propagated
    /// through intermediate nodes with distance-based decay.
    pub fn calculate_weighted_trust_scores(
        &self,
        graph: &TrustGraph,
        time_provider: &dyn TimeProvider,
        max_path_length: usize,
    ) -> HashMap<Did, f64> {
        let nodes = graph.get_all_nodes();
        let mut scores: HashMap<Did, f64> = HashMap::new();
        let current_time = time_provider.unix_seconds();

        for node in &nodes {
            let mut total_score;
            let mut direct_trust_sum = 0.0;
            let mut indirect_trust_sum = 0.0;
            let mut direct_count = 0;
            let mut indirect_count = 0;

            // Calculate direct trust (path length 1)
            if let Some(incoming_edges) = graph.get_incoming_edges(node) {
                for edge in incoming_edges.values() {
                    let decayed_weight = self.apply_time_decay(edge, current_time);
                    direct_trust_sum += decayed_weight;
                    direct_count += 1;
                }
            }

            // Calculate indirect trust (path length 2 to max_path_length)
            for path_length in 2..=max_path_length {
                let indirect_paths = self.find_trust_paths_of_length(graph, node, path_length);
                for path in indirect_paths {
                    let path_trust = self.calculate_path_trust(&path, current_time);
                    // Apply distance decay
                    let distance_decay = 1.0 / (path_length as f64).powi(2);
                    indirect_trust_sum += path_trust * distance_decay;
                    indirect_count += 1;
                }
            }

            // Combine direct and indirect trust
            let direct_avg = if direct_count > 0 {
                direct_trust_sum / direct_count as f64
            } else {
                0.0
            };
            let indirect_avg = if indirect_count > 0 {
                indirect_trust_sum / indirect_count as f64
            } else {
                0.0
            };

            total_score = self.config.direct_trust_weight * direct_avg
                + (1.0 - self.config.direct_trust_weight) * indirect_avg;

            total_score = total_score.clamp(self.config.min_score, self.config.max_score);
            scores.insert(node.clone(), total_score);
        }

        scores
    }

    /// Calculate trust score for a specific target node from a source perspective
    ///
    /// This method finds the best trust path from source to target and calculates
    /// the effective trust score along that path.
    pub fn calculate_pairwise_trust(
        &self,
        graph: &TrustGraph,
        source: &Did,
        target: &Did,
        time_provider: &dyn TimeProvider,
        max_path_length: usize,
    ) -> f64 {
        // Check for direct trust first
        if let Some(edge) = graph.get_edge(source, target) {
            return self.apply_time_decay(edge, time_provider.unix_seconds());
        }

        // Find best indirect path
        let paths = self.find_all_trust_paths(graph, source, target, max_path_length);
        if paths.is_empty() {
            return 0.0;
        }

        let current_time = time_provider.unix_seconds();
        let mut best_trust: f64 = 0.0;

        for path in paths {
            let path_trust = self.calculate_path_trust(&path, current_time);
            // Apply distance decay based on path length
            let distance_decay = 1.0 / (path.len() as f64);
            let effective_trust = path_trust * distance_decay;
            best_trust = best_trust.max(effective_trust);
        }

        best_trust.clamp(self.config.min_score, self.config.max_score)
    }

    /// Apply time-based decay to a trust edge based on its age
    fn apply_time_decay(&self, edge: &TrustEdge, current_time: u64) -> f64 {
        let age_seconds = current_time.saturating_sub(edge.updated_at);
        let age_days = age_seconds as f64 / 86400.0; // Convert to days

        // Exponential decay with half-life of 90 days
        let half_life_days = 90.0;
        let decay_factor = 0.5_f64.powf(age_days / half_life_days);

        edge.weight * decay_factor
    }

    /// Calculate the trust value along a path by taking the minimum edge weight
    fn calculate_path_trust(&self, path: &[TrustEdge], current_time: u64) -> f64 {
        if path.is_empty() {
            return 0.0;
        }

        // Trust along a path is limited by the weakest link
        path.iter()
            .map(|edge| self.apply_time_decay(edge, current_time))
            .fold(f64::INFINITY, f64::min)
            .min(self.config.max_score)
    }

    /// Find all trust paths from source to target up to max_path_length
    fn find_all_trust_paths(
        &self,
        graph: &TrustGraph,
        source: &Did,
        target: &Did,
        max_path_length: usize,
    ) -> Vec<Vec<TrustEdge>> {
        let mut paths = Vec::new();
        let mut current_path = Vec::new();
        let mut visited = std::collections::HashSet::new();

        self.dfs_find_paths(
            graph,
            source,
            target,
            max_path_length,
            &mut current_path,
            &mut visited,
            &mut paths,
        );

        paths
    }

    /// Recursive depth-first search to find all paths
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::only_used_in_recursion)]
    fn dfs_find_paths(
        &self,
        graph: &TrustGraph,
        current: &Did,
        target: &Did,
        remaining_depth: usize,
        current_path: &mut Vec<TrustEdge>,
        visited: &mut std::collections::HashSet<Did>,
        all_paths: &mut Vec<Vec<TrustEdge>>,
    ) {
        if remaining_depth == 0 {
            return;
        }

        visited.insert(current.clone());

        if let Some(outgoing_edges) = graph.get_outgoing_edges(current) {
            for (next_node, edge) in outgoing_edges {
                if next_node == target {
                    // Found a path to target
                    current_path.push(edge.clone());
                    all_paths.push(current_path.clone());
                    current_path.pop();
                } else if !visited.contains(next_node) {
                    // Continue exploring
                    current_path.push(edge.clone());
                    self.dfs_find_paths(
                        graph,
                        next_node,
                        target,
                        remaining_depth - 1,
                        current_path,
                        visited,
                        all_paths,
                    );
                    current_path.pop();
                }
            }
        }

        visited.remove(current);
    }

    /// Find trust paths of a specific length ending at target
    fn find_trust_paths_of_length(
        &self,
        graph: &TrustGraph,
        target: &Did,
        path_length: usize,
    ) -> Vec<Vec<TrustEdge>> {
        let mut paths = Vec::new();

        // Use BFS to find paths of exact length
        let mut queue = Vec::new();

        // Initialize with all nodes that have edges to target
        if let Some(incoming_edges) = graph.get_incoming_edges(target) {
            for (source, edge) in incoming_edges {
                queue.push((source.clone(), vec![edge.clone()], 1));
            }
        }

        while let Some((current_node, current_path, current_length)) = queue.pop() {
            if current_length == path_length {
                paths.push(current_path);
                continue;
            }

            if current_length < path_length {
                if let Some(incoming_edges) = graph.get_incoming_edges(&current_node) {
                    for (source, edge) in incoming_edges {
                        // Avoid cycles
                        if !current_path.iter().any(|e| e.from == *source) {
                            let mut new_path = current_path.clone();
                            new_path.insert(0, edge.clone());
                            queue.push((source.clone(), new_path, current_length + 1));
                        }
                    }
                }
            }
        }

        paths
    }
}

impl Default for TrustCalculationEngine {
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
        Did::from_str(&format!("did:test:{id}")).unwrap()
    }

    #[test]
    fn test_pagerank_simple_graph() {
        let mut graph = TrustGraph::new();
        let engine = TrustCalculationEngine::new();
        let time_provider = FixedTimeProvider::new(1000);

        let alice = create_test_did("alice");
        let bob = create_test_did("bob");
        let charlie = create_test_did("charlie");

        // Create a simple triangle: Alice -> Bob -> Charlie -> Alice
        graph.add_edge(TrustEdge::new(alice.clone(), bob.clone(), 0.8, 950));
        graph.add_edge(TrustEdge::new(bob.clone(), charlie.clone(), 0.9, 960));
        graph.add_edge(TrustEdge::new(charlie.clone(), alice.clone(), 0.7, 970));

        let scores = engine.calculate_pagerank_scores(&graph, &time_provider);

        // All nodes should have positive scores
        assert!(scores.get(&alice).unwrap() > &0.0);
        assert!(scores.get(&bob).unwrap() > &0.0);
        assert!(scores.get(&charlie).unwrap() > &0.0);

        // Sum of all scores should be approximately 1.0
        let total: f64 = scores.values().sum();
        assert!((total - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_weighted_trust_scores() {
        let mut graph = TrustGraph::new();
        let engine = TrustCalculationEngine::new();
        let time_provider = FixedTimeProvider::new(1000);

        let alice = create_test_did("alice");
        let bob = create_test_did("bob");
        let charlie = create_test_did("charlie");

        // Alice trusts Bob, Bob trusts Charlie
        graph.add_edge(TrustEdge::new(alice.clone(), bob.clone(), 0.9, 950));
        graph.add_edge(TrustEdge::new(bob.clone(), charlie.clone(), 0.8, 960));

        let scores = engine.calculate_weighted_trust_scores(&graph, &time_provider, 3);

        // Bob should have highest score (direct trust from Alice)
        // Charlie should have lower score (indirect trust through Bob)
        // Alice should have lowest score (no incoming trust)
        assert!(scores.get(&bob).unwrap() > scores.get(&charlie).unwrap());
        assert!(scores.get(&charlie).unwrap() > scores.get(&alice).unwrap_or(&0.0));
    }

    #[test]
    fn test_pairwise_trust_direct() {
        let mut graph = TrustGraph::new();
        let engine = TrustCalculationEngine::new();
        let time_provider = FixedTimeProvider::new(1000);

        let alice = create_test_did("alice");
        let bob = create_test_did("bob");

        // Direct trust relationship
        graph.add_edge(TrustEdge::new(alice.clone(), bob.clone(), 0.8, 950));

        let trust = engine.calculate_pairwise_trust(&graph, &alice, &bob, &time_provider, 3);

        // Should be close to 0.8 with some time decay
        assert!(trust > 0.7 && trust <= 0.8);
    }

    #[test]
    fn test_pairwise_trust_indirect() {
        let mut graph = TrustGraph::new();
        let engine = TrustCalculationEngine::new();
        let time_provider = FixedTimeProvider::new(1000);

        let alice = create_test_did("alice");
        let bob = create_test_did("bob");
        let charlie = create_test_did("charlie");

        // Indirect trust: Alice -> Bob -> Charlie
        graph.add_edge(TrustEdge::new(alice.clone(), bob.clone(), 0.9, 950));
        graph.add_edge(TrustEdge::new(bob.clone(), charlie.clone(), 0.8, 960));

        let trust = engine.calculate_pairwise_trust(&graph, &alice, &charlie, &time_provider, 3);

        // Should be positive but less than direct trust due to path length and weakest link
        assert!(trust > 0.0);
        assert!(trust < 0.8); // Less than the minimum edge weight
    }

    #[test]
    fn test_time_decay() {
        let engine = TrustCalculationEngine::new();
        let alice = create_test_did("alice");
        let bob = create_test_did("bob");

        // Edge created 180 days ago (2 half-lives with 90-day half-life)
        let old_time = 1000;
        let current_time = old_time + 180 * 86400; // 180 days later
        let edge = TrustEdge::new(alice, bob, 0.8, old_time);

        let decayed_weight = engine.apply_time_decay(&edge, current_time);

        // After 2 half-lives, should be approximately 0.8 * 0.25 = 0.2
        assert!((decayed_weight - 0.2).abs() < 0.05);
    }

    #[test]
    fn test_empty_graph() {
        let graph = TrustGraph::new();
        let engine = TrustCalculationEngine::new();
        let time_provider = FixedTimeProvider::new(1000);

        let scores = engine.calculate_pagerank_scores(&graph, &time_provider);
        assert!(scores.is_empty());

        let weighted_scores = engine.calculate_weighted_trust_scores(&graph, &time_provider, 3);
        assert!(weighted_scores.is_empty());
    }

    #[test]
    fn test_trust_calculation_config() {
        let config = TrustCalculationConfig {
            damping_factor: 0.9,
            max_iterations: 50,
            convergence_threshold: 1e-4,
            min_score: 0.05,
            max_score: 0.95,
            direct_trust_weight: 0.8,
        };

        let engine = TrustCalculationEngine::with_config(config.clone());
        assert_eq!(engine.config.damping_factor, 0.9);
        assert_eq!(engine.config.max_iterations, 50);
        assert_eq!(engine.config.min_score, 0.05);
    }
}
