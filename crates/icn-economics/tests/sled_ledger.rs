#[cfg(feature = "persist-sled")]
mod tests {
    use icn_common::Did;
    use icn_economics::ledger::SledManaLedger;
    use std::str::FromStr;
    use tempfile::tempdir;

    #[test]
    fn sled_ledger_credit_all_persists() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("mana.sled");
        let ledger = SledManaLedger::new(path.clone()).unwrap();
        let alice = Did::from_str("did:example:alice").unwrap();
        let bob = Did::from_str("did:example:bob").unwrap();
        let charlie = Did::from_str("did:example:charlie").unwrap();
        ledger.set_balance(&alice, 5).unwrap();
        ledger.set_balance(&bob, 7).unwrap();
        ledger.set_balance(&charlie, 0).unwrap();
        ledger.credit_all(3).unwrap();
        drop(ledger);
        let ledger2 = SledManaLedger::new(path).unwrap();
        assert_eq!(ledger2.get_balance(&alice), 8);
        assert_eq!(ledger2.get_balance(&bob), 10);
        assert_eq!(ledger2.get_balance(&charlie), 3);
    }
}
