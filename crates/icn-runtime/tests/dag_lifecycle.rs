use icn_common::{Cid, Did};
use icn_identity::{ExecutionReceipt, SignatureBytes};
use icn_mesh::{JobId, JobKind, JobSpec, MeshJobBid, Resources};
use icn_runtime::context::{
    LocalMeshSubmitReceiptMessage, MeshNetworkServiceType, RuntimeContext, StubMeshNetworkService,
};
use std::str::FromStr;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

fn get_stub_network_service(context: &Arc<RuntimeContext>) -> Arc<StubMeshNetworkService> {
    match &*context.mesh_network_service {
        MeshNetworkServiceType::Stub(stub) => Arc::new(stub.clone()),
        _ => panic!("Expected StubMeshNetworkService"),
    }
}

#[tokio::test]
async fn lifecycle_reconstructs_spec_and_tracks_bid() {
    let submitter = Did::from_str("did:icn:test:submitter").unwrap();
    let ctx = RuntimeContext::new_testing(submitter.clone(), Some(1000)).unwrap();
    ctx.default_receipt_wait_ms = 500;

    let stub = get_stub_network_service(&ctx);

    let spec = JobSpec {
        kind: JobKind::GenericPlaceholder,
        inputs: vec![],
        outputs: vec![],
        required_resources: Resources {
            cpu_cores: 2,
            memory_mb: 0,
        },
    };
    let spec_json = serde_json::to_string(&spec).unwrap();
    let manifest_cid = Cid::new_v1_sha256(0x55, b"manifest_spec");

    let job_id = ctx
        .handle_submit_job(manifest_cid, spec_json.clone(), 50)
        .await
        .unwrap();

    let bid1 = MeshJobBid {
        job_id: job_id.clone(),
        executor_did: Did::from_str("did:icn:test:exec1").unwrap(),
        price_mana: 1,
        resources: Resources {
            cpu_cores: 1,
            memory_mb: 0,
        },
        signature: SignatureBytes(vec![]),
    };
    let bid2 = MeshJobBid {
        job_id: job_id.clone(),
        executor_did: Did::from_str("did:icn:test:exec2").unwrap(),
        price_mana: 10,
        resources: Resources {
            cpu_cores: 2,
            memory_mb: 0,
        },
        signature: SignatureBytes(vec![]),
    };

    stub.stage_bid(job_id.clone(), bid1.clone()).await;
    stub.stage_bid(job_id.clone(), bid2.clone()).await;

    let receipt = ExecutionReceipt {
        job_id: Cid::from(job_id.clone()),
        executor_did: bid2.executor_did.clone(),
        result_cid: Cid::new_v1_sha256(0x55, b"result"),
        cpu_ms: 10,
        success: true,
        sig: SignatureBytes(vec![]),
    };
    stub.stage_receipt(
        job_id.clone(),
        LocalMeshSubmitReceiptMessage {
            receipt: receipt.clone(),
        },
    )
    .await;

    for _ in 0..20 {
        if let Some(lifecycle) = ctx.get_job_status(&job_id).await.unwrap() {
            if lifecycle.receipt.is_some() {
                break;
            }
        }
        sleep(Duration::from_millis(50)).await;
    }

    let lifecycle = ctx.get_job_status(&job_id).await.unwrap().expect("job");
    assert_eq!(lifecycle.job.spec_json, spec_json);
    let assignment = lifecycle.assignment.expect("assignment");
    assert_eq!(assignment.winning_bid_id, "bid_1");
    assert_eq!(assignment.assigned_executor_did, bid2.executor_did);
}
