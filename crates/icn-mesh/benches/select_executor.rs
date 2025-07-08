use criterion::{criterion_group, criterion_main, Criterion};
use icn_common::{Cid, Did};
use icn_economics::ManaLedger;
use icn_identity::SignatureBytes;
use icn_mesh::{select_executor, JobId, JobSpec, MeshJobBid, Resources, SelectionPolicy};
use icn_reputation::InMemoryReputationStore;

struct BenchLedger {
    balance: u64,
}

impl ManaLedger for BenchLedger {
    fn get_balance(&self, _did: &Did) -> u64 {
        self.balance
    }

    fn set_balance(&self, _did: &Did, _amount: u64) -> Result<(), icn_common::CommonError> {
        Ok(())
    }

    fn spend(&self, _did: &Did, _amount: u64) -> Result<(), icn_common::CommonError> {
        Ok(())
    }

    fn credit(&self, _did: &Did, _amount: u64) -> Result<(), icn_common::CommonError> {
        Ok(())
    }
}

fn dummy_job_id() -> JobId {
    JobId(Cid::new_v1_sha256(0x55, b"select_executor_bench"))
}

fn create_bids(job_id: &JobId, count: usize) -> Vec<MeshJobBid> {
    (0..count)
        .map(|i| MeshJobBid {
            job_id: job_id.clone(),
            executor_did: Did::new("bench", &format!("exec{i}")),
            price_mana: ((i % 10) + 1) as u64,
            resources: Resources {
                cpu_cores: 1,
                memory_mb: 512,
            },
            signature: SignatureBytes(Vec::new()),
        })
        .collect()
}

fn bench_select_executor(c: &mut Criterion) {
    let rep_store = InMemoryReputationStore::new();
    let ledger = BenchLedger { balance: 100 };
    let job_id = dummy_job_id();
    let spec = JobSpec::default();
    let policy = SelectionPolicy::default();

    for &count in &[10usize, 100, 1000] {
        let bids = create_bids(&job_id, count);
        c.bench_function(&format!("select_executor_{count}"), |b| {
            b.iter(|| {
                select_executor(&job_id, &spec, bids.clone(), &policy, &rep_store, &ledger);
            });
        });
    }
}

criterion_group!(mesh_benches, bench_select_executor);
criterion_main!(mesh_benches);
