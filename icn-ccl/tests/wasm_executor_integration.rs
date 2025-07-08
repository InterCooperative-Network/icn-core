use icn_ccl::{compile_ccl_file_to_wasm, compile_ccl_source_to_wasm};
use icn_common::{Cid, DagBlock};
use icn_dag::InMemoryDagStore;
use icn_identity::{did_key_from_verifying_key, generate_ed25519_keypair, SignatureBytes};
use icn_mesh::{ActualMeshJob, JobId, JobSpec};
use icn_runtime::context::{RuntimeContext, StubMeshNetworkService, StubSigner};
use icn_runtime::executor::{JobExecutor, WasmExecutor, WasmExecutorConfig};
use std::str::FromStr;
use std::sync::Arc;
use std::thread;
use tokio::runtime::Runtime;
use tokio::sync::Mutex as TokioMutex;

fn ctx_with_temp_store(did: &str, mana: u64) -> Arc<RuntimeContext> {
    let temp = tempfile::tempdir().unwrap();
    let dag_store = Arc::new(TokioMutex::new(InMemoryDagStore::new()));
    let ctx = RuntimeContext::new_with_ledger_path(
        icn_common::Did::from_str(did).unwrap(),
        Arc::new(StubMeshNetworkService::new()),
        Arc::new(StubSigner::new().unwrap()),
        Arc::new(icn_identity::KeyDidResolver),
        dag_store,
        temp.path().join("mana"),
        temp.path().join("reputation"),
        None,
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
        store.put(&block).unwrap();
    }
    let cid = block.cid.clone();

    let (sk, vk) = generate_ed25519_keypair();
    let node_did = icn_common::Did::from_str(&did_key_from_verifying_key(&vk)).unwrap();

    let job = ActualMeshJob {
        id: JobId(Cid::new_v1_sha256(0x55, b"job")),
        manifest_cid: cid,
        spec: JobSpec::default(),
        creator_did: node_did.clone(),
        cost_mana: 0,
        max_execution_wait_ms: None,
        signature: SignatureBytes(vec![]),
    };

    let signer = Arc::new(StubSigner::new_with_keys(sk, vk).unwrap());
    let exec = WasmExecutor::new(ctx.clone(), signer, WasmExecutorConfig::default());
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
        store.put(&block).unwrap();
    }
    let cid = block.cid.clone();

    let (sk, vk) = generate_ed25519_keypair();
    let node_did = icn_common::Did::from_str(&did_key_from_verifying_key(&vk)).unwrap();

    let job = ActualMeshJob {
        id: JobId(Cid::new_v1_sha256(0x55, b"jobadd")),
        manifest_cid: cid,
        spec: JobSpec::default(),
        creator_did: node_did.clone(),
        cost_mana: 0,
        max_execution_wait_ms: None,
        signature: SignatureBytes(vec![]),
    };

    let signer = Arc::new(StubSigner::new_with_keys(sk, vk).unwrap());
    let exec = WasmExecutor::new(ctx.clone(), signer, WasmExecutorConfig::default());
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
    let ts = 0u64;
    let author = icn_common::Did::new("key", "tester");
    let sig_opt = None;
    let cid_calc = icn_common::compute_merkle_cid(0x71, &wasm, &[], ts, &author, &sig_opt, &None);
    let block = DagBlock {
        cid: cid_calc.clone(),
        data: wasm.clone(),
        links: vec![],
        timestamp: ts,
        author_did: author,
        signature: sig_opt,
        scope: None,
    };
    {
        let mut store = ctx.dag_store.lock().await;
        store.put(&block).unwrap();
    }
    let cid = block.cid.clone();

    let (sk, vk) = generate_ed25519_keypair();
    let node_did = icn_common::Did::from_str(&did_key_from_verifying_key(&vk)).unwrap();

    let job = ActualMeshJob {
        id: JobId(Cid::new_v1_sha256(0x55, b"job2")),
        manifest_cid: cid,
        spec: JobSpec::default(),
        creator_did: node_did.clone(),
        cost_mana: 0,
        max_execution_wait_ms: None,
        signature: SignatureBytes(vec![]),
    };

    let signer = Arc::new(StubSigner::new_with_keys(sk, vk).unwrap());
    let exec = WasmExecutor::new(ctx.clone(), signer, WasmExecutorConfig::default());
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
        store.put(&block).unwrap();
    }

    let (sk, vk) = generate_ed25519_keypair();
    let node_did = icn_common::Did::from_str(&did_key_from_verifying_key(&vk)).unwrap();
    let job = ActualMeshJob {
        id: JobId(Cid::new_v1_sha256(0x55, b"job_simple")),
        manifest_cid: cid,
        spec: JobSpec::default(),
        creator_did: node_did.clone(),
        cost_mana: 0,
        max_execution_wait_ms: None,
        signature: SignatureBytes(vec![]),
    };

    let signer = Arc::new(StubSigner::new_with_keys(sk, vk).unwrap());
    let exec = WasmExecutor::new(ctx.clone(), signer, WasmExecutorConfig::default());
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

#[tokio::test(flavor = "multi_thread")]
async fn wasm_executor_runs_compiled_file() {
    let contract_path =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/contracts/example.ccl");
    let (wasm, _) = compile_ccl_file_to_wasm(&contract_path).expect("compile file");

    let ctx = ctx_with_temp_store("did:key:zFileExec", 10);
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
        store.put(&block).unwrap();
    }

    let (sk, vk) = generate_ed25519_keypair();
    let node_did = icn_common::Did::from_str(&did_key_from_verifying_key(&vk)).unwrap();
    let job = ActualMeshJob {
        id: JobId(Cid::new_v1_sha256(0x55, b"job_file")),
        manifest_cid: cid,
        spec: JobSpec {
            kind: icn_mesh::JobKind::CclWasm,
            ..Default::default()
        },
        creator_did: node_did.clone(),
        cost_mana: 0,
        max_execution_wait_ms: None,
        signature: SignatureBytes(vec![]),
    };

    let signer = Arc::new(StubSigner::new_with_keys(sk, vk).unwrap());
    let exec = WasmExecutor::new(ctx.clone(), signer, WasmExecutorConfig::default());
    let job_clone = job.clone();
    let handle = std::thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        rt.block_on(async { exec.execute_job(&job_clone).await })
    });
    let receipt = handle.join().unwrap().unwrap();
    assert_eq!(receipt.executor_did, node_did);
    let expected_cid = Cid::new_v1_sha256(0x55, &11i64.to_le_bytes());
    assert_eq!(receipt.result_cid, expected_cid);
}

#[tokio::test(flavor = "multi_thread")]
async fn wasm_executor_runs_while_loop() {
    let contract_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/contracts/while_counter.ccl");
    let (wasm, _) = compile_ccl_file_to_wasm(&contract_path).expect("compile file");

    let ctx = ctx_with_temp_store("did:key:zWhile", 10);
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
        store.put(&block).unwrap();
    }

    let (sk, vk) = generate_ed25519_keypair();
    let node_did = icn_common::Did::from_str(&did_key_from_verifying_key(&vk)).unwrap();
    let job = ActualMeshJob {
        id: JobId(Cid::new_v1_sha256(0x55, b"job_while")),
        manifest_cid: cid,
        spec: JobSpec::default(),
        creator_did: node_did.clone(),
        cost_mana: 0,
        max_execution_wait_ms: None,
        signature: SignatureBytes(vec![]),
    };

    let signer = Arc::new(StubSigner::new_with_keys(sk, vk).unwrap());
    let exec = WasmExecutor::new(ctx.clone(), signer, WasmExecutorConfig::default());
    let job_clone = job.clone();
    let handle = std::thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        rt.block_on(async { exec.execute_job(&job_clone).await })
    });
    let receipt = handle.join().unwrap().unwrap();
    assert_eq!(receipt.executor_did, node_did);
    let expected_cid = Cid::new_v1_sha256(0x55, &5i64.to_le_bytes());
    assert_eq!(receipt.result_cid, expected_cid);
}

#[tokio::test(flavor = "multi_thread")]
async fn contract_queries_reputation() {
    let source = "fn run() -> Integer { return host_get_reputation(); }";
    let (wasm, _meta) = compile_ccl_source_to_wasm(source).expect("compile ccl");

    let ctx = ctx_with_temp_store("did:key:zRepQuery", 10);
    ctx.reputation_store
        .record_execution(&ctx.current_identity, true, 0);
    ctx.reputation_store
        .record_execution(&ctx.current_identity, true, 0);
    let expected_rep = ctx.reputation_store.get_reputation(&ctx.current_identity);

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
        store.put(&block).unwrap();
    }

    let (sk, vk) = generate_ed25519_keypair();
    let node_did = icn_common::Did::from_str(&did_key_from_verifying_key(&vk)).unwrap();
    let job = ActualMeshJob {
        id: JobId(Cid::new_v1_sha256(0x55, b"job_rep")),
        manifest_cid: cid,
        spec: JobSpec::default(),
        creator_did: node_did.clone(),
        cost_mana: 0,
        max_execution_wait_ms: None,
        signature: SignatureBytes(vec![]),
    };

    let signer = Arc::new(StubSigner::new_with_keys(sk, vk).unwrap());
    let exec = WasmExecutor::new(ctx.clone(), signer, WasmExecutorConfig::default());
    let job_clone = job.clone();
    let handle = std::thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        rt.block_on(async { exec.execute_job(&job_clone).await })
    });
    let receipt = handle.join().unwrap().unwrap();
    assert_eq!(receipt.executor_did, node_did);
    let expected_cid = Cid::new_v1_sha256(0x55, &(expected_rep as i64).to_le_bytes());
    assert_eq!(receipt.result_cid, expected_cid);
}
