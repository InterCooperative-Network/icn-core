// icn-ccl/tests/integration_tests.rs
use icn_ccl::{compile_ccl_source_to_wasm, CclError, ContractMetadata};
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

    match compile_ccl_source_to_wasm(ccl_source) {
        Ok((wasm_bytecode, metadata)) => {
            assert!(
                !wasm_bytecode.is_empty(),
                "WASM bytecode should not be empty"
            );
            assert!(
                wasm_bytecode.starts_with(b"\0asm"),
                "Output should be valid WASM"
            );

            println!(
                "Generated WASM (first 16 bytes): {:?}",
                &wasm_bytecode[0..std::cmp::min(16, wasm_bytecode.len())]
            );
            println!("Generated Metadata: {:?}", metadata);

            assert!(
                metadata.cid.contains("bafy2bzace"),
                "Metadata CID placeholder missing"
            ); // Check placeholder
            assert_eq!(
                metadata.exports,
                vec!["mana_cost".to_string(), "can_bid".to_string()]
            ); // From stub
               // TODO: Update assertions once actual exports are derived from the AST
        }
        Err(e) => {
            panic!("CCL compilation failed: {:?}", e);
        }
    }
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
    assert!(res.is_ok(), "Compilation should succeed");
}

// TODO: Add more integration tests:
// - Semantic errors (type mismatch, undefined variable)
// - Correct metadata generation (exports, inputs based on CCL code)
// - Test CLI check function
// - Test CLI format function (once implemented)
// - Test CLI explain function (once implemented)
