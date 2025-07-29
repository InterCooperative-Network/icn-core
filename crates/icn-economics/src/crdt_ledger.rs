//! CRDT-backed mana ledger for conflict-free distributed mana management.
//!
//! This module provides a ManaLedger implementation that uses CRDTs to ensure
//! conflict-free replication across multiple nodes. Each account's mana balance
//! is tracked using a PN-Counter CRDT, allowing concurrent operations across
//! nodes without conflicts.

use crate::{LedgerEvent, ManaLedger};
use icn_common::{CommonError, Did};
use icn_crdt::{CRDTMap, NodeId, PNCounter, CRDT};
use log::{debug, error, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, RwLock};

/// CRDT-backed mana ledger that enables conflict-free distributed mana management.
///
/// Uses a CRDT Map where each DID maps to a PN-Counter tracking mana balance.
/// This allows multiple nodes to concurrently update mana balances without conflicts.
pub struct CRDTManaLedger {
    /// Node identifier for this ledger instance.
    node_id: NodeId,
    /// CRDT Map storing DID -> PN-Counter mappings for mana balances.
    balance_map: Arc<RwLock<CRDTMap<String, PNCounter>>>,
    /// Optional event recording for auditability.
    event_store: Option<Arc<RwLock<Box<dyn icn_eventstore::EventStore<LedgerEvent>>>>>,
}

/// Configuration for CRDT mana ledger.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CRDTManaLedgerConfig {
    /// Node identifier for this ledger instance.
    pub node_id: String,
    /// Initial mana balances for accounts (for bootstrapping).
    pub initial_balances: HashMap<String, u64>,
    /// Whether to enable event store recording.
    pub enable_event_store: bool,
}

impl Default for CRDTManaLedgerConfig {
    fn default() -> Self {
        Self {
            node_id: "default_node".to_string(),
            initial_balances: HashMap::new(),
            enable_event_store: false,
        }
    }
}

impl CRDTManaLedger {
    /// Create a new CRDT mana ledger with the given configuration.
    pub fn new(config: CRDTManaLedgerConfig) -> Self {
        let node_id = NodeId::new(config.node_id);
        let balance_map = CRDTMap::new("mana_balances".to_string());

        let ledger = Self {
            node_id,
            balance_map: Arc::new(RwLock::new(balance_map)),
            event_store: None,
        };

        // Initialize with provided balances
        for (did_str, amount) in config.initial_balances {
            if let Ok(did) = Did::from_str(&did_str) {
                if let Err(e) = ledger.set_balance(&did, amount) {
                    warn!("Failed to set initial balance for {did_str}: {e}");
                }
            }
        }

        ledger
    }

    /// Create a new CRDT mana ledger with a specific node ID.
    pub fn with_node_id(node_id: String) -> Self {
        Self::new(CRDTManaLedgerConfig {
            node_id,
            ..Default::default()
        })
    }

    /// Add an event store for recording ledger operations.
    pub fn with_event_store(
        mut self,
        event_store: Box<dyn icn_eventstore::EventStore<LedgerEvent>>,
    ) -> Self {
        self.event_store = Some(Arc::new(RwLock::new(event_store)));
        self
    }

    /// Get the node ID for this ledger instance.
    pub fn node_id(&self) -> &NodeId {
        &self.node_id
    }

    /// Merge state from another CRDT mana ledger.
    ///
    /// This enables synchronization between distributed ledger instances.
    pub fn merge(&self, other: &Self) -> Result<(), CommonError> {
        let mut our_map = self
            .balance_map
            .write()
            .map_err(|_| CommonError::LockError("Failed to acquire write lock".to_string()))?;

        let other_map = other
            .balance_map
            .read()
            .map_err(|_| CommonError::LockError("Failed to acquire read lock".to_string()))?;

        our_map.merge(&*other_map);

        debug!("Merged CRDT mana ledger state from node {}", other.node_id);
        Ok(())
    }

    /// Get a snapshot of current balances for debugging or reporting.
    pub fn get_all_balances(&self) -> Result<HashMap<Did, u64>, CommonError> {
        let map = self
            .balance_map
            .read()
            .map_err(|_| CommonError::LockError("Failed to acquire read lock".to_string()))?;

        let mut balances = HashMap::new();

        for key in map.keys() {
            if let Ok(did) = Did::from_str(&key) {
                if let Some(counter) = map.get(&key) {
                    let balance = counter.get_total().max(0) as u64;
                    balances.insert(did, balance);
                }
            }
        }

        Ok(balances)
    }

    /// Get statistics about the CRDT ledger.
    pub fn get_stats(&self) -> Result<CRDTManaLedgerStats, CommonError> {
        let map = self
            .balance_map
            .read()
            .map_err(|_| CommonError::LockError("Failed to acquire read lock".to_string()))?;

        let account_count = map.len() as u64;
        let balances = self.get_all_balances()?;
        let total_mana = balances.values().sum::<u64>();
        let average_balance = if account_count > 0 {
            total_mana / account_count
        } else {
            0
        };

        let max_balance = balances.values().max().copied().unwrap_or(0);
        let min_balance = balances.values().min().copied().unwrap_or(0);

        Ok(CRDTManaLedgerStats {
            account_count,
            total_mana,
            average_balance,
            max_balance,
            min_balance,
            node_id: self.node_id.clone(),
        })
    }

    /// Record an event in the event store if configured.
    fn record_event(&self, event: LedgerEvent) {
        if let Some(event_store) = &self.event_store {
            if let Ok(mut store) = event_store.write() {
                if let Err(e) = store.append(&event) {
                    warn!("Failed to record ledger event: {e}");
                }
            }
        }
    }

    /// Get or create a PN-Counter for the given DID.
    fn get_or_create_counter(&self, did: &Did) -> Result<PNCounter, CommonError> {
        let mut map = self
            .balance_map
            .write()
            .map_err(|_| CommonError::LockError("Failed to acquire write lock".to_string()))?;

        let did_str = did.to_string();

        if let Some(counter) = map.get(&did_str) {
            Ok(counter.clone())
        } else {
            // Create new counter for this DID
            let counter_id = format!("mana_{did_str}");
            let counter = PNCounter::new(counter_id);

            map.put(did_str, counter.clone(), self.node_id.clone())
                .map_err(|e| CommonError::CRDTError(format!("Failed to create counter: {e}")))?;

            debug!("Created new mana counter for DID: {did}");
            Ok(counter)
        }
    }

    /// Update a counter in the map after modification.
    fn update_counter(&self, did: &Did, counter: PNCounter) -> Result<(), CommonError> {
        let mut map = self
            .balance_map
            .write()
            .map_err(|_| CommonError::LockError("Failed to acquire write lock".to_string()))?;

        let did_str = did.to_string();
        map.put(did_str, counter, self.node_id.clone())
            .map_err(|e| CommonError::CRDTError(format!("Failed to update counter: {e}")))?;

        Ok(())
    }
}

/// Statistics about a CRDT mana ledger.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CRDTManaLedgerStats {
    /// Number of accounts in the ledger.
    pub account_count: u64,
    /// Total mana across all accounts.
    pub total_mana: u64,
    /// Average mana balance per account.
    pub average_balance: u64,
    /// Highest balance in the ledger.
    pub max_balance: u64,
    /// Lowest balance in the ledger.
    pub min_balance: u64,
    /// Node ID of this ledger instance.
    pub node_id: NodeId,
}

impl Clone for CRDTManaLedger {
    fn clone(&self) -> Self {
        Self {
            node_id: self.node_id.clone(),
            balance_map: self.balance_map.clone(),
            event_store: None, // Event store is not cloneable
        }
    }
}

impl std::fmt::Debug for CRDTManaLedger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CRDTManaLedger")
            .field("node_id", &self.node_id)
            .field("balance_map", &"<CRDTMap>")
            .field("event_store", &self.event_store.is_some())
            .finish()
    }
}

impl ManaLedger for CRDTManaLedger {
    fn get_balance(&self, did: &Did) -> u64 {
        match self.balance_map.read() {
            Ok(map) => {
                let did_str = did.to_string();
                if let Some(counter) = map.get(&did_str) {
                    counter.get_total().max(0) as u64
                } else {
                    0
                }
            }
            Err(_) => {
                error!("Failed to acquire read lock for balance check");
                0
            }
        }
    }

    fn set_balance(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
        debug!("Setting balance for DID {did} to {amount}");

        let current_balance = self.get_balance(did);

        if amount > current_balance {
            // Need to credit the difference
            let credit_amount = amount - current_balance;
            self.credit(did, credit_amount)?;
        } else if amount < current_balance {
            // Need to spend the difference
            let spend_amount = current_balance - amount;
            self.spend(did, spend_amount)?;
        }
        // If amounts are equal, no operation needed

        self.record_event(LedgerEvent::SetBalance {
            did: did.clone(),
            amount,
        });

        Ok(())
    }

    fn spend(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
        if amount == 0 {
            return Ok(());
        }

        debug!("Spending {amount} mana for DID {did}");

        // Check if we have sufficient balance
        let current_balance = self.get_balance(did);
        if current_balance < amount {
            return Err(CommonError::InsufficientFunds(format!(
                "Insufficient mana: has {current_balance}, needs {amount}"
            )));
        }

        // Get the counter and perform decrement
        let mut counter = self.get_or_create_counter(did)?;
        counter
            .decrement(&self.node_id, amount)
            .map_err(|e| CommonError::CRDTError(format!("Failed to decrement counter: {e}")))?;

        // Update the counter in the map
        self.update_counter(did, counter)?;

        self.record_event(LedgerEvent::Debit {
            did: did.clone(),
            amount,
        });

        debug!("Successfully spent {amount} mana for DID {did}");
        Ok(())
    }

    fn credit(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
        if amount == 0 {
            return Ok(());
        }

        debug!("Crediting {amount} mana to DID {did}");

        // Get the counter and perform increment
        let mut counter = self.get_or_create_counter(did)?;
        counter
            .increment(&self.node_id, amount)
            .map_err(|e| CommonError::CRDTError(format!("Failed to increment counter: {e}")))?;

        // Update the counter in the map
        self.update_counter(did, counter)?;

        self.record_event(LedgerEvent::Credit {
            did: did.clone(),
            amount,
        });

        debug!("Successfully credited {amount} mana to DID {did}");
        Ok(())
    }

    fn credit_all(&self, amount: u64) -> Result<(), CommonError> {
        if amount == 0 {
            return Ok(());
        }

        debug!("Crediting {amount} mana to all accounts");

        let accounts = self.all_accounts();
        for did in accounts {
            self.credit(&did, amount)?;
        }

        debug!(
            "Successfully credited {} mana to {} accounts",
            amount,
            self.all_accounts().len()
        );
        Ok(())
    }

    fn all_accounts(&self) -> Vec<Did> {
        match self.balance_map.read() {
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
    fn test_crdt_mana_ledger_creation() {
        let ledger = CRDTManaLedger::with_node_id("test_node".to_string());
        assert_eq!(ledger.node_id().as_str(), "test_node");
        assert_eq!(ledger.get_balance(&alice_did()), 0);
    }

    #[test]
    fn test_crdt_mana_ledger_initial_balances() {
        let mut initial_balances = HashMap::new();
        initial_balances.insert("did:key:alice".to_string(), 100);
        initial_balances.insert("did:key:bob".to_string(), 50);

        let config = CRDTManaLedgerConfig {
            node_id: "test_node".to_string(),
            initial_balances,
            enable_event_store: false,
        };

        let ledger = CRDTManaLedger::new(config);
        assert_eq!(ledger.get_balance(&alice_did()), 100);
        assert_eq!(ledger.get_balance(&bob_did()), 50);
    }

    #[test]
    fn test_crdt_mana_ledger_credit_spend() {
        let ledger = CRDTManaLedger::with_node_id("test_node".to_string());

        // Credit some mana
        ledger.credit(&alice_did(), 100).unwrap();
        assert_eq!(ledger.get_balance(&alice_did()), 100);

        // Spend some mana
        ledger.spend(&alice_did(), 30).unwrap();
        assert_eq!(ledger.get_balance(&alice_did()), 70);

        // Try to spend more than available
        let result = ledger.spend(&alice_did(), 100);
        assert!(result.is_err());
        assert_eq!(ledger.get_balance(&alice_did()), 70);
    }

    #[test]
    fn test_crdt_mana_ledger_set_balance() {
        let ledger = CRDTManaLedger::with_node_id("test_node".to_string());

        // Set initial balance
        ledger.set_balance(&alice_did(), 100).unwrap();
        assert_eq!(ledger.get_balance(&alice_did()), 100);

        // Increase balance
        ledger.set_balance(&alice_did(), 150).unwrap();
        assert_eq!(ledger.get_balance(&alice_did()), 150);

        // Decrease balance
        ledger.set_balance(&alice_did(), 75).unwrap();
        assert_eq!(ledger.get_balance(&alice_did()), 75);
    }

    #[test]
    fn test_crdt_mana_ledger_merge() {
        let ledger1 = CRDTManaLedger::with_node_id("node1".to_string());
        let ledger2 = CRDTManaLedger::with_node_id("node2".to_string());

        // Each ledger credits different amounts to Alice
        ledger1.credit(&alice_did(), 50).unwrap();
        ledger2.credit(&alice_did(), 30).unwrap();

        // Bob gets mana only in ledger2
        ledger2.credit(&bob_did(), 20).unwrap();

        // Verify initial state
        assert_eq!(ledger1.get_balance(&alice_did()), 50);
        assert_eq!(ledger1.get_balance(&bob_did()), 0);
        assert_eq!(ledger2.get_balance(&alice_did()), 30);
        assert_eq!(ledger2.get_balance(&bob_did()), 20);

        // Merge ledger2 into ledger1
        ledger1.merge(&ledger2).unwrap();

        // After merge, Alice should have 80 (50 + 30) and Bob should have 20
        assert_eq!(ledger1.get_balance(&alice_did()), 80);
        assert_eq!(ledger1.get_balance(&bob_did()), 20);
    }

    #[test]
    fn test_crdt_mana_ledger_credit_all() {
        let ledger = CRDTManaLedger::with_node_id("test_node".to_string());

        // Set up some accounts
        ledger.set_balance(&alice_did(), 10).unwrap();
        ledger.set_balance(&bob_did(), 20).unwrap();

        // Credit all accounts
        ledger.credit_all(5).unwrap();

        assert_eq!(ledger.get_balance(&alice_did()), 15);
        assert_eq!(ledger.get_balance(&bob_did()), 25);
    }

    #[test]
    fn test_crdt_mana_ledger_all_accounts() {
        let ledger = CRDTManaLedger::with_node_id("test_node".to_string());

        // Initially no accounts
        assert_eq!(ledger.all_accounts().len(), 0);

        // Add some accounts
        ledger.credit(&alice_did(), 10).unwrap();
        ledger.credit(&bob_did(), 20).unwrap();

        let accounts = ledger.all_accounts();
        assert_eq!(accounts.len(), 2);
        assert!(accounts.contains(&alice_did()));
        assert!(accounts.contains(&bob_did()));
    }

    #[test]
    fn test_crdt_mana_ledger_stats() {
        let ledger = CRDTManaLedger::with_node_id("test_node".to_string());

        ledger.credit(&alice_did(), 100).unwrap();
        ledger.credit(&bob_did(), 200).unwrap();

        let stats = ledger.get_stats().unwrap();
        assert_eq!(stats.account_count, 2);
        assert_eq!(stats.total_mana, 300);
        assert_eq!(stats.average_balance, 150);
        assert_eq!(stats.max_balance, 200);
        assert_eq!(stats.min_balance, 100);
        assert_eq!(stats.node_id.as_str(), "test_node");
    }

    #[test]
    fn test_crdt_mana_ledger_concurrent_operations() {
        let ledger1 = CRDTManaLedger::with_node_id("node1".to_string());
        let ledger2 = CRDTManaLedger::with_node_id("node2".to_string());

        // Simulate concurrent operations on the same account
        ledger1.credit(&alice_did(), 50).unwrap();
        ledger1.spend(&alice_did(), 10).unwrap(); // Alice now has 40 in ledger1

        ledger2.credit(&alice_did(), 30).unwrap();
        ledger2.spend(&alice_did(), 5).unwrap(); // Alice now has 25 in ledger2

        // Before merge
        assert_eq!(ledger1.get_balance(&alice_did()), 40);
        assert_eq!(ledger2.get_balance(&alice_did()), 25);

        // Merge the states
        ledger1.merge(&ledger2).unwrap();

        // After merge: Alice should have total credits (50 + 30) - total debits (10 + 5) = 65
        assert_eq!(ledger1.get_balance(&alice_did()), 65);
    }
}
