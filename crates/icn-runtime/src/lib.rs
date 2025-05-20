#![doc = include_str!("../README.md")]

//! # ICN Runtime Crate
//! This crate provides the execution environment for InterCooperative Network (ICN) logic,
//! possibly including WebAssembly (WASM) runtimes and host interaction capabilities.
//! It focuses on a secure, performant, and modular execution environment with well-defined host functions.

pub mod abi;
pub mod context;

use icn_common::{NodeInfo, CommonError, ICN_CORE_VERSION, Did};
use context::{RuntimeContext, HostAbiError, JobId};
use std::str::FromStr;

/// Placeholder function demonstrating use of common types for runtime operations.
/// This function is not directly part of the Host ABI layer discussed below but serves as an example.
pub fn execute_icn_script(info: &NodeInfo, script_id: &str) -> Result<String, CommonError> {
    Ok(format!("Executed script {} for node: {} (v{})", script_id, info.name, info.version))
}

// --- Host ABI Functions ---
// These functions are intended to be callable from a WASM environment,
// mediated by the `HostEnvironment` and using a `RuntimeContext`.

/// ABI Index: (defined in `abi::ABI_HOST_SUBMIT_MESH_JOB`)
/// Submits a job to the mesh network using the provided runtime context.
///
/// The `job_spec_json` is expected to be a JSON string describing the job.
/// This will be parsed and transformed into a `MeshJob` structure internally.
///
/// TODO: Implement full logic for job_spec_json parsing into a `Vec<u8>` or structured type for `ctx.submit_mesh_job`.
/// TODO: WASM bindings will need to handle memory marshalling for `job_spec_json`.
pub fn host_submit_mesh_job(ctx: &mut RuntimeContext, job_spec_json: &str) -> Result<JobId, HostAbiError> {
    // TODO: record metric `icn_runtime_abi_call_total{method="host_submit_mesh_job"}`
    println!("[RUNTIME_ABI] host_submit_mesh_job called with spec: {}", job_spec_json);

    if job_spec_json.is_empty() {
        return Err(HostAbiError::InvalidParameters("Job specification JSON cannot be empty".to_string()));
    }
    // For now, let's assume job_spec_json can be directly used or converted to Vec<u8>.
    // A real implementation would parse this JSON into a more structured MeshJob or its data payload.
    let job_data = job_spec_json.as_bytes().to_vec(); 
    ctx.submit_mesh_job(job_data)
}

/// ABI Index: (defined in `abi::ABI_HOST_ACCOUNT_GET_MANA`)
/// Retrieves the current mana for the calling account/identity, using the provided runtime context.
///
/// The `account_id_str` is the string representation of the DID for which mana is requested.
/// In many cases, this will be the `current_identity` within the `ctx`,
/// but the API allows specifying it for potential future flexibility (e.g., admin queries).
///
/// TODO: WASM bindings will need to handle memory marshalling for `account_id_str`.
pub fn host_account_get_mana(ctx: &RuntimeContext, account_id_str: &str) -> Result<u64, HostAbiError> {
    // TODO: record metric `icn_runtime_abi_call_total{method="host_account_get_mana"}`
    println!("[RUNTIME_ABI] host_account_get_mana called for account: {}", account_id_str);

    if account_id_str.is_empty() {
        return Err(HostAbiError::InvalidParameters("Account ID string cannot be empty".to_string()));
    }

    let account_did = Did::from_str(account_id_str)
        .map_err(|e| HostAbiError::InvalidParameters(format!("Invalid DID format for account_id: {}", e)))?;
    
    ctx.get_mana(&account_did)
}

/// ABI Index: (defined in `abi::ABI_HOST_ACCOUNT_SPEND_MANA`)
/// Attempts to spend mana from the specified account, using the provided runtime context.
///
/// The `account_id_str` is the string representation of the DID from which mana should be spent.
/// The `amount` is the quantity of mana to spend.
///
/// Policy Note: `RuntimeContext::spend_mana` currently only allows spending from `ctx.current_identity`.
///
/// TODO: WASM bindings will need to handle memory marshalling for `account_id_str` and `amount`.
pub fn host_account_spend_mana(ctx: &mut RuntimeContext, account_id_str: &str, amount: u64) -> Result<(), HostAbiError> {
    // TODO: record metric `icn_runtime_abi_call_total{method="host_account_spend_mana"}`
    println!("[RUNTIME_ABI] host_account_spend_mana called for account: {} amount: {}", account_id_str, amount);

    if account_id_str.is_empty() {
        return Err(HostAbiError::InvalidParameters("Account ID string cannot be empty".to_string()));
    }
    if amount == 0 {
        return Err(HostAbiError::InvalidParameters("Spend amount must be greater than zero".to_string()));
    }

    let account_did = Did::from_str(account_id_str)
        .map_err(|e| HostAbiError::InvalidParameters(format!("Invalid DID format for account_id: {}", e)))?;
    
    ctx.spend_mana(&account_did, amount)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::abi;
    use super::context::{RuntimeContext, HostAbiError, JobId, SimpleManaLedger};
    use icn_common::Did;
    use std::str::FromStr;

    const TEST_IDENTITY_DID_STR: &str = "did:icn:test:dummy_executor";
    const OTHER_IDENTITY_DID_STR: &str = "did:icn:test:other_account";

    fn create_test_context() -> RuntimeContext {
        let test_did = Did::from_str(TEST_IDENTITY_DID_STR).expect("Failed to create test DID");
        RuntimeContext::new(test_did)
    }

    fn create_test_context_with_mana(initial_mana: u64) -> RuntimeContext {
        let test_did = Did::from_str(TEST_IDENTITY_DID_STR).expect("Failed to create test DID");
        let mut ctx = RuntimeContext::new(test_did.clone());
        ctx.mana_ledger.set_balance(&test_did, initial_mana);
        ctx
    }

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
    #[should_panic(expected = "RuntimeContext::submit_mesh_job: Hook into mesh job queue and return actual JobId")]
    fn test_host_submit_mesh_job_calls_context() {
        let mut ctx = create_test_context();
        let job_spec = "{\"name\": \"test_job_001\"}";
        let result = host_submit_mesh_job(&mut ctx, job_spec);
        assert!(result.is_ok());
    }

    #[test]
    fn test_host_submit_mesh_job_empty_spec() {
        let mut ctx = create_test_context();
        let job_spec = "";
        let result = host_submit_mesh_job(&mut ctx, job_spec);
        assert!(result.is_err());
        match result.err().unwrap() {
            HostAbiError::InvalidParameters(msg) => {
                assert_eq!(msg, "Job specification JSON cannot be empty");
            }
            e => panic!("Expected InvalidParameters error, got {:?}", e),
        }
    }

    #[test]
    #[should_panic(expected = "RuntimeContext::get_mana: Read mana from repository for the account")]
    fn test_host_account_get_mana_calls_context() {
        let ctx = create_test_context_with_mana(100);
        let account_id = ctx.current_identity.to_string();
        let _ = host_account_get_mana(&ctx, &account_id);
    }

    #[test]
    fn test_host_account_get_mana_empty_id() {
        let ctx = create_test_context();
        let account_id = "";
        let result = host_account_get_mana(&ctx, account_id);
        assert!(result.is_err());
        match result.err().unwrap() {
            HostAbiError::InvalidParameters(msg) => {
                assert_eq!(msg, "Account ID string cannot be empty");
            }
            e => panic!("Expected InvalidParameters error, got {:?}", e),
        }
    }

    #[test]
    fn test_host_account_get_mana_invalid_did_format() {
        let ctx = create_test_context();
        let account_id = "not-a-valid-did";
        let result = host_account_get_mana(&ctx, account_id);
        assert!(result.is_err());
        match result.err().unwrap() {
            HostAbiError::InvalidParameters(msg) => {
                assert!(msg.contains("Invalid DID format for account_id"));
            }
            e => panic!("Expected InvalidParameters error for DID format, got {:?}", e),
        }
    }

    #[test]
    fn test_host_account_spend_mana_successful() {
        let mut ctx = create_test_context_with_mana(100);
        let spend_amount = 30;
        let result = host_account_spend_mana(&mut ctx, TEST_IDENTITY_DID_STR, spend_amount);
        assert!(result.is_ok(), "Expected successful spend, got {:?}", result.err());
        let remaining_mana = ctx.get_mana(&ctx.current_identity).unwrap();
        assert_eq!(remaining_mana, 70, "Mana not deducted correctly");
    }

    #[test]
    fn test_host_account_spend_mana_insufficient_funds() {
        let mut ctx = create_test_context_with_mana(20);
        let spend_amount = 30;
        let result = host_account_spend_mana(&mut ctx, TEST_IDENTITY_DID_STR, spend_amount);
        assert!(result.is_err());
        match result.err().unwrap() {
            HostAbiError::InsufficientMana => { /* Expected */ }
            e => panic!("Expected InsufficientMana, got {:?}", e),
        }
        let remaining_mana = ctx.get_mana(&ctx.current_identity).unwrap();
        assert_eq!(remaining_mana, 20, "Mana should not change on failed spend");
    }

    #[test]
    fn test_host_account_spend_mana_account_not_found() {
        let mut ctx = create_test_context_with_mana(100);
        let non_existent_did_str = "did:icn:test:non_existent_account";
        let result = host_account_spend_mana(&mut ctx, non_existent_did_str, 10);
        assert!(result.is_err());
        match result.err().unwrap() {
            HostAbiError::AccountNotFound(did) => {
                assert_eq!(did.to_string(), non_existent_did_str);
            }
            e => panic!("Expected AccountNotFound, got {:?}", e),
        }
    }

    #[test]
    fn test_host_account_spend_mana_zero_amount() {
        let mut ctx = create_test_context_with_mana(100);
        let result = host_account_spend_mana(&mut ctx, TEST_IDENTITY_DID_STR, 0);
        assert!(result.is_err());
        match result.err().unwrap() {
            HostAbiError::InvalidParameters(msg) => {
                assert_eq!(msg, "Spend amount must be greater than zero");
            }
            e => panic!("Expected InvalidParameters for zero amount, got {:?}", e),
        }
    }

    #[test]
    fn test_host_account_spend_mana_empty_account_id() {
        let mut ctx = create_test_context_with_mana(100);
        let result = host_account_spend_mana(&mut ctx, "", 10);
        assert!(result.is_err());
        match result.err().unwrap() {
            HostAbiError::InvalidParameters(msg) => {
                assert_eq!(msg, "Account ID string cannot be empty");
            }
            e => panic!("Expected InvalidParameters for empty account ID, got {:?}", e),
        }
    }

    #[test]
    fn test_host_account_spend_mana_invalid_did_format() {
        let mut ctx = create_test_context_with_mana(100);
        let result = host_account_spend_mana(&mut ctx, "not-a-valid-did", 10);
        assert!(result.is_err());
        match result.err().unwrap() {
            HostAbiError::InvalidParameters(msg) => {
                assert!(msg.contains("Invalid DID format for account_id"));
            }
            e => panic!("Expected InvalidParameters for invalid DID, got {:?}", e),
        }
    }

    #[test]
    fn test_host_account_spend_mana_for_other_account_fails_policy() {
        let mut ctx = create_test_context_with_mana(100);
        let other_did = Did::from_str(OTHER_IDENTITY_DID_STR).unwrap();
        ctx.mana_ledger.set_balance(&other_did, 50);

        let result = host_account_spend_mana(&mut ctx, OTHER_IDENTITY_DID_STR, 10);
        assert!(result.is_err());
        match result.err().unwrap() {
            HostAbiError::InvalidParameters(msg) => {
                assert!(msg.contains("Attempting to spend mana for an account other than the current context identity"));
            }
            e => panic!("Expected InvalidParameters due to policy violation, got {:?}", e),
        }
        let other_mana = ctx.get_mana(&other_did).unwrap();
        assert_eq!(other_mana, 50);
    }
}
