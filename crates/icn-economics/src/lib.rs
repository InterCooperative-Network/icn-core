#![doc = include_str!("../README.md")]

//! # ICN Economics Crate
//! This crate handles the economic protocols of the InterCooperative Network (ICN).
//! It manages token models, ledger interactions, transaction logic, and incentive mechanisms,
//! aiming for security, accuracy, and interoperability.

use icn_common::{CommonError, Did, NodeInfo};
mod ledger;
pub use ledger::FileManaLedger;

// Placeholder for EconError enum
#[derive(Debug)]
pub enum EconError {
    InsufficientBalance(String),
    AdapterError(String),
    PolicyViolation(String),
}

pub trait ManaLedger: Send + Sync {
    fn get_balance(&self, did: &Did) -> u64;
    fn set_balance(&self, did: &Did, amount: u64) -> Result<(), CommonError>;
    fn spend(&self, did: &Did, amount: u64) -> Result<(), EconError>;
    fn credit(&self, did: &Did, amount: u64) -> Result<(), EconError>;
}

#[derive(Debug)]
pub struct ManaRepositoryAdapter<L: ManaLedger> {
    ledger: L,
}

impl<L: ManaLedger> ManaRepositoryAdapter<L> {
    pub fn new(ledger: L) -> Self {
        ManaRepositoryAdapter { ledger }
    }

    pub fn spend_mana(&self, did: &Did, amount: u64) -> Result<(), EconError> {
        self.ledger.spend(did, amount)
    }
}

// Placeholder for ResourcePolicyEnforcer struct
#[derive(Debug)]
pub struct ResourcePolicyEnforcer<L: ManaLedger> {
    adapter: ManaRepositoryAdapter<L>,
}

impl<L: ManaLedger> ResourcePolicyEnforcer<L> {
    pub fn new(adapter: ManaRepositoryAdapter<L>) -> Self {
        ResourcePolicyEnforcer { adapter }
    }

    pub fn spend_mana(&self, did: &Did, amount: u64) -> Result<(), EconError> {
        println!(
            "[ResourcePolicyEnforcer STUB] Enforcing spend_mana for DID {did:?}, amount {amount}"
        );
        // TODO: Add actual policy logic here if needed before calling adapter
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
    fn test_file_mana_ledger_persistence() {
        let dir = tempdir().unwrap();
        let ledger_path = dir.path().join("mana.json");
        let ledger = FileManaLedger::new(ledger_path.clone()).unwrap();
        let did = Did::from_str("did:example:alice").unwrap();
        ledger.set_balance(&did, 50).unwrap();
        ledger.credit(&did, 20).unwrap();
        ledger.spend(&did, 30).unwrap();
        assert_eq!(ledger.get_balance(&did), 40);
        drop(ledger);

        let ledger2 = FileManaLedger::new(ledger_path).unwrap();
        assert_eq!(ledger2.get_balance(&did), 40);
    }
}
