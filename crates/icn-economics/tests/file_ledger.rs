use icn_common::Did;
use icn_economics::ledger::FileManaLedger;
use std::str::FromStr;
use tempfile::tempdir;

#[test]
fn file_ledger_spend_persists() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("mana.json");
    let ledger = FileManaLedger::new(path.clone()).unwrap();
    let did = Did::from_str("did:example:alice").unwrap();
    ledger.set_balance(&did, 100).unwrap();
    ledger.spend(&did, 40).unwrap();
    drop(ledger);
    let ledger2 = FileManaLedger::new(path).unwrap();
    assert_eq!(ledger2.get_balance(&did), 60);
}

#[test]
fn file_ledger_credit_persists() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("mana.json");
    let ledger = FileManaLedger::new(path.clone()).unwrap();
    let did = Did::from_str("did:example:bob").unwrap();
    ledger.set_balance(&did, 10).unwrap();
    ledger.credit(&did, 15).unwrap();
    drop(ledger);
    let ledger2 = FileManaLedger::new(path).unwrap();
    assert_eq!(ledger2.get_balance(&did), 25);
}

#[test]
fn file_ledger_credit_all_persists() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("mana.json");
    let ledger = FileManaLedger::new(path.clone()).unwrap();
    let alice = Did::from_str("did:example:alice").unwrap();
    let bob = Did::from_str("did:example:bob").unwrap();
    let charlie = Did::from_str("did:example:charlie").unwrap();
    ledger.set_balance(&alice, 5).unwrap();
    ledger.set_balance(&bob, 7).unwrap();
    ledger.set_balance(&charlie, 0).unwrap();
    ledger.credit_all(3).unwrap();
    drop(ledger);
    let ledger2 = FileManaLedger::new(path).unwrap();
    assert_eq!(ledger2.get_balance(&alice), 8);
    assert_eq!(ledger2.get_balance(&bob), 10);
    assert_eq!(ledger2.get_balance(&charlie), 3);
}
