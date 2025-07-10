use proptest::prelude::*;
use icn_ccl::compile_ccl_source_to_wasm;
use icn_common::{compute_merkle_cid, Cid, DagBlock, Did};
use icn_identity::SignatureBytes;
use icn_mesh::{ActualMeshJob, JobId, JobSpec};
use icn_runtime::context::{RuntimeContext, StubSigner};
use icn_runtime::executor::{JobExecutor, WasmExecutor, WasmExecutorConfig};
use std::sync::Arc;

proptest! {
    #[test]
    fn compiled_addition_deterministic(a in 0i64..1000, b in 0i64..1000) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let ctx = RuntimeContext::new_with_stubs("did:key:zProp").unwrap();
            ctx.mana_ledger.set_balance(&ctx.current_identity, 5).unwrap();
            let source = format!("fn run() -> Integer {{ return {} + {}; }}", a, b);
            let (wasm, _) = compile_ccl_source_to_wasm(&source).unwrap();

            let ts = 0u64;
            let author = Did::new("key", "tester");
            let sig_opt = None;
            let cid = compute_merkle_cid(0x71, &wasm, &[], ts, &author, &sig_opt, &None);
            let block = DagBlock { cid: cid.clone(), data: wasm.clone(), links: vec![], timestamp: ts, author_did: author, signature: sig_opt, scope: None };
            {
                let mut store = ctx.dag_store.lock().await;
                store.put(&block).await.unwrap();
            }

            let job = ActualMeshJob {
                id: JobId(Cid::new_v1_sha256(0x55, b"prop")),
                manifest_cid: block.cid.clone(),
                spec: JobSpec::default(),
                creator_did: ctx.current_identity.clone(),
                cost_mana: 0,
                max_execution_wait_ms: None,
                signature: SignatureBytes(vec![]),
            };
            let signer = Arc::new(StubSigner::new());
            let exec = WasmExecutor::new(ctx.clone(), signer, WasmExecutorConfig::default());
            let receipt1 = exec.execute_job(&job).await.unwrap();
            let receipt2 = exec.execute_job(&job).await.unwrap();
            let expected = Cid::new_v1_sha256(0x55, &(a + b).to_le_bytes());
            assert_eq!(receipt1.result_cid, expected);
            assert_eq!(receipt2.result_cid, expected);
        });
    }
}

