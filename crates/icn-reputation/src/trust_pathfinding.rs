//! Trust Path Discovery
//!
//! This module implements algorithms for finding optimal trust paths between
//! cooperatives through intermediaries, including shortest path and best trust path algorithms.

use crate::trust_graph::{TrustGraph, TrustEdge};
use icn_common::{Did, TimeProvider};
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};
use std::cmp::Ordering;

/// Represents a path through the trust graph
#[derive(Debug, Clone)]
pub struct TrustPath {
    /// The sequence of trust edges forming this path
    pub edges: Vec<TrustEdge>,
    /// The total trust score along this path
    pub trust_score: f64,
    /// The length of the path (number of hops)
    pub length: usize,
    /// The starting node of the path
    pub source: Did,
    /// The ending node of the path
    pub target: Did,
}

impl TrustPath {
    /// Create a new trust path
    pub fn new(source: Did, target: Did) -> Self {
        Self {
            edges: Vec::new(),
            trust_score: 1.0,
            length: 0,
            source,
            target,
        }
    }

    /// Add an edge to the path and update metrics
    pub fn add_edge(&mut self, edge: TrustEdge, decay_factor: f64) {
        // Trust score is minimum of all edges (weakest link principle)
        let edge_weight = edge.weight * decay_factor;
        if self.edges.is_empty() {
            self.trust_score = edge_weight;
        } else {
            self.trust_score = self.trust_score.min(edge_weight);
        }
        
        self.edges.push(edge);
        self.length += 1;
    }

    /// Get all intermediate nodes in the path (excluding source and target)
    pub fn get_intermediate_nodes(&self) -> Vec<Did> {
        if self.edges.is_empty() {
            return Vec::new();
        }

        let mut nodes = Vec::new();
        for i in 0..self.edges.len() - 1 {
            nodes.push(self.edges[i].to.clone());
        }
        nodes
    }

    /// Check if this path contains a specific node
    pub fn contains_node(&self, node: &Did) -> bool {
        if node == &self.source || node == &self.target {
            return true;
        }
        self.edges.iter().any(|edge| &edge.to == node)
    }

    /// Calculate effective trust score with distance penalty
    pub fn effective_trust_score(&self, distance_penalty: f64) -> f64 {
        if self.length == 0 {
            return 0.0;
        }
        self.trust_score * (distance_penalty.powi(self.length as i32 - 1))
    }
}

/// State for Dijkstra's shortest path algorithm
#[derive(Debug, Clone)]
struct PathState {
    node: Did,
    trust_score: f64,
    path: Vec<TrustEdge>,
    distance: usize,
}

impl Eq for PathState {}

impl PartialEq for PathState {
    fn eq(&self, other: &Self) -> bool {
        self.trust_score.eq(&other.trust_score)
    }
}

impl Ord for PathState {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for max-heap (we want highest trust scores first)
        other.trust_score.partial_cmp(&self.trust_score).unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for PathState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Configuration for trust path discovery algorithms
#[derive(Debug, Clone)]
pub struct PathDiscoveryConfig {
    /// Maximum path length to consider
    pub max_path_length: usize,
    /// Distance penalty factor (applied exponentially with path length)
    pub distance_penalty: f64,
    /// Time decay half-life in seconds
    pub time_decay_half_life: u64,
    /// Minimum trust threshold for considering a path
    pub min_trust_threshold: f64,
    /// Maximum number of paths to return
    pub max_paths: usize,
}

impl Default for PathDiscoveryConfig {
    fn default() -> Self {
        Self {
            max_path_length: 6, // Six degrees of separation
            distance_penalty: 0.8,
            time_decay_half_life: 90 * 24 * 3600, // 90 days
            min_trust_threshold: 0.01,
            max_paths: 10,
        }
    }
}

/// Engine for discovering trust paths between entities
pub struct TrustPathfinder {
    config: PathDiscoveryConfig,
}

impl TrustPathfinder {
    /// Create a new pathfinder with default configuration
    pub fn new() -> Self {
        Self {
            config: PathDiscoveryConfig::default(),
        }
    }

    /// Create a new pathfinder with custom configuration
    pub fn with_config(config: PathDiscoveryConfig) -> Self {
        Self { config }
    }

    /// Find the best trust path between source and target
    ///
    /// Uses a modified Dijkstra's algorithm that maximizes trust score instead of minimizing distance.
    pub fn find_best_path(
        &self,
        graph: &TrustGraph,
        source: &Did,
        target: &Did,
        time_provider: &dyn TimeProvider,
    ) -> Option<TrustPath> {
        if source == target {
            return Some(TrustPath::new(source.clone(), target.clone()));
        }

        let current_time = time_provider.unix_seconds();
        let mut visited = HashSet::new();
        let mut heap = BinaryHeap::new();

        // Initialize with source node
        heap.push(PathState {
            node: source.clone(),
            trust_score: 1.0,
            path: Vec::new(),
            distance: 0,
        });

        while let Some(current_state) = heap.pop() {
            if visited.contains(&current_state.node) {
                continue;
            }

            visited.insert(current_state.node.clone());

            // Check if we reached the target
            if current_state.node == *target {
                let mut path = TrustPath::new(source.clone(), target.clone());
                for edge in current_state.path {
                    let decay = self.calculate_time_decay(&edge, current_time);
                    path.add_edge(edge, decay);
                }
                return Some(path);
            }

            // Don't explore beyond max path length
            if current_state.distance >= self.config.max_path_length {
                continue;
            }

            // Explore neighbors
            if let Some(outgoing_edges) = graph.get_outgoing_edges(&current_state.node) {
                for (neighbor, edge) in outgoing_edges {
                    if visited.contains(neighbor) {
                        continue;
                    }

                    let decay = self.calculate_time_decay(edge, current_time);
                    let edge_weight = edge.weight * decay;
                    
                    // Calculate new trust score (minimum along path)
                    let new_trust_score = current_state.trust_score.min(edge_weight);
                    
                    // Apply distance penalty
                    let effective_trust = new_trust_score * self.config.distance_penalty.powi(current_state.distance as i32);
                    
                    if effective_trust >= self.config.min_trust_threshold {
                        let mut new_path = current_state.path.clone();
                        new_path.push(edge.clone());

                        heap.push(PathState {
                            node: neighbor.clone(),
                            trust_score: new_trust_score,
                            path: new_path,
                            distance: current_state.distance + 1,
                        });
                    }
                }
            }
        }

        None // No path found
    }

    /// Find multiple good trust paths between source and target
    ///
    /// Returns up to max_paths diverse paths, sorted by effective trust score.
    pub fn find_multiple_paths(
        &self,
        graph: &TrustGraph,
        source: &Did,
        target: &Did,
        time_provider: &dyn TimeProvider,
    ) -> Vec<TrustPath> {
        if source == target {
            return vec![TrustPath::new(source.clone(), target.clone())];
        }

        let current_time = time_provider.unix_seconds();
        let mut all_paths = Vec::new();
        let mut used_nodes = HashSet::new();

        // Find multiple paths by excluding previously used intermediate nodes
        for _iteration in 0..self.config.max_paths {
            if let Some(path) = self.find_path_avoiding_nodes(
                graph,
                source,
                target,
                &used_nodes,
                current_time,
            ) {
                // Add intermediate nodes to exclusion set for diversity
                for intermediate in path.get_intermediate_nodes() {
                    used_nodes.insert(intermediate);
                }
                all_paths.push(path);
            } else {
                break; // No more paths available
            }
        }

        // Sort by effective trust score (highest first)
        all_paths.sort_by(|a, b| {
            b.effective_trust_score(self.config.distance_penalty)
                .partial_cmp(&a.effective_trust_score(self.config.distance_penalty))
                .unwrap_or(Ordering::Equal)
        });

        all_paths
    }

    /// Find the shortest trust path (minimum number of hops) between source and target
    pub fn find_shortest_path(
        &self,
        graph: &TrustGraph,
        source: &Did,
        target: &Did,
        time_provider: &dyn TimeProvider,
    ) -> Option<TrustPath> {
        if source == target {
            return Some(TrustPath::new(source.clone(), target.clone()));
        }

        let current_time = time_provider.unix_seconds();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        // BFS to find shortest path
        queue.push_back((source.clone(), Vec::new(), 0));

        while let Some((current_node, current_path, distance)) = queue.pop_front() {
            if visited.contains(&current_node) {
                continue;
            }

            visited.insert(current_node.clone());

            if current_node == *target {
                let mut path = TrustPath::new(source.clone(), target.clone());
                for edge in current_path {
                    let decay = self.calculate_time_decay(&edge, current_time);
                    path.add_edge(edge, decay);
                }
                return Some(path);
            }

            if distance >= self.config.max_path_length {
                continue;
            }

            if let Some(outgoing_edges) = graph.get_outgoing_edges(&current_node) {
                for (neighbor, edge) in outgoing_edges {
                    if !visited.contains(neighbor) {
                        let decay = self.calculate_time_decay(edge, current_time);
                        let edge_weight = edge.weight * decay;
                        
                        if edge_weight >= self.config.min_trust_threshold {
                            let mut new_path = current_path.clone();
                            new_path.push(edge.clone());
                            queue.push_back((neighbor.clone(), new_path, distance + 1));
                        }
                    }
                }
            }
        }

        None
    }

    /// Find all reachable nodes from a source within a given trust threshold
    pub fn find_reachable_nodes(
        &self,
        graph: &TrustGraph,
        source: &Did,
        min_trust: f64,
        time_provider: &dyn TimeProvider,
    ) -> HashMap<Did, f64> {
        let current_time = time_provider.unix_seconds();
        let mut reachable = HashMap::new();
        let mut visited = HashSet::new();
        let mut heap = BinaryHeap::new();

        heap.push(PathState {
            node: source.clone(),
            trust_score: 1.0,
            path: Vec::new(),
            distance: 0,
        });

        while let Some(current_state) = heap.pop() {
            if visited.contains(&current_state.node) {
                continue;
            }

            visited.insert(current_state.node.clone());
            
            if current_state.trust_score >= min_trust {
                reachable.insert(current_state.node.clone(), current_state.trust_score);
            }

            if current_state.distance >= self.config.max_path_length {
                continue;
            }

            if let Some(outgoing_edges) = graph.get_outgoing_edges(&current_state.node) {
                for (neighbor, edge) in outgoing_edges {
                    if !visited.contains(neighbor) {
                        let decay = self.calculate_time_decay(edge, current_time);
                        let edge_weight = edge.weight * decay;
                        let new_trust_score = current_state.trust_score.min(edge_weight);
                        
                        if new_trust_score >= self.config.min_trust_threshold {
                            heap.push(PathState {
                                node: neighbor.clone(),
                                trust_score: new_trust_score,
                                path: Vec::new(), // We don't need the full path for this query
                                distance: current_state.distance + 1,
                            });
                        }
                    }
                }
            }
        }

        reachable
    }

    /// Calculate time-based decay factor for a trust edge
    fn calculate_time_decay(&self, edge: &TrustEdge, current_time: u64) -> f64 {
        let age_seconds = current_time.saturating_sub(edge.updated_at);
        let half_life = self.config.time_decay_half_life as f64;
        0.5_f64.powf(age_seconds as f64 / half_life)
    }

    /// Find a path while avoiding certain intermediate nodes
    fn find_path_avoiding_nodes(
        &self,
        graph: &TrustGraph,
        source: &Did,
        target: &Did,
        avoid_nodes: &HashSet<Did>,
        current_time: u64,
    ) -> Option<TrustPath> {
        let mut visited = HashSet::new();
        let mut heap = BinaryHeap::new();

        heap.push(PathState {
            node: source.clone(),
            trust_score: 1.0,
            path: Vec::new(),
            distance: 0,
        });

        while let Some(current_state) = heap.pop() {
            if visited.contains(&current_state.node) {
                continue;
            }

            visited.insert(current_state.node.clone());

            if current_state.node == *target {
                let mut path = TrustPath::new(source.clone(), target.clone());
                for edge in current_state.path {
                    let decay = self.calculate_time_decay(&edge, current_time);
                    path.add_edge(edge, decay);
                }
                return Some(path);
            }

            if current_state.distance >= self.config.max_path_length {
                continue;
            }

            if let Some(outgoing_edges) = graph.get_outgoing_edges(&current_state.node) {
                for (neighbor, edge) in outgoing_edges {
                    // Skip if neighbor is in avoid set (unless it's the target)
                    if neighbor != target && avoid_nodes.contains(neighbor) {
                        continue;
                    }

                    if !visited.contains(neighbor) {
                        let decay = self.calculate_time_decay(edge, current_time);
                        let edge_weight = edge.weight * decay;
                        let new_trust_score = current_state.trust_score.min(edge_weight);
                        
                        if new_trust_score >= self.config.min_trust_threshold {
                            let mut new_path = current_state.path.clone();
                            new_path.push(edge.clone());

                            heap.push(PathState {
                                node: neighbor.clone(),
                                trust_score: new_trust_score,
                                path: new_path,
                                distance: current_state.distance + 1,
                            });
                        }
                    }
                }
            }
        }

        None
    }
}

impl Default for TrustPathfinder {
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
    fn test_trust_path_creation() {
        let alice = create_test_did("alice");
        let bob = create_test_did("bob");
        let mut path = TrustPath::new(alice.clone(), bob.clone());

        assert_eq!(path.source, alice);
        assert_eq!(path.target, bob);
        assert_eq!(path.length, 0);
        assert_eq!(path.trust_score, 1.0);

        let edge = TrustEdge::new(alice.clone(), bob.clone(), 0.8, 1000);
        path.add_edge(edge, 1.0);

        assert_eq!(path.length, 1);
        assert_eq!(path.trust_score, 0.8);
    }

    #[test]
    fn test_find_best_path_direct() {
        let mut graph = TrustGraph::new();
        let pathfinder = TrustPathfinder::new();
        let time_provider = FixedTimeProvider::new(1000);

        let alice = create_test_did("alice");
        let bob = create_test_did("bob");

        // Direct edge
        graph.add_edge(TrustEdge::new(alice.clone(), bob.clone(), 0.9, 950));

        let path = pathfinder.find_best_path(&graph, &alice, &bob, &time_provider);
        assert!(path.is_some());

        let path = path.unwrap();
        assert_eq!(path.length, 1);
        assert_eq!(path.source, alice);
        assert_eq!(path.target, bob);
        assert!(path.trust_score > 0.8); // Should be close to 0.9 with minimal decay
    }

    #[test]
    fn test_find_best_path_indirect() {
        let mut graph = TrustGraph::new();
        let pathfinder = TrustPathfinder::new();
        let time_provider = FixedTimeProvider::new(1000);

        let alice = create_test_did("alice");
        let bob = create_test_did("bob");
        let charlie = create_test_did("charlie");

        // Indirect path: Alice -> Bob -> Charlie
        graph.add_edge(TrustEdge::new(alice.clone(), bob.clone(), 0.9, 950));
        graph.add_edge(TrustEdge::new(bob.clone(), charlie.clone(), 0.8, 960));

        let path = pathfinder.find_best_path(&graph, &alice, &charlie, &time_provider);
        assert!(path.is_some());

        let path = path.unwrap();
        assert_eq!(path.length, 2);
        assert_eq!(path.source, alice);
        assert_eq!(path.target, charlie);
        // Trust score should be limited by weakest link (0.8)
        assert!(path.trust_score <= 0.8);
    }

    #[test]
    fn test_find_shortest_path() {
        let mut graph = TrustGraph::new();
        let pathfinder = TrustPathfinder::new();
        let time_provider = FixedTimeProvider::new(1000);

        let alice = create_test_did("alice");
        let bob = create_test_did("bob");
        let charlie = create_test_did("charlie");
        let david = create_test_did("david");

        // Create two paths: Alice -> Bob -> David (length 2) and Alice -> Charlie -> David (length 2)
        // And a longer path: Alice -> Bob -> Charlie -> David (length 3)
        graph.add_edge(TrustEdge::new(alice.clone(), bob.clone(), 0.9, 950));
        graph.add_edge(TrustEdge::new(alice.clone(), charlie.clone(), 0.7, 950));
        graph.add_edge(TrustEdge::new(bob.clone(), david.clone(), 0.8, 960));
        graph.add_edge(TrustEdge::new(charlie.clone(), david.clone(), 0.9, 960));
        graph.add_edge(TrustEdge::new(bob.clone(), charlie.clone(), 0.6, 960));

        let path = pathfinder.find_shortest_path(&graph, &alice, &david, &time_provider);
        assert!(path.is_some());

        let path = path.unwrap();
        assert_eq!(path.length, 2); // Should find one of the 2-hop paths
    }

    #[test]
    fn test_find_multiple_paths() {
        let mut graph = TrustGraph::new();
        let pathfinder = TrustPathfinder::new();
        let time_provider = FixedTimeProvider::new(1000);

        let alice = create_test_did("alice");
        let bob = create_test_did("bob");
        let charlie = create_test_did("charlie");
        let david = create_test_did("david");

        // Create multiple paths from Alice to David
        graph.add_edge(TrustEdge::new(alice.clone(), bob.clone(), 0.9, 950));
        graph.add_edge(TrustEdge::new(alice.clone(), charlie.clone(), 0.8, 950));
        graph.add_edge(TrustEdge::new(bob.clone(), david.clone(), 0.8, 960));
        graph.add_edge(TrustEdge::new(charlie.clone(), david.clone(), 0.9, 960));

        let paths = pathfinder.find_multiple_paths(&graph, &alice, &david, &time_provider);
        
        // Should find both 2-hop paths
        assert!(paths.len() >= 2);
        assert!(paths.iter().all(|p| p.length == 2));
    }

    #[test]
    fn test_find_reachable_nodes() {
        let mut graph = TrustGraph::new();
        let pathfinder = TrustPathfinder::new();
        let time_provider = FixedTimeProvider::new(1000);

        let alice = create_test_did("alice");
        let bob = create_test_did("bob");
        let charlie = create_test_did("charlie");
        let david = create_test_did("david");

        // Alice can reach Bob, Bob can reach Charlie, Charlie can reach David
        graph.add_edge(TrustEdge::new(alice.clone(), bob.clone(), 0.9, 950));
        graph.add_edge(TrustEdge::new(bob.clone(), charlie.clone(), 0.8, 960));
        graph.add_edge(TrustEdge::new(charlie.clone(), david.clone(), 0.7, 970));

        let reachable = pathfinder.find_reachable_nodes(&graph, &alice, 0.6, &time_provider);

        // Alice should be able to reach Bob and Charlie with sufficient trust
        assert!(reachable.contains_key(&alice)); // Alice reaches herself with trust 1.0
        assert!(reachable.contains_key(&bob));
        assert!(reachable.contains_key(&charlie));
        // David might not be reachable if trust drops below threshold due to weakest link
    }

    #[test]
    fn test_no_path_exists() {
        let mut graph = TrustGraph::new();
        let pathfinder = TrustPathfinder::new();
        let time_provider = FixedTimeProvider::new(1000);

        let alice = create_test_did("alice");
        let bob = create_test_did("bob");
        let charlie = create_test_did("charlie");

        // Alice -> Bob, but no path to Charlie
        graph.add_edge(TrustEdge::new(alice.clone(), bob.clone(), 0.9, 950));

        let path = pathfinder.find_best_path(&graph, &alice, &charlie, &time_provider);
        assert!(path.is_none());
    }

    #[test]
    fn test_same_source_target() {
        let pathfinder = TrustPathfinder::new();
        let time_provider = FixedTimeProvider::new(1000);
        let graph = TrustGraph::new();

        let alice = create_test_did("alice");

        let path = pathfinder.find_best_path(&graph, &alice, &alice, &time_provider);
        assert!(path.is_some());

        let path = path.unwrap();
        assert_eq!(path.length, 0);
        assert_eq!(path.source, alice);
        assert_eq!(path.target, alice);
    }

    #[test]
    fn test_trust_path_effective_score() {
        let alice = create_test_did("alice");
        let bob = create_test_did("bob");
        let mut path = TrustPath::new(alice.clone(), bob.clone());

        let edge = TrustEdge::new(alice, bob, 0.8, 1000);
        path.add_edge(edge, 1.0);

        // With no distance penalty (penalty = 1.0)
        assert_eq!(path.effective_trust_score(1.0), 0.8);

        // With distance penalty (penalty = 0.8)
        assert_eq!(path.effective_trust_score(0.8), 0.8); // No penalty for length 1

        // Add another edge to test distance penalty
        let charlie = create_test_did("charlie");
        let edge2 = TrustEdge::new(bob.clone(), charlie, 0.9, 1000);
        path.add_edge(edge2, 1.0);

        // Now length is 2, so penalty should apply
        let expected = 0.8 * 0.8; // trust_score * penalty^(length-1)
        assert_eq!(path.effective_trust_score(0.8), expected);
    }
}