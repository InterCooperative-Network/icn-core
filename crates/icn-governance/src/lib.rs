#![doc = include_str!("../README.md")]

//! # ICN Governance Crate
//! This crate defines the mechanisms for network governance within the InterCooperative Network (ICN).
//! It handles proposal systems, voting procedures, quorum logic, and decision execution,
//! focusing on transparency, fairness, and flexibility.

use icn_common::{NodeInfo, CommonError, ICN_CORE_VERSION};

/// Placeholder function demonstrating use of common types for governance.
pub fn submit_governance_proposal(info: &NodeInfo, proposal_id: u32) -> Result<String, CommonError> {
    Ok(format!("Submitted governance proposal {} from node: {} (v{})", proposal_id, info.name, info.version))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_submit_governance_proposal() {
        let node_info = NodeInfo {
            name: "GovNode".to_string(),
            version: ICN_CORE_VERSION.to_string(),
            status_message: "Governance active".to_string(),
        };
        let result = submit_governance_proposal(&node_info, 101);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("101"));
    }
}
