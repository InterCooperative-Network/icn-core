use criterion::{criterion_group, criterion_main, Criterion};
use downcast_rs::Downcast;
use icn_common::{Cid, Did};
use icn_identity::{
    did_key_from_verifying_key, generate_ed25519_keypair, ExecutionReceipt, SignatureBytes,
};
use icn_mesh::{JobId, JobSpec, MeshJobBid, Resources};
use icn_runtime::context::{LocalMeshSubmitReceiptMessage, RuntimeContext, StubMeshNetworkService};
use std::str::FromStr;
use tokio::runtime::Runtime;

async fn queue_and_process_job() {
    let ctx = std::sync::Arc::new(
        RuntimeContext::new_with_stubs_and_mana("did:icn:bench:node", 100).unwrap(),
    );

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
        executor_capabilities: Vec::new(),
        executor_federations: Vec::new(),
        executor_trust_scope: None,
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
    stub.stage_receipt(job_id.clone(), LocalMeshSubmitReceiptMessage { receipt })
        .await;

    // Test the job submission API
    let manifest_cid = Cid::new_v1_sha256(0x55, b"manifest");
    let job_spec = JobSpec::default();
    let spec_bytes = bincode::serialize(&job_spec).expect("Failed to serialize job spec");
    let cost_mana = 10;

    let submitted_job_id = ctx
        .handle_submit_job(manifest_cid, spec_bytes, cost_mana)
        .await
        .unwrap();

    // Simple verification that job was submitted
    assert_eq!(submitted_job_id, job_id);
}

fn bench_job_manager(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    c.bench_function("queue_and_process_job", |b| {
        b.iter(|| rt.block_on(queue_and_process_job()));
    });
}

criterion_group!(benches, bench_job_manager);
criterion_main!(benches);
