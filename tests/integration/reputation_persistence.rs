#[cfg(feature = "persist-sled")]
mod reputation_persistence {
    use icn_common::{Cid, Did};
    use icn_identity::{did_key_from_verifying_key, generate_ed25519_keypair, SignatureBytes};
    use icn_mesh::{select_executor, MeshJobBid, JobSpec, Resources, SelectionPolicy};
    use icn_reputation::sled_store::SledReputationStore;
    use icn_reputation::ReputationStore;
    use icn_runtime::context::SimpleManaLedger;
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
}
