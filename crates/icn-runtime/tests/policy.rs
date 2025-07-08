use icn_common::{Cid, Did};
use icn_governance::scoped_policy::InMemoryPolicyEnforcer;
use icn_identity::{ExecutionReceipt, SignatureBytes};
use icn_runtime::{
    context::{HostAbiError, RuntimeContext, StubDagStore, StubMeshNetworkService, StubSigner},
    host_anchor_receipt, ReputationUpdater,
};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

#[tokio::test]
async fn anchor_receipt_denied_by_policy() {
    let did = Did::from_str("did:icn:test:denied").unwrap();
    let ctx = RuntimeContext::new_with_ledger_path(
        did.clone(),
        Arc::new(StubMeshNetworkService::new()),
        Arc::new(StubSigner::new().unwrap()),
        Arc::new(icn_identity::KeyDidResolver),
        Arc::new(TokioMutex::new(StubDagStore::new())),
        PathBuf::from("./mana_ledger.sled"),
        PathBuf::from("./reputation.sled"),
        Some(Arc::new(InMemoryPolicyEnforcer::new(
            HashSet::new(),
            HashSet::new(),
            HashMap::new(),
        ))),
    );

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
