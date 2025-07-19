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
pub mod stdlib;
pub mod governance_std;

pub use error::CclError;
pub use metadata::ContractMetadata;
pub use stdlib::StandardLibrary;

/// Compiles a CCL source string into WASM bytecode and metadata.
pub fn compile_ccl_source_to_wasm(source: &str) -> Result<(Vec<u8>, ContractMetadata), CclError> {
    use sha2::{Digest, Sha256};
    use icn_common::{compute_merkle_cid, Did};

    let ast_node = parser::parse_ccl_source(source)?;

    let mut semantic_analyzer = semantic_analyzer::SemanticAnalyzer::new();
    semantic_analyzer.analyze(&ast_node)?;

    let optimizer = optimizer::Optimizer::new();
    let optimized_ast = optimizer.optimize(ast_node)?;

    let mut backend = wasm_backend::WasmBackend::new();
    let (wasm, mut meta) = backend.compile_to_wasm(&optimized_ast)?;

    // Compute the CID for the generated WASM so executors can fetch it via the
    // runtime DAG APIs. This mirrors the behavior in the CLI helper.
    let ts = 0u64;
    let author = Did::new("key", "tester");
    let sig_opt = None;
    let cid = compute_merkle_cid(0x71, &wasm, &[], ts, &author, &sig_opt, &None);
    meta.cid = cid.to_string();

    // Hash the source code for auditing purposes
    let digest = Sha256::digest(source.as_bytes());
    meta.source_hash = format!("sha256:{:x}", digest);

    Ok((wasm, meta))
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
    use std::fs;

    let source_code = fs::read_to_string(path).map_err(|e| {
        CclError::IoError(format!(
            "Failed to read source file {}: {}",
            path.display(),
            e
        ))
    })?;

    compile_ccl_source_to_wasm(&source_code)
}

// Re-export CLI helper functions for easier access by icn-cli
pub use cli::{check_ccl_file, compile_ccl_file, explain_ccl_policy, format_ccl_file};
