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
        MeshNetworkServiceType::Stub(stub) => Arc::new(stub.clone()),
        _ => panic!("Expected StubMeshNetworkService"),
    }
}

#[tokio::test]
async fn lifecycle_reconstructs_spec_and_tracks_bid() {
    let submitter = Did::from_str("did:icn:test:submitter").unwrap();
    let ctx = RuntimeContext::new_for_testing(submitter.clone(), Some(1000)).unwrap();
    ctx.default_receipt_wait_ms = 500;

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

    ctx.submit_bid(bid1.clone()).await.unwrap();
    ctx.submit_bid(bid2.clone()).await.unwrap();

    sleep(Duration::from_millis(100)).await;

    let receipt = ExecutionReceipt {
        job_id: job_id.clone(),
        executor_did: bid1.executor_did.clone(),
        result: icn_identity::ExecutionResult {
            success: true,
            output_cids: vec![Cid::new_v1_sha256(0x55, b"result")],
            error_message: None,
            gas_used: 10,
        },
        signature: SignatureBytes(vec![0; 64]),
    };

    stub.send_receipt(LocalMeshSubmitReceiptMessage {
        receipt: receipt.clone(),
    })
    .await
    .unwrap();

    sleep(Duration::from_millis(200)).await;

    let lifecycle = ctx.get_job_lifecycle(&job_id).await.unwrap();
    assert_eq!(lifecycle.job.spec_json, Some(spec_json));
    assert_eq!(lifecycle.bids.len(), 2);
    assert_eq!(lifecycle.receipts.len(), 1);
    assert_eq!(lifecycle.receipts[0].executor_did, bid1.executor_did);
}
