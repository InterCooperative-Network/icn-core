use icn_common::{Cid, DagBlock, Did};
use icn_identity::SignatureBytes;
use icn_mesh::{ActualMeshJob, JobId, JobKind, JobSpec};
use icn_runtime::context::RuntimeContext;
use icn_runtime::executor::{JobExecutor, WasmExecutor, WasmExecutorConfig, WasmSecurityLimits};
use std::sync::Arc;

#[tokio::test]
async fn validator_enforces_memory_pages() {
    let ctx = RuntimeContext::new_with_stubs_and_mana("did:key:zMemPages", 1).unwrap();
    let wasm = "(module (memory 2) (func (export \"run\") (result i64) i64.const 1))";
    let wasm_bytes = wat::parse_str(wasm).unwrap();
    let block = DagBlock {
        cid: Cid::new_v1_sha256(0x71, &wasm_bytes),
        data: wasm_bytes,
        links: vec![],
        timestamp: 0,
        author_did: Did::new("key", "tester"),
        signature: None,
        scope: None,
    };
    {
        let mut store = ctx.dag_store.lock().await;
        store.put(&block).await.unwrap();
    }
    let job = ActualMeshJob {
        id: JobId(Cid::new_v1_sha256(0x55, b"mem_pages")),
        manifest_cid: block.cid.clone(),
        spec: JobSpec {
            kind: JobKind::CclWasm,
            ..Default::default()
        },
        creator_did: ctx.current_identity.clone(),
        cost_mana: 0,
        max_execution_wait_ms: None,
        signature: SignatureBytes(vec![]),
    };
    let signer = Arc::new(icn_runtime::context::StubSigner::new());
    let mut limits = WasmSecurityLimits::default();
    limits.max_memory_pages = 1;
    let config = WasmExecutorConfig {
        max_memory: 10 * 1024 * 1024,
        fuel: 1_000_000,
        security_limits: limits,
    };
    let exec = WasmExecutor::new(ctx.clone(), signer, config);
    assert!(exec.execute_job(&job).await.is_err());
}

#[tokio::test]
async fn validator_enforces_function_limit() {
    let ctx = RuntimeContext::new_with_stubs_and_mana("did:key:zFuncLimit", 1).unwrap();
    let wasm = r#"(module
        (func $f1 (result i64) i64.const 1)
        (func $f2 (result i64) i64.const 2)
        (func (export \"run\") (result i64) call $f1)
    )"#;
    let wasm_bytes = wat::parse_str(wasm).unwrap();
    let block = DagBlock {
        cid: Cid::new_v1_sha256(0x71, &wasm_bytes),
        data: wasm_bytes,
        links: vec![],
        timestamp: 0,
        author_did: Did::new("key", "tester"),
        signature: None,
        scope: None,
    };
    {
        let mut store = ctx.dag_store.lock().await;
        store.put(&block).await.unwrap();
    }
    let job = ActualMeshJob {
        id: JobId(Cid::new_v1_sha256(0x55, b"func_limit")),
        manifest_cid: block.cid.clone(),
        spec: JobSpec {
            kind: JobKind::CclWasm,
            ..Default::default()
        },
        creator_did: ctx.current_identity.clone(),
        cost_mana: 0,
        max_execution_wait_ms: None,
        signature: SignatureBytes(vec![]),
    };
    let signer = Arc::new(icn_runtime::context::StubSigner::new());
    let mut limits = WasmSecurityLimits::default();
    limits.max_functions = 1;
    let config = WasmExecutorConfig {
        max_memory: 10 * 1024 * 1024,
        fuel: 1_000_000,
        security_limits: limits,
    };
    let exec = WasmExecutor::new(ctx.clone(), signer, config);
    assert!(exec.execute_job(&job).await.is_err());
}

#[tokio::test]
async fn resource_limiter_blocks_growth() {
    let ctx = RuntimeContext::new_with_stubs_and_mana("did:key:zGrow", 1).unwrap();
    let wasm = r#"(module
        (memory 1)
        (func (export \"run\") (result i64)
            (if (memory.grow (i32.const 2)) (then i64.const 1) else i64.const 0))
    )"#;
    let wasm_bytes = wat::parse_str(wasm).unwrap();
    let block = DagBlock {
        cid: Cid::new_v1_sha256(0x71, &wasm_bytes),
        data: wasm_bytes,
        links: vec![],
        timestamp: 0,
        author_did: Did::new("key", "tester"),
        signature: None,
        scope: None,
    };
    {
        let mut store = ctx.dag_store.lock().await;
        store.put(&block).await.unwrap();
    }
    let job = ActualMeshJob {
        id: JobId(Cid::new_v1_sha256(0x55, b"grow")),
        manifest_cid: block.cid.clone(),
        spec: JobSpec {
            kind: JobKind::CclWasm,
            ..Default::default()
        },
        creator_did: ctx.current_identity.clone(),
        cost_mana: 0,
        max_execution_wait_ms: None,
        signature: SignatureBytes(vec![]),
    };
    let signer = Arc::new(icn_runtime::context::StubSigner::new());
    let config = WasmExecutorConfig {
        max_memory: 64 * 1024,
        fuel: 1_000_000,
        security_limits: WasmSecurityLimits::default(),
    };
    let exec = WasmExecutor::new(ctx.clone(), signer, config);
    assert!(exec.execute_job(&job).await.is_err());
}
