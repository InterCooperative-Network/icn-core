use icn_common::Did;
use icn_identity::{did_key_from_verifying_key, generate_ed25519_keypair, SignatureBytes};
use icn_mesh::{score_bid, JobSpec, MeshJobBid, Resources, SelectionPolicy};
use icn_economics::ManaLedger;
use icn_reputation::InMemoryReputationStore;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Mutex;

struct InMemoryLedger {
    balances: Mutex<HashMap<Did, u64>>,
}

impl InMemoryLedger {
    fn new() -> Self {
        Self { balances: Mutex::new(HashMap::new()) }
    }
}

impl icn_economics::ManaLedger for InMemoryLedger {
    fn get_balance(&self, did: &Did) -> u64 {
        *self.balances.lock().unwrap().get(did).unwrap_or(&0)
    }
    fn set_balance(&self, did: &Did, amount: u64) -> Result<(), icn_common::CommonError> {
        self.balances.lock().unwrap().insert(did.clone(), amount);
        Ok(())
    }
    fn spend(&self, did: &Did, amount: u64) -> Result<(), icn_common::CommonError> {
        let mut map = self.balances.lock().unwrap();
        let bal = map.get_mut(did).ok_or_else(|| icn_common::CommonError::DatabaseError("account".into()))?;
        if *bal < amount {
            return Err(icn_common::CommonError::PolicyDenied("insufficient".into()));
        }
        *bal -= amount;
        Ok(())
    }
    fn credit(&self, did: &Did, amount: u64) -> Result<(), icn_common::CommonError> {
        let mut map = self.balances.lock().unwrap();
        let entry = map.entry(did.clone()).or_insert(0);
        *entry += amount;
        Ok(())
    }
}

#[test]
fn resource_weight_affects_score() {
    let (sk_fast, vk_fast) = generate_ed25519_keypair();
    let fast = Did::from_str(&did_key_from_verifying_key(&vk_fast)).unwrap();
    let (sk_slow, vk_slow) = generate_ed25519_keypair();
    let slow = Did::from_str(&did_key_from_verifying_key(&vk_slow)).unwrap();

    let rep_store = InMemoryReputationStore::new();
    rep_store.set_score(fast.clone(), 1);
    rep_store.set_score(slow.clone(), 1);

    let ledger = InMemoryLedger::new();
    ledger.set_balance(&fast, 100).unwrap();
    ledger.set_balance(&slow, 100).unwrap();

    let spec = JobSpec {
        required_resources: Resources { cpu_cores: 2, memory_mb: 1024 },
        ..JobSpec::default()
    };

    let bid_fast = MeshJobBid {
        job_id: icn_mesh::JobId(icn_common::Cid::new_v1_sha256(0x55, b"job")),
        executor_did: fast.clone(),
        price_mana: 10,
        resources: Resources { cpu_cores: 4, memory_mb: 4096 },
        signature: SignatureBytes(vec![]),
    }
    .sign(&sk_fast)
    .unwrap();

    let bid_slow = MeshJobBid {
        job_id: icn_mesh::JobId(icn_common::Cid::new_v1_sha256(0x55, b"job")),
        executor_did: slow.clone(),
        price_mana: 10,
        resources: Resources { cpu_cores: 1, memory_mb: 512 },
        signature: SignatureBytes(vec![]),
    }
    .sign(&sk_slow)
    .unwrap();

    let policy = SelectionPolicy { weight_price: 1.0, weight_reputation: 0.0, weight_resources: 10.0 };

    let fast_score = score_bid(&bid_fast, &spec, &policy, &rep_store, ledger.get_balance(&fast));
    let slow_score = score_bid(&bid_slow, &spec, &policy, &rep_store, ledger.get_balance(&slow));
    assert!(fast_score > slow_score);
}
