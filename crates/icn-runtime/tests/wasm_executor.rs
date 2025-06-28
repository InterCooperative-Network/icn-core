use icn_common::Cid;
use icn_common::DagBlock;
use icn_identity::{did_key_from_verifying_key, generate_ed25519_keypair, SignatureBytes};
use icn_mesh::{ActualMeshJob, JobSpec};
use icn_runtime::context::RuntimeContext;
use icn_runtime::executor::{JobExecutor, WasmExecutor};
use std::str::FromStr;
use serde_json;

fn bytes_to_wat_string(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("\\{:02x}", b)).collect()
}

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
async fn wasm_executor_host_calls_with_json() {
    let ctx = RuntimeContext::new_with_stubs_and_mana("did:key:zExecJson", 80);
    let (sk, vk) = generate_ed25519_keypair();
    let node_did = icn_common::Did::from_str(&did_key_from_verifying_key(&vk)).unwrap();

    let submit_job = ActualMeshJob {
        id: Cid::new_v1_sha256(0x55, b"sjob"),
        manifest_cid: Cid::new_v1_sha256(0x71, b"man"),
        spec: JobSpec::default(),
        creator_did: node_did.clone(),
        cost_mana: 5,
        max_execution_wait_ms: Some(10),
        signature: SignatureBytes(vec![]),
    };
    let job_json = serde_json::to_string(&submit_job).unwrap();

    let mut receipt = icn_identity::ExecutionReceipt {
        job_id: Cid::new_v1_sha256(0x55, b"rid"),
        executor_did: node_did.clone(),
        result_cid: Cid::new_v1_sha256(0x55, b"res"),
        cpu_ms: 2,
        success: true,
        sig: SignatureBytes(Vec::new()),
    };
    let mut msg = Vec::new();
    msg.extend_from_slice(receipt.job_id.to_string().as_bytes());
    msg.extend_from_slice(node_did.to_string().as_bytes());
    msg.extend_from_slice(receipt.result_cid.to_string().as_bytes());
    msg.extend_from_slice(&receipt.cpu_ms.to_le_bytes());
    msg.push(receipt.success as u8);
    receipt.sig = SignatureBytes(ctx.signer.sign(&msg).unwrap());
    let receipt_json = serde_json::to_string(&receipt).unwrap();

    let job_hex = bytes_to_wat_string(job_json.as_bytes());
    let receipt_hex = bytes_to_wat_string(receipt_json.as_bytes());
    let job_len = job_json.len();
    let receipt_len = receipt_json.len();
    let receipt_offset = 4096;
    let module_wat = format!(
        "(module\n  (import \"icn\" \"host_submit_mesh_job\" (func $s (param i32 i32)))\n  (import \"icn\" \"host_anchor_receipt\" (func $a (param i32 i32)))\n  (memory (export \"memory\") 1)\n  (data (i32.const 0) \"{job_hex}\")\n  (data (i32.const {receipt_offset}) \"{receipt_hex}\")\n  (func (export \"run\") (result i64)\n    i32.const 0\n    i32.const {job_len}\n    call $s\n    i32.const {receipt_offset}\n    i32.const {receipt_len}\n    call $a\n    i64.const 0)\n)",
        job_hex = job_hex,
        receipt_hex = receipt_hex,
        job_len = job_len,
        receipt_len = receipt_len,
        receipt_offset = receipt_offset,
    );
    let wasm_bytes = wat::parse_str(&module_wat).unwrap();
    let block = DagBlock { cid: Cid::new_v1_sha256(0x71, &wasm_bytes), data: wasm_bytes, links: vec![] };
    {
        let mut store = ctx.dag_store.lock().await;
        store.put(&block).unwrap();
    }

    let job = ActualMeshJob {
        id: Cid::new_v1_sha256(0x55, b"exec"),
        manifest_cid: block.cid.clone(),
        spec: JobSpec::default(),
        creator_did: node_did.clone(),
        cost_mana: 0,
        max_execution_wait_ms: None,
        signature: SignatureBytes(vec![]),
    };

    let exec = WasmExecutor::new(ctx.clone(), node_did.clone(), sk);
    exec.execute_job(&job).await.unwrap();

    assert!(!ctx.pending_mesh_jobs.lock().await.is_empty());
    assert!(ctx.reputation_store.get_reputation(&node_did) > 0);
}
