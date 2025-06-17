// icn-ccl/src/lib.rs
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::ptr_arg)]
#![allow(clippy::new_without_default)]
#![allow(clippy::only_used_in_recursion)]
#![allow(clippy::clone_on_copy)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::len_zero)]

pub mod ast;
pub mod error;
// pub mod grammar; // If ccl.pest is directly included or re-exported - REMOVING THIS
pub mod cli;
pub mod metadata;
pub mod optimizer;
pub mod parser;
pub mod semantic_analyzer;
pub mod wasm_backend; // Expose functions for CLI layer

pub use error::CclError;
pub use metadata::ContractMetadata;

/// Compiles a CCL source string into WASM bytecode and metadata.
pub fn compile_ccl_source_to_wasm(source: &str) -> Result<(Vec<u8>, ContractMetadata), CclError> {
    let ast_node = parser::parse_ccl_source(source)?;

    let mut semantic_analyzer = semantic_analyzer::SemanticAnalyzer::new();
    semantic_analyzer.analyze(&ast_node)?;

    let optimizer = optimizer::Optimizer::new();
    let optimized_ast = optimizer.optimize(ast_node)?; // ast_node might be consumed or cloned

    let backend = wasm_backend::WasmBackend::new();
    backend.compile_to_wasm(&optimized_ast)
}

/// Reads a CCL source file from disk and compiles it to WASM bytecode and metadata.
///
/// This is similar to [`cli::compile_ccl_file`] but returns the generated WASM
/// bytes directly instead of writing them to disk. It also fills out the
/// [`ContractMetadata`] with a placeholder CID and the SHA-256 hash of the
/// source for auditing purposes.
pub fn compile_ccl_file_to_wasm(
    path: &std::path::Path,
) -> Result<(Vec<u8>, ContractMetadata), CclError> {
    use sha2::{Digest, Sha256};
    use std::cmp::min;
    use std::fs;

    let source_code = fs::read_to_string(path).map_err(|e| {
        CclError::IoError(format!(
            "Failed to read source file {}: {}",
            path.display(),
            e
        ))
    })?;

    let (wasm, mut meta) = compile_ccl_source_to_wasm(&source_code)?;

    // Placeholder CID calculation until real DAG integration is wired in.
    meta.cid = format!("bafy2bzace{}", hex::encode(&wasm[0..min(10, wasm.len())]));

    let digest = Sha256::digest(source_code.as_bytes());
    meta.source_hash = format!("sha256:{:x}", digest);

    Ok((wasm, meta))
}

// Re-export CLI helper functions for easier access by icn-cli
pub use cli::{check_ccl_file, compile_ccl_file, explain_ccl_policy, format_ccl_file};
