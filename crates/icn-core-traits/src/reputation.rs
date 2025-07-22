//! Reputation service traits and types

use crate::CoreTraitsError;
use async_trait::async_trait;
use icn_common::{CommonError, Did};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Reputation store trait for managing reputation data
pub trait ReputationStore: Send + Sync {
    /// Get reputation score for a DID
    fn get_reputation(&self, did: &Did) -> u32;

    /// Set reputation score for a DID
    fn set_reputation(&mut self, did: &Did, score: u32) -> Result<(), CommonError>;

    /// Update reputation score for a DID
    fn update_reputation(&mut self, did: &Did, delta: i32) -> Result<u32, CommonError>;

    /// Get all reputation scores
    fn get_all_reputations(&self) -> HashMap<Did, u32>;

    /// Check if a DID has a reputation score
    fn has_reputation(&self, did: &Did) -> bool;

    /// Remove reputation score for a DID
    fn remove_reputation(&mut self, did: &Did) -> Result<(), CommonError>;
}

/// Reputation events that can be observed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReputationEvent {
    /// Reputation score updated
    ReputationUpdated {
        did: Did,
        old_score: u32,
        new_score: u32,
    },
    /// New reputation entry created
    ReputationCreated { did: Did, score: u32 },
    /// Reputation entry removed
    ReputationRemoved { did: Did, last_score: u32 },
    /// Reputation calculation completed
    ReputationCalculated {
        did: Did,
        score: u32,
        factors: HashMap<String, f64>,
    },
}

/// Advanced reputation provider with async operations
#[async_trait]
pub trait ReputationProvider: Send + Sync {
    /// Calculate reputation score based on various factors
    async fn calculate_reputation(&self, did: &Did) -> Result<u32, CoreTraitsError>;

    /// Get reputation score asynchronously
    async fn get_reputation_async(&self, did: &Did) -> Result<u32, CoreTraitsError>;

    /// Update reputation based on execution quality
    async fn update_reputation_from_execution(
        &self,
        did: &Did,
        execution_quality: f64,
        context: HashMap<String, String>,
    ) -> Result<u32, CoreTraitsError>;

    /// Get reputation trend over time
    async fn get_reputation_trend(
        &self,
        did: &Did,
        days: u32,
    ) -> Result<Vec<(u64, u32)>, CoreTraitsError>; // (timestamp, reputation)

    /// Get top reputation scores
    async fn get_top_reputations(&self, limit: usize) -> Result<Vec<(Did, u32)>, CoreTraitsError>;

    /// Subscribe to reputation events
    async fn subscribe_to_events(
        &self,
    ) -> Result<tokio::sync::mpsc::Receiver<ReputationEvent>, CoreTraitsError>;

    /// Calculate trust path between two DIDs
    async fn calculate_trust_path(
        &self,
        from: &Did,
        to: &Did,
    ) -> Result<Option<Vec<Did>>, CoreTraitsError>;
}

/// Simple reputation provider for basic operations
pub trait SimpleReputationProvider: Send + Sync {
    /// Get reputation score
    fn get_reputation(&self, did: &Did) -> u32;

    /// Check if DID meets minimum reputation threshold
    fn meets_threshold(&self, did: &Did, threshold: u32) -> bool {
        self.get_reputation(did) >= threshold
    }

    /// Get reputation as a normalized score (0.0 to 1.0)
    fn get_normalized_reputation(&self, did: &Did, max_score: u32) -> f64 {
        let score = self.get_reputation(did) as f64;
        if max_score == 0 {
            0.0
        } else {
            (score / max_score as f64).min(1.0)
        }
    }
}
