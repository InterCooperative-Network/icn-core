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

// Re-export CLI helper functions for easier access by icn-cli
pub use cli::{check_ccl_file, compile_ccl_file, explain_ccl_policy, format_ccl_file};
