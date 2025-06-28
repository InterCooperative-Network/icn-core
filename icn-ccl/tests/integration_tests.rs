// icn-ccl/tests/integration_tests.rs
#![allow(clippy::uninlined_format_args)]
use icn_ccl::{
    ast::{ActionNode, AstNode, ExpressionNode, PolicyStatementNode},
    compile_ccl_source_to_wasm,
    parser::parse_ccl_source,
    CclError, ContractMetadata,
};
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

    let res = compile_ccl_source_to_wasm(ccl_source).expect("compile success");
    let (wasm, meta) = res;
    assert!(wasm.starts_with(b"\0asm"));
    assert!(meta.exports.contains(&"get_cost".to_string()));
}

#[test]
fn test_compile_valid_function() {
    let src = "fn add() -> Integer { return 2 + 3; }";
    let result = compile_ccl_source_to_wasm(src).expect("compile success");
    let (wasm, meta) = result;
    assert!(wasm.starts_with(b"\0asm"));
    assert!(meta.exports.contains(&"add".to_string()));
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
fn test_compile_semantic_error_type_mismatch() {
    let src = "fn wrong() -> Bool { return 1; }";
    let res = compile_ccl_source_to_wasm(src);
    assert!(matches!(res, Err(CclError::TypeError(_))));
}

#[test]
fn test_compile_semantic_error_undefined_var() {
    let src = "fn bad() -> Integer { return x; }";
    let res = compile_ccl_source_to_wasm(src);
    assert!(matches!(res, Err(CclError::SemanticError(_))));
}

#[test]
fn test_compile_semantic_error_binary_op() {
    let src = "fn bad() -> Integer { let a = 1 + \"s\"; return 0; }";
    let res = compile_ccl_source_to_wasm(src);
    assert!(matches!(res, Err(CclError::TypeError(_))));
}

#[test]
fn test_compile_metadata_exports() {
    let src = "fn run() -> Integer { return 42; } fn cost() -> Mana { return 1; }";
    let (_wasm, meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(meta.exports.contains(&"run".to_string()));
    assert!(meta.exports.contains(&"cost".to_string()));
    assert!(meta.inputs.is_empty());
    assert!(!meta.cid.is_empty());
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

#[test]
fn test_parse_rule_definition() {
    let source = "rule allow_all when true then allow";
    let ast = parse_ccl_source(source).expect("parse rule");
    let expected = AstNode::Policy(vec![PolicyStatementNode::RuleDef(
        AstNode::RuleDefinition {
            name: "allow_all".to_string(),
            condition: ExpressionNode::BooleanLiteral(true),
            action: ActionNode::Allow,
        },
    )]);
    assert_eq!(ast, expected);
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
        cid: Cid::new_v1_sha256(0x71, &wasm),
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

#[test]
fn test_cli_format_ccl_file() {
    let dir = tempdir().expect("Failed to create temp dir");
    let src = "fn main()->Bool{return true;}";
    let path = create_dummy_ccl_file(dir.path(), "fmt.ccl", src);

    let formatted = icn_ccl::cli::format_ccl_file(&path, false).expect("format");
    assert!(formatted.contains("fn main() -> Bool"));
    assert!(formatted.contains("return true;"));

    let inplace = icn_ccl::cli::format_ccl_file(&path, true).expect("format inplace");
    let updated = fs::read_to_string(&path).unwrap();
    assert_eq!(inplace, updated);
}

#[test]
fn test_cli_explain_ccl_policy() {
    let dir = tempdir().expect("Failed to create temp dir");
    let src = "fn main() -> Bool { return true; } rule r when true then allow";
    let path = create_dummy_ccl_file(dir.path(), "explain.ccl", src);

    let explanation = icn_ccl::cli::explain_ccl_policy(&path, None).expect("explain");
    assert!(explanation.contains("Function `main`"));
    assert!(explanation.contains("Rule `r`"));

    let only =
        icn_ccl::cli::explain_ccl_policy(&path, Some("main".to_string())).expect("explain target");
    assert!(only.contains("Function `main`"));
    assert!(!only.contains("Rule `r`"));
}

#[test]
fn test_cli_check_ccl_file_ok() {
    let dir = tempdir().expect("Failed to create temp dir");
    let src = "fn main() -> Bool { return true; }";
    let path = create_dummy_ccl_file(dir.path(), "check_ok.ccl", src);

    let res = icn_ccl::cli::check_ccl_file(&path);
    assert!(res.is_ok());
}

#[test]
fn test_cli_check_ccl_file_type_error() {
    let dir = tempdir().expect("Failed to create temp dir");
    let src = "fn wrong() -> Bool { return 1; }";
    let path = create_dummy_ccl_file(dir.path(), "check_type_error.ccl", src);

    let res = icn_ccl::cli::check_ccl_file(&path);
    assert!(matches!(res, Err(CclError::TypeError(_))));
}

#[test]
fn test_cli_check_ccl_file_undefined_var() {
    let dir = tempdir().expect("Failed to create temp dir");
    let src = "fn bad() -> Integer { return x; }";
    let path = create_dummy_ccl_file(dir.path(), "check_undef.ccl", src);

    let res = icn_ccl::cli::check_ccl_file(&path);
    assert!(matches!(res, Err(CclError::SemanticError(_))));
}

#[test]
fn test_compile_semantic_error_missing_return() {
    let src = "fn bad() -> Integer { let a = 1; }";
    let res = compile_ccl_source_to_wasm(src);
    assert!(matches!(res, Err(CclError::SemanticError(_))));
}

#[test]
fn test_compile_rule_condition_must_be_bool() {
    let src = "rule wrong when 1 then allow";
    let res = compile_ccl_source_to_wasm(src);
    assert!(matches!(res, Err(CclError::TypeError(_))));
}

#[test]
fn test_parse_rule_with_charge() {
    let source = "rule pay when true then charge 5";
    let ast = parse_ccl_source(source).expect("parse rule");
    let expected = AstNode::Policy(vec![PolicyStatementNode::RuleDef(
        AstNode::RuleDefinition {
            name: "pay".to_string(),
            condition: ExpressionNode::BooleanLiteral(true),
            action: ActionNode::Charge(ExpressionNode::IntegerLiteral(5)),
        },
    )]);
    assert_eq!(ast, expected);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_wasm_executor_runs_addition() {
    use icn_common::Cid;
    use icn_identity::{did_key_from_verifying_key, generate_ed25519_keypair, SignatureBytes};
    use icn_mesh::{ActualMeshJob, JobSpec};
    use icn_runtime::context::RuntimeContext;
    use icn_runtime::executor::{JobExecutor, WasmExecutor};
    use std::str::FromStr;

    let source = "fn run() -> Integer { return 40 + 2; }";
    let (wasm, _) = compile_ccl_source_to_wasm(source).expect("compile ccl");

    use icn_common::DagBlock;

    let ctx = RuntimeContext::new_with_stubs_and_mana("did:key:zAddExecInt", 10);
    let cid_calc = icn_common::compute_merkle_cid(0x71, &wasm, &[]);
    let block = DagBlock {
        cid: cid_calc.clone(),
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
        id: Cid::new_v1_sha256(0x55, b"jobadd"),
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
