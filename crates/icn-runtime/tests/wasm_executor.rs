use icn_common::{compute_merkle_cid, Cid, DagBlock, Did};
use icn_identity::{did_key_from_verifying_key, generate_ed25519_keypair, SignatureBytes};
use icn_mesh::{ActualMeshJob, JobSpec};
use icn_runtime::context::{RuntimeContext, StubSigner};
use icn_runtime::executor::{JobExecutor, WasmExecutor};
use icn_runtime::host_submit_mesh_job;
use std::str::FromStr;
use std::sync::Arc;

#[tokio::test(flavor = "multi_thread")]
async fn wasm_executor_runs_wasm() {
    let ctx = RuntimeContext::new_with_stubs_and_mana("did:key:zTestExec", 42);
    let (sk, vk) = generate_ed25519_keypair();
    let node_did = did_key_from_verifying_key(&vk);
    let node_did = icn_common::Did::from_str(&node_did).unwrap();

    let wasm = r#"(module
        (import "icn" "host_account_get_mana" (func $get (result i64)))
        (func (export "run") (result i64)
            call $get
        )
    )"#;
    let wasm_bytes = wat::parse_str(wasm).unwrap();
    let ts = 0u64;
    let author = Did::new("key", "tester");
    let sig_opt = None;
    let cid = compute_merkle_cid(0x71, &wasm_bytes, &[], ts, &author, &sig_opt);
    let block = DagBlock {
        cid: cid.clone(),
        data: wasm_bytes,
        links: vec![],
        timestamp: ts,
        author_did: author,
        signature: sig_opt,
    };
    {
        let mut store = ctx.dag_store.lock().await;
        store.put(&block).unwrap();
    }
    let cid = block.cid.clone();

    let job = ActualMeshJob {
        id: Cid::new_v1_sha256(0x55, b"job"),
        manifest_cid: cid,
        spec: JobSpec::default(),
        creator_did: node_did.clone(),
        cost_mana: 0,
        max_execution_wait_ms: None,
        signature: SignatureBytes(vec![]),
    };

    let signer = Arc::new(StubSigner::new_with_keys(sk, vk));
    let exec = WasmExecutor::new(ctx.clone(), signer);
    let receipt = exec.execute_job(&job).await.unwrap();
    assert_eq!(receipt.executor_did, node_did);
}

#[tokio::test(flavor = "multi_thread")]
async fn wasm_executor_runs_compiled_ccl_contract() {
    let ctx = RuntimeContext::new_with_stubs_and_mana("did:key:zCclExec", 10);
    let (sk, vk) = generate_ed25519_keypair();
    let node_did = icn_common::Did::from_str(&did_key_from_verifying_key(&vk)).unwrap();

    let source = "fn run() -> Integer { return 3 + 4; }";
    let (wasm, _) = icn_ccl::compile_ccl_source_to_wasm(source).unwrap();
    let ts = 0u64;
    let author = Did::new("key", "tester");
    let sig_opt = None;
    let cid_calc = compute_merkle_cid(0x71, &wasm, &[], ts, &author, &sig_opt);
    let block = DagBlock {
        cid: cid_calc.clone(),
        data: wasm.clone(),
        links: vec![],
        timestamp: ts,
        author_did: author,
        signature: sig_opt,
    };
    {
        let mut store = ctx.dag_store.lock().await;
        store.put(&block).unwrap();
    }
    let cid = block.cid.clone();

    let job = ActualMeshJob {
        id: Cid::new_v1_sha256(0x55, b"job_ccl"),
        manifest_cid: cid,
        spec: JobSpec::default(),
        creator_did: node_did.clone(),
        cost_mana: 0,
        max_execution_wait_ms: None,
        signature: SignatureBytes(vec![]),
    };

    let signer = Arc::new(StubSigner::new_with_keys(sk, vk));
    let exec = WasmExecutor::new(ctx.clone(), signer);
    let receipt = exec.execute_job(&job).await.unwrap();
    assert_eq!(receipt.executor_did, node_did);
    let expected_cid = Cid::new_v1_sha256(0x55, &7i64.to_le_bytes());
    assert_eq!(receipt.result_cid, expected_cid);
}

#[tokio::test(flavor = "multi_thread")]
async fn wasm_executor_host_submit_mesh_job_json() {
    use icn_mesh::{JobKind, Resources};

    let ctx = RuntimeContext::new_with_stubs_and_mana("did:key:zHostSubmit", 50);
    let (sk, vk) = generate_ed25519_keypair();
    let node_did = icn_common::Did::from_str(&did_key_from_verifying_key(&vk)).unwrap();

    let complex_job = ActualMeshJob {
        id: Cid::new_v1_sha256(0x55, b"embedded"),
        manifest_cid: Cid::new_v1_sha256(0x71, b"embedded"),
        spec: JobSpec {
            kind: JobKind::GenericPlaceholder,
            inputs: vec![
                Cid::new_v1_sha256(0x71, b"in1"),
                Cid::new_v1_sha256(0x71, b"in2"),
            ],
            outputs: vec!["out1".to_string(), "out2".to_string()],
            required_resources: Resources {
                cpu_cores: 2,
                memory_mb: 256,
            },
        },
        creator_did: node_did.clone(),
        cost_mana: 10,
        max_execution_wait_ms: Some(1000),
        signature: SignatureBytes(vec![]),
    };

    let job_json = serde_json::to_string(&complex_job).unwrap();
    let escaped = job_json.replace('\\', "\\\\").replace('"', "\\\"");
    let wasm = format!(
        "(module\n  (import \"icn\" \"host_submit_mesh_job\" (func $s (param i32 i32)))\n  (import \"icn\" \"host_account_get_mana\" (func $g (result i64)))\n  (memory (export \"memory\") 1)\n  (data (i32.const 0) \"{data}\")\n  (func (export \"run\") (result i64)\n    i32.const 0\n    i32.const {len}\n    call $s\n    call $g)\n)",
        data = escaped,
        len = job_json.len()
    );

    let wasm_bytes = wat::parse_str(&wasm).unwrap();
    let ts = 0u64;
    let author = Did::new("key", "tester");
    let sig_opt = None;
    let cid_calc = compute_merkle_cid(0x71, &wasm_bytes, &[], ts, &author, &sig_opt);
    let block = DagBlock {
        cid: cid_calc.clone(),
        data: wasm_bytes,
        links: vec![],
        timestamp: ts,
        author_did: author,
        signature: sig_opt,
    };
    {
        let mut store = ctx.dag_store.lock().await;
        store.put(&block).unwrap();
    }

    let job = ActualMeshJob {
        id: Cid::new_v1_sha256(0x55, b"job_host"),
        manifest_cid: block.cid.clone(),
        spec: JobSpec::default(),
        creator_did: node_did.clone(),
        cost_mana: 0,
        max_execution_wait_ms: None,
        signature: SignatureBytes(vec![]),
    };

    let signer = Arc::new(StubSigner::new_with_keys(sk, vk));
    let exec = WasmExecutor::new(ctx.clone(), signer);
    let receipt = exec.execute_job(&job).await.unwrap();

    let expected_cid = Cid::new_v1_sha256(0x55, &(40i64).to_le_bytes());
    assert_eq!(receipt.result_cid, expected_cid);
    assert_eq!(ctx.get_mana(&ctx.current_identity).await.unwrap(), 40);
    let pending = ctx.pending_mesh_jobs.lock().await;
    assert_eq!(pending.len(), 1);
}

#[tokio::test(flavor = "multi_thread")]
async fn wasm_executor_host_anchor_receipt_json() {
    use icn_identity::ExecutionReceipt;

    let ctx = RuntimeContext::new_with_stubs_and_mana("did:key:zHostAnchor", 10);
    let (exec_sk, exec_vk) = generate_ed25519_keypair();
    let executor_did = icn_common::Did::from_str(&did_key_from_verifying_key(&exec_vk)).unwrap();
    let (node_sk, node_vk) = generate_ed25519_keypair();
    let node_did = icn_common::Did::from_str(&did_key_from_verifying_key(&node_vk)).unwrap();

    let receipt = ExecutionReceipt {
        job_id: Cid::new_v1_sha256(0x55, b"jid"),
        executor_did: executor_did.clone(),
        result_cid: Cid::new_v1_sha256(0x55, b"res"),
        cpu_ms: 5,
        success: true,
        sig: SignatureBytes(vec![]),
    };
    let receipt = receipt.sign_with_key(&exec_sk).unwrap();
    let receipt_json = serde_json::to_string(&receipt).unwrap();
    let escaped = receipt_json.replace('\\', "\\\\").replace('"', "\\\"");
    let wasm = format!(
        "(module\n  (import \"icn\" \"host_anchor_receipt\" (func $a (param i32 i32)))\n  (memory (export \"memory\") 1)\n  (data (i32.const 0) \"{data}\")\n  (func (export \"run\") (result i64)\n    i32.const 0\n    i32.const {len}\n    call $a\n    i64.const 1)\n)",
        data = escaped,
        len = receipt_json.len()
    );

    let wasm_bytes = wat::parse_str(&wasm).unwrap();
    let ts = 0u64;
    let author = Did::new("key", "tester");
    let sig_opt = None;
    let cid_calc = compute_merkle_cid(0x71, &wasm_bytes, &[], ts, &author, &sig_opt);
    let block = DagBlock {
        cid: cid_calc.clone(),
        data: wasm_bytes,
        links: vec![],
        timestamp: ts,
        author_did: author,
        signature: sig_opt,
    };
    {
        let mut store = ctx.dag_store.lock().await;
        store.put(&block).unwrap();
    }

    let job = ActualMeshJob {
        id: Cid::new_v1_sha256(0x55, b"job_anchor"),
        manifest_cid: block.cid.clone(),
        spec: JobSpec::default(),
        creator_did: node_did.clone(),
        cost_mana: 0,
        max_execution_wait_ms: None,
        signature: SignatureBytes(vec![]),
    };

    let signer = Arc::new(StubSigner::new_with_keys(node_sk, node_vk));
    let exec = WasmExecutor::new(ctx.clone(), signer);
    let _ = exec.execute_job(&job).await.unwrap();

    let rec_bytes = serde_json::to_vec(&receipt).unwrap();
    let expected = Cid::new_v1_sha256(0x71, &rec_bytes);
    let store = ctx.dag_store.lock().await;
    assert!(store.get(&expected).unwrap().is_some());
    assert!(ctx.reputation_store.get_reputation(&executor_did) > 0);
}

#[tokio::test(flavor = "multi_thread")]
async fn submit_compiled_ccl_runs_via_executor() {
    let ctx = RuntimeContext::new_with_stubs_and_mana("did:key:zEndToEnd", 5);
    let source = "fn run() -> Integer { return 9; }";
    let (wasm, _) = icn_ccl::compile_ccl_source_to_wasm(source).unwrap();
    let ts = 0u64;
    let author = Did::new("key", "tester");
    let sig_opt = None;
    let cid_calc = compute_merkle_cid(0x71, &wasm, &[], ts, &author, &sig_opt);
    let block = DagBlock {
        cid: cid_calc.clone(),
        data: wasm.clone(),
        links: vec![],
        timestamp: ts,
        author_did: author,
        signature: sig_opt,
    };
    {
        let mut store = ctx.dag_store.lock().await;
        store.put(&block).unwrap();
    }
    let cid = block.cid.clone();
    let job = ActualMeshJob {
        id: Cid::new_v1_sha256(0x55, b"auto"),
        manifest_cid: cid.clone(),
        spec: JobSpec::default(),
        creator_did: ctx.current_identity.clone(),
        cost_mana: 0,
        max_execution_wait_ms: None,
        signature: SignatureBytes(vec![]),
    };
    let job_json = serde_json::to_string(&job).unwrap();
    let job_id = host_submit_mesh_job(&ctx, &job_json).await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    let states = ctx.job_states.lock().await;
    if let Some(icn_mesh::JobState::Completed { receipt }) = states.get(&job_id) {
        let expected = Cid::new_v1_sha256(0x55, &9i64.to_le_bytes());
        assert_eq!(receipt.result_cid, expected);
    } else {
        panic!("job not completed");
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn queued_compiled_ccl_executes() {
    let ctx = RuntimeContext::new_with_stubs_and_mana("did:key:zQueueExec", 5);
    let source = "fn run() -> Integer { return 4; }";
    let (wasm, _) = icn_ccl::compile_ccl_source_to_wasm(source).unwrap();
    let ts = 0u64;
    let author = Did::new("key", "tester");
    let sig_opt = None;
    let cid_calc = compute_merkle_cid(0x71, &wasm, &[], ts, &author, &sig_opt);
    let block = DagBlock {
        cid: cid_calc.clone(),
        data: wasm.clone(),
        links: vec![],
        timestamp: ts,
        author_did: author,
        signature: sig_opt,
    };
    {
        let mut store = ctx.dag_store.lock().await;
        store.put(&block).unwrap();
    }
    let cid = block.cid.clone();
    let job = ActualMeshJob {
        id: Cid::new_v1_sha256(0x55, b"queued"),
        manifest_cid: cid.clone(),
        spec: JobSpec::default(),
        creator_did: ctx.current_identity.clone(),
        cost_mana: 0,
        max_execution_wait_ms: None,
        signature: SignatureBytes(vec![]),
    };
    ctx.internal_queue_mesh_job(job.clone()).await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    let states = ctx.job_states.lock().await;
    if let Some(icn_mesh::JobState::Completed { receipt }) = states.get(&job.id) {
        let expected = Cid::new_v1_sha256(0x55, &4i64.to_le_bytes());
        assert_eq!(receipt.result_cid, expected);
    } else {
        panic!("job not completed");
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn compiled_example_contract_file_runs() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let contract_path = manifest_dir.join("../../icn-ccl/tests/contracts/example.ccl");
    let source = std::fs::read_to_string(contract_path).expect("read example");
    let (wasm, _) = icn_ccl::compile_ccl_source_to_wasm(&source).unwrap();

    let ctx = RuntimeContext::new_with_stubs_and_mana("did:key:zExampleExec", 5);
    let block = DagBlock {
        cid: Cid::new_v1_sha256(0x71, &wasm),
        data: wasm.clone(),
        links: vec![],
        timestamp: 0,
        author_did: Did::new("key", "tester"),
        signature: None,
    };
    {
        let mut store = ctx.dag_store.lock().await;
        store.put(&block).unwrap();
    }
    let cid = block.cid.clone();
    let job = ActualMeshJob {
        id: Cid::new_v1_sha256(0x55, b"example"),
        manifest_cid: cid.clone(),
        spec: JobSpec::default(),
        creator_did: ctx.current_identity.clone(),
        cost_mana: 0,
        max_execution_wait_ms: None,
        signature: SignatureBytes(vec![]),
    };
    let signer = std::sync::Arc::new(StubSigner::new());
    let exec = WasmExecutor::new(ctx.clone(), signer);
    let receipt = exec.execute_job(&job).await.unwrap();
    let expected = Cid::new_v1_sha256(0x55, &11i64.to_le_bytes());
    assert_eq!(receipt.result_cid, expected);
}
