use icn_common::Cid;
use icn_identity::{
    did_key_from_verifying_key, generate_ed25519_keypair, ExecutionReceipt, SignatureBytes,
};
use icn_mesh::{ActualMeshJob, JobId, JobSpec};
use icn_runtime::context::RuntimeContext;
use icn_runtime::{
    wasm_host_account_get_mana, wasm_host_account_spend_mana, wasm_host_anchor_receipt,
    wasm_host_get_pending_mesh_jobs, wasm_host_submit_mesh_job,
};
use std::str::FromStr;
use wasmtime::{Engine, Linker, Module, Store};

#[tokio::test]
async fn wasm_host_api_functions() {
    let ctx = RuntimeContext::new_with_stubs_and_mana("did:key:zHostTest", 50).unwrap();
    let engine = Engine::default();
    let module_wat = r#"(module
        (import "icn" "wasm_host_submit_mesh_job" (func $submit (param i32 i32 i32 i32) (result i32)))
        (import "icn" "wasm_host_get_pending_mesh_jobs" (func $get_jobs (param i32 i32) (result i32)))
        (import "icn" "wasm_host_account_get_mana" (func $get_mana (param i32 i32) (result i64)))
        (import "icn" "wasm_host_account_spend_mana" (func $spend (param i32 i32 i64)))
        (import "icn" "wasm_host_anchor_receipt" (func $anchor (param i32 i32 i32 i32) (result i32)))
        (import "icn" "wasm_host_get_reputation" (func $get_rep (param i32 i32) (result i64)))
        (memory (export "memory") 1)
        (func (export "submit") (param i32 i32 i32 i32) (result i32) (local.get 0) (local.get 1) (local.get 2) (local.get 3) call $submit)
        (func (export "get_jobs") (param i32 i32) (result i32) (local.get 0) (local.get 1) call $get_jobs)
        (func (export "get_mana") (param i32 i32) (result i64) (local.get 0) (local.get 1) call $get_mana)
        (func (export "spend") (param i32 i32 i64) (local.get 0) (local.get 1) (local.get 2) call $spend)
        (func (export "anchor") (param i32 i32 i32 i32) (result i32) (local.get 0) (local.get 1) (local.get 2) (local.get 3) call $anchor)
        (func (export "get_rep") (param i32 i32) (result i64) (local.get 0) (local.get 1) call $get_rep)
    )"#;
    let module_bytes = wat::parse_str(module_wat).unwrap();
    let module = Module::new(&engine, module_bytes).unwrap();

    let mut linker = Linker::new(&engine);

    linker
        .func_wrap(
            "icn",
            "wasm_host_submit_mesh_job",
            wasm_host_submit_mesh_job,
        )
        .unwrap();
    linker
        .func_wrap(
            "icn",
            "wasm_host_get_pending_mesh_jobs",
            wasm_host_get_pending_mesh_jobs,
        )
        .unwrap();
    linker
        .func_wrap(
            "icn",
            "wasm_host_account_get_mana",
            wasm_host_account_get_mana,
        )
        .unwrap();
    linker
        .func_wrap("icn", "wasm_host_get_reputation", wasm_host_get_reputation)
        .unwrap();
    linker
        .func_wrap(
            "icn",
            "wasm_host_account_spend_mana",
            wasm_host_account_spend_mana,
        )
        .unwrap();
    linker
        .func_wrap("icn", "wasm_host_anchor_receipt", wasm_host_anchor_receipt)
        .unwrap();

    let mut store = Store::new(&engine, ctx.clone());
    let instance = linker.instantiate(&mut store, &module).unwrap();
    let memory = instance.get_memory(&mut store, "memory").unwrap();

    // Prepare job JSON in memory
    let job = ActualMeshJob {
        id: JobId(Cid::new_v1_sha256(0x55, b"job")),
        manifest_cid: Cid::new_v1_sha256(0x71, b"wasm"),
        spec: JobSpec::default(),
        creator_did: ctx.current_identity.clone(),
        cost_mana: 10,
        max_execution_wait_ms: None,
        signature: SignatureBytes(vec![]),
    };
    let job_json = serde_json::to_string(&job).unwrap();
    memory.write(&mut store, 0, job_json.as_bytes()).unwrap();

    // Submit job via wasm
    let submit = instance
        .get_typed_func::<(i32, i32, i32, i32), i32>(&mut store, "submit")
        .unwrap();
    let job_id_ptr = 500i32;
    let written_id = submit
        .call(&mut store, (0, job_json.len() as i32, job_id_ptr, 100))
        .unwrap();
    assert!(written_id > 0);
    let mut id_buf = vec![0u8; written_id as usize];
    memory
        .read(&mut store, job_id_ptr as usize, &mut id_buf)
        .unwrap();
    let _job_id_str = String::from_utf8(id_buf).unwrap();

    // DID string for mana calls
    let did_str = ctx.current_identity.to_string();
    memory.write(&mut store, 1000, did_str.as_bytes()).unwrap();
    let get_mana = instance
        .get_typed_func::<(i32, i32), i64>(&mut store, "get_mana")
        .unwrap();
    let mana_before = get_mana
        .call(&mut store, (1000, did_str.len() as i32))
        .unwrap();
    assert_eq!(mana_before, 40); // mana deducted by submit

    let spend = instance
        .get_typed_func::<(i32, i32, i64), ()>(&mut store, "spend")
        .unwrap();
    spend
        .call(&mut store, (1000, did_str.len() as i32, 5))
        .unwrap();
    let mana_after = get_mana
        .call(&mut store, (1000, did_str.len() as i32))
        .unwrap();
    assert_eq!(mana_after, 35);

    // Build and anchor receipt
    let pending = ctx.pending_mesh_jobs.lock().await;
    let job_id = pending[0].id.clone();
    drop(pending);

    let (_sk, vk) = generate_ed25519_keypair();
    let node_did = icn_common::Did::from_str(&did_key_from_verifying_key(&vk)).unwrap();
    let mut receipt = ExecutionReceipt {
        job_id: job_id.into(),
        executor_did: node_did.clone(),
        result_cid: Cid::new_v1_sha256(0x55, b"res"),
        cpu_ms: 1,
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
    memory
        .write(&mut store, 2000, receipt_json.as_bytes())
        .unwrap();
    let anchor = instance
        .get_typed_func::<(i32, i32, i32, i32), i32>(&mut store, "anchor")
        .unwrap();
    let anchor_ptr = 4000i32;
    let cid_len = anchor
        .call(
            &mut store,
            (2000, receipt_json.len() as i32, anchor_ptr, 100),
        )
        .unwrap();
    assert!(cid_len > 0);
    let mut cid_buf = vec![0u8; cid_len as usize];
    memory
        .read(&mut store, anchor_ptr as usize, &mut cid_buf)
        .unwrap();
    let _anchor_cid = String::from_utf8(cid_buf).unwrap();

    // reputation updated
    assert!(ctx.reputation_store.get_reputation(&node_did) > 0);
    // verify via wasm import
    memory
        .write(&mut store, 5000, node_did.to_string().as_bytes())
        .unwrap();
    let get_rep = instance
        .get_typed_func::<(i32, i32), i64>(&mut store, "get_rep")
        .unwrap();
    let rep_val = get_rep
        .call(&mut store, (5000, node_did.to_string().len() as i32))
        .unwrap();
    assert!(rep_val > 0);

    // get pending jobs via wasm
    let get_jobs = instance
        .get_typed_func::<(i32, i32), i32>(&mut store, "get_jobs")
        .unwrap();
    let buf_ptr = 3000i32;
    let written = get_jobs.call(&mut store, (buf_ptr, 1024)).unwrap();
    let mut buf = vec![0u8; written as usize];
    memory.read(&mut store, buf_ptr as usize, &mut buf).unwrap();
    let jobs: Vec<ActualMeshJob> = serde_json::from_slice(&buf).unwrap();
    assert!(!jobs.is_empty());
}

#[tokio::test]
async fn wasm_host_api_error_paths() {
    let ctx = RuntimeContext::new_with_stubs_and_mana("did:key:zHostErr", 10).unwrap();
    let engine = Engine::default();
    let module_wat = r#"(module
        (import "icn" "wasm_host_submit_mesh_job" (func $submit (param i32 i32 i32 i32) (result i32)))
        (import "icn" "wasm_host_get_pending_mesh_jobs" (func $get (param i32 i32) (result i32)))
        (memory (export "memory") 1)
        (func (export "submit_invalid") (result i32)
            i32.const 0
            i32.const 0
            i32.const 100
            i32.const 10
            call $submit)
        (func (export "get_small") (result i32)
            i32.const 200
            i32.const 2
            call $get)
    )"#;
    let module_bytes = wat::parse_str(module_wat).unwrap();
    let module = Module::new(&engine, module_bytes).unwrap();

    let mut linker = Linker::new(&engine);
    linker
        .func_wrap(
            "icn",
            "wasm_host_submit_mesh_job",
            wasm_host_submit_mesh_job,
        )
        .unwrap();
    linker
        .func_wrap(
            "icn",
            "wasm_host_get_pending_mesh_jobs",
            wasm_host_get_pending_mesh_jobs,
        )
        .unwrap();

    let mut store = Store::new(&engine, ctx.clone());
    let instance = linker.instantiate(&mut store, &module).unwrap();
    let memory = instance.get_memory(&mut store, "memory").unwrap();

    // Call invalid submit (empty JSON)
    let submit_invalid = instance
        .get_typed_func::<(), i32>(&mut store, "submit_invalid")
        .unwrap();
    let written = submit_invalid.call(&mut store, ()).unwrap();
    assert_eq!(written, 0);

    // Request pending jobs with tiny buffer (should just return length of '[]')
    let get_small = instance
        .get_typed_func::<(), i32>(&mut store, "get_small")
        .unwrap();
    let len = get_small.call(&mut store, ()).unwrap();
    assert_eq!(len, 2);
    let mut buf = vec![0u8; len as usize];
    memory.read(&mut store, 200usize, &mut buf).unwrap();
    assert_eq!(&buf, b"[]");
}
