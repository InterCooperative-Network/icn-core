//! ICN CCL Debugger
//!
//! This crate provides debugging capabilities for CCL contracts:
//! - Step-by-step execution of CCL/WASM contracts
//! - Breakpoint support
//! - Variable inspection
//! - Call stack analysis
//! - Contract state examination

pub mod debugger;
pub mod breakpoints;
pub mod inspector;
pub mod execution;

pub use debugger::CclDebugger;