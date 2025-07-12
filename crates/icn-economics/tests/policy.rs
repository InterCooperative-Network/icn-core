use icn_common::{CommonError, Did};
use icn_economics::{ledger::FileManaLedger, ManaRepositoryAdapter, ResourcePolicyEnforcer};
use std::str::FromStr;
use tempfile::tempdir;

#[test]
fn file_ledger_basic_ops() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("mana.json");
    let ledger = FileManaLedger::new(path.clone()).unwrap();
    let alice = Did::from_str("did:example:alice").unwrap();

    ledger.set_balance(&alice, 50).unwrap();
    ledger.credit(&alice, 10).unwrap();
    ledger.spend(&alice, 20).unwrap();

    drop(ledger);
    let ledger2 = FileManaLedger::new(path).unwrap();
    assert_eq!(ledger2.get_balance(&alice), 40);
}

#[test]
fn policy_enforcer_spend_success() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("mana.json");
    let ledger = FileManaLedger::new(path).unwrap();
    let did = Did::from_str("did:example:alice").unwrap();
    ledger.set_balance(&did, 150).unwrap();

    let adapter = ManaRepositoryAdapter::new(ledger);
    let enforcer = ResourcePolicyEnforcer::new(adapter);
    let result = enforcer.spend_mana(&did, 100);
    assert!(result.is_ok());
}

#[test]
fn policy_enforcer_insufficient_balance() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("mana.json");
    let ledger = FileManaLedger::new(path).unwrap();
    let did = Did::from_str("did:example:bob").unwrap();
    ledger.set_balance(&did, 20).unwrap();

    let adapter = ManaRepositoryAdapter::new(ledger);
    let enforcer = ResourcePolicyEnforcer::new(adapter);
    let result = enforcer.spend_mana(&did, 30);
    assert!(matches!(result, Err(CommonError::PolicyDenied(_))));
}

#[test]
fn policy_enforcer_exceeds_limit() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("mana.json");
    let ledger = FileManaLedger::new(path).unwrap();
    let did = Did::from_str("did:example:carol").unwrap();
    ledger.set_balance(&did, 5000).unwrap();

    let adapter = ManaRepositoryAdapter::new(ledger);
    let enforcer = ResourcePolicyEnforcer::new(adapter);
    let over_limit = ResourcePolicyEnforcer::<FileManaLedger>::MAX_SPEND_LIMIT + 1;
    let result = enforcer.spend_mana(&did, over_limit);
    assert!(matches!(result, Err(CommonError::PolicyDenied(_))));
}

