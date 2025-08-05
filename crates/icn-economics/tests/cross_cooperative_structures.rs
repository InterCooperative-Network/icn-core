use icn_economics::automation::{
    EconomicOptimizer, OptimizationConstraint, OptimizationResult,
    CrossCooperativeRequest, RequestStatus, FederationEconomicState,
    CrossCooperativePolicy, CrossCooperativePricingStrategy,
    EconomicHealthMetrics,
};
use std::collections::HashMap;

#[test]
fn test_economic_optimizer_default() {
    let optimizer = EconomicOptimizer::default();
    assert_eq!(optimizer.learning_rate, 0.01);
    assert!(optimizer.optimization_targets.is_empty());
    assert!(optimizer.constraints.is_empty());
    assert!(optimizer.performance_history.is_empty());
}

#[test]  
fn test_cross_cooperative_request_creation() {
    let request = CrossCooperativeRequest {
        request_id: "test_req_001".to_string(),
        requesting_federation: "test-federation".to_string(),
        resource_type: "cpu".to_string(),
        amount: 100,
        max_price: 10.0,
        urgency: 0.8,
        min_trust_level: 0.5,
        expires_at: 1234567890,
        status: RequestStatus::Open,
    };

    assert_eq!(request.request_id, "test_req_001");
    assert_eq!(request.resource_type, "cpu");
    assert_eq!(request.amount, 100);
    assert_eq!(request.urgency, 0.8);
    assert!(matches!(request.status, RequestStatus::Open));
}

#[test]
fn test_federation_economic_state() {
    let initial_resources = HashMap::from([
        ("cpu".to_string(), 1000u64),
        ("memory".to_string(), 2000u64),
    ]);

    let federation_state = FederationEconomicState {
        federation_id: "test-federation".to_string(),
        health_metrics: EconomicHealthMetrics {
            overall_health: 0.8,
            mana_inequality: 0.2,
            resource_efficiency: 0.9,
            market_liquidity: 0.7,
            price_stability: 0.85,
            activity_level: 0.6,
            last_updated: 1234567890,
        },
        available_resources: initial_resources.clone(),
        trust_levels: HashMap::new(),
        resource_requests: Vec::new(),
        last_sync: 1234567890,
    };

    assert_eq!(federation_state.federation_id, "test-federation");
    assert_eq!(federation_state.health_metrics.overall_health, 0.8);
    assert_eq!(federation_state.available_resources.get("cpu"), Some(&1000));
    assert_eq!(federation_state.available_resources.get("memory"), Some(&2000));
}

#[test]
fn test_cross_cooperative_pricing_strategies() {
    // Test different pricing strategies
    let market_strategy = CrossCooperativePricingStrategy::MarketWithTrustDiscount {
        base_markup: 1.2,
        trust_discount: 0.1,
    };

    let cost_plus_strategy = CrossCooperativePricingStrategy::CostPlus {
        markup_percentage: 0.15,
    };

    let mutual_aid_strategy = CrossCooperativePricingStrategy::MutualAid {
        cost_recovery_rate: 1.05,
    };

    let dynamic_strategy = CrossCooperativePricingStrategy::Dynamic {
        base_price: 10.0,
        demand_multiplier: 0.2,
    };

    // Just verify they can be created and matched
    match market_strategy {
        CrossCooperativePricingStrategy::MarketWithTrustDiscount { base_markup, trust_discount } => {
            assert_eq!(base_markup, 1.2);
            assert_eq!(trust_discount, 0.1);
        },
        _ => panic!("Unexpected strategy type"),
    }

    match cost_plus_strategy {
        CrossCooperativePricingStrategy::CostPlus { markup_percentage } => {
            assert_eq!(markup_percentage, 0.15);
        },
        _ => panic!("Unexpected strategy type"),
    }

    match mutual_aid_strategy {
        CrossCooperativePricingStrategy::MutualAid { cost_recovery_rate } => {
            assert_eq!(cost_recovery_rate, 1.05);
        },
        _ => panic!("Unexpected strategy type"),
    }

    match dynamic_strategy {
        CrossCooperativePricingStrategy::Dynamic { base_price, demand_multiplier } => {
            assert_eq!(base_price, 10.0);
            assert_eq!(demand_multiplier, 0.2);
        },
        _ => panic!("Unexpected strategy type"),
    }
}

#[test]
fn test_cross_cooperative_policy() {
    let policy = CrossCooperativePolicy {
        policy_id: "test_policy_001".to_string(),
        min_trust_level: 0.3,
        max_resource_share: 0.25,
        pricing_strategy: CrossCooperativePricingStrategy::MarketWithTrustDiscount {
            base_markup: 1.15,
            trust_discount: 0.05,
        },
        auto_approval_threshold: 500,
        local_priority_weight: 1.2,
    };

    assert_eq!(policy.policy_id, "test_policy_001");
    assert_eq!(policy.min_trust_level, 0.3);
    assert_eq!(policy.max_resource_share, 0.25);
    assert_eq!(policy.auto_approval_threshold, 500);
    assert_eq!(policy.local_priority_weight, 1.2);
}

#[test]
fn test_request_status_transitions() {
    // Test various request statuses
    let open_status = RequestStatus::Open;
    let partial_status = RequestStatus::PartiallyFulfilled { fulfilled_amount: 50 };
    let fulfilled_status = RequestStatus::Fulfilled;
    let expired_status = RequestStatus::Expired;
    let cancelled_status = RequestStatus::Cancelled;

    assert!(matches!(open_status, RequestStatus::Open));
    
    match partial_status {
        RequestStatus::PartiallyFulfilled { fulfilled_amount } => {
            assert_eq!(fulfilled_amount, 50);
        },
        _ => panic!("Unexpected status"),
    }

    assert!(matches!(fulfilled_status, RequestStatus::Fulfilled));
    assert!(matches!(expired_status, RequestStatus::Expired));
    assert!(matches!(cancelled_status, RequestStatus::Cancelled));
}

#[test]
fn test_optimization_constraint() {
    let constraint = OptimizationConstraint {
        constraint_type: "max_inequality".to_string(),
        min_value: None,
        max_value: Some(0.4),
        weight: 0.8,
    };

    assert_eq!(constraint.constraint_type, "max_inequality");
    assert_eq!(constraint.min_value, None);
    assert_eq!(constraint.max_value, Some(0.4));
    assert_eq!(constraint.weight, 0.8);
}

#[test]
fn test_optimization_result() {
    let mut metric_scores = HashMap::new();
    metric_scores.insert("economic_health".to_string(), 0.85);
    metric_scores.insert("resource_efficiency".to_string(), 0.92);

    let result = OptimizationResult {
        timestamp: 1234567890,
        objective_value: 0.88,
        metric_scores,
        constraint_violations: vec!["inequality_threshold".to_string()],
        duration_ms: 150,
    };

    assert_eq!(result.timestamp, 1234567890);
    assert_eq!(result.objective_value, 0.88);
    assert_eq!(result.duration_ms, 150);
    assert_eq!(result.metric_scores.get("economic_health"), Some(&0.85));
    assert_eq!(result.constraint_violations.len(), 1);
    assert_eq!(result.constraint_violations[0], "inequality_threshold");
}