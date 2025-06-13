use icn_common::Cid;
use icn_identity::{did_key_from_verifying_key, generate_ed25519_keypair, SignatureBytes};
use icn_mesh::{ActualMeshJob, JobSpec};
use icn_runtime::context::RuntimeContext;
use icn_common::DagBlock;
use icn_runtime::executor::{JobExecutor, WasmExecutor};
use std::str::FromStr;
use std::thread;
use tokio::runtime::Runtime;

#[tokio::test(flavor = "multi_thread")]
#[ignore = "nested tokio runtime not yet supported"]
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
        cid: Cid::new_v1_dummy(0x71, 0x11, &wasm_bytes),
        data: wasm_bytes,
        links: vec![],
    };
    {
        let mut store = ctx.dag_store.lock().await;
        store.put(&block).unwrap();
    }
    let cid = block.cid.clone();

    let job = ActualMeshJob {
        id: Cid::new_v1_dummy(0x55, 0x11, b"job"),
        manifest_cid: cid,
        spec: JobSpec::GenericPlaceholder,
        creator_did: node_did.clone(),
        cost_mana: 0,
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
}
