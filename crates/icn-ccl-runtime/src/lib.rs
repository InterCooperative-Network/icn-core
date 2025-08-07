//! # ICN CCL Runtime
//! 
//! Enhanced Cooperative Contract Language runtime with security, federation support,
//! and economic metering for the InterCooperative Network.
//!
//! This crate provides:
//! - Secure WASM contract execution with resource limits
//! - Capability-based security model
//! - Federation-aware contract scoping
//! - Economic metering via mana consumption
//! - Democratic governance and economic standard library
//! - Formal verification and invariant checking

use icn_common::{CommonError, Did, Cid, SystemTimeProvider, TimeProvider};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BTreeSet, HashSet};
use thiserror::Error;
use sha2::Digest;

pub mod execution;
pub mod security;
pub mod federation;
pub mod metering;
pub mod stdlib;
pub mod verification;

pub use execution::{ContractExecutor, ExecutionContext, ExecutionResult};
pub use security::{Capability, SecurityConfig, ResourceLimits};
pub use federation::{ContractScope, FederationId, CrossFederationProtocol};
pub use metering::{ManaMetering, EconomicEnforcement};
pub use stdlib::{DemocraticGovernanceContract, MutualCreditContract, JobMarketplaceContract};
pub use verification::{PropertyTester, InvariantChecker, FormalVerifier};

/// CCL Runtime errors
#[derive(Error, Debug)]
pub enum CclRuntimeError {
    #[error("Execution error: {0}")]
    ExecutionError(String),
    
    #[error("Security violation: {0}")]
    SecurityViolation(String),
    
    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),
    
    #[error("Insufficient mana: required {required}, available {available}")]
    InsufficientMana { required: u64, available: u64 },
    
    #[error("Permission denied: missing capability {0:?}")]
    PermissionDenied(Capability),
    
    #[error("Federation error: {0}")]
    FederationError(String),
    
    #[error("Contract not found: {0}")]
    ContractNotFound(String),
    
    #[error("Invalid contract state")]
    InvalidContractState,
    
    #[error("Common error: {0}")]
    CommonError(#[from] CommonError),
}

/// Contract deployment status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContractState {
    Deploying,
    Active,
    Paused,
    Upgrading,
    Deprecated,
    Destroyed,
}

/// Contract address type
pub type ContractAddress = String;

/// Time epoch for scheduling and coordination
pub type Epoch = u64;

/// WASM bytecode
pub type WasmCode = Vec<u8>;

/// Contract deployment metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractDeployment {
    pub address: ContractAddress,
    pub code_hash: String,
    pub deployer: Did,
    pub scope: ContractScope,
    pub timestamp: u64,
    pub state: ContractState,
    pub resource_limits: ResourceLimits,
    pub capabilities: BTreeSet<Capability>,
}

/// Contract function call parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractCall {
    pub contract: ContractAddress,
    pub function: String,
    pub args: Vec<u8>,
    pub mana_limit: u64,
    pub caller: Did,
}

/// Contract call result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractCallResult {
    pub success: bool,
    pub return_value: Option<Vec<u8>>,
    pub mana_consumed: u64,
    pub events: Vec<ContractEvent>,
    pub error: Option<String>,
}

/// Contract event emitted during execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractEvent {
    pub contract: ContractAddress,
    pub event_type: String,
    pub data: Vec<u8>,
    pub block_height: u64,
    pub timestamp: u64,
}

/// High-level CCL runtime interface
pub struct CclRuntime {
    executor: ContractExecutor,
    metering: ManaMetering,
    security: SecurityConfig,
    federation: CrossFederationProtocol,
    contracts: HashMap<ContractAddress, ContractDeployment>,
}

impl CclRuntime {
    /// Create a new CCL runtime instance
    pub fn new(
        security_config: SecurityConfig,
        federation_id: FederationId,
    ) -> Result<Self, CclRuntimeError> {
        Ok(CclRuntime {
            executor: ContractExecutor::new(security_config.clone())?,
            metering: ManaMetering::new(),
            security: security_config,
            federation: CrossFederationProtocol::new(federation_id),
            contracts: HashMap::new(),
        })
    }
    
    /// Deploy a new contract
    pub async fn deploy_contract(
        &mut self,
        code: WasmCode,
        deployer: Did,
        scope: ContractScope,
        init_args: Vec<u8>,
    ) -> Result<ContractAddress, CclRuntimeError> {
        // Validate deployment permissions
        self.security.check_deploy_permission(&deployer)?;
        
        // Calculate deployment cost
        let deploy_cost = self.metering.calculate_deploy_cost(&code);
        
        // Check mana balance
        self.metering.check_mana_balance(&deployer, deploy_cost)?;
        
        // Generate contract address
        let address = self.generate_contract_address(&code, &deployer);
        
        // Validate WASM code
        self.security.validate_wasm_code(&code)?;
        
        // Create execution context
        let context = ExecutionContext::new(
            deployer.clone(),
            address.clone(),
            self.security.default_resource_limits.clone(),
            scope.clone(),
        );
        
        // Deploy contract
        self.executor.deploy_contract(
            address.clone(),
            code.clone(),
            context,
            init_args,
        ).await?;
        
        // Charge deployment cost
        self.metering.charge_mana(&deployer, deploy_cost)?;
        
        // Store contract metadata
        let deployment = ContractDeployment {
            address: address.clone(),
            code_hash: hex::encode(sha2::Sha256::digest(&code)),
            deployer,
            scope,
            timestamp: crate::current_timestamp(),
            state: ContractState::Active,
            resource_limits: self.security.default_resource_limits.clone(),
            capabilities: self.security.default_capabilities.clone(),
        };
        
        self.contracts.insert(address.clone(), deployment);
        
        Ok(address)
    }
    
    /// Call a contract function
    pub async fn call_contract(
        &mut self,
        call: ContractCall,
    ) -> Result<ContractCallResult, CclRuntimeError> {
        // Check if contract exists
        let contract = self.contracts.get(&call.contract)
            .ok_or_else(|| CclRuntimeError::ContractNotFound(call.contract.clone()))?;
        
        // Check contract state
        if contract.state != ContractState::Active {
            return Err(CclRuntimeError::InvalidContractState);
        }
        
        // Check permissions
        self.security.check_call_permission(&call.caller, &contract.scope)?;
        
        // Check mana limit
        self.metering.check_mana_balance(&call.caller, call.mana_limit)?;
        
        // Create execution context
        let context = ExecutionContext::new(
            call.caller.clone(),
            call.contract.clone(),
            contract.resource_limits.clone(),
            contract.scope.clone(),
        );
        
        // Execute contract function
        let result = self.executor.call_function(
            call.contract.clone(),
            call.function,
            call.args,
            context,
        ).await?;
        
        // Charge for execution
        self.metering.charge_mana(&call.caller, result.mana_consumed)?;
        
        // Refund unused mana
        if call.mana_limit > result.mana_consumed {
            self.metering.refund_mana(&call.caller, call.mana_limit - result.mana_consumed)?;
        }
        
        Ok(ContractCallResult {
            success: result.success,
            return_value: result.return_value,
            mana_consumed: result.mana_consumed,
            events: result.events,
            error: result.error,
        })
    }
    
    /// Get contract metadata
    pub fn get_contract(&self, address: &ContractAddress) -> Option<&ContractDeployment> {
        self.contracts.get(address)
    }
    
    /// List all contracts
    pub fn list_contracts(&self) -> Vec<&ContractDeployment> {
        self.contracts.values().collect()
    }
    
    /// Pause a contract (governance action)
    pub fn pause_contract(
        &mut self,
        address: &ContractAddress,
        caller: &Did,
    ) -> Result<(), CclRuntimeError> {
        // Check governance permission
        self.security.check_governance_permission(caller)?;
        
        if let Some(contract) = self.contracts.get_mut(address) {
            contract.state = ContractState::Paused;
            Ok(())
        } else {
            Err(CclRuntimeError::ContractNotFound(address.clone()))
        }
    }
    
    /// Resume a paused contract
    pub fn resume_contract(
        &mut self,
        address: &ContractAddress,
        caller: &Did,
    ) -> Result<(), CclRuntimeError> {
        // Check governance permission
        self.security.check_governance_permission(caller)?;
        
        if let Some(contract) = self.contracts.get_mut(address) {
            if contract.state == ContractState::Paused {
                contract.state = ContractState::Active;
                Ok(())
            } else {
                Err(CclRuntimeError::InvalidContractState)
            }
        } else {
            Err(CclRuntimeError::ContractNotFound(address.clone()))
        }
    }
    
    /// Generate deterministic contract address
    fn generate_contract_address(&self, code: &WasmCode, deployer: &Did) -> ContractAddress {
        let mut hasher = sha2::Sha256::new();
        hasher.update(code);
        hasher.update(deployer.to_string().as_bytes());
        hasher.update(&crate::current_timestamp().to_be_bytes());
        format!("contract_{}", hex::encode(hasher.finalize())[..16].to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::Did;
    
    #[tokio::test]
    async fn test_runtime_creation() {
        let security_config = SecurityConfig::default();
        let federation_id = FederationId::new("test_fed".to_string());
        
        let runtime = CclRuntime::new(security_config, federation_id);
        assert!(runtime.is_ok());
    }
    
    #[tokio::test]
    async fn test_contract_deployment() {
        let security_config = SecurityConfig::default();
        let federation_id = FederationId::new("test_fed".to_string());
        let mut runtime = CclRuntime::new(security_config, federation_id).unwrap();
        
        let deployer = Did::new("key", "test_deployer");
        let code = vec![0x00, 0x61, 0x73, 0x6d]; // Basic WASM header
        let scope = ContractScope::Local("test_org".to_string());
        
        // This will fail due to mana checks, but tests the flow
        let result = runtime.deploy_contract(code, deployer, scope, vec![]).await;
        
        // Should fail due to mana balance check
        assert!(result.is_err());
    }
}

/// Helper function to get current timestamp
pub fn current_timestamp() -> u64 {
    SystemTimeProvider.unix_seconds()
}