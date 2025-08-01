use icn_common::{Cid, Did};
use icn_identity::{ExecutionReceipt, SignatureBytes};
use icn_mesh::{JobKind, JobSpec, MeshJobBid, Resources};
use icn_runtime::context::{
    LocalMeshSubmitReceiptMessage, MeshNetworkServiceType, RuntimeContext, StubMeshNetworkService,
};
use std::str::FromStr;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

fn get_stub_network_service(context: &Arc<RuntimeContext>) -> Arc<StubMeshNetworkService> {
    match &*context.mesh_network_service {
        MeshNetworkServiceType::Stub(_stub) => Arc::new(StubMeshNetworkService::new()),
        _ => panic!("Expected StubMeshNetworkService"),
    }
}

#[tokio::test]
async fn lifecycle_reconstructs_spec_and_tracks_bid() {
    let submitter = Did::from_str("did:icn:test:submitter").unwrap();
    let ctx = RuntimeContext::new_for_testing(submitter.clone(), Some(1000)).unwrap();

    let stub = get_stub_network_service(&ctx);

    let spec = JobSpec {
        kind: JobKind::GenericPlaceholder,
        inputs: vec![],
        outputs: vec![],
        required_resources: Resources {
            cpu_cores: 2,
            memory_mb: 0,
            storage_mb: 0,
        },
        required_capabilities: vec![],
        required_trust_scope: None,
        min_executor_reputation: None,
        allowed_federations: vec![],
    };
    let spec_json = serde_json::to_string(&spec).unwrap();
    let spec_json_bytes = spec_json.as_bytes().to_vec();
    let manifest_cid = Cid::new_v1_sha256(0x55, b"manifest_spec");

    let job_id = ctx
        .handle_submit_job(manifest_cid, spec_json_bytes, 50)
        .await
        .unwrap();

    let bid1 = MeshJobBid {
        job_id: job_id.clone(),
        executor_did: Did::from_str("did:icn:test:exec1").unwrap(),
        price_mana: 1,
        resources: Resources {
            cpu_cores: 1,
            memory_mb: 0,
            storage_mb: 0,
        },
        executor_capabilities: vec![],
        executor_federations: vec![],
        executor_trust_scope: None,
        signature: SignatureBytes(vec![0; 64]),
    };

    let bid2 = MeshJobBid {
        job_id: job_id.clone(),
        executor_did: Did::from_str("did:icn:test:exec2").unwrap(),
        price_mana: 2,
        resources: Resources {
            cpu_cores: 2,
            memory_mb: 1,
            storage_mb: 0,
        },
        executor_capabilities: vec![],
        executor_federations: vec![],
        executor_trust_scope: None,
        signature: SignatureBytes(vec![0; 64]),
    };

    stub.stage_bid(job_id.clone(), bid1.clone()).await;
    stub.stage_bid(job_id.clone(), bid2).await;

    sleep(Duration::from_millis(100)).await;

    let receipt = ExecutionReceipt {
        job_id: job_id.clone().into(),
        executor_did: bid1.executor_did.clone(),
        result_cid: Cid::new_v1_sha256(0x55, b"result"),
        cpu_ms: 10,
        success: true,
        sig: SignatureBytes(vec![0; 64]),
    };

    stub.stage_receipt(job_id.clone(), LocalMeshSubmitReceiptMessage {
        receipt: receipt.clone(),
    })
    .await;

    sleep(Duration::from_millis(200)).await;

    let status = ctx.get_job_status(&job_id).await.unwrap();
    let lifecycle = status.expect("Job status should be available");
    assert_eq!(lifecycle.job.spec_json, Some(spec_json));
    assert_eq!(lifecycle.bids.len(), 2);
    assert!(lifecycle.receipt.is_some());
    assert_eq!(lifecycle.receipt.as_ref().unwrap().executor_did, bid1.executor_did);
}
