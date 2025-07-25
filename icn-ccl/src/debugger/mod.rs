// icn-ccl/src/debugger/mod.rs
//! CCL/WASM debugger integration
//! 
//! This module provides debugging capabilities for CCL contracts by:
//! - Generating source maps from CCL to WASM
//! - Integrating with WASM debugging tools
//! - Providing breakpoint and step-through debugging

pub mod source_map;
pub mod wasm_debugger;

pub use source_map::SourceMap;
pub use wasm_debugger::WasmDebugger;