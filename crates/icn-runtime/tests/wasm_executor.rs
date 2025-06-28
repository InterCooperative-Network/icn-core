use icn_common::Cid;
use icn_common::DagBlock;
use icn_identity::{did_key_from_verifying_key, generate_ed25519_keypair, SignatureBytes};
use icn_mesh::{ActualMeshJob, JobSpec};
use icn_runtime::context::RuntimeContext;
use icn_runtime::executor::{JobExecutor, WasmExecutor};
use std::str::FromStr;

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
    let block = DagBlock {
        cid: Cid::new_v1_sha256(0x71, &wasm_bytes),
        data: wasm_bytes,
        links: vec![],
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

    let exec = WasmExecutor::new(ctx.clone(), node_did.clone(), sk);
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
    let block = DagBlock {
        cid: Cid::new_v1_sha256(0x71, &wasm),
        data: wasm.clone(),
        links: vec![],
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

    let exec = WasmExecutor::new(ctx.clone(), node_did.clone(), sk);
    let receipt = exec.execute_job(&job).await.unwrap();
    assert_eq!(receipt.executor_did, node_did);
    let expected_cid = Cid::new_v1_sha256(0x55, &7i64.to_le_bytes());
    assert_eq!(receipt.result_cid, expected_cid);
}
