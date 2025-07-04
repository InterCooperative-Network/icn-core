#[cfg(feature = "persist-sqlite")]
mod tests {
    use icn_common::Did;
    use icn_economics::ledger::SqliteManaLedger;
    use std::str::FromStr;
    use tempfile::tempdir;

    #[test]
    fn sqlite_ledger_credit_all_persists() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("mana.sqlite");
        let ledger = SqliteManaLedger::new(path.clone()).unwrap();
        let alice = Did::from_str("did:example:alice").unwrap();
        let bob = Did::from_str("did:example:bob").unwrap();
        let charlie = Did::from_str("did:example:charlie").unwrap();
        ledger.set_balance(&alice, 10).unwrap();
        ledger.set_balance(&bob, 20).unwrap();
        ledger.set_balance(&charlie, 1).unwrap();
        ledger.credit_all(2).unwrap();
        drop(ledger);
        let ledger2 = SqliteManaLedger::new(path).unwrap();
        assert_eq!(ledger2.get_balance(&alice), 12);
        assert_eq!(ledger2.get_balance(&bob), 22);
        assert_eq!(ledger2.get_balance(&charlie), 3);
    }
}
