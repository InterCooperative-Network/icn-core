use std::sync::Arc;
use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion};
use icn_common::{Cid, Did};
use icn_identity::SignatureBytes;
use icn_mesh::{
    select_executor, ActualMeshJob, JobId, JobSpec, MeshJobBid, Resources, SelectionPolicy,
};
use icn_runtime::context::{
    JobAssignmentNotice, LocalMeshSubmitReceiptMessage, RuntimeContext, StubDagStore,
    StubMeshNetworkService, StubSigner,
};
use tokio::runtime::Runtime;
use tokio::sync::Mutex as TokioMutex;

async fn queue_and_process_job() {
    let network = Arc::new(StubMeshNetworkService::new());
    let dag_store = Arc::new(TokioMutex::new(StubDagStore::new()));
    let ctx = RuntimeContext::new(
        Did::new("bench", "submitter"),
        network.clone(),
        Arc::new(StubSigner::new()),
        Arc::new(icn_identity::KeyDidResolver),
        dag_store,
    );
    ctx.mana_ledger
        .set_balance(&ctx.current_identity, 100)
        .expect("set balance");

    let job = ActualMeshJob {
        id: JobId(Cid::new_v1_sha256(0x55, b"bench_job")),
        manifest_cid: Cid::new_v1_sha256(0x55, b"manifest"),
        spec: JobSpec::default(),
        creator_did: ctx.current_identity.clone(),
        cost_mana: 10,
        max_execution_wait_ms: None,
        signature: SignatureBytes(Vec::new()),
    };

    let exec_did = Did::new("bench", "exec1");
    let bid = MeshJobBid {
        job_id: job.id.clone(),
        executor_did: exec_did.clone(),
        price_mana: 5,
        resources: Resources::default(),
        signature: SignatureBytes(Vec::new()),
    };
    network.stage_bid(job.id.clone(), bid).await;

    let receipt = icn_identity::ExecutionReceipt {
        job_id: job.id.clone().into(),
        executor_did: exec_did.clone(),
        result_cid: Cid::new_v1_sha256(0x55, b"result"),
        cpu_ms: 10,
        success: true,
        sig: SignatureBytes(Vec::new()),
    };
    network
        .stage_receipt(LocalMeshSubmitReceiptMessage { receipt })
        .await;

    ctx.internal_queue_mesh_job(job.clone())
        .await
        .expect("queue job");

    ctx.mesh_network_service
        .announce_job(&job)
        .await
        .expect("announce");
    let bids = ctx
        .mesh_network_service
        .collect_bids_for_job(&job.id, Duration::from_millis(10))
        .await
        .expect("collect");
    let policy = SelectionPolicy::default();
    let exec = select_executor(
        &job.id,
        &job.spec,
        bids,
        &policy,
        ctx.reputation_store.as_ref(),
        &ctx.mana_ledger,
    )
    .expect("selected executor");
    ctx.mesh_network_service
        .notify_executor_of_assignment(&JobAssignmentNotice {
            job_id: job.id.clone(),
            executor_did: exec.clone(),
        })
        .await
        .expect("notify");
    let _ = ctx
        .mesh_network_service
        .try_receive_receipt(&job.id, &exec, Duration::from_millis(10))
        .await
        .expect("receipt");
}

fn bench_job_manager(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    c.bench_function("queue_and_process_job", |b| {
        b.to_async(&rt).iter(|| queue_and_process_job());
    });
}

criterion_group!(runtime_benches, bench_job_manager);
criterion_main!(runtime_benches);
