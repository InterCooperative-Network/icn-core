// icn-ccl/tests/integration_tests.rs
#![allow(clippy::uninlined_format_args)]
use icn_ccl::{compile_ccl_source_to_wasm, CclError, ContractMetadata};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

fn create_dummy_ccl_file(dir: &Path, name: &str, content: &str) -> PathBuf {
    let file_path = dir.join(name);
    fs::write(&file_path, content).expect("Failed to write dummy CCL file");
    file_path
}

#[test]
fn test_compile_simple_policy_end_to_end() {
    let ccl_source = r#"
        // Simple policy for testing
        fn get_cost() -> Mana {
            return 10;
        }
    "#;

    let res = compile_ccl_source_to_wasm(ccl_source);
    assert!(matches!(res, Err(CclError::TypeError(_))));
}

#[test]
fn test_compile_ccl_file_cli_function() {
    let dir = tempdir().expect("Failed to create temp dir");
    let source_content = "fn main() -> Bool { return true; }"; // Simplified
    let source_path = create_dummy_ccl_file(dir.path(), "test_policy.ccl", source_content);

    let output_wasm_path = dir.path().join("test_policy.wasm");
    let output_meta_path = dir.path().join("test_policy.json");

    match icn_ccl::cli::compile_ccl_file(&source_path, &output_wasm_path, &output_meta_path) {
        Ok(metadata) => {
            assert!(output_wasm_path.exists(), ".wasm file should be created");
            assert!(
                output_meta_path.exists(),
                ".json metadata file should be created"
            );

            let wasm_bytes = fs::read(&output_wasm_path).unwrap();
            assert!(wasm_bytes.starts_with(b"\0asm"));

            let meta_content = fs::read_to_string(&output_meta_path).unwrap();
            let parsed_meta: ContractMetadata = serde_json::from_str(&meta_content).unwrap();
            assert_eq!(parsed_meta.cid, metadata.cid); // Check consistency

            let expected_hash = {
                let digest = Sha256::digest(source_content.as_bytes());
                format!("sha256:{:x}", digest)
            };
            assert_eq!(metadata.source_hash, expected_hash);
            assert_eq!(parsed_meta.source_hash, expected_hash);

            println!(
                "CLI compile_ccl_file test successful. Metadata: {:?}",
                metadata
            );
        }
        Err(e) => panic!("compile_ccl_file failed: {:?}", e),
    }
}

#[test]
fn test_compile_invalid_ccl() {
    let invalid_ccl_source = "fn broken syntax here {";
    let result = compile_ccl_source_to_wasm(invalid_ccl_source);
    assert!(
        matches!(result, Err(CclError::ParsingError(_))),
        "Expected parsing error for invalid syntax"
    );
}

#[test]
fn test_compile_with_rule_and_if() {
    let source = r#"
        fn main() -> Bool {
            let a = 5;
            if a > 1 { return true; } else { return false; }
        }

        rule basic when true then allow
    "#;

    let res = compile_ccl_source_to_wasm(source);
    assert!(res.is_err());
}

#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn test_wasm_executor_with_ccl() {
    use icn_common::Cid;
    use icn_identity::{did_key_from_verifying_key, generate_ed25519_keypair, SignatureBytes};
    use icn_mesh::{ActualMeshJob, JobSpec};
    use icn_runtime::context::RuntimeContext;
    use icn_runtime::executor::{JobExecutor, WasmExecutor};
    use std::str::FromStr;

    let source = "fn run() -> Integer { return 7 + 5; }";
    let (wasm, _) = compile_ccl_source_to_wasm(source).expect("compile ccl");

    use icn_common::DagBlock;

    let ctx = RuntimeContext::new_with_stubs_and_mana("did:key:zTestExec", 10);
    let block = DagBlock {
        cid: Cid::new_v1_dummy(0x71, 0x12, &wasm),
        data: wasm.clone(),
        links: vec![],
    };
    {
        let mut store = ctx.dag_store.lock().await;
        store.put(&block).unwrap();
    }
    let cid = block.cid.clone();

    let (sk, vk) = generate_ed25519_keypair();
    let node_did = did_key_from_verifying_key(&vk);
    let node_did = icn_common::Did::from_str(&node_did).unwrap();

    let job = ActualMeshJob {
        id: Cid::new_v1_dummy(0x55, 0x12, b"job"),
        manifest_cid: cid,
        spec: JobSpec::GenericPlaceholder,
        creator_did: node_did.clone(),
        cost_mana: 0,
        max_execution_wait_ms: None,
        signature: SignatureBytes(vec![]),
    };

    let exec = WasmExecutor::new(ctx.clone(), node_did.clone(), sk);
    let receipt = exec.execute_job(&job).await.unwrap();
    assert_eq!(receipt.executor_did, node_did);
}

// TODO: Add more integration tests:
// - Semantic errors (type mismatch, undefined variable)
// - Correct metadata generation (exports, inputs based on CCL code)
// - Test CLI check function
// - Test CLI format function (once implemented)
// - Test CLI explain function (once implemented)
