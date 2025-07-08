use criterion::{criterion_group, criterion_main, Criterion};
use icn_common::{Cid, Did};
use icn_identity::{
    did_key_from_verifying_key, generate_ed25519_keypair, ExecutionReceipt, SignatureBytes,
    SigningKey,
};
use icn_mesh::{ActualMeshJob, JobId, JobSpec, JobState, MeshJobBid, Resources};
use icn_runtime::context::{LocalMeshSubmitReceiptMessage, RuntimeContext, StubMeshNetworkService};
use std::str::FromStr;
use tokio::runtime::Runtime;
use tokio::time::{sleep, Duration};

async fn queue_and_process_job() {
    let ctx = RuntimeContext::new_with_stubs_and_mana("did:icn:bench:node", 100).unwrap();
    ctx.default_receipt_wait_ms = 50;

    let stub = ctx
        .mesh_network_service
        .as_any()
        .downcast_ref::<StubMeshNetworkService>()
        .expect("stub service");

    let (exec_sk, exec_vk) = generate_ed25519_keypair();
    let exec_did = Did::from_str(&did_key_from_verifying_key(&exec_vk)).unwrap();

    let job_id = JobId(Cid::new_v1_sha256(0x55, b"benchjob"));

    let unsigned_bid = MeshJobBid {
        job_id: job_id.clone(),
        executor_did: exec_did.clone(),
        price_mana: 10,
        resources: Resources::default(),
        signature: SignatureBytes(Vec::new()),
    };
    let signed_bid = unsigned_bid.clone().sign(&exec_sk).unwrap();
    stub.stage_bid(job_id.clone(), signed_bid).await;

    let receipt = ExecutionReceipt {
        job_id: job_id.clone().into(),
        executor_did: exec_did.clone(),
        result_cid: Cid::new_v1_sha256(0x55, b"result"),
        cpu_ms: 1,
        success: true,
        sig: SignatureBytes(Vec::new()),
    }
    .sign_with_key(&exec_sk)
    .unwrap();
    stub.stage_receipt(LocalMeshSubmitReceiptMessage { receipt })
        .await;

    let ctx_clone = ctx.clone();
    ctx_clone.spawn_mesh_job_manager().await;

    let job = ActualMeshJob {
        id: job_id.clone(),
        manifest_cid: Cid::new_v1_sha256(0x55, b"manifest"),
        spec: JobSpec::default(),
        creator_did: ctx.current_identity.clone(),
        cost_mana: 10,
        max_execution_wait_ms: Some(50),
        signature: SignatureBytes(Vec::new()),
    };
    ctx.internal_queue_mesh_job(job).await.unwrap();

    for _ in 0..20 {
        if let Some(state) = ctx.job_states.get(&job_id).map(|s| s.value().clone()) {
            if matches!(state, JobState::Completed { .. }) {
                break;
            }
        }
        sleep(Duration::from_millis(10)).await;
    }
}

fn bench_job_manager(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    c.bench_function("queue_and_process_job", |b| {
        b.to_async(&rt).iter(|| queue_and_process_job());
    });
}

criterion_group!(benches, bench_job_manager);
criterion_main!(benches);
