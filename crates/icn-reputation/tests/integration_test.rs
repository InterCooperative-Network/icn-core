//! Comprehensive Integration Tests for Trust Graph System
//!
//! These tests validate the complete trust graph implementation including
//! all major components: graph structure, calculation algorithms, pathfinding,
//! decay models, and aggregation.

use icn_common::{Did, FixedTimeProvider, TimeProvider};
use icn_reputation::{
    TrustGraph, TrustEdge, TrustCalculationEngine, TrustPathfinder,
    TrustDecayCalculator, TrustAggregator, DecayModel, TrustSignal,
    AggregationConfig, CombinationMethod, PathDiscoveryConfig,
    DistanceDecayConfig, InteractionDecayConfig,
};
use std::str::FromStr;
use std::collections::HashMap;

fn create_test_did(id: &str) -> Did {
    Did::from_str(&format!("did:test:{}", id)).unwrap()
}

fn setup_test_network() -> (TrustGraph, Vec<Did>) {
    let mut graph = TrustGraph::new();
    let alice = create_test_did("alice");
    let bob = create_test_did("bob");
    let charlie = create_test_did("charlie");
    let david = create_test_did("david");
    let eve = create_test_did("eve");

    // Create a more complex network topology
    // Alice: central hub with high trust relationships
    graph.add_edge(TrustEdge::new(alice.clone(), bob.clone(), 0.9, 1000));
    graph.add_edge(TrustEdge::new(alice.clone(), charlie.clone(), 0.85, 1050));
    graph.add_edge(TrustEdge::new(alice.clone(), david.clone(), 0.8, 1100));

    // Bob: well-connected secondary hub
    graph.add_edge(TrustEdge::new(bob.clone(), charlie.clone(), 0.75, 1150));
    graph.add_edge(TrustEdge::new(bob.clone(), david.clone(), 0.7, 1200));
    graph.add_edge(TrustEdge::new(bob.clone(), eve.clone(), 0.65, 1250));

    // Charlie: bridge between different clusters
    graph.add_edge(TrustEdge::new(charlie.clone(), david.clone(), 0.8, 1300));
    graph.add_edge(TrustEdge::new(charlie.clone(), eve.clone(), 0.7, 1350));

    // David: trusts back to create cycles
    graph.add_edge(TrustEdge::new(david.clone(), alice.clone(), 0.6, 1400));
    graph.add_edge(TrustEdge::new(david.clone(), eve.clone(), 0.75, 1450));

    // Eve: peripheral node with some connections
    graph.add_edge(TrustEdge::new(eve.clone(), alice.clone(), 0.55, 1500));

    let nodes = vec![alice, bob, charlie, david, eve];
    (graph, nodes)
}

#[test]
fn test_trust_graph_basic_operations() {
    let (mut graph, nodes) = setup_test_network();
    
    // Test basic graph properties
    assert_eq!(graph.node_count(), 5);
    assert_eq!(graph.edge_count(), 11);
    
    // Test edge retrieval
    let alice = &nodes[0];
    let bob = &nodes[1];
    let edge = graph.get_edge(alice, bob).unwrap();
    assert_eq!(edge.weight, 0.9);
    
    // Test outgoing edges
    let outgoing = graph.get_outgoing_edges(alice).unwrap();
    assert_eq!(outgoing.len(), 3);
    
    // Test incoming edges
    let incoming = graph.get_incoming_edges(alice).unwrap();
    assert_eq!(incoming.len(), 2); // David and Eve trust Alice
    
    // Test edge removal
    let removed = graph.remove_edge(alice, bob);
    assert!(removed.is_some());
    assert_eq!(graph.edge_count(), 10);
    
    // Test edge not found after removal
    assert!(graph.get_edge(alice, bob).is_none());
}

#[test]
fn test_pagerank_calculation() {
    let (graph, nodes) = setup_test_network();
    let engine = TrustCalculationEngine::new();
    let time_provider = FixedTimeProvider::new(2000);
    
    let scores = engine.calculate_pagerank_scores(&graph, &time_provider);
    
    // Verify all nodes have scores
    assert_eq!(scores.len(), 5);
    
    // Verify scores are normalized (sum ≈ 1.0)
    let total: f64 = scores.values().sum();
    assert!((total - 1.0).abs() < 0.1);
    
    // Alice should have high score due to central position
    let alice_score = scores.get(&nodes[0]).unwrap();
    let eve_score = scores.get(&nodes[4]).unwrap();
    assert!(alice_score > eve_score);
    
    // All scores should be positive
    for score in scores.values() {
        assert!(*score > 0.0);
    }
}

#[test]
fn test_weighted_trust_calculation() {
    let (graph, nodes) = setup_test_network();
    let engine = TrustCalculationEngine::new();
    let time_provider = FixedTimeProvider::new(2000);
    
    let scores = engine.calculate_weighted_trust_scores(&graph, &time_provider, 4);
    
    // Verify scores for all nodes
    assert_eq!(scores.len(), 5);
    
    // Alice should have high score due to many incoming edges
    let alice = &nodes[0];
    let eve = &nodes[4];
    let alice_score = scores.get(alice).unwrap();
    let eve_score = scores.get(eve).unwrap();
    assert!(alice_score > eve_score);
    
    // All scores should be in valid range
    for score in scores.values() {
        assert!(*score >= 0.0 && *score <= 1.0);
    }
}

#[test]
fn test_trust_pathfinding() {
    let (graph, nodes) = setup_test_network();
    let pathfinder = TrustPathfinder::new();
    let time_provider = FixedTimeProvider::new(2000);
    
    let alice = &nodes[0];
    let eve = &nodes[4];
    
    // Test best path finding
    let best_path = pathfinder.find_best_path(&graph, alice, eve, &time_provider);
    assert!(best_path.is_some());
    
    let path = best_path.unwrap();
    assert!(path.length > 0);
    assert!(path.trust_score > 0.0);
    assert_eq!(path.source, *alice);
    assert_eq!(path.target, *eve);
    
    // Test shortest path
    let shortest = pathfinder.find_shortest_path(&graph, alice, eve, &time_provider);
    assert!(shortest.is_some());
    
    // Test multiple paths
    let multiple = pathfinder.find_multiple_paths(&graph, alice, eve, &time_provider);
    assert!(!multiple.is_empty());
    
    // Paths should be sorted by trust score (highest first)
    for i in 1..multiple.len() {
        let prev_effective = multiple[i-1].effective_trust_score(0.8);
        let curr_effective = multiple[i].effective_trust_score(0.8);
        assert!(prev_effective >= curr_effective);
    }
    
    // Test reachability
    let reachable = pathfinder.find_reachable_nodes(&graph, alice, 0.3, &time_provider);
    assert!(reachable.len() >= 2); // At least Alice herself and some others
    assert!(reachable.contains_key(alice));
}

#[test]
fn test_trust_decay_models() {
    let alice = create_test_did("alice");
    let bob = create_test_did("bob");
    let old_time = 1000;
    let current_time = old_time + 90 * 24 * 3600; // 90 days later
    
    // Test exponential decay
    let exp_calculator = TrustDecayCalculator::with_time_decay(
        DecayModel::Exponential { half_life_seconds: 90 * 24 * 3600 }
    );
    let edge = TrustEdge::new(alice.clone(), bob.clone(), 1.0, old_time);
    let exp_decay = exp_calculator.calculate_time_decay(&edge, current_time);
    assert!((exp_decay - 0.5).abs() < 0.01); // Should be ~0.5 after 1 half-life
    
    // Test linear decay
    let lin_calculator = TrustDecayCalculator::with_time_decay(
        DecayModel::Linear { decay_period_seconds: 90 * 24 * 3600 }
    );
    let lin_decay = lin_calculator.calculate_time_decay(&edge, current_time);
    assert!(lin_decay < 0.01); // Should be near 0 after full period
    
    // Test distance decay
    let distance_decay_1 = exp_calculator.calculate_distance_decay(1);
    let distance_decay_3 = exp_calculator.calculate_distance_decay(3);
    assert!(distance_decay_1 > distance_decay_3);
    
    // Test interaction decay/boost
    let mut interactive_edge = TrustEdge::new(alice.clone(), bob.clone(), 1.0, old_time);
    interactive_edge.interaction_count = 50;
    let interaction_factor = exp_calculator.calculate_interaction_decay(&interactive_edge, old_time + 3600);
    assert!(interaction_factor > 1.0); // Should have boost for interactions
    
    // Test combined decay
    let combined = exp_calculator.calculate_combined_decay(&interactive_edge, 2, old_time + 3600);
    assert!(combined > 0.0 && combined <= 1.0);
}

#[test]
fn test_trust_aggregation() {
    let aggregator = TrustAggregator::new();
    let time_provider = FixedTimeProvider::new(2000);
    let alice = create_test_did("alice");
    let bob = create_test_did("bob");
    
    // Create diverse trust signals
    let signals = vec![
        TrustSignal::DirectTrust {
            from: alice.clone(),
            to: bob.clone(),
            weight: 0.85,
        },
        TrustSignal::ReputationTrust {
            entity: bob.clone(),
            score: 0.9,
        },
        TrustSignal::ActivityTrust {
            entity: bob.clone(),
            interaction_count: 25,
            recency_score: 0.8,
        },
        TrustSignal::NetworkTrust {
            entity: bob.clone(),
            centrality_score: 0.7,
        },
        TrustSignal::PerformanceTrust {
            entity: bob.clone(),
            success_rate: 0.95,
            sample_size: 100,
        },
    ];
    
    let result = aggregator.aggregate_trust_for_entity(&bob, signals, &time_provider);
    
    // Verify aggregation result
    assert_eq!(result.entity, bob);
    assert!(result.trust_score > 0.0 && result.trust_score <= 1.0);
    assert!(result.confidence > 0.0 && result.confidence <= 1.0);
    assert_eq!(result.signal_count, 5);
    assert!(!result.signal_breakdown.is_empty());
    assert!(result.signal_variance >= 0.0);
    assert_eq!(result.timestamp, 2000);
    
    // Test with insufficient signals
    let few_signals = vec![
        TrustSignal::DirectTrust {
            from: alice.clone(),
            to: bob.clone(),
            weight: 0.8,
        },
    ];
    
    let low_confidence_result = aggregator.aggregate_trust_for_entity(&bob, few_signals, &time_provider);
    assert!(low_confidence_result.confidence < result.confidence);
}

#[test]
fn test_aggregation_combination_methods() {
    let alice = create_test_did("alice");
    let bob = create_test_did("bob");
    let time_provider = FixedTimeProvider::new(2000);
    
    let signals = vec![
        TrustSignal::DirectTrust { from: alice.clone(), to: bob.clone(), weight: 0.8 },
        TrustSignal::ReputationTrust { entity: bob.clone(), score: 0.9 },
    ];
    
    // Test weighted mean
    let mean_config = AggregationConfig {
        combination_method: CombinationMethod::WeightedMean,
        ..Default::default()
    };
    let mean_aggregator = TrustAggregator::with_config(mean_config);
    let mean_result = mean_aggregator.aggregate_trust_for_entity(&bob, signals.clone(), &time_provider);
    
    // Test geometric mean
    let geom_config = AggregationConfig {
        combination_method: CombinationMethod::WeightedGeometricMean,
        ..Default::default()
    };
    let geom_aggregator = TrustAggregator::with_config(geom_config);
    let geom_result = geom_aggregator.aggregate_trust_for_entity(&bob, signals.clone(), &time_provider);
    
    // Geometric mean should be lower than arithmetic mean for these values
    assert!(geom_result.trust_score < mean_result.trust_score);
    
    // Test custom combination
    let custom_config = AggregationConfig {
        combination_method: CombinationMethod::Custom { alpha: 2.0, beta: 1.0 },
        ..Default::default()
    };
    let custom_aggregator = TrustAggregator::with_config(custom_config);
    let custom_result = custom_aggregator.aggregate_trust_for_entity(&bob, signals, &time_provider);
    
    // All should produce valid scores
    assert!(mean_result.trust_score > 0.0 && mean_result.trust_score <= 1.0);
    assert!(geom_result.trust_score > 0.0 && geom_result.trust_score <= 1.0);
    assert!(custom_result.trust_score > 0.0 && custom_result.trust_score <= 1.0);
}

#[test]
fn test_decay_graph_cleanup() {
    let mut graph = TrustGraph::new();
    let decay_calculator = TrustDecayCalculator::with_time_decay(
        DecayModel::Linear { decay_period_seconds: 86400 } // 1 day
    );
    let time_provider = FixedTimeProvider::new(2000 + 2 * 86400); // 2 days later
    
    let alice = create_test_did("alice");
    let bob = create_test_did("bob");
    let charlie = create_test_did("charlie");
    
    // Add edges with different ages
    graph.add_edge(TrustEdge::new(alice.clone(), bob.clone(), 1.0, 2000)); // 2 days old
    graph.add_edge(TrustEdge::new(bob.clone(), charlie.clone(), 1.0, 2000 + 43200)); // 1.5 days old
    
    assert_eq!(graph.edge_count(), 2);
    
    // Apply decay cleanup (both edges should be removed due to linear decay)
    let removed_count = decay_calculator.apply_decay_to_graph(&mut graph, &time_provider, 0.1);
    
    assert_eq!(removed_count, 2);
    assert_eq!(graph.edge_count(), 0);
}

#[test]
fn test_pairwise_trust_calculation() {
    let (graph, nodes) = setup_test_network();
    let engine = TrustCalculationEngine::new();
    let time_provider = FixedTimeProvider::new(2000);
    
    let alice = &nodes[0];
    let bob = &nodes[1];
    let eve = &nodes[4];
    
    // Direct trust (Alice -> Bob)
    let direct_trust = engine.calculate_pairwise_trust(&graph, alice, bob, &time_provider, 3);
    assert!(direct_trust > 0.8); // Should be close to original 0.9 with minimal decay
    
    // Indirect trust (Alice -> Eve)
    let indirect_trust = engine.calculate_pairwise_trust(&graph, alice, eve, &time_provider, 3);
    assert!(indirect_trust > 0.0);
    assert!(indirect_trust < direct_trust); // Indirect should be less than direct
    
    // No path (if we create isolated nodes)
    let isolated = create_test_did("isolated");
    let no_trust = engine.calculate_pairwise_trust(&graph, alice, &isolated, &time_provider, 3);
    assert_eq!(no_trust, 0.0);
}

#[test]
fn test_trust_path_properties() {
    let alice = create_test_did("alice");
    let bob = create_test_did("bob");
    let charlie = create_test_did("charlie");
    
    let mut path = icn_reputation::TrustPath::new(alice.clone(), charlie.clone());
    
    // Add edges to create Alice -> Bob -> Charlie path
    let edge1 = TrustEdge::new(alice.clone(), bob.clone(), 0.8, 1000);
    let edge2 = TrustEdge::new(bob.clone(), charlie.clone(), 0.9, 1000);
    
    path.add_edge(edge1, 1.0);
    path.add_edge(edge2, 1.0);
    
    // Test path properties
    assert_eq!(path.length, 2);
    assert_eq!(path.trust_score, 0.8); // Minimum of the two edges
    assert_eq!(path.source, alice);
    assert_eq!(path.target, charlie);
    
    // Test intermediate nodes
    let intermediates = path.get_intermediate_nodes();
    assert_eq!(intermediates.len(), 1);
    assert_eq!(intermediates[0], bob);
    
    // Test node containment
    assert!(path.contains_node(&alice));
    assert!(path.contains_node(&bob));
    assert!(path.contains_node(&charlie));
    
    let david = create_test_did("david");
    assert!(!path.contains_node(&david));
    
    // Test effective trust score with distance penalty
    let effective = path.effective_trust_score(0.8);
    let expected = 0.8 * 0.8; // trust_score * penalty^(length-1)
    assert!((effective - expected).abs() < 1e-6);
}

#[test]
fn test_graph_expired_edge_cleanup() {
    let mut graph = TrustGraph::new();
    let time_provider = FixedTimeProvider::new(2000);
    
    let alice = create_test_did("alice");
    let bob = create_test_did("bob");
    let charlie = create_test_did("charlie");
    
    // Add edges with different ages
    let old_edge = TrustEdge::new(alice.clone(), bob.clone(), 0.8, 500); // 1500s old
    let new_edge = TrustEdge::new(bob.clone(), charlie.clone(), 0.9, 1800); // 200s old
    
    graph.add_edge(old_edge);
    graph.add_edge(new_edge);
    assert_eq!(graph.edge_count(), 2);
    
    // Cleanup with 1000s threshold
    graph.cleanup_expired_edges(&time_provider, 1000);
    
    // Old edge should be removed, new edge should remain
    assert_eq!(graph.edge_count(), 1);
    assert!(graph.get_edge(&alice, &bob).is_none());
    assert!(graph.get_edge(&bob, &charlie).is_some());
}

#[test]
fn test_complex_trust_aggregation_scenario() {
    let (graph, nodes) = setup_test_network();
    let aggregator = TrustAggregator::new();
    let time_provider = FixedTimeProvider::new(2000);
    
    let alice = &nodes[0];
    
    // Test aggregation for multiple entities
    let all_entities = vec![nodes[1].clone(), nodes[2].clone(), nodes[3].clone(), nodes[4].clone()];
    let aggregated_results = aggregator.aggregate_trust_for_entities(&all_entities, &graph, alice, &time_provider);
    
    // Should have results for all entities
    assert_eq!(aggregated_results.len(), 4);
    
    // Results should be sorted by trust score (highest first)
    for i in 1..aggregated_results.len() {
        assert!(aggregated_results[i-1].trust_score >= aggregated_results[i].trust_score);
    }
    
    // All results should have positive scores and confidence
    for result in &aggregated_results {
        assert!(result.trust_score > 0.0);
        assert!(result.confidence > 0.0);
        assert!(result.signal_count > 0);
    }
}