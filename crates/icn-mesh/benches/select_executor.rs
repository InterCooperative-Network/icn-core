use std::collections::HashMap;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, black_box};
use icn_common::{Cid, Did, CommonError};
use icn_economics::ManaLedger;
use icn_mesh::{select_executor, MeshJobBid, JobId, JobSpec, SelectionPolicy, Resources};
use icn_reputation::{InMemoryReputationStore, ReputationStore};
use std::sync::Mutex;

#[derive(Default)]
struct BenchLedger {
    balances: Mutex<HashMap<Did, u64>>, 
}

impl BenchLedger {
    fn set_balance(&self, did: &Did, amount: u64) {
        self.balances.lock().unwrap().insert(did.clone(), amount);
    }
}

impl ManaLedger for BenchLedger {
    fn get_balance(&self, did: &Did) -> u64 {
        *self.balances.lock().unwrap().get(did).unwrap_or(&0)
    }
    fn set_balance(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
        self.balances.lock().unwrap().insert(did.clone(), amount);
        Ok(())
    }
    fn spend(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
        let mut map = self.balances.lock().unwrap();
        let bal = map.get_mut(did).ok_or_else(|| CommonError::DatabaseError("account".into()))?;
        if *bal < amount {
            return Err(CommonError::PolicyDenied("insufficient".into()));
        }
        *bal -= amount;
        Ok(())
    }
    fn credit(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
        let mut map = self.balances.lock().unwrap();
        let entry = map.entry(did.clone()).or_insert(0);
        *entry += amount;
        Ok(())
    }
    fn credit_all(&self, amount: u64) -> Result<(), CommonError> {
        let mut map = self.balances.lock().unwrap();
        for val in map.values_mut() { *val += amount; }
        Ok(())
    }
    fn all_accounts(&self) -> Vec<Did> {
        self.balances.lock().unwrap().keys().cloned().collect()
    }
}

fn build_bids(job_id: &JobId, count: usize) -> (Vec<MeshJobBid>, BenchLedger, InMemoryReputationStore) {
    let ledger = BenchLedger::default();
    let rep = InMemoryReputationStore::new();
    let mut bids = Vec::with_capacity(count);
    for i in 0..count {
        let did = Did::new("icn", &format!("exec{i}"));
        ledger.set_balance(&did, 1000);
        rep.set_score(did.clone(), 1);
        let bid = MeshJobBid {
            job_id: job_id.clone(),
            executor_did: did,
            price_mana: (i % 10 + 1) as u64,
            resources: Resources { cpu_cores: 1, memory_mb: 512 },
            signature: icn_identity::SignatureBytes(Vec::new()),
        };
        bids.push(bid);
    }
    (bids, ledger, rep)
}

fn bench_select_executor(c: &mut Criterion) {
    let policy = SelectionPolicy::default();
    let job_id = JobId(Cid::new_v1_sha256(0x55, b"bench_job"));
    let spec = JobSpec::default();

    let mut group = c.benchmark_group("select_executor");
    for &count in &[1usize, 10, 100, 1000] {
        let (bids, ledger, rep) = build_bids(&job_id, count);
        group.bench_with_input(BenchmarkId::from_parameter(count), &bids, |b, bids| {
            b.iter(|| {
                let result = select_executor(black_box(&job_id), &spec, bids.clone(), &policy, &rep, &ledger);
                black_box(result);
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_select_executor);
criterion_main!(benches);

