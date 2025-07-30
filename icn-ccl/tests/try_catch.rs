use icn_ccl::compile_ccl_file_to_wasm;
use icn_common::{Cid, DagBlock};
use icn_identity::{did_key_from_verifying_key, generate_ed25519_keypair, SignatureBytes};
use icn_mesh::{ActualMeshJob, JobId, JobSpec};
use icn_runtime::{
    context::RuntimeContext,
    decode_result_i32, encode_result_i32,
    executor::{JobExecutor, WasmExecutor, WasmExecutorConfig},
};
use std::path::Path;
use std::str::FromStr;

async fn compile_and_run(path: &Path, tag: &[u8]) -> icn_identity::ExecutionReceipt {
    let (wasm, _) = compile_ccl_file_to_wasm(path).expect("compile");
    let ctx = RuntimeContext::new_with_stubs_and_mana("did:key:zTry", 10).unwrap();
    let ts = 0u64;
    let author = icn_common::Did::new("key", "tester");
    let sig_opt = None;
    let cid = icn_common::compute_merkle_cid(0x71, &wasm, &[], ts, &author, &sig_opt, &None);
    let block = DagBlock {
        cid: cid.clone(),
        data: wasm,
        links: vec![],
        timestamp: ts,
        author_did: author,
        signature: sig_opt,
        scope: None,
    };
    {
        let mut store = ctx.dag_store.store.lock().await;
        store.put(&block).await.unwrap();
    }
    let (sk, vk) = generate_ed25519_keypair();
    let node_did = icn_common::Did::from_str(&did_key_from_verifying_key(&vk)).unwrap();
    let job = ActualMeshJob {
        id: JobId(Cid::new_v1_sha256(0x55, tag)),
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
    receipt
}

#[tokio::test(flavor = "multi_thread")]
async fn catch_arm_executed() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/contracts/try_catch_ok.ccl");
    let receipt = compile_and_run(&path, b"job_try_ok").await;
    let encoded = encode_result_i32(Ok(42));
    let expected = Cid::new_v1_sha256(0x55, &encoded.to_le_bytes());
    assert_eq!(receipt.result_cid, expected);
    assert_eq!(decode_result_i32(encoded), Ok(42));
}

#[tokio::test(flavor = "multi_thread")]
async fn error_propagates() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/contracts/try_catch_propagate.ccl");
    let receipt = compile_and_run(&path, b"job_try_err").await;
    let encoded = encode_result_i32(Err(9));
    let expected = Cid::new_v1_sha256(0x55, &encoded.to_le_bytes());
    assert_eq!(receipt.result_cid, expected);
    assert_eq!(decode_result_i32(encoded), Err(9));
}
