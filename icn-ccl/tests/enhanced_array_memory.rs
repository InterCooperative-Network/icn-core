use icn_ccl::compile_ccl_source_to_wasm;
use icn_common::{Cid, DagBlock};
use icn_dag::InMemoryDagStore;
use icn_identity::{did_key_from_verifying_key, generate_ed25519_keypair, SignatureBytes};
use icn_mesh::{ActualMeshJob, JobId, JobSpec};
use icn_runtime::context::{RuntimeContext, StubMeshNetworkService, StubSigner};
use icn_runtime::executor::{WasmExecutor, WasmExecutorConfig};
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
        Arc::new(StubSigner::new()),
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
async fn array_element_assignment_integers() {
    let source = r#"
        fn run() -> Integer {
            let mut nums = [10, 20, 30];
            nums[1] = 99;
            return nums[1];
        }
    "#;
    let (wasm, _) = compile_ccl_source_to_wasm(source).expect("compile");

    let ctx = ctx_with_temp_store("did:key:zArrAssign", 10);
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
        id: JobId(Cid::new_v1_sha256(0x55, b"arr_assign")),
        manifest_cid: cid,
        spec: JobSpec::default(),
        creator_did: node_did.clone(),
        cost_mana: 0,
        max_execution_wait_ms: None,
        signature: SignatureBytes(vec![]),
    };

    let signer = Arc::new(StubSigner::new_with_keys(sk, vk));
    let exec = WasmExecutor::new(ctx.clone(), signer, WasmExecutorConfig::default());
    let job_clone = job.clone();
    let handle = thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        rt.block_on(async { exec.execute_job(&job_clone).await })
    });
    let receipt = handle.join().unwrap().unwrap();
    assert_eq!(receipt.executor_did, node_did);
    // Should return 99 (the assigned value)
    let expected_cid = Cid::new_v1_sha256(0x55, &99i64.to_le_bytes());
    assert_eq!(receipt.result_cid, expected_cid);
}

#[tokio::test(flavor = "multi_thread")]
async fn array_bounds_checking() {
    let source = r#"
        fn run() -> Integer {
            let mut nums = [1, 2, 3];
            let result = if nums[5] == 0 { 1 } else { 0 };
            return result;
        }
    "#;

    // This should compile but fail at runtime with bounds check
    let compile_result = compile_ccl_source_to_wasm(source);
    assert!(compile_result.is_ok(), "Should compile successfully");

    // TODO: Add runtime bounds checking test when runtime error handling is implemented
}

#[tokio::test(flavor = "multi_thread")]
async fn array_dynamic_growth_stress_test() {
    let source = r#"
        fn run() -> Integer {
            let mut nums = [1];
            let mut i = 0;
            while i < 10 {
                array_push(nums, i * 2);
                i = i + 1;
            }
            return array_len(nums);
        }
    "#;
    let (wasm, _) = compile_ccl_source_to_wasm(source).expect("compile");

    let ctx = ctx_with_temp_store("did:key:zArrGrowth", 10);
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
        id: JobId(Cid::new_v1_sha256(0x55, b"arr_growth")),
        manifest_cid: cid,
        spec: JobSpec::default(),
        creator_did: node_did.clone(),
        cost_mana: 0,
        max_execution_wait_ms: None,
        signature: SignatureBytes(vec![]),
    };

    let signer = Arc::new(StubSigner::new_with_keys(sk, vk));
    let exec = WasmExecutor::new(ctx.clone(), signer, WasmExecutorConfig::default());
    let job_clone = job.clone();
    let handle = thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        rt.block_on(async { exec.execute_job(&job_clone).await })
    });
    let receipt = handle.join().unwrap().unwrap();
    assert_eq!(receipt.executor_did, node_did);
    // Should return 11 (1 initial + 10 pushed)
    let expected_cid = Cid::new_v1_sha256(0x55, &11i64.to_le_bytes());
    assert_eq!(receipt.result_cid, expected_cid);
}
