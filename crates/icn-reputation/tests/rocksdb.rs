#[cfg(feature = "persist-rocksdb")]
mod tests {
    use icn_common::Did;
    use icn_identity::{did_key_from_verifying_key, generate_ed25519_keypair};
    use icn_reputation::rocksdb_store::RocksdbReputationStore;
    use icn_reputation::ReputationStore;
    use std::path::PathBuf;
    use std::str::FromStr;
    use tempfile::tempdir;

    #[test]
    fn rocksdb_round_trip() {
        let dir = tempdir().unwrap();
        let path: PathBuf = dir.path().join("rep.rocks");
        let store = RocksdbReputationStore::new(path.clone()).unwrap();

        let (_sk, vk) = generate_ed25519_keypair();
        let did = Did::from_str(&did_key_from_verifying_key(&vk)).unwrap();

        store.record_execution(&did, true, 1000);
        assert_eq!(store.get_reputation(&did), 2);
        store.record_proof_attempt(&did, true);
        assert_eq!(store.get_reputation(&did), 3);

        drop(store);
        let reopened = RocksdbReputationStore::new(path).unwrap();
        assert_eq!(reopened.get_reputation(&did), 3);
    }
}
