use icn_common::{Cid, Did};
use icn_identity::SignatureBytes;
use icn_mesh::{ActualMeshJob, JobId, JobKind, JobSpec, Resources};
use icn_runtime::context::RuntimeContext;
use std::str::FromStr;
use std::sync::Arc;

#[tokio::test]
async fn bid_price_deterministic() {
    let did = Did::from_str("did:icn:test:exec").unwrap();
    let ctx = RuntimeContext::new_testing(did.clone(), Some(1000)).unwrap();

    let job = ActualMeshJob {
        id: JobId(Cid::new_v1_sha256(0x55, b"bid_price")),
        manifest_cid: Cid::new_v1_sha256(0x55, b"man"),
        spec: JobSpec {
            kind: JobKind::GenericPlaceholder,
            inputs: vec![],
            outputs: vec![],
            required_resources: Resources {
                cpu_cores: 1,
                memory_mb: 128,
                storage_mb: 0,
            },
        },
        creator_did: did.clone(),
        cost_mana: 10,
        max_execution_wait_ms: None,
        signature: SignatureBytes(vec![]),
    };

    let bid1 = RuntimeContext::evaluate_and_bid_on_job(&ctx, &job)
        .await
        .unwrap()
        .expect("bid1");
    let bid2 = RuntimeContext::evaluate_and_bid_on_job(&ctx, &job)
        .await
        .unwrap()
        .expect("bid2");
    assert_eq!(bid1.price_mana, bid2.price_mana);
}
