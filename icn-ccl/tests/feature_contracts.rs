use icn_ccl::compile_ccl_file_to_wasm;
use icn_common::{Cid, DagBlock};
use icn_identity::{did_key_from_verifying_key, generate_ed25519_keypair, SignatureBytes};
use icn_mesh::{ActualMeshJob, JobId, JobSpec};
use icn_runtime::context::RuntimeContext;
use icn_runtime::executor::{JobExecutor, WasmExecutor, WasmExecutorConfig};
use std::str::FromStr;

async fn compile_and_run(path: &std::path::Path, job_tag: &[u8], expected: i64) {
    let (wasm, _) = compile_ccl_file_to_wasm(path).expect("compile file");
    let ctx = RuntimeContext::new_with_stubs_and_mana("did:key:zFeature", 10).unwrap();
    let ts = 0u64;
    let author = icn_common::Did::new("key", "tester");
    let sig_opt = None;
    let cid = icn_common::compute_merkle_cid(0x71, &wasm, &[], ts, &author, &sig_opt, &None);
    let block = DagBlock {
        cid: cid.clone(),
        data: wasm.clone(),
        links: vec![],
        timestamp: ts,
        author_did: author,
        signature: sig_opt,
        scope: None,
    };
    {
        let mut store = ctx.dag_store.lock().await;
        store.put(&block).await.unwrap();
    }
    let (sk, vk) = generate_ed25519_keypair();
    let node_did = icn_common::Did::from_str(&did_key_from_verifying_key(&vk)).unwrap();
    let job = ActualMeshJob {
        id: JobId(Cid::new_v1_sha256(0x55, job_tag)),
        manifest_cid: cid,
        spec: JobSpec::default(),
        creator_did: node_did.clone(),
        cost_mana: 0,
        max_execution_wait_ms: None,
        signature: SignatureBytes(vec![]),
    };
    let signer = std::sync::Arc::new(icn_runtime::context::StubSigner::new_with_keys(sk, vk));
    let exec = WasmExecutor::new(ctx.clone(), signer, WasmExecutorConfig::default());
    let receipt = exec.execute_job(&job).await.unwrap();
    assert_eq!(receipt.executor_did, node_did);
    let expected_cid = Cid::new_v1_sha256(0x55, &expected.to_le_bytes());
    assert_eq!(receipt.result_cid, expected_cid);
}

#[tokio::test(flavor = "multi_thread")]
async fn else_if_chain_contract_runs() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/contracts/else_if_chain.ccl");
    compile_and_run(&path, b"job_elif", 2).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn array_push_pop_contract_runs() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/contracts/array_push_pop.ccl");
    compile_and_run(&path, b"job_arr", 9).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn string_concat_contract_runs() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/contracts/string_concat.ccl");
    compile_and_run(&path, b"job_str", 10).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn budgeting_contract_runs() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ccl-lib/budgeting.ccl");
    compile_and_run(&path, b"job_budget", 1).await;
}
