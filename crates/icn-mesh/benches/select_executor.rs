use criterion::{black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use icn_common::{Cid, Did};
use icn_economics::ManaLedger;
use icn_identity::{did_key_from_verifying_key, generate_ed25519_keypair, SignatureBytes};
use icn_mesh::{select_executor, JobId, JobSpec, MeshJobBid, Resources, SelectionPolicy};
use icn_reputation::InMemoryReputationStore;
use std::str::FromStr;

// Simple in-memory ledger for benchmarks
struct BenchLedger {
    inner: std::collections::HashMap<Did, u64>,
}
impl BenchLedger {
    fn new() -> Self {
        Self {
            inner: std::collections::HashMap::new(),
        }
    }
}
impl ManaLedger for BenchLedger {
    fn get_balance(&self, did: &Did) -> u64 {
        *self.inner.get(did).unwrap_or(&0)
    }
    fn set_balance(&self, did: &Did, amount: u64) -> Result<(), icn_common::CommonError> {
        self.inner.insert(did.clone(), amount);
        Ok(())
    }
    fn spend(&self, did: &Did, amount: u64) -> Result<(), icn_common::CommonError> {
        let bal = self
            .inner
            .get_mut(did)
            .ok_or_else(|| icn_common::CommonError::DatabaseError("account".into()))?;
        if *bal < amount {
            return Err(icn_common::CommonError::PolicyDenied("insufficient".into()));
        }
        *bal -= amount;
        Ok(())
    }
    fn credit(&self, did: &Did, amount: u64) -> Result<(), icn_common::CommonError> {
        let bal = self.inner.entry(did.clone()).or_insert(0);
        *bal += amount;
        Ok(())
    }
}

fn bench_select_executor(c: &mut Criterion) {
    let job_id = JobId(Cid::new_v1_sha256(0x55, b"benchjob"));
    let spec = JobSpec::default();
    let policy = SelectionPolicy::default();
    let mut group = c.benchmark_group("select_executor");

    for &num_bids in &[10usize, 100, 1000] {
        group.bench_with_input(BenchmarkId::from_parameter(num_bids), &num_bids, |b, &n| {
            // Prepare bids, ledger and reputation store
            let rep_store = InMemoryReputationStore::new();
            let ledger = BenchLedger::new();
            let mut bids = Vec::with_capacity(n);
            for i in 0..n {
                let (_sk, vk) = generate_ed25519_keypair();
                let did = Did::from_str(&did_key_from_verifying_key(&vk)).unwrap();
                rep_store.set_score(did.clone(), i as u64);
                ledger.set_balance(&did, 100).unwrap();
                bids.push(MeshJobBid {
                    job_id: job_id.clone(),
                    executor_did: did,
                    price_mana: (i % 10) as u64,
                    resources: Resources {
                        cpu_cores: 1,
                        memory_mb: 512,
                    },
                    signature: SignatureBytes(Vec::new()),
                });
            }
            b.iter_batched(
                || bids.clone(),
                |bids_vec| {
                    black_box(select_executor(
                        &job_id, &spec, bids_vec, &policy, &rep_store, &ledger,
                    ));
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

criterion_group!(benches, bench_select_executor);
criterion_main!(benches);
