#[cfg(feature = "persist-rocksdb")]
mod persistence_rocksdb {
    use icn_common::{compute_merkle_cid, DagBlock, Did};
    use icn_runtime::context::{RuntimeContext, StubMeshNetworkService, StubSigner};
    use std::sync::Arc;
    use tempfile::tempdir;

    fn sample_block() -> DagBlock {
        let data = b"hello".to_vec();
        let timestamp = 0u64;
        let author = Did::new("key", "tester");
        let sig = None;
        let cid = compute_merkle_cid(0x71, &data, &[], timestamp, &author, &None, &sig);
        DagBlock {
            cid,
            data,
            links: vec![],
            timestamp,
            author_did: author,
            scope: None,
            signature: sig,
        }
    }

    #[tokio::test]
    async fn ledger_and_dag_survive_restart() {
        let dir = tempdir().unwrap();
        let dag_path = dir.path().join("dag.rocks");
        let mana_path = dir.path().join("mana.rocks");
        let rep_path = dir.path().join("rep.rocks");

        let id = Did::new("key", "tester");
        let ctx1 = RuntimeContext::new_with_paths(
            id.clone(),
            Arc::new(StubMeshNetworkService::new()),
            Arc::new(StubSigner::new()),
            Arc::new(icn_identity::KeyDidResolver),
            dag_path.clone(),
            mana_path.clone(),
            rep_path.clone(),
        )
        .unwrap();

        ctx1.credit_mana(&id, 42).await.unwrap();
        let block = sample_block();
        ctx1.dag_store.lock().await.put(&block).unwrap();
        drop(ctx1);

        let ctx2 = RuntimeContext::new_with_paths(
            id.clone(),
            Arc::new(StubMeshNetworkService::new()),
            Arc::new(StubSigner::new()),
            Arc::new(icn_identity::KeyDidResolver),
            dag_path,
            mana_path,
            rep_path,
        )
        .unwrap();

        assert_eq!(ctx2.mana_ledger.get_balance(&id), 42);
        assert!(ctx2
            .dag_store
            .lock()
            .await
            .get(&block.cid)
            .unwrap()
            .is_some());
    }
}
