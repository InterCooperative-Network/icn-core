use icn_common::{Cid, DagBlock, Did, compute_merkle_cid};
use icn_ccl::compile_ccl_source_to_wasm;
use icn_runtime::{context::{RuntimeContext, StubDagStore, StubMeshNetworkService, StubSigner}, host_submit_mesh_job};
use icn_mesh::{ActualMeshJob, JobSpec, JobKind};
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;
use serde_json;

#[tokio::test]
async fn compiled_policy_executes_via_host_abi() {
    // Setup runtime context with stub services and mana
    let dag_store = Arc::new(TokioMutex::new(StubDagStore::new()));
    let ctx = RuntimeContext::new_with_mana_ledger(
        Did::new("key", "tester"),
        Arc::new(StubMeshNetworkService::new()),
        Arc::new(StubSigner::new().unwrap()),
        Arc::new(icn_identity::KeyDidResolver),
        dag_store.clone(),
        icn_runtime::context::SimpleManaLedger::default(),
        std::path::PathBuf::from("./rep.sled"),
    );
    ctx.mana_ledger.set_balance(&ctx.current_identity, 5).unwrap();

    // Compile simple CCL policy
    let source = "fn run() -> Integer { return 2; }";
    let (wasm, mut meta) = compile_ccl_source_to_wasm(source).unwrap();

    // Store WASM module
    let ts = 0u64;
    let author = ctx.current_identity.clone();
    let sig = None;
    let wasm_cid = compute_merkle_cid(0x71, &wasm, &[], ts, &author, &sig);
    let wasm_block = DagBlock { cid: wasm_cid.clone(), data: wasm, links: vec![], timestamp: ts, author_did: author.clone(), signature: sig };
    {
        let mut store = dag_store.lock().await;
        store.put(&wasm_block).unwrap();
    }

    // Store metadata referencing the wasm CID
    meta.cid = wasm_cid.to_string();
    let meta_bytes = serde_json::to_vec(&meta).unwrap();
    let meta_cid = compute_merkle_cid(0x80, &meta_bytes, &[], ts, &author, &None, &None);
    let meta_block = DagBlock { cid: meta_cid.clone(), data: meta_bytes, links: vec![], timestamp: ts, author_did: author, signature: None, scope: None };
    {
        let mut store = dag_store.lock().await;
        store.put(&meta_block).unwrap();
    }

    // Submit job referencing metadata CID
    let job = ActualMeshJob {
        id: Cid::new_v1_sha256(0x55, b"job"),
        manifest_cid: meta_cid,
        spec: JobSpec { kind: JobKind::CclWasm, ..Default::default() },
        creator_did: ctx.current_identity.clone(),
        cost_mana: 0,
        max_execution_wait_ms: None,
        signature: icn_identity::SignatureBytes(vec![]),
    };
    let job_json = serde_json::to_string(&job).unwrap();
    let job_id = host_submit_mesh_job(&ctx, &job_json).await.unwrap();

    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    let states = ctx.job_states.lock().await;
    if let Some(icn_mesh::JobState::Completed { receipt }) = states.get(&job_id) {
        let expected = Cid::new_v1_sha256(0x55, &2i64.to_le_bytes());
        assert_eq!(receipt.result_cid, expected);
    } else {
        panic!("job not completed");
    }
}
