// icn-ccl/tests/test_developer_tooling.rs
//! Integration tests for CCL developer tooling

use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn test_package_creation() {
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path();
    
    let result = icn_ccl::cli_commands::init_package(
        "test-package".to_string(),
        temp_path,
        "Test Author".to_string(),
        Some("test@example.com".to_string()),
    );
    
    assert!(result.is_ok());
    
    let package_dir = temp_path.join("test-package");
    assert!(package_dir.exists());
    assert!(package_dir.join("package.ccl").exists());
    assert!(package_dir.join("src").exists());
    assert!(package_dir.join("src/main.ccl").exists());
    assert!(package_dir.join("README.md").exists());
}

#[test]
fn test_ccl_compilation() {
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path();
    
    // Create a simple CCL file
    let ccl_content = "fn test() -> Integer { return 42; }";
    let ccl_file = temp_path.join("test.ccl");
    std::fs::write(&ccl_file, ccl_content).unwrap();
    
    // Compile it
    let result = icn_ccl::compile_ccl_file_to_wasm(&ccl_file);
    assert!(result.is_ok());
    
    let (wasm_bytes, metadata) = result.unwrap();
    assert!(!wasm_bytes.is_empty());
    assert!(!metadata.cid.is_empty());
}

#[test]
fn test_lsp_completion() {
    use icn_ccl::lsp::completion::provide_completions;
    use icn_ccl::lsp::server::DocumentState;
    use tower_lsp::lsp_types::Position;
    use url::Url;
    
    let doc_state = DocumentState {
        uri: Url::parse("file:///test.ccl").unwrap(),
        text: "contract Test { ".to_string(),
        version: 1,
        ast: None,
        semantic_errors: Vec::new(),
        parse_errors: Vec::new(),
    };
    
    let position = Position { line: 0, character: 16 };
    let completions = provide_completions(&doc_state, position);
    
    // Should have CCL keywords, types, and stdlib functions
    assert!(!completions.is_empty());
    
    // Check for some expected completions
    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();
    assert!(labels.contains(&"function".to_string()));
    assert!(labels.contains(&"u32".to_string()));
    assert!(labels.contains(&"log".to_string()));
}

#[test]
fn test_source_map_creation() {
    use icn_ccl::debugger::SourceMap;
    
    let source_map = SourceMap::new(
        "TestContract".to_string(),
        "/test/contract.ccl".to_string(),
        "/test/contract.wasm".to_string(),
    );
    
    assert_eq!(source_map.contract_name, "TestContract");
    assert_eq!(source_map.ccl_source_file, "/test/contract.ccl");
    assert_eq!(source_map.wasm_module, "/test/contract.wasm");
    
    // Test JSON serialization
    let json = source_map.to_json().unwrap();
    assert!(json.contains("TestContract"));
    
    // Test deserialization
    let deserialized = SourceMap::from_json(&json).unwrap();
    assert_eq!(deserialized.contract_name, source_map.contract_name);
}

#[test]
fn test_package_manifest() {
    use icn_ccl::package::{PackageManifest, VersionReq};
    use icn_ccl::package::manifest::Author;
    
    let author = Author {
        name: "Test Author".to_string(),
        email: Some("test@example.com".to_string()),
        did: None,
    };
    
    let mut manifest = PackageManifest::new(
        "test-package".to_string(),
        "0.1.0".to_string(),
        vec![author],
    );
    
    manifest.add_dependency("governance-lib".to_string(), VersionReq::new("^1.0.0"));
    
    assert!(manifest.has_dependency("governance-lib"));
    assert!(!manifest.has_dependency("nonexistent"));
    
    // Test TOML serialization
    let toml = manifest.to_toml().unwrap();
    assert!(toml.contains("test-package"));
    assert!(toml.contains("governance-lib"));
}