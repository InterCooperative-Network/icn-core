// icn-ccl/src/lsp/mod.rs
//! Language Server Protocol implementation for CCL
//!
//! This module provides LSP features including:
//! - Syntax highlighting and diagnostics
//! - Autocompletion for CCL keywords, functions, and variables
//! - Go-to-definition and find references
//! - Hover information for symbols
//! - Document formatting

pub mod completion;
pub mod diagnostics;
pub mod formatting;
pub mod hover;
pub mod navigation;
pub mod server;

pub use server::CclLanguageServer;
