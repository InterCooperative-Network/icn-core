#[cfg(feature = "persist-sqlite")]
mod tests {
    use icn_common::Did;
    use icn_economics::ledger::TokenClassId;
    use icn_economics::ledger::{ResourceLedger, SqliteResourceLedger, TokenClass};
    use std::str::FromStr;
    use tempfile::tempdir;

    #[test]
    fn sqlite_resource_ledger_persists() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("resources.sqlite");
        let ledger = SqliteResourceLedger::new(path.clone()).unwrap();
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
        ledger.mint(&class_id, &alice, 20).unwrap();
        ledger.transfer(&class_id, &alice, &bob, 8).unwrap();
        drop(ledger);
        let ledger2 = SqliteResourceLedger::new(path).unwrap();
        assert_eq!(ledger2.get_balance(&class_id, &alice), 12);
        assert_eq!(ledger2.get_balance(&class_id, &bob), 8);
    }
}
