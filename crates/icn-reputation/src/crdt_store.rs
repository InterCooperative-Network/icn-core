//! CRDT-backed reputation store for conflict-free distributed reputation management.
//!
//! This module provides a ReputationStore implementation that uses CRDTs to ensure
//! conflict-free replication across multiple nodes. Each DID's reputation is tracked
//! using a PN-Counter CRDT that handles both positive and negative reputation changes.

use crate::ReputationStore;
use icn_common::{CommonError, Did};
use icn_crdt::{CRDTMap, NodeId, PNCounter, CRDT};
use log::{debug, error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, RwLock};

/// CRDT-backed reputation store that enables conflict-free distributed reputation management.
///
/// Uses a CRDT Map where each DID maps to a PN-Counter tracking reputation score.
/// This allows multiple nodes to concurrently update reputation without conflicts.
pub struct CRDTReputationStore {
    /// Node identifier for this reputation store instance.
    node_id: NodeId,
    /// CRDT Map storing DID -> PN-Counter mappings for reputation scores.
    reputation_map: Arc<RwLock<CRDTMap<String, PNCounter>>>,
    /// Configuration for reputation scoring.
    config: CRDTReputationConfig,
}

/// Configuration for CRDT reputation store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CRDTReputationConfig {
    /// Node identifier for this store instance.
    pub node_id: String,
    /// Base score for successful executions.
    pub success_reward: u64,
    /// Penalty for failed executions.
    pub failure_penalty: u64,
    /// Reward for successful proof attempts.
    pub proof_success_reward: u64,
    /// Penalty for failed proof attempts.
    pub proof_failure_penalty: u64,
    /// CPU time multiplier for execution scoring (score per millisecond).
    pub cpu_time_multiplier: f64,
    /// Initial reputation scores for accounts (for bootstrapping).
    pub initial_scores: HashMap<String, u64>,
}

impl Default for CRDTReputationConfig {
    fn default() -> Self {
        Self {
            node_id: "default_reputation_node".to_string(),
            success_reward: 10,
            failure_penalty: 5,
            proof_success_reward: 2,
            proof_failure_penalty: 1,
            cpu_time_multiplier: 0.001, // Small multiplier for CPU time bonus
            initial_scores: HashMap::new(),
        }
    }
}

/// Statistics about a CRDT reputation store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CRDTReputationStats {
    /// Number of accounts with reputation scores.
    pub account_count: u64,
    /// Total reputation across all accounts.
    pub total_reputation: u64,
    /// Average reputation per account.
    pub average_reputation: u64,
    /// Highest reputation score.
    pub max_reputation: u64,
    /// Lowest reputation score.
    pub min_reputation: u64,
    /// Node ID of this reputation store instance.
    pub node_id: NodeId,
}

impl CRDTReputationStore {
    /// Create a new CRDT reputation store with the given configuration.
    pub fn new(config: CRDTReputationConfig) -> Self {
        let node_id = NodeId::new(config.node_id.clone());
        let reputation_map = CRDTMap::new("reputation_scores".to_string());

        let store = Self {
            node_id,
            reputation_map: Arc::new(RwLock::new(reputation_map)),
            config,
        };

        // Initialize with provided scores
        for (did_str, score) in &store.config.initial_scores {
            if let Ok(did) = Did::from_str(did_str) {
                store.set_score(did, *score);
            }
        }

        store
    }

    /// Create a new CRDT reputation store with a specific node ID.
    pub fn with_node_id(node_id: String) -> Self {
        Self::new(CRDTReputationConfig {
            node_id,
            ..Default::default()
        })
    }

    /// Get the node ID for this reputation store instance.
    pub fn node_id(&self) -> &NodeId {
        &self.node_id
    }

    /// Merge state from another CRDT reputation store.
    ///
    /// This enables synchronization between distributed reputation store instances.
    pub fn merge(&self, other: &Self) -> Result<(), CommonError> {
        let mut our_map = self
            .reputation_map
            .write()
            .map_err(|_| CommonError::LockError("Failed to acquire write lock".to_string()))?;

        let other_map = other
            .reputation_map
            .read()
            .map_err(|_| CommonError::LockError("Failed to acquire read lock".to_string()))?;

        our_map.merge(&*other_map);

        debug!(
            "Merged CRDT reputation store state from node {}",
            other.node_id
        );
        Ok(())
    }

    /// Set the reputation score for a specific DID.
    ///
    /// This is useful for initialization or administrative changes.
    pub fn set_score(&self, did: Did, score: u64) {
        if let Err(e) = self.set_score_internal(&did, score) {
            error!("Failed to set reputation score for {did}: {e}");
        }
    }

    /// Internal method to set reputation score.
    fn set_score_internal(&self, did: &Did, score: u64) -> Result<(), CommonError> {
        debug!("Setting reputation score for DID {did} to {score}");

        let current_score = self.get_reputation(did);

        if score > current_score {
            // Need to add the difference
            let add_amount = score - current_score;
            self.adjust_reputation(did, add_amount as i64)?;
        } else if score < current_score {
            // Need to subtract the difference
            let sub_amount = current_score - score;
            self.adjust_reputation(did, -(sub_amount as i64))?;
        }
        // If scores are equal, no operation needed

        Ok(())
    }

    /// Adjust reputation by a delta amount (can be positive or negative).
    fn adjust_reputation(&self, did: &Did, delta: i64) -> Result<(), CommonError> {
        if delta == 0 {
            return Ok(());
        }

        debug!("Adjusting reputation for DID {did} by {delta}");

        // Get or create the counter
        let mut counter = self.get_or_create_counter(did)?;

        if delta > 0 {
            counter
                .increment(&self.node_id, delta as u64)
                .map_err(|e| {
                    CommonError::CRDTError(format!("Failed to increment reputation: {e}"))
                })?;
        } else {
            counter
                .decrement(&self.node_id, (-delta) as u64)
                .map_err(|e| {
                    CommonError::CRDTError(format!("Failed to decrement reputation: {e}"))
                })?;
        }

        // Update the counter in the map
        self.update_counter(did, counter)?;

        debug!(
            "Successfully adjusted reputation for DID {did} by {delta}"
        );
        Ok(())
    }

    /// Get or create a PN-Counter for the given DID.
    fn get_or_create_counter(&self, did: &Did) -> Result<PNCounter, CommonError> {
        let mut map = self
            .reputation_map
            .write()
            .map_err(|_| CommonError::LockError("Failed to acquire write lock".to_string()))?;

        let did_str = did.to_string();

        if let Some(counter) = map.get(&did_str) {
            Ok(counter.clone())
        } else {
            // Create new counter for this DID
            let counter_id = format!("reputation_{did_str}");
            let counter = PNCounter::new(counter_id);

            map.put(did_str, counter.clone(), self.node_id.clone())
                .map_err(|e| {
                    CommonError::CRDTError(format!("Failed to create reputation counter: {e}"))
                })?;

            debug!("Created new reputation counter for DID: {did}");
            Ok(counter)
        }
    }

    /// Update a counter in the map after modification.
    fn update_counter(&self, did: &Did, counter: PNCounter) -> Result<(), CommonError> {
        let mut map = self
            .reputation_map
            .write()
            .map_err(|_| CommonError::LockError("Failed to acquire write lock".to_string()))?;

        let did_str = did.to_string();
        map.put(did_str, counter, self.node_id.clone())
            .map_err(|e| {
                CommonError::CRDTError(format!("Failed to update reputation counter: {e}"))
            })?;

        Ok(())
    }

    /// Get a snapshot of current reputation scores for debugging or reporting.
    pub fn get_all_scores(&self) -> Result<HashMap<Did, u64>, CommonError> {
        let map = self
            .reputation_map
            .read()
            .map_err(|_| CommonError::LockError("Failed to acquire read lock".to_string()))?;

        let mut scores = HashMap::new();

        for key in map.keys() {
            if let Ok(did) = Did::from_str(&key) {
                if let Some(counter) = map.get(&key) {
                    let score = counter.get_total().max(0) as u64;
                    scores.insert(did, score);
                }
            }
        }

        Ok(scores)
    }

    /// Get statistics about the reputation store.
    pub fn get_stats(&self) -> Result<CRDTReputationStats, CommonError> {
        let scores = self.get_all_scores()?;
        let account_count = scores.len() as u64;
        let total_reputation = scores.values().sum::<u64>();
        let average_reputation = if account_count > 0 {
            total_reputation / account_count
        } else {
            0
        };

        let max_reputation = scores.values().max().copied().unwrap_or(0);
        let min_reputation = scores.values().min().copied().unwrap_or(0);

        Ok(CRDTReputationStats {
            account_count,
            total_reputation,
            average_reputation,
            max_reputation,
            min_reputation,
            node_id: self.node_id.clone(),
        })
    }

    /// Get all DIDs with reputation scores.
    pub fn all_accounts(&self) -> Vec<Did> {
        match self.reputation_map.read() {
            Ok(map) => {
                let mut accounts = Vec::new();
                for key in map.keys() {
                    if let Ok(did) = Did::from_str(&key) {
                        accounts.push(did);
                    }
                }
                accounts
            }
            Err(_) => {
                error!("Failed to acquire read lock for account listing");
                Vec::new()
            }
        }
    }
}

impl Clone for CRDTReputationStore {
    fn clone(&self) -> Self {
        Self {
            node_id: self.node_id.clone(),
            reputation_map: self.reputation_map.clone(),
            config: self.config.clone(),
        }
    }
}

impl std::fmt::Debug for CRDTReputationStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CRDTReputationStore")
            .field("node_id", &self.node_id)
            .field("reputation_map", &"<CRDTMap>")
            .field("config", &self.config)
            .finish()
    }
}

impl ReputationStore for CRDTReputationStore {
    fn get_reputation(&self, did: &Did) -> u64 {
        match self.reputation_map.read() {
            Ok(map) => {
                let did_str = did.to_string();
                if let Some(counter) = map.get(&did_str) {
                    counter.get_total().max(0) as u64
                } else {
                    0
                }
            }
            Err(_) => {
                error!("Failed to acquire read lock for reputation check");
                0
            }
        }
    }

    fn record_execution(&self, executor: &Did, success: bool, cpu_ms: u64) {
        crate::metrics::EXECUTION_RECORDS.inc();

        let base_delta = if success {
            self.config.success_reward as i64
        } else {
            -(self.config.failure_penalty as i64)
        };

        // Add bonus for CPU time efficiency (only for successful executions)
        let cpu_bonus = if success {
            (cpu_ms as f64 * self.config.cpu_time_multiplier) as i64
        } else {
            0
        };

        let total_delta = base_delta + cpu_bonus;

        if let Err(e) = self.adjust_reputation(executor, total_delta) {
            error!("Failed to record execution for {executor}: {e}");
        } else {
            debug!(
                "Recorded execution for {executor}: success={success}, cpu_ms={cpu_ms}, delta={total_delta}"
            );
        }
    }

    fn record_proof_attempt(&self, prover: &Did, success: bool) {
        crate::metrics::PROOF_ATTEMPTS.inc();

        let delta = if success {
            self.config.proof_success_reward as i64
        } else {
            -(self.config.proof_failure_penalty as i64)
        };

        if let Err(e) = self.adjust_reputation(prover, delta) {
            error!("Failed to record proof attempt for {prover}: {e}");
        } else {
            debug!(
                "Recorded proof attempt for {prover}: success={success}, delta={delta}"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn alice_did() -> Did {
        Did::from_str("did:key:alice").unwrap()
    }

    fn bob_did() -> Did {
        Did::from_str("did:key:bob").unwrap()
    }

    #[test]
    fn test_crdt_reputation_store_creation() {
        let store = CRDTReputationStore::with_node_id("test_node".to_string());
        assert_eq!(store.node_id().as_str(), "test_node");
        assert_eq!(store.get_reputation(&alice_did()), 0);
    }

    #[test]
    fn test_crdt_reputation_store_initial_scores() {
        let mut initial_scores = HashMap::new();
        initial_scores.insert("did:key:alice".to_string(), 100);
        initial_scores.insert("did:key:bob".to_string(), 50);

        let config = CRDTReputationConfig {
            node_id: "test_node".to_string(),
            initial_scores,
            ..Default::default()
        };

        let store = CRDTReputationStore::new(config);
        assert_eq!(store.get_reputation(&alice_did()), 100);
        assert_eq!(store.get_reputation(&bob_did()), 50);
    }

    #[test]
    fn test_crdt_reputation_store_set_score() {
        let store = CRDTReputationStore::with_node_id("test_node".to_string());

        // Set initial score
        store.set_score(alice_did(), 100);
        assert_eq!(store.get_reputation(&alice_did()), 100);

        // Increase score
        store.set_score(alice_did(), 150);
        assert_eq!(store.get_reputation(&alice_did()), 150);

        // Decrease score
        store.set_score(alice_did(), 75);
        assert_eq!(store.get_reputation(&alice_did()), 75);
    }

    #[test]
    fn test_crdt_reputation_store_record_execution() {
        let store = CRDTReputationStore::with_node_id("test_node".to_string());

        // Record successful execution
        store.record_execution(&alice_did(), true, 1000);
        let score_after_success = store.get_reputation(&alice_did());
        assert!(score_after_success > 0);

        // Record failed execution
        store.record_execution(&alice_did(), false, 500);
        let score_after_failure = store.get_reputation(&alice_did());
        assert!(score_after_failure < score_after_success);
    }

    #[test]
    fn test_crdt_reputation_store_record_proof_attempt() {
        let store = CRDTReputationStore::with_node_id("test_node".to_string());

        // Record successful proof
        store.record_proof_attempt(&alice_did(), true);
        let score_after_success = store.get_reputation(&alice_did());
        assert_eq!(score_after_success, 2); // Default proof_success_reward

        // Record failed proof
        store.record_proof_attempt(&alice_did(), false);
        let score_after_failure = store.get_reputation(&alice_did());
        assert_eq!(score_after_failure, 1); // 2 - 1 (default proof_failure_penalty)
    }

    #[test]
    fn test_crdt_reputation_store_merge() {
        let store1 = CRDTReputationStore::with_node_id("node1".to_string());
        let store2 = CRDTReputationStore::with_node_id("node2".to_string());

        // Each store records different executions for Alice
        store1.record_execution(&alice_did(), true, 1000); // +10 base + 1 cpu bonus
        store2.record_execution(&alice_did(), true, 2000); // +10 base + 2 cpu bonus

        // Bob gets reputation only in store2
        store2.record_execution(&bob_did(), true, 500); // +10 base + 0.5 cpu bonus

        // Verify initial state
        let alice_score1 = store1.get_reputation(&alice_did());
        let alice_score2 = store2.get_reputation(&alice_did());
        assert!(alice_score1 > 10);
        assert!(alice_score2 > 10);
        assert_eq!(store1.get_reputation(&bob_did()), 0);
        assert!(store2.get_reputation(&bob_did()) > 10);

        // Merge store2 into store1
        store1.merge(&store2).unwrap();

        // After merge, Alice should have combined scores and Bob should have his score
        let alice_final = store1.get_reputation(&alice_did());
        let bob_final = store1.get_reputation(&bob_did());

        assert!(alice_final >= alice_score1 + alice_score2);
        assert!(bob_final > 10);
    }

    #[test]
    fn test_crdt_reputation_store_all_accounts() {
        let store = CRDTReputationStore::with_node_id("test_node".to_string());

        // Initially no accounts
        assert_eq!(store.all_accounts().len(), 0);

        // Add some accounts
        store.record_execution(&alice_did(), true, 1000);
        store.record_execution(&bob_did(), true, 1000);

        let accounts = store.all_accounts();
        assert_eq!(accounts.len(), 2);
        assert!(accounts.contains(&alice_did()));
        assert!(accounts.contains(&bob_did()));
    }

    #[test]
    fn test_crdt_reputation_store_stats() {
        let store = CRDTReputationStore::with_node_id("test_node".to_string());

        store.set_score(alice_did(), 100);
        store.set_score(bob_did(), 200);

        let stats = store.get_stats().unwrap();
        assert_eq!(stats.account_count, 2);
        assert_eq!(stats.total_reputation, 300);
        assert_eq!(stats.average_reputation, 150);
        assert_eq!(stats.max_reputation, 200);
        assert_eq!(stats.min_reputation, 100);
        assert_eq!(stats.node_id.as_str(), "test_node");
    }

    #[test]
    fn test_crdt_reputation_store_concurrent_operations() {
        let store1 = CRDTReputationStore::with_node_id("node1".to_string());
        let store2 = CRDTReputationStore::with_node_id("node2".to_string());

        // Simulate concurrent operations on the same account
        store1.record_execution(&alice_did(), true, 1000); // Success
        store1.record_execution(&alice_did(), false, 500); // Failure

        store2.record_execution(&alice_did(), true, 2000); // Success
        store2.record_proof_attempt(&alice_did(), true); // Proof success

        // Before merge
        let alice_score1 = store1.get_reputation(&alice_did());
        let alice_score2 = store2.get_reputation(&alice_did());

        // Merge the states
        store1.merge(&store2).unwrap();

        // After merge: Alice should have combined reputation from all operations
        let alice_final = store1.get_reputation(&alice_did());
        assert!(alice_final >= alice_score1.max(alice_score2));
    }

    #[test]
    fn test_crdt_reputation_store_configuration() {
        let config = CRDTReputationConfig {
            node_id: "test_node".to_string(),
            success_reward: 20,
            failure_penalty: 10,
            proof_success_reward: 5,
            proof_failure_penalty: 2,
            ..Default::default()
        };

        let store = CRDTReputationStore::new(config);

        // Test with custom rewards
        store.record_execution(&alice_did(), true, 0);
        assert_eq!(store.get_reputation(&alice_did()), 20);

        store.record_execution(&alice_did(), false, 0);
        assert_eq!(store.get_reputation(&alice_did()), 10); // 20 - 10

        store.record_proof_attempt(&alice_did(), true);
        assert_eq!(store.get_reputation(&alice_did()), 15); // 10 + 5
    }
}
