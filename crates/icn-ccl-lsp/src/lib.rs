//! ICN CCL Language Server Protocol Implementation
//!
//! This crate provides LSP support for CCL including:
//! - Syntax highlighting and validation
//! - Auto-completion for CCL functions and stdlib
//! - Go-to-definition for symbols
//! - Inline documentation and hover information
//! - Diagnostic messages for compilation errors

pub mod server;
pub mod completion;
pub mod diagnostics;
pub mod hover;

pub use server::{CclLanguageServer, start_lsp_server};