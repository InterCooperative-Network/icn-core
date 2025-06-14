// icn-ccl/src/cli.rs
use crate::error::CclError;
use crate::metadata::ContractMetadata;
use crate::optimizer::Optimizer;
use crate::parser::parse_ccl_source;
use crate::semantic_analyzer::SemanticAnalyzer;
use crate::wasm_backend::WasmBackend;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
// use icn_common::Cid; // If calculating actual CIDs

// This function would be called by `icn-cli ccl compile ...`
pub fn compile_ccl_file(
    source_path: &PathBuf,
    output_wasm_path: &PathBuf,
    output_meta_path: &PathBuf,
) -> Result<ContractMetadata, CclError> {
    println!(
        "[CCL CLI Lib] Compiling {} to {} (meta: {})",
        source_path.display(),
        output_wasm_path.display(),
        output_meta_path.display()
    );

    let source_code = fs::read_to_string(source_path).map_err(|e| {
        CclError::IoError(format!(
            "Failed to read source file {}: {}",
            source_path.display(),
            e
        ))
    })?;

    let ast = parse_ccl_source(&source_code)?;

    let mut semantic_analyzer = SemanticAnalyzer::new();
    semantic_analyzer.analyze(&ast)?;

    let optimizer = Optimizer::new();
    let optimized_ast = optimizer.optimize(ast)?; // AST might change to IR here

    let wasm_backend = WasmBackend::new();
    let (wasm_bytecode, mut metadata) = wasm_backend.compile_to_wasm(&optimized_ast)?;

    // Calculate CID of wasm_bytecode (placeholder)
    // In reality, use icn_dag or similar to produce a real CID
    let wasm_cid_placeholder = format!(
        "bafy2bzace{}",
        hex::encode(&wasm_bytecode[0..min(10, wasm_bytecode.len())])
    ); // Very rough placeholder
    metadata.cid = wasm_cid_placeholder;

    // Calculate SHA-256 hash of the source code
    let hash = Sha256::digest(source_code.as_bytes());
    metadata.source_hash = format!("sha256:{:x}", hash);

    fs::write(output_wasm_path, &wasm_bytecode).map_err(|e| {
        CclError::IoError(format!(
            "Failed to write WASM file {}: {}",
            output_wasm_path.display(),
            e
        ))
    })?;

    let metadata_json = serde_json::to_string_pretty(&metadata).map_err(|e| {
        CclError::InternalCompilerError(format!("Failed to serialize metadata: {}", e))
    })?;
    fs::write(output_meta_path, metadata_json).map_err(|e| {
        CclError::IoError(format!(
            "Failed to write metadata file {}: {}",
            output_meta_path.display(),
            e
        ))
    })?;

    println!(
        "[CCL CLI Lib] Compilation successful. WASM: {}, Meta: {}",
        output_wasm_path.display(),
        output_meta_path.display()
    );
    Ok(metadata)
}

// This function would be called by `icn-cli ccl lint ...` or `icn-cli ccl check ...`
pub fn check_ccl_file(source_path: &PathBuf) -> Result<(), CclError> {
    println!("[CCL CLI Lib] Checking/Linting {}", source_path.display());
    let source_code = fs::read_to_string(source_path)?;
    let ast = parse_ccl_source(&source_code)?;
    let mut semantic_analyzer = SemanticAnalyzer::new();
    semantic_analyzer.analyze(&ast)?;
    println!("[CCL CLI Lib] {} passed checks.", source_path.display());
    Ok(())
}

// This function would be called by `icn-cli ccl fmt ...`
pub fn format_ccl_file(source_path: &PathBuf, _inplace: bool) -> Result<String, CclError> {
    println!(
        "[CCL CLI Lib] Formatting {} (Inplace: {}) (Formatting logic pending)",
        source_path.display(),
        _inplace
    );
    let source_code = fs::read_to_string(source_path)?;
    // TODO: Implement actual CCL auto-formatter using the Pest grammar and AST
    // For now, just return the original source
    Ok(source_code)
}

// This function would be called by `icn-cli ccl explain ...`
pub fn explain_ccl_policy(
    source_path: &PathBuf,
    _target_construct: Option<String>,
) -> Result<String, CclError> {
    println!(
        "[CCL CLI Lib] Explaining {} (Target: {:?}) (Explanation logic pending)",
        source_path.display(),
        _target_construct
    );
    // TODO: Implement logic to analyze a CCL policy and explain its behavior,
    // potentially focusing on a specific rule or function.
    Ok(format!("Explanation for {} (stub):\nThis policy is designed to govern resources based on CCL principles.", source_path.display()))
}

// Helper for min, replace with std::cmp::min
use std::cmp::min;
