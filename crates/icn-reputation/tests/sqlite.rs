#[cfg(feature = "persist-sqlite")]
mod tests {
    use icn_common::Did;
    use icn_identity::{did_key_from_verifying_key, generate_ed25519_keypair};
    use icn_reputation::sqlite_store::SqliteReputationStore;
    use icn_reputation::AsyncReputationStore;
    use std::path::PathBuf;
    use std::str::FromStr;
    use tempfile::tempdir;

    #[tokio::test]
    async fn sqlite_round_trip() {
        let dir = tempdir().unwrap();
        let path: PathBuf = dir.path().join("rep.sqlite");
        let store = SqliteReputationStore::new(path.clone()).await.unwrap();

        let (_sk, vk) = generate_ed25519_keypair();
        let did = Did::from_str(&did_key_from_verifying_key(&vk)).unwrap();

        store.record_execution(&did, true, 1000).await;
        assert_eq!(store.get_reputation(&did).await, 2);
        store.record_proof_attempt(&did, true).await;
        assert_eq!(store.get_reputation(&did).await, 3);

        drop(store);
        let reopened = SqliteReputationStore::new(path).await.unwrap();
        assert_eq!(reopened.get_reputation(&did).await, 3);
    }
}
