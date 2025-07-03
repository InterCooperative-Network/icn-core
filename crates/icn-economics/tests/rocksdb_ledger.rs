#[cfg(feature = "persist-rocksdb")]
mod tests {
    use icn_common::Did;
    use icn_economics::ledger::RocksdbManaLedger;
    use std::str::FromStr;
    use tempfile::tempdir;

    #[test]
    fn rocksdb_credit_all_persists() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("mana.rocks");
        let ledger = RocksdbManaLedger::new(path.clone()).unwrap();
        let alice = Did::from_str("did:example:alice").unwrap();
        let bob = Did::from_str("did:example:bob").unwrap();
        ledger.set_balance(&alice, 1).unwrap();
        ledger.set_balance(&bob, 2).unwrap();
        ledger.credit_all(5).unwrap();
        drop(ledger);
        let ledger2 = RocksdbManaLedger::new(path).unwrap();
        assert_eq!(ledger2.get_balance(&alice), 6);
        assert_eq!(ledger2.get_balance(&bob), 7);
    }
}
