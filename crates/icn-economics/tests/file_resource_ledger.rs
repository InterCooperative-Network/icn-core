use icn_common::Did;
use icn_economics::ledger::TokenClassId;
use icn_economics::ledger::{FileResourceLedger, ResourceLedger, TokenClass};
use std::str::FromStr;
use tempfile::tempdir;

#[test]
fn file_resource_ledger_persists() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("resources.json");
    let ledger = FileResourceLedger::new(path.clone()).unwrap();
    let class_id: TokenClassId = "gold".to_string();
    ledger
        .create_class(
            &class_id,
            TokenClass {
                name: "Gold".into(),
            },
        )
        .unwrap();
    let alice = Did::from_str("did:example:alice").unwrap();
    let bob = Did::from_str("did:example:bob").unwrap();
    ledger.mint(&class_id, &alice, 50).unwrap();
    ledger.transfer(&class_id, &alice, &bob, 20).unwrap();
    drop(ledger);
    let ledger2 = FileResourceLedger::new(path).unwrap();
    assert_eq!(ledger2.get_balance(&class_id, &alice), 30);
    assert_eq!(ledger2.get_balance(&class_id, &bob), 20);
}
