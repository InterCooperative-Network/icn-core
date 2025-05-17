#![doc = include_str!("../README.md")]

//! # ICN Runtime Crate
//! This crate provides the execution environment for InterCooperative Network (ICN) logic,
//! possibly including WebAssembly (WASM) runtimes and host interaction capabilities.
//! It focuses on a secure, performant, and modular execution environment with well-defined host functions.

use icn_common::{NodeInfo, CommonError, ICN_CORE_VERSION};

/// Placeholder function demonstrating use of common types for runtime operations.
pub fn execute_icn_script(info: &NodeInfo, script_id: &str) -> Result<String, CommonError> {
    Ok(format!("Executed script {} for node: {} (v{})", script_id, info.name, info.version))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_icn_script() {
        let node_info = NodeInfo {
            name: "RuntimeNode".to_string(),
            version: ICN_CORE_VERSION.to_string(),
            status_message: "Runtime active".to_string(),
        };
        let result = execute_icn_script(&node_info, "script-xyz");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("script-xyz"));
    }
}
