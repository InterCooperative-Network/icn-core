use std::time::Duration;
use std::sync::Arc;

use criterion::{criterion_group, criterion_main, Criterion};
use icn_common::Cid;
use icn_identity::{SignatureBytes, generate_ed25519_keypair, did_key_from_verifying_key};
use icn_mesh::{ActualMeshJob, JobId, JobSpec, Resources, MeshJobBid};
use icn_runtime::context::{RuntimeContext, StubMeshNetworkService, LocalMeshSubmitReceiptMessage};
use icn_runtime::context::JobState;
use tokio::runtime::Runtime;

async fn queue_and_process() {
    let ctx = RuntimeContext::new_with_stubs_and_mana("did:icn:test:bench_submitter", 100).unwrap();
    let network = ctx
        .mesh_network_service
        .clone()
        .downcast_arc::<StubMeshNetworkService>()
        .expect("stub net");
    let (_sk, vk) = generate_ed25519_keypair();
    let exec_did = did_key_from_verifying_key(&vk);
    let exec_ctx = RuntimeContext::new_with_stubs_and_mana(&exec_did, 50).unwrap();

    let job = ActualMeshJob {
        id: JobId(Cid::new_v1_sha256(0x55, b"bench_job")),
        manifest_cid: Cid::new_v1_sha256(0x55, b"manifest"),
        spec: JobSpec { kind: icn_mesh::JobKind::Echo { payload: "hi".into() }, ..Default::default() },
        creator_did: ctx.current_identity.clone(),
        cost_mana: 5,
        max_execution_wait_ms: None,
        signature: SignatureBytes(Vec::new()),
    };

    let bid = MeshJobBid {
        job_id: job.id.clone(),
        executor_did: exec_ctx.current_identity.clone(),
        price_mana: 1,
        resources: Resources { cpu_cores: 1, memory_mb: 512 },
        signature: SignatureBytes(Vec::new()),
    };
    network.stage_bid(job.id.clone(), bid).await;

    // prepare receipt signed by executor
    let receipt = icn_identity::ExecutionReceipt {
        job_id: job.id.clone().into(),
        executor_did: exec_ctx.current_identity.clone(),
        result_cid: Cid::new_v1_sha256(0x55, b"result"),
        cpu_ms: 1,
        success: true,
        sig: SignatureBytes(Vec::new()),
    };
    let mut msg = Vec::new();
    msg.extend_from_slice(receipt.job_id.to_string().as_bytes());
    msg.extend_from_slice(exec_ctx.current_identity.to_string().as_bytes());
    msg.extend_from_slice(receipt.result_cid.to_string().as_bytes());
    msg.extend_from_slice(&receipt.cpu_ms.to_le_bytes());
    msg.push(receipt.success as u8);
    let sig = exec_ctx.signer.sign(&msg).unwrap();
    let mut signed_receipt = receipt.clone();
    signed_receipt.sig = SignatureBytes(sig);
    network
        .stage_receipt(LocalMeshSubmitReceiptMessage { receipt: signed_receipt })
        .await;

    ctx.internal_queue_mesh_job(job.clone()).await.unwrap();
    ctx.clone()
        .wait_for_and_process_receipt(job, exec_ctx.current_identity.clone())
        .await
        .unwrap();

    assert!(matches!(ctx.job_states.get(&JobId(Cid::new_v1_sha256(0x55, b"bench_job"))).map(|s| s.value().clone()), Some(JobState::Completed { .. })));
}

fn bench_job_manager(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    c.bench_function("queue_and_process_job", |b| {
        b.to_async(&rt).iter(|| async {
            queue_and_process().await;
        });
    });
}

criterion_group!(benches, bench_job_manager);
criterion_main!(benches);
