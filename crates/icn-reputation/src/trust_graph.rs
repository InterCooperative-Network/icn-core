//! Trust Graph Data Structures and Core Types
//!
//! This module provides the foundational data structures for representing
//! trust relationships between cooperatives in the ICN network.

use icn_common::{Did, TimeProvider};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a trust relationship between two cooperatives/entities
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrustEdge {
    /// The entity extending trust (source)
    pub from: Did,
    /// The entity receiving trust (target)  
    pub to: Did,
    /// Raw trust score (0.0 to 1.0)
    pub weight: f64,
    /// Unix timestamp when this trust relationship was established
    pub created_at: u64,
    /// Unix timestamp when this trust relationship was last updated
    pub updated_at: u64,
    /// Number of successful interactions that contributed to this trust
    pub interaction_count: u64,
    /// Optional metadata about the trust relationship
    pub metadata: Option<HashMap<String, String>>,
}

impl TrustEdge {
    /// Create a new trust edge with the given parameters
    pub fn new(from: Did, to: Did, weight: f64, timestamp: u64) -> Self {
        Self {
            from,
            to,
            weight: weight.clamp(0.0, 1.0), // Ensure weight is in valid range
            created_at: timestamp,
            updated_at: timestamp,
            interaction_count: 0,
            metadata: None,
        }
    }

    /// Update the trust weight and timestamp
    pub fn update_weight(&mut self, new_weight: f64, timestamp: u64) {
        self.weight = new_weight.clamp(0.0, 1.0);
        self.updated_at = timestamp;
    }

    /// Increment the interaction count
    pub fn increment_interactions(&mut self) {
        self.interaction_count = self.interaction_count.saturating_add(1);
    }

    /// Check if this trust edge has expired based on a time threshold
    pub fn is_expired(&self, current_time: u64, max_age_seconds: u64) -> bool {
        current_time.saturating_sub(self.updated_at) > max_age_seconds
    }
}

/// Core trust graph structure representing the network of trust relationships
#[derive(Debug, Clone, Default)]
pub struct TrustGraph {
    /// Adjacency list representation: from_did -> (to_did -> trust_edge)
    edges: HashMap<Did, HashMap<Did, TrustEdge>>,
    /// Reverse index for efficient incoming edge queries
    incoming_edges: HashMap<Did, HashMap<Did, TrustEdge>>,
    /// Cached trust scores for performance (did -> score)
    cached_scores: HashMap<Did, f64>,
    /// Timestamp when scores were last computed
    last_score_update: u64,
}

impl TrustGraph {
    /// Create a new empty trust graph
    pub fn new() -> Self {
        Self::default()
    }

    /// Add or update a trust edge in the graph
    pub fn add_edge(&mut self, edge: TrustEdge) {
        let from = edge.from.clone();
        let to = edge.to.clone();

        // Add to forward adjacency list
        self.edges
            .entry(from.clone())
            .or_default()
            .insert(to.clone(), edge.clone());

        // Add to reverse adjacency list
        self.incoming_edges
            .entry(to)
            .or_default()
            .insert(from, edge);

        // Invalidate cached scores since graph structure changed
        self.cached_scores.clear();
    }

    /// Remove a trust edge from the graph
    pub fn remove_edge(&mut self, from: &Did, to: &Did) -> Option<TrustEdge> {
        let removed = self.edges.get_mut(from)?.remove(to);

        if removed.is_some() {
            // Also remove from incoming edges
            if let Some(incoming) = self.incoming_edges.get_mut(to) {
                incoming.remove(from);
            }

            // Invalidate cached scores
            self.cached_scores.clear();
        }

        removed
    }

    /// Get a trust edge between two entities
    pub fn get_edge(&self, from: &Did, to: &Did) -> Option<&TrustEdge> {
        self.edges.get(from)?.get(to)
    }

    /// Get all outgoing edges from an entity
    pub fn get_outgoing_edges(&self, from: &Did) -> Option<&HashMap<Did, TrustEdge>> {
        self.edges.get(from)
    }

    /// Get all incoming edges to an entity
    pub fn get_incoming_edges(&self, to: &Did) -> Option<&HashMap<Did, TrustEdge>> {
        self.incoming_edges.get(to)
    }

    /// Get all entities (DIDs) in the graph
    pub fn get_all_nodes(&self) -> Vec<Did> {
        let mut nodes = std::collections::HashSet::new();

        for (from, edges) in &self.edges {
            nodes.insert(from.clone());
            for to in edges.keys() {
                nodes.insert(to.clone());
            }
        }

        nodes.into_iter().collect()
    }

    /// Get the number of edges in the graph
    pub fn edge_count(&self) -> usize {
        self.edges.values().map(|edges| edges.len()).sum()
    }

    /// Get the number of nodes in the graph
    pub fn node_count(&self) -> usize {
        self.get_all_nodes().len()
    }

    /// Clean up expired edges based on a time provider
    pub fn cleanup_expired_edges(
        &mut self,
        time_provider: &dyn TimeProvider,
        max_age_seconds: u64,
    ) {
        let current_time = time_provider.unix_seconds();
        let mut edges_to_remove = Vec::new();

        // Collect expired edges
        for (from, edges) in &self.edges {
            for (to, edge) in edges {
                if edge.is_expired(current_time, max_age_seconds) {
                    edges_to_remove.push((from.clone(), to.clone()));
                }
            }
        }

        // Remove expired edges
        for (from, to) in edges_to_remove {
            self.remove_edge(&from, &to);
        }
    }

    /// Clear all cached scores
    pub fn invalidate_cache(&mut self) {
        self.cached_scores.clear();
        self.last_score_update = 0;
    }

    /// Get a cached trust score if available and not stale
    pub fn get_cached_score(
        &self,
        did: &Did,
        max_cache_age: u64,
        current_time: u64,
    ) -> Option<f64> {
        if current_time.saturating_sub(self.last_score_update) > max_cache_age {
            return None;
        }
        self.cached_scores.get(did).copied()
    }

    /// Cache a computed trust score
    pub fn cache_score(&mut self, did: Did, score: f64, timestamp: u64) {
        self.cached_scores.insert(did, score);
        self.last_score_update = timestamp;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::FixedTimeProvider;
    use std::str::FromStr;

    fn create_test_did(id: &str) -> Did {
        Did::from_str(&format!("did:test:{}", id)).unwrap()
    }

    #[test]
    fn test_trust_edge_creation() {
        let from = create_test_did("alice");
        let to = create_test_did("bob");
        let edge = TrustEdge::new(from.clone(), to.clone(), 0.8, 1000);

        assert_eq!(edge.from, from);
        assert_eq!(edge.to, to);
        assert_eq!(edge.weight, 0.8);
        assert_eq!(edge.created_at, 1000);
        assert_eq!(edge.updated_at, 1000);
        assert_eq!(edge.interaction_count, 0);
    }

    #[test]
    fn test_trust_edge_weight_clamping() {
        let from = create_test_did("alice");
        let to = create_test_did("bob");

        // Test upper bound clamping
        let edge1 = TrustEdge::new(from.clone(), to.clone(), 1.5, 1000);
        assert_eq!(edge1.weight, 1.0);

        // Test lower bound clamping
        let edge2 = TrustEdge::new(from, to, -0.5, 1000);
        assert_eq!(edge2.weight, 0.0);
    }

    #[test]
    fn test_trust_edge_expiration() {
        let from = create_test_did("alice");
        let to = create_test_did("bob");
        let edge = TrustEdge::new(from, to, 0.8, 1000);

        // Not expired within threshold
        assert!(!edge.is_expired(1500, 1000));

        // Expired beyond threshold
        assert!(edge.is_expired(2500, 1000));
    }

    #[test]
    fn test_trust_graph_basic_operations() {
        let mut graph = TrustGraph::new();
        let alice = create_test_did("alice");
        let bob = create_test_did("bob");
        let charlie = create_test_did("charlie");

        // Add edges
        let edge1 = TrustEdge::new(alice.clone(), bob.clone(), 0.8, 1000);
        let edge2 = TrustEdge::new(bob.clone(), charlie.clone(), 0.9, 1000);

        graph.add_edge(edge1.clone());
        graph.add_edge(edge2.clone());

        assert_eq!(graph.edge_count(), 2);
        assert_eq!(graph.node_count(), 3);

        // Test edge retrieval
        let retrieved_edge = graph.get_edge(&alice, &bob).unwrap();
        assert_eq!(retrieved_edge.weight, 0.8);

        // Test outgoing edges
        let outgoing = graph.get_outgoing_edges(&alice).unwrap();
        assert_eq!(outgoing.len(), 1);
        assert!(outgoing.contains_key(&bob));

        // Test incoming edges
        let incoming = graph.get_incoming_edges(&bob).unwrap();
        assert_eq!(incoming.len(), 1);
        assert!(incoming.contains_key(&alice));
    }

    #[test]
    fn test_trust_graph_edge_removal() {
        let mut graph = TrustGraph::new();
        let alice = create_test_did("alice");
        let bob = create_test_did("bob");

        let edge = TrustEdge::new(alice.clone(), bob.clone(), 0.8, 1000);
        graph.add_edge(edge);

        assert_eq!(graph.edge_count(), 1);

        // Remove edge
        let removed = graph.remove_edge(&alice, &bob);
        assert!(removed.is_some());
        assert_eq!(graph.edge_count(), 0);

        // Verify edge is gone
        assert!(graph.get_edge(&alice, &bob).is_none());
    }

    #[test]
    fn test_trust_graph_cleanup_expired_edges() {
        let mut graph = TrustGraph::new();
        let time_provider = FixedTimeProvider::new(2000);
        let alice = create_test_did("alice");
        let bob = create_test_did("bob");
        let charlie = create_test_did("charlie");

        // Add edges with different timestamps
        let old_edge = TrustEdge::new(alice.clone(), bob.clone(), 0.8, 500); // Old
        let new_edge = TrustEdge::new(bob.clone(), charlie.clone(), 0.9, 1800); // Recent

        graph.add_edge(old_edge);
        graph.add_edge(new_edge);

        assert_eq!(graph.edge_count(), 2);

        // Cleanup with 1000 second threshold
        graph.cleanup_expired_edges(&time_provider, 1000);

        // Old edge should be removed, new edge should remain
        assert_eq!(graph.edge_count(), 1);
        assert!(graph.get_edge(&alice, &bob).is_none());
        assert!(graph.get_edge(&bob, &charlie).is_some());
    }

    #[test]
    fn test_trust_graph_cache_operations() {
        let mut graph = TrustGraph::new();
        let alice = create_test_did("alice");

        // Initially no cached score
        assert!(graph.get_cached_score(&alice, 1000, 2000).is_none());

        // Cache a score
        graph.cache_score(alice.clone(), 0.85, 2000);

        // Retrieve cached score (not stale)
        let cached = graph.get_cached_score(&alice, 1000, 2500);
        assert_eq!(cached, Some(0.85));

        // Cache should be stale now
        let stale = graph.get_cached_score(&alice, 1000, 4000);
        assert!(stale.is_none());
    }
}
