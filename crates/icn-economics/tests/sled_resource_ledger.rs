#[cfg(feature = "persist-sled")]
mod tests {
    use icn_common::Did;
    use icn_economics::ledger::TokenClassId;
    use icn_economics::ledger::{ResourceLedger, SledResourceLedger, TokenClass};
    use std::str::FromStr;
    use tempfile::tempdir;

    #[test]
    fn sled_resource_ledger_persists() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("resources.sled");
        let ledger = SledResourceLedger::new(path.clone()).unwrap();
        let class_id: TokenClassId = "gold".to_string();
        ledger
            .create_class(
                &class_id,
                TokenClass {
                    name: "Gold".into(),
                    description: "Test gold token".into(),
                    symbol: "GLD".into(),
                    decimals: 2,
                    token_type: icn_economics::ledger::TokenType::Fungible,
                    transferability: icn_economics::ledger::TransferabilityRule::FreelyTransferable,
                    scoping_rules: icn_economics::ledger::ScopingRules {
                        geographic_scope: None,
                        community_scope: None,
                        validity_period: None,
                        max_supply: None,
                        min_balance: None,
                    },
                    issuer: Did::from_str("did:example:issuer").unwrap(),
                    created_at: 0,
                    metadata: std::collections::HashMap::new(),
                },
            )
            .unwrap();
        let alice = Did::from_str("did:example:alice").unwrap();
        let bob = Did::from_str("did:example:bob").unwrap();
        ledger.mint(&class_id, &alice, 10).unwrap();
        ledger.transfer(&class_id, &alice, &bob, 5).unwrap();
        drop(ledger);
        let ledger2 = SledResourceLedger::new(path).unwrap();
        assert_eq!(ledger2.get_balance(&class_id, &alice), 5);
        assert_eq!(ledger2.get_balance(&class_id, &bob), 5);
    }
}
