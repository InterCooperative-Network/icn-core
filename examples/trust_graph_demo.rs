//! Trust Graph Integration Example
//!
//! This example demonstrates how to use the ICN trust graph system
//! for cooperative trust evaluation and pathfinding.

use icn_common::{Did, FixedTimeProvider, TimeProvider};
use icn_reputation::{
    TrustGraph, TrustEdge, TrustCalculationEngine, TrustPathfinder,
    TrustDecayCalculator, TrustAggregator, DecayModel, PathDiscoveryConfig,
    AggregationConfig, CombinationMethod, TrustSignal
};
use std::str::FromStr;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("ICN Trust Graph Integration Example");
    println!("===================================");
    
    // Create some test DIDs for cooperatives
    let food_coop = Did::from_str("did:icn:coop:food-collective")?;
    let tech_coop = Did::from_str("did:icn:coop:tech-workers")?; 
    let housing_coop = Did::from_str("did:icn:coop:housing-union")?;
    let energy_coop = Did::from_str("did:icn:coop:renewable-energy")?;
    let credit_union = Did::from_str("did:icn:coop:community-credit")?;
    
    // Initialize the trust graph system components
    let mut trust_graph = TrustGraph::new();
    let calculation_engine = TrustCalculationEngine::new();
    let pathfinder = TrustPathfinder::new();
    let decay_calculator = TrustDecayCalculator::with_time_decay(
        DecayModel::Exponential { half_life_seconds: 180 * 24 * 3600 } // 6 months
    );
    let aggregator = TrustAggregator::new();
    let time_provider = FixedTimeProvider::new(1640995200); // Jan 1, 2022
    
    println!("\n1. Building Trust Relationships");
    println!("------------------------------");
    
    // Build a network of trust relationships between cooperatives
    // Food coop has established relationships
    trust_graph.add_edge(TrustEdge::new(food_coop.clone(), tech_coop.clone(), 0.85, 1640908800));
    trust_graph.add_edge(TrustEdge::new(food_coop.clone(), housing_coop.clone(), 0.9, 1640822400));
    trust_graph.add_edge(TrustEdge::new(food_coop.clone(), credit_union.clone(), 0.95, 1640736000));
    
    // Tech coop relationships
    trust_graph.add_edge(TrustEdge::new(tech_coop.clone(), energy_coop.clone(), 0.8, 1640649600));
    trust_graph.add_edge(TrustEdge::new(tech_coop.clone(), housing_coop.clone(), 0.75, 1640563200));
    
    // Housing coop relationships  
    trust_graph.add_edge(TrustEdge::new(housing_coop.clone(), credit_union.clone(), 0.88, 1640476800));
    trust_graph.add_edge(TrustEdge::new(housing_coop.clone(), energy_coop.clone(), 0.82, 1640390400));
    
    // Energy coop relationships
    trust_graph.add_edge(TrustEdge::new(energy_coop.clone(), credit_union.clone(), 0.77, 1640304000));
    
    // Credit union trusts most others (financial services)
    trust_graph.add_edge(TrustEdge::new(credit_union.clone(), food_coop.clone(), 0.83, 1640217600));
    trust_graph.add_edge(TrustEdge::new(credit_union.clone(), tech_coop.clone(), 0.79, 1640131200));
    
    println!("Built trust graph with {} nodes and {} edges", 
             trust_graph.node_count(), trust_graph.edge_count());
    
    println!("\n2. PageRank-Style Trust Scores");
    println!("------------------------------");
    
    // Calculate global trust scores using PageRank algorithm
    let pagerank_scores = calculation_engine.calculate_pagerank_scores(&trust_graph, &time_provider);
    
    let mut scores: Vec<_> = pagerank_scores.iter().collect();
    scores.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
    
    for (coop, score) in scores {
        println!("{}: {:.4}", coop.id_string, score);
    }
    
    println!("\n3. Trust Path Discovery");
    println!("----------------------");
    
    // Find trust path from food coop to energy coop
    if let Some(path) = pathfinder.find_best_path(&trust_graph, &food_coop, &energy_coop, &time_provider) {
        println!("Best trust path from {} to {}:", 
                 food_coop.id_string, energy_coop.id_string);
        println!("  Path length: {} hops", path.length);
        println!("  Trust score: {:.4}", path.trust_score);
        println!("  Path: {} ", food_coop.id_string);
        for edge in &path.edges {
            println!("    -> {} (trust: {:.3})", edge.to.id_string, edge.weight);
        }
    }
    
    // Find multiple diverse paths
    let multiple_paths = pathfinder.find_multiple_paths(&trust_graph, &food_coop, &energy_coop, &time_provider);
    println!("\nFound {} alternative paths", multiple_paths.len());
    
    for (i, path) in multiple_paths.iter().enumerate() {
        println!("  Path {}: {} hops, trust {:.4}", i + 1, path.length, path.trust_score);
    }
    
    println!("\n4. Trust Decay Analysis");
    println!("----------------------");
    
    // Analyze trust decay in the network
    let decay_stats = decay_calculator.calculate_decay_statistics(&trust_graph, &time_provider);
    println!("Decay Statistics:");
    println!("  Total edges: {}", decay_stats.total_edges);
    println!("  Average age: {} days", decay_stats.average_age_seconds / 86400);
    println!("  Average decay factor: {:.4}", decay_stats.average_decay_factor);
    println!("  Significantly decayed: {:.1}%", decay_stats.significant_decay_percentage());
    
    println!("\n5. Trust Aggregation");
    println!("-------------------");
    
    // Aggregate trust signals for the tech coop from food coop's perspective
    let signals = vec![
        TrustSignal::DirectTrust {
            from: food_coop.clone(),
            to: tech_coop.clone(),
            weight: 0.85,
        },
        TrustSignal::ReputationTrust {
            entity: tech_coop.clone(),
            score: 0.78,
        },
        TrustSignal::ActivityTrust {
            entity: tech_coop.clone(),
            interaction_count: 25,
            recency_score: 0.9,
        },
        TrustSignal::NetworkTrust {
            entity: tech_coop.clone(),
            centrality_score: 0.6,
        },
    ];
    
    let aggregated = aggregator.aggregate_trust_for_entity(&tech_coop, signals, &time_provider);
    
    println!("Aggregated trust for {}:", tech_coop.id_string);
    println!("  Final score: {:.4}", aggregated.trust_score);
    println!("  Confidence: {:.4}", aggregated.confidence);
    println!("  Signals used: {}", aggregated.signal_count);
    println!("  Signal variance: {:.4}", aggregated.signal_variance);
    
    println!("  Signal breakdown:");
    for (signal_type, score) in &aggregated.signal_breakdown {
        println!("    {}: {:.4}", signal_type, score);
    }
    
    println!("\n6. Reachability Analysis");
    println!("-----------------------");
    
    // Find all cooperatives reachable from food coop with minimum trust
    let reachable = pathfinder.find_reachable_nodes(&trust_graph, &food_coop, 0.5, &time_provider);
    
    println!("Cooperatives reachable from {} with trust >= 0.5:", food_coop.id_string);
    for (coop, trust_score) in &reachable {
        if coop != &food_coop {
            println!("  {}: {:.4}", coop.id_string, trust_score);
        }
    }
    
    println!("\n7. Weighted Trust Calculation");
    println!("-----------------------------");
    
    // Calculate weighted trust scores considering direct and indirect relationships
    let weighted_scores = calculation_engine.calculate_weighted_trust_scores(&trust_graph, &time_provider, 4);
    
    let mut weighted: Vec<_> = weighted_scores.iter().collect();
    weighted.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
    
    println!("Weighted trust scores (considering path length):");
    for (coop, score) in weighted {
        println!("  {}: {:.4}", coop.id_string, score);
    }
    
    println!("\nTrust graph analysis complete!");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_trust_graph_example() {
        // Run the example as a test to ensure it works
        assert!(main().is_ok());
    }
}