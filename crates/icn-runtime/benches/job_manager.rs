use std::sync::Arc;
use std::time::{Duration, Instant};

use criterion::{criterion_group, criterion_main, Criterion};
use icn_common::{Cid, Did};
use icn_identity::SignatureBytes;
use icn_mesh::{ActualMeshJob, JobId, JobSpec, MeshJobBid, Resources};
use icn_runtime::context::{
    LocalMeshSubmitReceiptMessage, RuntimeContext, StubDagStore, StubMeshNetworkService, StubSigner,
};
use icn_runtime::host_submit_mesh_job;
use tokio::runtime::Runtime;
use tokio::sync::Mutex as TokioMutex;

fn create_context() -> Arc<RuntimeContext> {
    let network: Arc<StubMeshNetworkService> = Arc::new(StubMeshNetworkService::new());
    let dag = Arc::new(TokioMutex::new(StubDagStore::new()));
    let did = Did::new("key", "bench_submitter");
    let ctx = RuntimeContext::new(
        did.clone(),
        network.clone(),
        Arc::new(StubSigner::new()),
        Arc::new(icn_identity::KeyDidResolver),
        dag,
    );
    ctx.mana_ledger.set_balance(&did, 100).expect("set balance");
    ctx
}

async fn run_once() {
    let ctx = create_context();
    let network = ctx
        .mesh_network_service
        .clone()
        .downcast_arc::<StubMeshNetworkService>()
        .expect("stub network");
    ctx.spawn_mesh_job_manager().await;

    let manifest_cid = Cid::new_v1_sha256(0x55, b"manifest");
    let job = ActualMeshJob {
        id: JobId(Cid::new_v1_sha256(0x55, b"job")),
        manifest_cid: manifest_cid.clone(),
        spec: JobSpec::default(),
        creator_did: ctx.current_identity.clone(),
        cost_mana: 10,
        max_execution_wait_ms: Some(1000),
        signature: SignatureBytes(vec![]),
    };
    let job_json = serde_json::to_string(&job).unwrap();

    let job_id = host_submit_mesh_job(&ctx, &job_json).await.unwrap();

    let exec_did = Did::new("key", "bench_exec");
    ctx.mana_ledger
        .set_balance(&exec_did, 100)
        .expect("set balance");

    let bid = MeshJobBid {
        job_id: job_id.clone(),
        executor_did: exec_did.clone(),
        price_mana: 5,
        resources: Resources::default(),
        signature: SignatureBytes(vec![]),
    };
    network.stage_bid(job_id.clone(), bid).await;

    let receipt = icn_identity::ExecutionReceipt {
        job_id: job_id.clone().into(),
        executor_did: exec_did.clone(),
        result_cid: Cid::new_v1_sha256(0x55, b"result"),
        cpu_ms: 1,
        success: true,
        sig: SignatureBytes(vec![]),
    };
    network
        .stage_receipt(LocalMeshSubmitReceiptMessage { receipt })
        .await;

    for _ in 0..200 {
        {
            let states = ctx.job_states.lock().await;
            if matches!(
                states.get(&job_id),
                Some(icn_mesh::JobState::Completed { .. })
            ) {
                break;
            }
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

fn bench_job_manager(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    c.bench_function("queue_and_process", |b| {
        b.iter_custom(|iters| {
            rt.block_on(async {
                let mut total = Duration::ZERO;
                for _ in 0..iters {
                    let start = Instant::now();
                    run_once().await;
                    total += start.elapsed();
                }
                total
            })
        });
    });
}

criterion_group!(benches, bench_job_manager);
criterion_main!(benches);
