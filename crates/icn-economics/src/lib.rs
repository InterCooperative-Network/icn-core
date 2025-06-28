#![doc = include_str!("../README.md")]

//! # ICN Economics Crate
//! This crate handles the economic protocols of the InterCooperative Network (ICN).
//! It manages token models, ledger interactions, transaction logic, and incentive mechanisms,
//! aiming for security, accuracy, and interoperability.

use icn_common::{CommonError, Did, NodeInfo};
mod ledger;
pub use ledger::{FileManaLedger, SledManaLedger};

/// Errors that can occur during mana accounting operations.
#[derive(Debug)]
pub enum EconError {
    InsufficientBalance(String),
    AdapterError(String),
    PolicyViolation(String),
}

/// Abstraction over the persistence layer storing account balances.
pub trait ManaLedger: Send + Sync {
    /// Retrieve the mana balance for a DID.
    fn get_balance(&self, did: &Did) -> u64;
    /// Persist a new balance for a DID.
    fn set_balance(&self, did: &Did, amount: u64) -> Result<(), CommonError>;
    /// Spend mana from the account.
    fn spend(&self, did: &Did, amount: u64) -> Result<(), EconError>;
    /// Credit mana to the account.
    fn credit(&self, did: &Did, amount: u64) -> Result<(), EconError>;
}

/// Thin wrapper exposing convenience methods over a [`ManaLedger`].
#[derive(Debug)]
pub struct ManaRepositoryAdapter<L: ManaLedger> {
    ledger: L,
}

impl<L: ManaLedger> ManaRepositoryAdapter<L> {
    /// Construct a new adapter around the provided ledger implementation.
    pub fn new(ledger: L) -> Self {
        ManaRepositoryAdapter { ledger }
    }

    /// Deduct mana from an account via the underlying ledger.
    pub fn spend_mana(&self, did: &Did, amount: u64) -> Result<(), EconError> {
        self.ledger.spend(did, amount)
    }

    /// Retrieve the account balance.
    pub fn get_balance(&self, did: &Did) -> u64 {
        self.ledger.get_balance(did)
    }

    /// Credits the specified account with additional mana.
    pub fn credit_mana(&self, did: &Did, amount: u64) -> Result<(), EconError> {
        self.ledger.credit(did, amount)
    }
}

/// Enforces spending limits and forwards to a [`ManaRepositoryAdapter`].
#[derive(Debug)]
pub struct ResourcePolicyEnforcer<L: ManaLedger> {
    adapter: ManaRepositoryAdapter<L>,
}

impl<L: ManaLedger> ResourcePolicyEnforcer<L> {
    /// Maximum mana spend allowed per single operation.
    pub const MAX_SPEND_LIMIT: u64 = 1000;

    /// Create a new enforcer using the supplied adapter.
    pub fn new(adapter: ManaRepositoryAdapter<L>) -> Self {
        ResourcePolicyEnforcer { adapter }
    }

    /// Spend mana after applying basic policy checks.
    pub fn spend_mana(&self, did: &Did, amount: u64) -> Result<(), EconError> {
        println!("[ResourcePolicyEnforcer] Enforcing spend_mana for DID {did:?}, amount {amount}");

        if amount == 0 {
            return Err(EconError::PolicyViolation(
                "Spend amount must be greater than zero".into(),
            ));
        }

        let available = self.adapter.get_balance(did);
        if available < amount {
            return Err(EconError::InsufficientBalance(format!(
                "Insufficient mana for DID {did}"
            )));
        }

        if amount > Self::MAX_SPEND_LIMIT {
            return Err(EconError::PolicyViolation(format!(
                "Spend amount {amount} exceeds limit {limit}",
                limit = Self::MAX_SPEND_LIMIT
            )));
        }

        self.adapter.spend_mana(did, amount)
    }
}

/// Exposes a public function to charge mana, wrapping ResourcePolicyEnforcer.
pub fn charge_mana<L: ManaLedger>(ledger: L, did: &Did, amount: u64) -> Result<(), EconError> {
    let mana_adapter = ManaRepositoryAdapter::new(ledger);
    let policy_enforcer = ResourcePolicyEnforcer::new(mana_adapter);

    println!("[icn-economics] charge_mana called for DID {did:?}, amount {amount}");
    policy_enforcer.spend_mana(did, amount)
}

/// Placeholder function demonstrating use of common types for economics.
pub fn process_economic_event(info: &NodeInfo, event_details: &str) -> Result<String, CommonError> {
    Ok(format!(
        "Processed economic event '{} ' for node: {} (v{})",
        event_details, info.name, info.version
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::ICN_CORE_VERSION;
    use std::str::FromStr;
    use tempfile::tempdir;

    #[test]
    fn test_process_economic_event() {
        let node_info = NodeInfo {
            name: "EcoNode".to_string(),
            version: ICN_CORE_VERSION.to_string(),
            status_message: "Economics active".to_string(),
        };
        let result = process_economic_event(&node_info, "test_transaction");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("test_transaction"));
    }

    #[test]
    fn test_sled_mana_ledger_persistence() {
        let dir = tempdir().unwrap();
        let ledger_path = dir.path().join("mana.sled");
        let ledger = SledManaLedger::new(ledger_path.clone()).unwrap();
        let did = Did::from_str("did:example:alice").unwrap();
        ledger.set_balance(&did, 50).unwrap();
        ledger.credit(&did, 20).unwrap();
        ledger.spend(&did, 30).unwrap();
        assert_eq!(ledger.get_balance(&did), 40);
        drop(ledger);

        let ledger2 = SledManaLedger::new(ledger_path).unwrap();
        assert_eq!(ledger2.get_balance(&did), 40);
    }

    #[test]
    fn test_resource_policy_enforcer_spend_success() {
        let dir = tempdir().unwrap();
        let ledger_path = dir.path().join("mana.sled");
        let ledger = SledManaLedger::new(ledger_path.clone()).unwrap();
        let did = Did::from_str("did:example:alice").unwrap();
        ledger.set_balance(&did, 150).unwrap();

        let adapter = ManaRepositoryAdapter::new(ledger);
        let enforcer = ResourcePolicyEnforcer::new(adapter);
        let result = enforcer.spend_mana(&did, 100);
        assert!(result.is_ok());

        drop(enforcer); // release Sled DB before reopening
        let ledger_check = SledManaLedger::new(ledger_path).unwrap();
        assert_eq!(ledger_check.get_balance(&did), 50);
    }

    #[test]
    fn test_resource_policy_enforcer_insufficient_balance() {
        let dir = tempdir().unwrap();
        let ledger_path = dir.path().join("mana.sled");
        let ledger = SledManaLedger::new(ledger_path).unwrap();
        let did = Did::from_str("did:example:bob").unwrap();
        ledger.set_balance(&did, 20).unwrap();

        let adapter = ManaRepositoryAdapter::new(ledger);
        let enforcer = ResourcePolicyEnforcer::new(adapter);
        let result = enforcer.spend_mana(&did, 30);
        assert!(matches!(result, Err(EconError::InsufficientBalance(_))));
    }

    #[test]
    fn test_resource_policy_enforcer_exceeds_limit() {
        let dir = tempdir().unwrap();
        let ledger_path = dir.path().join("mana.sled");
        let ledger = SledManaLedger::new(ledger_path).unwrap();
        let did = Did::from_str("did:example:carol").unwrap();
        ledger.set_balance(&did, 5000).unwrap();

        let adapter = ManaRepositoryAdapter::new(ledger);
        let enforcer = ResourcePolicyEnforcer::new(adapter);
        let over_limit = ResourcePolicyEnforcer::<SledManaLedger>::MAX_SPEND_LIMIT + 1;
        let result = enforcer.spend_mana(&did, over_limit);
        assert!(matches!(result, Err(EconError::PolicyViolation(_))));
    }
}
