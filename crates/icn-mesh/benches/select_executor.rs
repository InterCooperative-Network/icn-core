use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use icn_common::{Cid, Did};
use icn_economics::ManaLedger;
use icn_identity::SignatureBytes;
use icn_mesh::{select_executor, JobId, JobSpec, MeshJobBid, Resources, SelectionPolicy};
use icn_reputation::InMemoryReputationStore;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Mutex;

struct BenchLedger {
    balances: Mutex<HashMap<Did, u64>>,
}

impl BenchLedger {
    fn new() -> Self {
        Self {
            balances: Mutex::new(HashMap::new()),
        }
    }
}

impl ManaLedger for BenchLedger {
    fn get_balance(&self, did: &Did) -> u64 {
        *self.balances.lock().unwrap().get(did).unwrap_or(&0)
    }
    fn set_balance(&self, did: &Did, amount: u64) -> Result<(), icn_common::CommonError> {
        self.balances.lock().unwrap().insert(did.clone(), amount);
        Ok(())
    }
    fn spend(&self, did: &Did, amount: u64) -> Result<(), icn_common::CommonError> {
        let mut map = self.balances.lock().unwrap();
        let bal = map
            .get_mut(did)
            .ok_or_else(|| icn_common::CommonError::DatabaseError("account".into()))?;
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

fn bench_select_executor(c: &mut Criterion) {
    let mut group = c.benchmark_group("select_executor");
    for &num_bids in &[10usize, 100, 1000] {
        group.bench_with_input(BenchmarkId::from_parameter(num_bids), &num_bids, |b, &n| {
            let job_id = JobId(Cid::new_v1_sha256(0x55, b"bench_job"));
            let job_spec = JobSpec::default();
            let policy = SelectionPolicy::default();
            let rep_store = InMemoryReputationStore::new();
            let ledger = BenchLedger::new();
            let mut bids = Vec::with_capacity(n);
            for i in 0..n {
                let did = Did::from_str(&format!("did:icn:test:{i}")).unwrap();
                rep_store.set_score(did.clone(), (i % 10) as u64);
                ledger.set_balance(&did, 100).unwrap();
                bids.push(MeshJobBid {
                    job_id: job_id.clone(),
                    executor_did: did,
                    price_mana: (i % 50 + 1) as u64,
                    resources: Resources {
                        cpu_cores: 2,
                        memory_mb: 1024,
                    },
                    signature: SignatureBytes(vec![]),
                });
            }
            b.iter(|| {
                let _ = select_executor(
                    &job_id,
                    &job_spec,
                    bids.clone(),
                    &policy,
                    &rep_store,
                    &ledger,
                );
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_select_executor);
criterion_main!(benches);
