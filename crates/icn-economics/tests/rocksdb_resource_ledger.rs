#[cfg(feature = "persist-rocksdb")]
mod tests {
    use icn_common::Did;
    use icn_economics::ledger::{ResourceLedger, RocksdbResourceLedger, TokenClass, TokenClassId, TokenType, TransferabilityRule, ScopingRules};
    use std::str::FromStr;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tempfile::tempdir;

    #[test]
    fn rocksdb_resource_ledger_persists() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("resources.rocks");
        let ledger = RocksdbResourceLedger::new(path.clone()).unwrap();
        let class_id: TokenClassId = "gold".to_string();
        ledger
            .create_class(
                &class_id,
                TokenClass {
                    name: "Gold".into(),
                    description: "A precious metal token".into(),
                    symbol: "GOLD".into(),
                    decimals: 0,
                    token_type: TokenType::Fungible,
                    transferability: TransferabilityRule::FreelyTransferable,
                    scoping_rules: ScopingRules {
                        geographic_scope: None,
                        community_scope: None,
                        validity_period: None,
                        max_supply: None,
                        min_balance: None,
                    },
                    issuer: Did::from_str("did:example:issuer").unwrap(),
                    created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    metadata: Default::default(),
                },
            )
            .unwrap();
        let alice = Did::from_str("did:example:alice").unwrap();
        let bob = Did::from_str("did:example:bob").unwrap();
        ledger.mint(&class_id, &alice, 7).unwrap();
        ledger.transfer(&class_id, &alice, &bob, 3).unwrap();
        drop(ledger);
        let ledger2 = RocksdbResourceLedger::new(path).unwrap();
        assert_eq!(ledger2.get_balance(&class_id, &alice), 4);
        assert_eq!(ledger2.get_balance(&class_id, &bob), 3);
    }
}
