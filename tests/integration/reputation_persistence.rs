#[cfg(feature = "persist-sled")]
mod reputation_persistence {
    use icn_common::{Cid, Did};
    use icn_identity::{did_key_from_verifying_key, generate_ed25519_keypair, SignatureBytes};
    use icn_mesh::{select_executor, MeshJobBid, JobSpec, Resources, SelectionPolicy};
    use icn_reputation::sled_store::SledReputationStore;
    use icn_reputation::ReputationStore;
    use icn_runtime::{
        context::{RuntimeContext, SimpleManaLedger, StubMeshNetworkService, StubSigner},
        host_anchor_receipt, ReputationUpdater,
    };
    use std::str::FromStr;
    use tempfile::tempdir;

    #[test]
    fn persisted_reputation_influences_selection() {
        let dir = tempdir().unwrap();
        let rep_path = dir.path().join("rep.sled");
        let mana_path = dir.path().join("mana.sled");

        let ledger = SimpleManaLedger::new(mana_path);
        let store = SledReputationStore::new(rep_path.clone()).unwrap();

        let (_ska, vka) = generate_ed25519_keypair();
        let did_a = Did::from_str(&did_key_from_verifying_key(&vka)).unwrap();
        let (_skb, vkb) = generate_ed25519_keypair();
        let did_b = Did::from_str(&did_key_from_verifying_key(&vkb)).unwrap();

        ledger.set_balance(&did_a, 100).unwrap();
        ledger.set_balance(&did_b, 100).unwrap();

        store.record_execution(&did_a, true, 1000);
        drop(store);

        let reopened = SledReputationStore::new(rep_path).unwrap();

        let job_id = Cid::new_v1_sha256(0x55, b"rep_job");
        let bid_a = MeshJobBid {
            job_id: job_id.clone(),
            executor_did: did_a.clone(),
            price_mana: 10,
            resources: Resources { cpu_cores: 1, memory_mb: 10 },
            signature: SignatureBytes(Vec::new()),
        };
        let bid_b = MeshJobBid {
            job_id: job_id.clone(),
            executor_did: did_b.clone(),
            price_mana: 10,
            resources: Resources { cpu_cores: 1, memory_mb: 10 },
            signature: SignatureBytes(Vec::new()),
        };

        let selected = select_executor(
            &job_id,
            &JobSpec::Echo { payload: "persist".into() },
            vec![bid_a, bid_b],
            &SelectionPolicy::default(),
            &reopened,
            &ledger,
        )
        .expect("executor selected");

        assert_eq!(selected, did_a);
    }

    #[tokio::test]
    async fn reputation_survives_restart() {
        let dir = tempdir().unwrap();
        let dag_path = dir.path().join("dag.sled");
        let mana_path = dir.path().join("mana.sled");
        let rep_path = dir.path().join("rep.sled");

        let did = Did::new("key", "tester");
        let ctx1 = RuntimeContext::new_with_paths(
            did.clone(),
            Arc::new(StubMeshNetworkService::new()),
            Arc::new(StubSigner::new()),
            Arc::new(icn_identity::KeyDidResolver),
            dag_path.clone(),
            mana_path.clone(),
            rep_path.clone(),
        )
        .unwrap();

        let mut receipt = icn_identity::ExecutionReceipt {
            job_id: Cid::new_v1_sha256(0x55, b"r"),
            executor_did: did.clone(),
            result_cid: Cid::new_v1_sha256(0x55, b"r"),
            cpu_ms: 1000,
            success: true,
            sig: SignatureBytes(Vec::new()),
        };

        let mut msg = Vec::new();
        msg.extend_from_slice(receipt.job_id.to_string().as_bytes());
        msg.extend_from_slice(did.to_string().as_bytes());
        msg.extend_from_slice(receipt.result_cid.to_string().as_bytes());
        msg.extend_from_slice(&receipt.cpu_ms.to_le_bytes());
        msg.push(receipt.success as u8);
        let sig = ctx1.signer.sign(&msg).unwrap();
        receipt.sig = SignatureBytes(sig);

        let json = serde_json::to_string(&receipt).unwrap();
        host_anchor_receipt(&ctx1, &json, &ReputationUpdater::new())
            .await
            .unwrap();
        drop(ctx1);

        let ctx2 = RuntimeContext::new_with_paths(
            did.clone(),
            Arc::new(StubMeshNetworkService::new()),
            Arc::new(StubSigner::new()),
            Arc::new(icn_identity::KeyDidResolver),
            dag_path,
            mana_path,
            rep_path,
        )
        .unwrap();

        assert_eq!(ctx2.reputation_store.get_reputation(&did), 2);
    }
}

#[cfg(feature = "persist-sqlite")]
mod reputation_persistence_sqlite {
    use icn_common::{Cid, Did};
    use icn_identity::{did_key_from_verifying_key, generate_ed25519_keypair, SignatureBytes};
    use icn_mesh::{select_executor, MeshJobBid, JobSpec, Resources, SelectionPolicy};
    use icn_reputation::sqlite_store::SqliteReputationStore;
    use icn_reputation::ReputationStore;
    use icn_runtime::{
        context::{RuntimeContext, SimpleManaLedger, StubMeshNetworkService, StubSigner},
        host_anchor_receipt, ReputationUpdater,
    };
    use std::str::FromStr;
    use tempfile::tempdir;

    #[test]
    fn persisted_reputation_influences_selection() {
        let dir = tempdir().unwrap();
        let rep_path = dir.path().join("rep.sqlite");
        let mana_path = dir.path().join("mana.sqlite");

        let ledger = SimpleManaLedger::new(mana_path);
        let store = SqliteReputationStore::new(rep_path.clone()).unwrap();

        let (_ska, vka) = generate_ed25519_keypair();
        let did_a = Did::from_str(&did_key_from_verifying_key(&vka)).unwrap();
        let (_skb, vkb) = generate_ed25519_keypair();
        let did_b = Did::from_str(&did_key_from_verifying_key(&vkb)).unwrap();

        ledger.set_balance(&did_a, 100).unwrap();
        ledger.set_balance(&did_b, 100).unwrap();

        store.record_execution(&did_a, true, 1000);
        drop(store);

        let reopened = SqliteReputationStore::new(rep_path).unwrap();

        let job_id = Cid::new_v1_sha256(0x55, b"rep_job");
        let bid_a = MeshJobBid {
            job_id: job_id.clone(),
            executor_did: did_a.clone(),
            price_mana: 10,
            resources: Resources { cpu_cores: 1, memory_mb: 10 },
            signature: SignatureBytes(Vec::new()),
        };
        let bid_b = MeshJobBid {
            job_id: job_id.clone(),
            executor_did: did_b.clone(),
            price_mana: 10,
            resources: Resources { cpu_cores: 1, memory_mb: 10 },
            signature: SignatureBytes(Vec::new()),
        };

        let selected = select_executor(
            &job_id,
            &JobSpec::Echo { payload: "persist".into() },
            vec![bid_a, bid_b],
            &SelectionPolicy::default(),
            &reopened,
            &ledger,
        )
        .expect("executor selected");

        assert_eq!(selected, did_a);
    }

    #[tokio::test]
    async fn reputation_survives_restart() {
        let dir = tempdir().unwrap();
        let dag_path = dir.path().join("dag.sqlite");
        let mana_path = dir.path().join("mana.sqlite");
        let rep_path = dir.path().join("rep.sqlite");

        let did = Did::new("key", "tester");
        let ctx1 = RuntimeContext::new_with_paths(
            did.clone(),
            Arc::new(StubMeshNetworkService::new()),
            Arc::new(StubSigner::new()),
            Arc::new(icn_identity::KeyDidResolver),
            dag_path.clone(),
            mana_path.clone(),
            rep_path.clone(),
        )
        .unwrap();

        let mut receipt = icn_identity::ExecutionReceipt {
            job_id: Cid::new_v1_sha256(0x55, b"r"),
            executor_did: did.clone(),
            result_cid: Cid::new_v1_sha256(0x55, b"r"),
            cpu_ms: 1000,
            success: true,
            sig: SignatureBytes(Vec::new()),
        };

        let mut msg = Vec::new();
        msg.extend_from_slice(receipt.job_id.to_string().as_bytes());
        msg.extend_from_slice(did.to_string().as_bytes());
        msg.extend_from_slice(receipt.result_cid.to_string().as_bytes());
        msg.extend_from_slice(&receipt.cpu_ms.to_le_bytes());
        msg.push(receipt.success as u8);
        let sig = ctx1.signer.sign(&msg).unwrap();
        receipt.sig = SignatureBytes(sig);

        let json = serde_json::to_string(&receipt).unwrap();
        host_anchor_receipt(&ctx1, &json, &ReputationUpdater::new())
            .await
            .unwrap();
        drop(ctx1);

        let ctx2 = RuntimeContext::new_with_paths(
            did.clone(),
            Arc::new(StubMeshNetworkService::new()),
            Arc::new(StubSigner::new()),
            Arc::new(icn_identity::KeyDidResolver),
            dag_path,
            mana_path,
            rep_path,
        )
        .unwrap();

        assert_eq!(ctx2.reputation_store.get_reputation(&did), 2);
    }
}

#[cfg(feature = "persist-rocksdb")]
mod reputation_persistence_rocks {
    use icn_common::{Cid, Did};
    use icn_identity::{did_key_from_verifying_key, generate_ed25519_keypair, SignatureBytes};
    use icn_mesh::{select_executor, MeshJobBid, JobSpec, Resources, SelectionPolicy};
    use icn_reputation::rocksdb_store::RocksdbReputationStore;
    use icn_reputation::ReputationStore;
    use icn_runtime::context::SimpleManaLedger;
    use std::str::FromStr;
    use tempfile::tempdir;

    #[test]
    fn persisted_reputation_influences_selection() {
        let dir = tempdir().unwrap();
        let rep_path = dir.path().join("rep.rocks");
        let mana_path = dir.path().join("mana.rocks");

        let ledger = SimpleManaLedger::new(mana_path);
        let store = RocksdbReputationStore::new(rep_path.clone()).unwrap();

        let (_ska, vka) = generate_ed25519_keypair();
        let did_a = Did::from_str(&did_key_from_verifying_key(&vka)).unwrap();
        let (_skb, vkb) = generate_ed25519_keypair();
        let did_b = Did::from_str(&did_key_from_verifying_key(&vkb)).unwrap();

        ledger.set_balance(&did_a, 100).unwrap();
        ledger.set_balance(&did_b, 100).unwrap();

        store.record_execution(&did_a, true, 1000);
        drop(store);

        let reopened = RocksdbReputationStore::new(rep_path).unwrap();

        let job_id = Cid::new_v1_sha256(0x55, b"rep_job");
        let bid_a = MeshJobBid {
            job_id: job_id.clone(),
            executor_did: did_a.clone(),
            price_mana: 10,
            resources: Resources { cpu_cores: 1, memory_mb: 10 },
            signature: SignatureBytes(Vec::new()),
        };
        let bid_b = MeshJobBid {
            job_id: job_id.clone(),
            executor_did: did_b.clone(),
            price_mana: 10,
            resources: Resources { cpu_cores: 1, memory_mb: 10 },
            signature: SignatureBytes(Vec::new()),
        };

        let selected = select_executor(
            &job_id,
            &JobSpec::Echo { payload: "persist".into() },
            vec![bid_a, bid_b],
            &SelectionPolicy::default(),
            &reopened,
            &ledger,
        )
        .expect("executor selected");

        assert_eq!(selected, did_a);
    }
}
