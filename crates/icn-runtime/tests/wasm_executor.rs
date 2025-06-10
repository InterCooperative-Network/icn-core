use icn_common::Cid;
use icn_identity::{did_key_from_verifying_key, generate_ed25519_keypair, SignatureBytes};
use icn_mesh::{ActualMeshJob, JobSpec};
use icn_runtime::context::RuntimeContext;
use icn_runtime::executor::{JobExecutor, WasmExecutor};
use std::str::FromStr;

#[tokio::test(flavor = "multi_thread")]
#[ignore]
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
    let cid = ctx.dag_store.put(&wasm_bytes).await.unwrap();

    let job = ActualMeshJob {
        id: Cid::new_v1_dummy(0x55, 0x11, b"job"),
        manifest_cid: cid,
        spec: JobSpec::GenericPlaceholder,
        creator_did: node_did.clone(),
        cost_mana: 0,
        signature: SignatureBytes(vec![]),
    };

    let exec = WasmExecutor::new(ctx.clone(), node_did.clone(), sk);
    let receipt = exec.execute_job(&job).await.unwrap();
    assert_eq!(receipt.executor_did, node_did);
}
