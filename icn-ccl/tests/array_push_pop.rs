use icn_ccl::compile_ccl_source_to_wasm;
use icn_common::{Cid, DagBlock};
use icn_dag::InMemoryDagStore;
use icn_identity::{did_key_from_verifying_key, generate_ed25519_keypair, SignatureBytes};
use icn_mesh::{ActualMeshJob, JobId, JobSpec};
use icn_runtime::context::{RuntimeContext, StubSigner};
use icn_runtime::executor::{JobExecutor, WasmExecutor, WasmExecutorConfig};
use std::str::FromStr;
use std::sync::Arc;
use std::thread;
use tokio::runtime::Runtime;
use tokio::sync::Mutex as TokioMutex;

fn ctx_with_temp_store(did: &str, mana: u64) -> Arc<RuntimeContext> {
    let ctx = RuntimeContext::new_with_stubs_and_mana(did, mana).unwrap();
    ctx
}

#[tokio::test(flavor = "multi_thread")]
async fn array_reallocation_integers() {
    let source = r#"
        fn run() -> Integer {
            let nums = [1,2];
            array_push(nums, 3);
            array_push(nums, 4);
            let l = array_len(nums);
            let v = array_pop(nums);
            return l + v;
        }
    "#;
    let (wasm, _) = compile_ccl_source_to_wasm(source).expect("compile");

    let ctx = ctx_with_temp_store("did:key:zArrInt", 10);
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
        let mut store = ctx.dag_store.store.lock().await;
        store.put(&block).await.unwrap();
    }
    let cid = block.cid.clone();

    let (sk, vk) = generate_ed25519_keypair();
    let node_did = icn_common::Did::from_str(&did_key_from_verifying_key(&vk)).unwrap();

    let job = ActualMeshJob {
        id: JobId(Cid::new_v1_sha256(0x55, b"arr_int")),
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
    let expected_cid = Cid::new_v1_sha256(0x55, &8i64.to_le_bytes());
    assert_eq!(receipt.result_cid, expected_cid);
}

#[tokio::test(flavor = "multi_thread")]
async fn array_push_bool_and_string() {
    let source = r#"
        fn run() -> Integer {
            let flags = [true];
            array_push(flags, false);
            let names = ["a"];
            array_push(names, "b");
            if array_pop(flags) { 1 } else { array_len(names) }
        }
    "#;
    let (wasm, _) = compile_ccl_source_to_wasm(source).expect("compile");

    let ctx = ctx_with_temp_store("did:key:zArrMix", 10);
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
        let mut store = ctx.dag_store.store.lock().await;
        store.put(&block).await.unwrap();
    }
    let cid = block.cid.clone();

    let (sk, vk) = generate_ed25519_keypair();
    let node_did = icn_common::Did::from_str(&did_key_from_verifying_key(&vk)).unwrap();

    let job = ActualMeshJob {
        id: JobId(Cid::new_v1_sha256(0x55, b"arr_mix")),
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
    // popped flag is false so branch returns array_len(names) == 2
    let expected_cid = Cid::new_v1_sha256(0x55, &2i64.to_le_bytes());
    assert_eq!(receipt.result_cid, expected_cid);
}
