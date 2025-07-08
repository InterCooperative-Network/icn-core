#[cfg(all(feature = "persist-rocksdb", feature = "enable-libp2p"))]
mod persistence_rocksdb {
    use icn_common::{compute_merkle_cid, DagBlock, Did};
    use icn_network::libp2p_service::{Libp2pNetworkService, NetworkConfig};
    use icn_network::NetworkService;
    use icn_runtime::context::{DefaultMeshNetworkService, RuntimeContext, StubSigner};
    use std::path::PathBuf;
    use std::sync::Arc;
    use tempfile::tempdir;

    fn sample_block() -> DagBlock {
        let data = b"hello".to_vec();
        let timestamp = 0u64;
        let author = Did::new("key", "tester");
        let sig = None;
        let cid = compute_merkle_cid(0x71, &data, &[], timestamp, &author, &sig, &None);
        DagBlock {
            cid,
            data,
            links: vec![],
            timestamp,
            author_did: author,
            signature: sig,
            scope: None,
        }
    }

    async fn create_ctx(
        id: Did,
        dag: PathBuf,
        mana: PathBuf,
        rep: PathBuf,
    ) -> Arc<RuntimeContext> {
        let service = Arc::new(
            Libp2pNetworkService::new(NetworkConfig::default())
                .await
                .unwrap(),
        ) as Arc<dyn NetworkService>;
        let mesh = Arc::new(DefaultMeshNetworkService::new(service));
        RuntimeContext::new_with_paths(
            id,
            mesh,
            Arc::new(StubSigner::new().unwrap()),
            Arc::new(icn_identity::KeyDidResolver),
            dag,
            mana,
            rep,
        )
        .unwrap()
    }

    #[tokio::test]
    async fn ledger_and_dag_survive_restart() {
        let dir = tempdir().unwrap();
        let dag_path = dir.path().join("dag.rocks");
        let mana_path = dir.path().join("mana.rocks");
        let rep_path = dir.path().join("rep.rocks");

        let id = Did::new("key", "tester");
        let ctx1 = create_ctx(id.clone(), dag_path.clone(), mana_path.clone(), rep_path.clone()).await;

        ctx1.credit_mana(&id, 42).await.unwrap();
        let block = sample_block();
        ctx1.dag_store.lock().await.put(&block).unwrap();
        drop(ctx1);

        let ctx2 = create_ctx(id.clone(), dag_path, mana_path, rep_path).await;

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

#[cfg(all(feature = "persist-rocksdb", not(feature = "enable-libp2p")))]
#[tokio::test]
async fn libp2p_feature_disabled_stub() {
    println!("libp2p feature disabled; skipping persistence test");
}
