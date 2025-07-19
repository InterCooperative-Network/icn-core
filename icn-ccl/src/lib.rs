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
pub mod stdlib;
pub mod wasm_backend; // Expose functions for CLI layer

pub use error::CclError;
pub use metadata::ContractMetadata;
pub use stdlib::StandardLibrary;

/// Compiles a CCL source string into WASM bytecode and metadata.
pub fn compile_ccl_source_to_wasm(source: &str) -> Result<(Vec<u8>, ContractMetadata), CclError> {
    use icn_common::{compute_merkle_cid, Did};
    use sha2::{Digest, Sha256};

    let mut ast_node = parser::parse_ccl_source(source)?;
    ast_node = expand_macros(ast_node, &StandardLibrary::new())?;

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

/// Expand macro definitions in the given AST using the standard library.
fn expand_macros(ast: ast::AstNode, stdlib: &StandardLibrary) -> Result<ast::AstNode, CclError> {
    use ast::{AstNode, PolicyStatementNode};

    if let AstNode::Policy(stmts) = ast {
        let mut expanded = Vec::new();
        let mut local_stdlib = stdlib.clone();
        
        // First pass: collect macro definitions and register them
        for stmt in &stmts {
            if let PolicyStatementNode::MacroDef { name, params, body } = stmt {
                local_stdlib.register_macro(name.clone(), params.clone(), body.clone());
            }
        }
        
        // Second pass: process statements 
        // For now, we just keep all statements including macro definitions
        // TODO: Implement full macro expansion in expressions
        for stmt in stmts {
            match stmt {
                PolicyStatementNode::MacroDef { .. } => {
                    // Keep macro definitions in the output
                    expanded.push(stmt);
                }
                other => {
                    // For now, just keep other statements as-is
                    // TODO: In a full implementation, we would recursively 
                    // traverse expressions in these statements to find and expand macro calls
                    expanded.push(other);
                }
            }
        }
        Ok(AstNode::Policy(expanded))
    } else {
        // For non-Policy nodes, return as-is for now
        // TODO: Implement macro expansion for other node types
        Ok(ast)
    }
}


