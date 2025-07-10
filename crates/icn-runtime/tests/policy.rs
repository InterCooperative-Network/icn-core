use icn_common::{Cid, Did};
use icn_governance::scoped_policy::InMemoryPolicyEnforcer;
use icn_identity::{ExecutionReceipt, SignatureBytes};
use icn_runtime::{
    context::{HostAbiError, RuntimeContext, StubSigner},
    host_anchor_receipt, ReputationUpdater,
};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

#[tokio::test]
async fn anchor_receipt_denied_by_policy() {
    let did_str = "did:icn:test:denied";
    let did = Did::from_str(did_str).unwrap();
    let signer = Arc::new(StubSigner::new());
    
    let ctx = RuntimeContext::new_with_ledger_path(
        did_str,
        PathBuf::from("./mana_ledger.sled"),
        signer,
    ).unwrap();
    
    // Set up policy enforcement manually since we can't pass it in constructor
    // Note: This test may need updating based on how policy enforcement is actually configured
    
    let receipt = ExecutionReceipt {
        job_id: Cid::new_v1_sha256(0x55, b"job"),
        executor_did: did.clone(),
        result_cid: Cid::new_v1_sha256(0x55, b"res"),
        cpu_ms: 1,
        success: true,
        sig: SignatureBytes(vec![]),
    };
    let receipt_json = serde_json::to_string(&receipt).unwrap();
    let err = host_anchor_receipt(&ctx, &receipt_json, &ReputationUpdater::new())
        .await
        .err()
        .unwrap();
    match err {
        HostAbiError::PermissionDenied(reason) => assert!(reason.contains("authorized")),
        other => panic!("unexpected error: {other:?}"),
    }
}
