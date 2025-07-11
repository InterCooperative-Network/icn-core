use icn_common::Did;
use icn_identity::{did_key_from_verifying_key, generate_ed25519_keypair, SignatureBytes};
use icn_mesh::{select_executor, JobId, JobSpec, MeshJobBid, Resources, SelectionPolicy};
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
fn executor_selection_prefers_reputation() {
    let job_id = JobId(icn_common::Cid::new_v1_sha256(0x55, b"job"));
    let (sk_h, vk_h) = generate_ed25519_keypair();
    let high = Did::from_str(&did_key_from_verifying_key(&vk_h)).unwrap();
    let (sk_l, vk_l) = generate_ed25519_keypair();
    let low = Did::from_str(&did_key_from_verifying_key(&vk_l)).unwrap();

    let rep_store = InMemoryReputationStore::new();
    rep_store.set_score(high.clone(), 5);
    rep_store.set_score(low.clone(), 1);

    let ledger = InMemoryLedger::new();
    ledger.set_balance(&high, 50).unwrap();
    ledger.set_balance(&low, 50).unwrap();

    let bid_high = MeshJobBid {
        job_id: job_id.clone(),
        executor_did: high.clone(),
        price_mana: 10,
        resources: Resources::default(),
        signature: SignatureBytes(vec![]),
    }
    .sign(&sk_h)
    .unwrap();
    let bid_low = MeshJobBid {
        job_id: job_id.clone(),
        executor_did: low.clone(),
        price_mana: 5,
        resources: Resources::default(),
        signature: SignatureBytes(vec![]),
    }
    .sign(&sk_l)
    .unwrap();

    let policy = SelectionPolicy::default();
    let spec = JobSpec::default();
    let selected = select_executor(
        &job_id,
        &spec,
        vec![bid_high, bid_low],
        &policy,
        &rep_store,
        &ledger,
    );
    assert_eq!(selected.unwrap(), high);
}
