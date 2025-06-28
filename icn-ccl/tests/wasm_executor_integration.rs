use icn_ccl::compile_ccl_source_to_wasm;
use icn_common::{Cid, DagBlock};
use icn_dag::sled_store::SledDagStore;
use icn_identity::{did_key_from_verifying_key, generate_ed25519_keypair, SignatureBytes};
use icn_mesh::{ActualMeshJob, JobSpec};
use icn_runtime::context::{RuntimeContext, StubMeshNetworkService, StubSigner};
use icn_runtime::executor::{JobExecutor, WasmExecutor};
use std::str::FromStr;
use std::sync::Arc;
use std::thread;
use tokio::runtime::Runtime;
use tokio::sync::Mutex as TokioMutex;

fn ctx_with_temp_store(did: &str, mana: u64) -> Arc<RuntimeContext> {
    let temp = tempfile::tempdir().unwrap();
    let dag_store = Arc::new(TokioMutex::new(
        SledDagStore::new(temp.path().join("dag")).unwrap(),
    ));
    let ctx = RuntimeContext::new_with_ledger_path(
        icn_common::Did::from_str(did).unwrap(),
        Arc::new(StubMeshNetworkService::new()),
        Arc::new(StubSigner::new()),
        Arc::new(icn_identity::KeyDidResolver),
        dag_store,
        temp.path().join("mana"),
    );
    ctx.mana_ledger
        .set_balance(&icn_common::Did::from_str(did).unwrap(), mana)
        .unwrap();
    ctx
}

#[tokio::test(flavor = "multi_thread")]
async fn wasm_executor_runs_compiled_ccl() {
    let source = "fn run() -> Integer { return 6; }";
    let (wasm, _) = compile_ccl_source_to_wasm(source).expect("compile ccl");

    let ctx = ctx_with_temp_store("did:key:zWasmExec", 10);
    let cid = icn_common::compute_merkle_cid(0x71, &wasm, &[]);
    let block = DagBlock {
        cid: cid.clone(),
        data: wasm.clone(),
        links: vec![],
    };
    {
        let mut store = ctx.dag_store.lock().await;
        store.put(&block).unwrap();
    }
    let cid = block.cid.clone();

    let (sk, vk) = generate_ed25519_keypair();
    let node_did = icn_common::Did::from_str(&did_key_from_verifying_key(&vk)).unwrap();

    let job = ActualMeshJob {
        id: Cid::new_v1_sha256(0x55, b"job"),
        manifest_cid: cid,
        spec: JobSpec::default(),
        creator_did: node_did.clone(),
        cost_mana: 0,
        max_execution_wait_ms: None,
        signature: SignatureBytes(vec![]),
    };

    let exec = WasmExecutor::new(ctx.clone(), node_did.clone(), sk);
    let job_clone = job.clone();
    let handle = thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        rt.block_on(async { exec.execute_job(&job_clone).await })
    });
    let receipt = handle.join().unwrap().unwrap();
    assert_eq!(receipt.executor_did, node_did);
    let expected_cid = Cid::new_v1_sha256(0x55, &6i64.to_le_bytes());
    assert_eq!(receipt.result_cid, expected_cid);
}

#[tokio::test(flavor = "multi_thread")]
async fn wasm_executor_runs_compiled_addition() {
    let source = "fn run() -> Integer { return 40 + 2; }";
    let (wasm, _) = compile_ccl_source_to_wasm(source).expect("compile ccl");

    let ctx = ctx_with_temp_store("did:key:zAddExec", 10);
    let cid = icn_common::compute_merkle_cid(0x71, &wasm, &[]);
    let block = DagBlock {
        cid: cid.clone(),
        data: wasm.clone(),
        links: vec![],
    };
    {
        let mut store = ctx.dag_store.lock().await;
        store.put(&block).unwrap();
    }
    let cid = block.cid.clone();

    let (sk, vk) = generate_ed25519_keypair();
    let node_did = icn_common::Did::from_str(&did_key_from_verifying_key(&vk)).unwrap();

    let job = ActualMeshJob {
        id: Cid::new_v1_sha256(0x55, b"jobadd"),
        manifest_cid: cid,
        spec: JobSpec::default(),
        creator_did: node_did.clone(),
        cost_mana: 0,
        max_execution_wait_ms: None,
        signature: SignatureBytes(vec![]),
    };

    let exec = WasmExecutor::new(ctx.clone(), node_did.clone(), sk);
    let job_clone = job.clone();
    let handle = thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        rt.block_on(async { exec.execute_job(&job_clone).await })
    });
    let receipt = handle.join().unwrap().unwrap();
    assert_eq!(receipt.executor_did, node_did);
    let expected_cid = Cid::new_v1_sha256(0x55, &42i64.to_le_bytes());
    assert_eq!(receipt.result_cid, expected_cid);
}

#[tokio::test(flavor = "multi_thread")]
async fn wasm_executor_fails_without_run() {
    let source = "fn not_run() -> Integer { return 1; }";
    let (wasm, _) = compile_ccl_source_to_wasm(source).expect("compile ccl");

    let ctx = ctx_with_temp_store("did:key:zWasmFail", 10);
    let cid_calc = icn_common::compute_merkle_cid(0x71, &wasm, &[]);
    let block = DagBlock {
        cid: cid_calc.clone(),
        data: wasm.clone(),
        links: vec![],
    };
    {
        let mut store = ctx.dag_store.lock().await;
        store.put(&block).unwrap();
    }
    let cid = block.cid.clone();

    let (sk, vk) = generate_ed25519_keypair();
    let node_did = icn_common::Did::from_str(&did_key_from_verifying_key(&vk)).unwrap();

    let job = ActualMeshJob {
        id: Cid::new_v1_sha256(0x55, b"job2"),
        manifest_cid: cid,
        spec: JobSpec::default(),
        creator_did: node_did.clone(),
        cost_mana: 0,
        max_execution_wait_ms: None,
        signature: SignatureBytes(vec![]),
    };

    let exec = WasmExecutor::new(ctx.clone(), node_did, sk);
    let job_clone = job.clone();
    let handle = thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        rt.block_on(async { exec.execute_job(&job_clone).await })
    });
    let result = handle.join().unwrap();
    assert!(result.is_err());
}

#[tokio::test(flavor = "multi_thread")]
async fn compile_and_execute_simple_contract() {
    let source = "fn run() -> Integer { return 8; }";
    let (wasm, meta) = compile_ccl_source_to_wasm(source).expect("compile ccl");
    assert!(meta.exports.contains(&"run".to_string()));

    let ctx = ctx_with_temp_store("did:key:zSimple", 5);
    let cid = icn_common::compute_merkle_cid(0x71, &wasm, &[]);
    let block = DagBlock {
        cid: cid.clone(),
        data: wasm.clone(),
        links: vec![],
    };
    {
        let mut store = ctx.dag_store.lock().await;
        store.put(&block).unwrap();
    }

    let (sk, vk) = generate_ed25519_keypair();
    let node_did = icn_common::Did::from_str(&did_key_from_verifying_key(&vk)).unwrap();
    let job = ActualMeshJob {
        id: Cid::new_v1_sha256(0x55, b"job_simple"),
        manifest_cid: cid,
        spec: JobSpec::default(),
        creator_did: node_did.clone(),
        cost_mana: 0,
        max_execution_wait_ms: None,
        signature: SignatureBytes(vec![]),
    };

    let exec = WasmExecutor::new(ctx.clone(), node_did.clone(), sk);
    let job_clone = job.clone();
    let handle = thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        rt.block_on(async { exec.execute_job(&job_clone).await })
    });
    let receipt = handle.join().unwrap().unwrap();
    assert_eq!(receipt.executor_did, node_did);
    let expected_cid = Cid::new_v1_sha256(0x55, &8i64.to_le_bytes());
    assert_eq!(receipt.result_cid, expected_cid);
}
