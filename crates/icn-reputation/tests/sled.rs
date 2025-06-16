#[cfg(feature = "persist-sled")]
mod tests {
    use icn_common::{Cid, Did};
    use icn_identity::{
        did_key_from_verifying_key, generate_ed25519_keypair, ExecutionReceipt, SignatureBytes,
    };
    use icn_reputation::{sled_store::SledReputationStore, ReputationStore};
    use std::str::FromStr;
    use tempfile::tempdir;

    #[test]
    fn sled_persists_score() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_path_buf();
        let store = SledReputationStore::new(path.clone()).unwrap();

        let (_sk, vk) = generate_ed25519_keypair();
        let did = Did::from_str(&did_key_from_verifying_key(&vk)).unwrap();
        let receipt = ExecutionReceipt {
            job_id: Cid::new_v1_dummy(0x55, 0x12, b"r"),
            executor_did: did.clone(),
            result_cid: Cid::new_v1_dummy(0x55, 0x12, b"r"),
            cpu_ms: 0,
            sig: SignatureBytes(vec![]),
        };
        store.record_receipt(&receipt);
        assert_eq!(store.get_reputation(&did), 1);
        drop(store);

        let store2 = SledReputationStore::new(path).unwrap();
        assert_eq!(store2.get_reputation(&did), 1);
    }
}
