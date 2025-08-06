//! Basic anti-gaming implementation for the Byzantine mana ledger

use crate::adversarial::{
    GameTheoreticSecurity, GamingDetectionResult, GamingIndicators, AntiGamingResult,
    BehaviorHistory, NetworkAnalysis, SybilDetectionResult, AntiGamingAction,
    SybilCountermeasure, IdentityCluster, ClusterType,
};
use icn_common::{CommonError, Did};
use std::collections::HashMap;

/// Basic anti-gaming engine implementation
#[derive(Debug)]
pub struct BasicAntiGamingEngine {
    /// Thresholds for gaming detection
    pub gaming_thresholds: GamingThresholds,
    /// Historical gaming detection results
    pub detection_history: Vec<GamingDetectionResult>,
}

#[derive(Debug, Clone)]
pub struct GamingThresholds {
    pub capacity_inflation_threshold: f64,
    pub reputation_farming_threshold: f64,
    pub transaction_manipulation_threshold: f64,
    pub collusion_threshold: f64,
    pub sybil_attack_threshold: f64,
}

impl Default for GamingThresholds {
    fn default() -> Self {
        Self {
            capacity_inflation_threshold: 0.7,
            reputation_farming_threshold: 0.6,
            transaction_manipulation_threshold: 0.8,
            collusion_threshold: 0.5,
            sybil_attack_threshold: 0.8,
        }
    }
}

impl BasicAntiGamingEngine {
    pub fn new() -> Self {
        Self {
            gaming_thresholds: GamingThresholds::default(),
            detection_history: Vec::new(),
        }
    }

    fn analyze_capacity_inflation(&self, behavior: &BehaviorHistory) -> f64 {
        // Simple heuristic: check for sudden capacity increases
        if behavior.capacity_claims.len() < 2 {
            return 0.0;
        }

        let mut max_increase = 0.0f64;
        for window in behavior.capacity_claims.windows(2) {
            if let [prev, curr] = window {
                let compute_increase = (curr.claimed_capacity.compute_capacity - prev.claimed_capacity.compute_capacity) 
                    / prev.claimed_capacity.compute_capacity.max(1.0);
                max_increase = max_increase.max(compute_increase);
            }
        }

        // Score based on rate of increase
        if max_increase > 2.0 {
            0.9 // Very suspicious
        } else if max_increase > 1.0 {
            0.6 // Moderately suspicious
        } else {
            max_increase * 0.3 // Low suspicion
        }
    }

    fn analyze_reputation_farming(&self, behavior: &BehaviorHistory) -> f64 {
        if behavior.reputation_changes.is_empty() {
            return 0.0;
        }

        // Check for patterns of small, frequent reputation gains
        let positive_changes: Vec<_> = behavior.reputation_changes
            .iter()
            .filter(|change| change.change_amount > 0)
            .collect();

        if positive_changes.is_empty() {
            return 0.0;
        }

        let avg_change = positive_changes.iter()
            .map(|change| change.change_amount as f64)
            .sum::<f64>() / positive_changes.len() as f64;

        let change_frequency = positive_changes.len() as f64 / 30.0; // Per month

        // Suspicious if many small, frequent changes
        if avg_change < 5.0 && change_frequency > 10.0 {
            0.8
        } else if avg_change < 10.0 && change_frequency > 5.0 {
            0.5
        } else {
            0.1
        }
    }

    fn analyze_transaction_manipulation(&self, behavior: &BehaviorHistory) -> f64 {
        let patterns = &behavior.transaction_patterns;
        
        // Check for artificial transaction patterns
        let mut suspicion_score = 0.0f64;

        // High frequency with low counterparty diversity suggests manipulation
        if patterns.transaction_frequency > 100.0 && patterns.counterparty_diversity < 0.3 {
            suspicion_score += 0.4;
        }

        // High temporal clustering suggests coordinated behavior
        if patterns.temporal_clustering > 0.8 {
            suspicion_score += 0.3;
        }

        // High outlier frequency in amounts suggests wash trading
        if patterns.amount_distribution.outlier_frequency > 0.3 {
            suspicion_score += 0.3;
        }

        suspicion_score.min(1.0)
    }

    fn analyze_collusion(&self, behavior: &BehaviorHistory) -> f64 {
        // Simple check: many connections to accounts with similar patterns
        if behavior.social_connections.len() > 20 {
            0.6 // Suspicious number of connections
        } else if behavior.social_connections.len() > 10 {
            0.3 // Moderately suspicious
        } else {
            0.1 // Low suspicion
        }
    }

    fn detect_sybil_clusters(&self, accounts: &[Did]) -> Vec<IdentityCluster> {
        // Simple clustering based on account creation patterns
        let mut clusters = Vec::new();
        
        // Group accounts created around the same time (simplified)
        let mut time_clusters = HashMap::new();
        for (i, account) in accounts.iter().enumerate() {
            // Simplified: use account index as proxy for creation time
            let time_bucket = i / 10; // Group every 10 accounts
            time_clusters.entry(time_bucket).or_insert_with(Vec::new).push(account.clone());
        }

        for (bucket, members) in time_clusters {
            if members.len() > 5 { // Suspicious if many accounts created together
                clusters.push(IdentityCluster {
                    cluster_id: format!("time_cluster_{}", bucket),
                    members,
                    cluster_type: ClusterType::SuspiciousSybil,
                    formation_time: bucket as u64 * 86400, // Simplified timestamp
                });
            }
        }

        clusters
    }
}

impl GameTheoreticSecurity for BasicAntiGamingEngine {
    fn detect_gaming_attempt(
        &self,
        _account: &Did,
        behavior_history: &BehaviorHistory,
    ) -> Result<GamingDetectionResult, CommonError> {
        let capacity_inflation_score = self.analyze_capacity_inflation(behavior_history);
        let reputation_farming_score = self.analyze_reputation_farming(behavior_history);
        let transaction_manipulation_score = self.analyze_transaction_manipulation(behavior_history);
        let collusion_score = self.analyze_collusion(behavior_history);
        
        let gaming_indicators = GamingIndicators {
            capacity_inflation_score,
            reputation_farming_score,
            transaction_manipulation_score,
            collusion_score,
            sybil_attack_score: 0.0, // Would need network analysis
        };

        // Calculate overall confidence score
        let confidence_score = capacity_inflation_score * 0.3 +
            reputation_farming_score * 0.2 +
            transaction_manipulation_score * 0.3 +
            collusion_score * 0.2;

        let gaming_detected = confidence_score > 0.5;

        let recommended_actions = if gaming_detected {
            if confidence_score > 0.8 {
                vec![AntiGamingAction::FreezeAccount { duration_hours: 24 }]
            } else if confidence_score > 0.6 {
                vec![AntiGamingAction::RequireAdditionalVerification]
            } else {
                vec![AntiGamingAction::ReduceCapacityWeight(0.8)]
            }
        } else {
            vec![]
        };

        Ok(GamingDetectionResult {
            gaming_detected,
            confidence_score,
            gaming_indicators,
            recommended_actions,
        })
    }

    fn apply_anti_gaming_measures(
        &self,
        _account: &Did,
        gaming_indicators: &GamingIndicators,
    ) -> Result<AntiGamingResult, CommonError> {
        let mut actions_applied = Vec::new();
        let mut adjusted_parameters = HashMap::new();
        let mut follow_up_required = false;

        // Apply measures based on indicators
        if gaming_indicators.capacity_inflation_score > self.gaming_thresholds.capacity_inflation_threshold {
            actions_applied.push(AntiGamingAction::ReduceCapacityWeight(0.5));
            adjusted_parameters.insert("capacity_weight".to_string(), 0.5);
        }

        if gaming_indicators.reputation_farming_score > self.gaming_thresholds.reputation_farming_threshold {
            actions_applied.push(AntiGamingAction::RequireAdditionalVerification);
            follow_up_required = true;
        }

        if gaming_indicators.transaction_manipulation_score > self.gaming_thresholds.transaction_manipulation_threshold {
            actions_applied.push(AntiGamingAction::FreezeAccount { duration_hours: 12 });
            follow_up_required = true;
        }

        Ok(AntiGamingResult {
            actions_applied,
            adjusted_parameters,
            follow_up_required,
        })
    }

    fn detect_sybil_attack(
        &self,
        accounts: &[Did],
        _network_analysis: &NetworkAnalysis,
    ) -> Result<SybilDetectionResult, CommonError> {
        let suspected_clusters = self.detect_sybil_clusters(accounts);
        
        let sybil_attack_detected = !suspected_clusters.is_empty();
        let confidence_score = if sybil_attack_detected {
            suspected_clusters.len() as f64 / accounts.len() as f64
        } else {
            0.0
        };

        let recommended_countermeasures = if sybil_attack_detected {
            vec![
                SybilCountermeasure::IncreaseVerificationRequirements,
                SybilCountermeasure::RequireIdentityProofs,
            ]
        } else {
            vec![]
        };

        Ok(SybilDetectionResult {
            sybil_attack_detected,
            confidence_score,
            suspected_sybil_clusters: suspected_clusters,
            recommended_countermeasures,
        })
    }
}

impl Default for BasicAntiGamingEngine {
    fn default() -> Self {
        Self::new()
    }
}