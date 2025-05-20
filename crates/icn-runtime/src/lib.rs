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

// --- Host ABI Function Stubs ---

/// ABI Index: 16 (Example)
/// Submits a job to the mesh network.
///
/// TODO: Implement full logic:
///       - Define proper request and response types (e.g., job specification, job ID).
///       - Interact with the `icn-mesh` crate to actually schedule the job.
///       - Handle errors from the mesh crate.
///       - Ensure `RuntimeContext::pending_mesh_jobs` is updated.
///       - Consider how this is called from a WASM environment (e.g., parameters, return values).
pub fn host_submit_mesh_job(job_spec_json: &str) -> Result<String, CommonError> {
    println!("[RUNTIME_STUB] host_submit_mesh_job called with spec: {}", job_spec_json);
    // For now, let's assume job_spec_json is a simple job identifier string
    // and we return a mock job ID.
    if job_spec_json.is_empty() {
        return Err(CommonError::InvalidInputError("Job specification cannot be empty".to_string()));
    }
    Ok(format!("mock_job_id_for_{}", job_spec_json))
}

/// ABI Index: (Example, TBD, e.g., 5)
/// Retrieves the current mana for the calling account/identity.
///
/// TODO: Implement full logic:
///       - Determine how the caller's identity/account is resolved from the RuntimeContext.
///       - Interact with `icn-economics` crate (`ManaRepositoryAdapter`, `SledManaLedger`).
///       - Define proper return type for mana (e.g., u64).
///       - Handle errors (e.g., account not found).
pub fn host_account_get_mana(account_id: &str) -> Result<u64, CommonError> {
    println!("[RUNTIME_STUB] host_account_get_mana called for account: {}", account_id);
    if account_id.is_empty() {
        return Err(CommonError::InvalidInputError("Account ID cannot be empty".to_string()));
    }
    // Return a mock mana value.
    Ok(1000)
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

    #[test]
    fn test_host_submit_mesh_job_stub_ok() {
        let job_spec = "test_job_spec_001";
        let result = host_submit_mesh_job(job_spec);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "mock_job_id_for_test_job_spec_001");
        // TODO: Add more comprehensive tests once integrated with icn-mesh:
        //       - Test with valid and invalid job specifications.
        //       - Mock interactions with `icn-mesh` and verify calls.
        //       - Test error handling paths.
    }

    #[test]
    fn test_host_submit_mesh_job_stub_empty_spec() {
        let job_spec = "";
        let result = host_submit_mesh_job(job_spec);
        assert!(result.is_err());
        match result.err().unwrap() {
            CommonError::InvalidInputError(msg) => {
                assert_eq!(msg, "Job specification cannot be empty");
            }
            _ => panic!("Expected InvalidInputError"),
        }
    }

    #[test]
    fn test_host_account_get_mana_stub_ok() {
        let account_id = "test_account_123";
        let result = host_account_get_mana(account_id);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1000);
        // TODO: Add more comprehensive tests once integrated with icn-economics:
        //       - Test with valid and non-existent account IDs.
        //       - Mock interactions with mana ledger.
        //       - Test different mana balances.
    }

    #[test]
    fn test_host_account_get_mana_stub_empty_id() {
        let account_id = "";
        let result = host_account_get_mana(account_id);
        assert!(result.is_err());
        match result.err().unwrap() {
            CommonError::InvalidInputError(msg) => {
                assert_eq!(msg, "Account ID cannot be empty");
            }
            _ => panic!("Expected InvalidInputError"),
        }
    }
}
