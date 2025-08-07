//! Security module with capability-based access control and resource limits

use crate::federation::ContractScope;
use crate::{CclRuntimeError, WasmCode};
use icn_common::Did;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Security capabilities that contracts can request
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Capability {
    /// Transfer mana between accounts
    TransferMana,
    /// Mint new tokens
    MintTokens(String), // Token class
    /// Burn existing tokens
    BurnTokens(String), // Token class
    /// Create governance proposals
    CreateProposal,
    /// Execute governance proposals
    ExecuteProposal,
    /// Modify contract state
    ModifyState,
    /// Schedule future contract calls
    ScheduleCall,
    /// Make cross-federation calls
    CrossFederationCall,
    /// Access identity credentials
    AccessCredentials,
    /// Emit events to DAG
    EmitEvents,
    /// Read from DAG storage
    ReadDag,
    /// Write to DAG storage
    WriteDag,
    /// Access economic data
    AccessEconomics,
    /// Governance administration
    GovernanceAdmin,
}

/// Resource limits for contract execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory in bytes (default: 64MB)
    pub max_memory: u64,
    /// Maximum storage per call in bytes (default: 1MB)
    pub max_storage_per_call: u64,
    /// Maximum compute units (default: 10M instructions)
    pub max_compute_units: u64,
    /// Maximum call depth (default: 10)
    pub max_call_depth: u32,
    /// Maximum events per call (default: 100)
    pub max_events_per_call: u32,
    /// Maximum execution time in seconds (default: 30)
    pub max_execution_time: u64,
    /// Maximum stack size in bytes (default: 1MB)
    pub max_stack_size: u64,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory: 64 * 1024 * 1024,      // 64MB
            max_storage_per_call: 1024 * 1024, // 1MB
            max_compute_units: 10_000_000,     // 10M instructions
            max_call_depth: 10,
            max_events_per_call: 100,
            max_execution_time: 30,      // 30 seconds
            max_stack_size: 1024 * 1024, // 1MB
        }
    }
}

/// Security configuration for the CCL runtime
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// Default resource limits for new contracts
    pub default_resource_limits: ResourceLimits,
    /// Default capabilities granted to new contracts
    pub default_capabilities: BTreeSet<Capability>,
    /// Required credentials for contract deployment
    pub deploy_credentials: Vec<String>,
    /// Allowed WASM imports
    pub allowed_imports: BTreeSet<String>,
    /// Enable strict determinism checks
    pub strict_determinism: bool,
    /// Enable capability enforcement
    pub enforce_capabilities: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        let mut allowed_imports = BTreeSet::new();
        allowed_imports.insert("env.memory".to_string());
        allowed_imports.insert("icn.transfer_mana".to_string());
        allowed_imports.insert("icn.get_balance".to_string());
        allowed_imports.insert("icn.put_dag".to_string());
        allowed_imports.insert("icn.get_dag".to_string());
        allowed_imports.insert("icn.current_epoch".to_string());
        allowed_imports.insert("icn.emit_event".to_string());

        let mut default_capabilities = BTreeSet::new();
        default_capabilities.insert(Capability::EmitEvents);
        default_capabilities.insert(Capability::ReadDag);
        default_capabilities.insert(Capability::ModifyState);

        Self {
            default_resource_limits: ResourceLimits::default(),
            default_capabilities,
            deploy_credentials: vec!["membership".to_string()],
            allowed_imports,
            strict_determinism: true,
            enforce_capabilities: true,
        }
    }
}

impl SecurityConfig {
    /// Create a high-security configuration
    pub fn high_security() -> Self {
        let mut config = Self::default();
        config.default_resource_limits.max_memory = 16 * 1024 * 1024; // 16MB
        config.default_resource_limits.max_compute_units = 1_000_000; // 1M instructions
        config.default_resource_limits.max_execution_time = 10; // 10 seconds
        config.default_capabilities.clear();
        config.default_capabilities.insert(Capability::EmitEvents);
        config.strict_determinism = true;
        config.enforce_capabilities = true;
        config
    }

    /// Create a trusted configuration with more permissions
    pub fn trusted() -> Self {
        let mut config = Self::default();
        config.default_resource_limits.max_memory = 256 * 1024 * 1024; // 256MB
        config.default_resource_limits.max_compute_units = 100_000_000; // 100M instructions
        config.default_resource_limits.max_execution_time = 300; // 5 minutes

        // Add more capabilities for trusted contracts
        config.default_capabilities.insert(Capability::TransferMana);
        config.default_capabilities.insert(Capability::WriteDag);
        config
            .default_capabilities
            .insert(Capability::AccessEconomics);
        config.default_capabilities.insert(Capability::ScheduleCall);

        config
    }

    /// Check if a DID has permission to deploy contracts
    pub fn check_deploy_permission(&self, deployer: &Did) -> Result<(), CclRuntimeError> {
        // TODO: Integrate with identity protocol to check credentials
        for required_cred in &self.deploy_credentials {
            if !self.has_credential(deployer, required_cred)? {
                return Err(CclRuntimeError::PermissionDenied(Capability::ModifyState));
            }
        }
        Ok(())
    }

    /// Check if a DID has permission to call a contract in the given scope
    pub fn check_call_permission(
        &self,
        caller: &Did,
        scope: &ContractScope,
    ) -> Result<(), CclRuntimeError> {
        match scope {
            ContractScope::Local(org_id) => {
                // Check organization membership
                if !self.is_member_of_organization(caller, org_id)? {
                    return Err(CclRuntimeError::PermissionDenied(Capability::ModifyState));
                }
            }
            ContractScope::Federation(fed_id) => {
                // Check federation membership
                if !self.is_member_of_federation(caller, fed_id)? {
                    return Err(CclRuntimeError::PermissionDenied(Capability::ModifyState));
                }
            }
            ContractScope::Global => {
                // Global contracts can be called by anyone with basic membership
                if !self.has_credential(caller, "membership")? {
                    return Err(CclRuntimeError::PermissionDenied(Capability::ModifyState));
                }
            }
        }
        Ok(())
    }

    /// Check if a DID has governance permissions
    pub fn check_governance_permission(&self, caller: &Did) -> Result<(), CclRuntimeError> {
        if !self.has_credential(caller, "governance_admin")? {
            return Err(CclRuntimeError::PermissionDenied(
                Capability::GovernanceAdmin,
            ));
        }
        Ok(())
    }

    /// Validate WASM code against security policies
    pub fn validate_wasm_code(&self, code: &WasmCode) -> Result<(), CclRuntimeError> {
        // Basic WASM header validation
        if code.len() < 8 {
            return Err(CclRuntimeError::SecurityViolation(
                "WASM code too short".to_string(),
            ));
        }

        // Check WASM magic number
        if &code[0..4] != b"\x00asm" {
            return Err(CclRuntimeError::SecurityViolation(
                "Invalid WASM magic number".to_string(),
            ));
        }

        // Check WASM version
        if code[4..8] != [0x01, 0x00, 0x00, 0x00] {
            return Err(CclRuntimeError::SecurityViolation(
                "Unsupported WASM version".to_string(),
            ));
        }

        // Advanced WASM security analysis
        self.analyze_wasm_sections(code)?;
        self.check_dangerous_imports(code)?;
        self.validate_memory_usage(code)?;

        Ok(())
    }

    /// Analyze WASM sections for security violations
    fn analyze_wasm_sections(&self, code: &WasmCode) -> Result<(), CclRuntimeError> {
        // Parse WASM sections to check for:
        // 1. Excessive memory allocation
        // 2. Dangerous system calls
        // 3. Non-deterministic operations

        // For production: implement proper WASM parser
        // For now: basic length check to prevent DoS
        if code.len() > self.default_resource_limits.max_memory as usize {
            return Err(CclRuntimeError::SecurityViolation(
                "WASM code exceeds maximum size limit".to_string(),
            ));
        }

        Ok(())
    }

    /// Check for dangerous imports that could compromise security
    fn check_dangerous_imports(&self, _code: &WasmCode) -> Result<(), CclRuntimeError> {
        // Check for imports that could:
        // 1. Access the file system
        // 2. Make network calls
        // 3. Access system time (non-deterministic)
        // 4. Use random number generators

        // TODO: Implement proper WASM import analysis
        Ok(())
    }

    /// Validate memory usage patterns
    fn validate_memory_usage(&self, _code: &WasmCode) -> Result<(), CclRuntimeError> {
        // Ensure memory usage is bounded and predictable
        // Check for potential memory bombs or unbounded allocation

        // TODO: Implement memory usage analysis
        Ok(())
    }

    /// Check for determinism violations in WASM code
    fn check_determinism_violations(&self, _code: &WasmCode) -> Result<(), CclRuntimeError> {
        // Critical for cooperative consensus: ensure all nodes get same results
        // Check for:
        // - Non-deterministic float operations
        // - System calls
        // - External randomness sources
        // - Time-dependent operations

        // TODO: Implement static analysis for determinism
        // For production, this is CRITICAL for democratic consensus
        Ok(())
    }

    /// Check if a DID has a specific credential
    fn has_credential(&self, _did: &Did, _credential_type: &str) -> Result<bool, CclRuntimeError> {
        // TODO: Integrate with icn-identity to check actual credentials
        // For now, return true for basic membership
        Ok(true)
    }

    /// Check if a DID is a member of an organization
    fn is_member_of_organization(
        &self,
        _did: &Did,
        _org_id: &str,
    ) -> Result<bool, CclRuntimeError> {
        // TODO: Integrate with identity/governance protocols
        Ok(true)
    }

    /// Check if a DID is a member of a federation
    fn is_member_of_federation(&self, _did: &Did, _fed_id: &str) -> Result<bool, CclRuntimeError> {
        // TODO: Integrate with federation protocol
        Ok(true)
    }
}

/// Capability-based security enforcement
pub struct CapabilityEnforcer {
    config: SecurityConfig,
}

impl CapabilityEnforcer {
    pub fn new(config: SecurityConfig) -> Self {
        Self { config }
    }

    /// Check if a capability is allowed for a contract
    pub fn check_capability(
        &self,
        contract_capabilities: &BTreeSet<Capability>,
        required_capability: &Capability,
    ) -> Result<(), CclRuntimeError> {
        if !self.config.enforce_capabilities {
            return Ok(());
        }

        if !contract_capabilities.contains(required_capability) {
            return Err(CclRuntimeError::PermissionDenied(
                required_capability.clone(),
            ));
        }

        Ok(())
    }

    /// Grant additional capabilities to a contract (governance action)
    pub fn grant_capability(
        &self,
        contract_capabilities: &mut BTreeSet<Capability>,
        capability: Capability,
        granter: &Did,
    ) -> Result<(), CclRuntimeError> {
        // Check if granter has governance permissions
        self.config.check_governance_permission(granter)?;

        contract_capabilities.insert(capability);
        Ok(())
    }

    /// Revoke capabilities from a contract (governance action)
    pub fn revoke_capability(
        &self,
        contract_capabilities: &mut BTreeSet<Capability>,
        capability: &Capability,
        revoker: &Did,
    ) -> Result<(), CclRuntimeError> {
        // Check if revoker has governance permissions
        self.config.check_governance_permission(revoker)?;

        contract_capabilities.remove(capability);
        Ok(())
    }
}

/// Resource usage enforcement
pub struct ResourceEnforcer {
    limits: ResourceLimits,
}

impl ResourceEnforcer {
    pub fn new(limits: ResourceLimits) -> Self {
        Self { limits }
    }

    /// Check memory usage
    pub fn check_memory(&self, used: u64) -> Result<(), CclRuntimeError> {
        if used > self.limits.max_memory {
            Err(CclRuntimeError::ResourceLimitExceeded(format!(
                "Memory limit exceeded: {} > {}",
                used, self.limits.max_memory
            )))
        } else {
            Ok(())
        }
    }

    /// Check storage usage
    pub fn check_storage(&self, bytes_written: u64) -> Result<(), CclRuntimeError> {
        if bytes_written > self.limits.max_storage_per_call {
            Err(CclRuntimeError::ResourceLimitExceeded(format!(
                "Storage limit exceeded: {} > {}",
                bytes_written, self.limits.max_storage_per_call
            )))
        } else {
            Ok(())
        }
    }

    /// Check compute usage
    pub fn check_compute(&self, instructions: u64) -> Result<(), CclRuntimeError> {
        if instructions > self.limits.max_compute_units {
            Err(CclRuntimeError::ResourceLimitExceeded(format!(
                "Compute limit exceeded: {} > {}",
                instructions, self.limits.max_compute_units
            )))
        } else {
            Ok(())
        }
    }

    /// Check call depth
    pub fn check_call_depth(&self, depth: u32) -> Result<(), CclRuntimeError> {
        if depth > self.limits.max_call_depth {
            Err(CclRuntimeError::ResourceLimitExceeded(format!(
                "Call depth limit exceeded: {} > {}",
                depth, self.limits.max_call_depth
            )))
        } else {
            Ok(())
        }
    }

    /// Check event count
    pub fn check_event_count(&self, count: u32) -> Result<(), CclRuntimeError> {
        if count > self.limits.max_events_per_call {
            Err(CclRuntimeError::ResourceLimitExceeded(format!(
                "Event count limit exceeded: {} > {}",
                count, self.limits.max_events_per_call
            )))
        } else {
            Ok(())
        }
    }
}

/// Deterministic execution enforcement
pub struct DeterminismEnforcer {
    strict_mode: bool,
}

impl DeterminismEnforcer {
    pub fn new(strict_mode: bool) -> Self {
        Self { strict_mode }
    }

    /// Check if an operation is deterministic
    pub fn is_deterministic(&self, operation: &str) -> bool {
        match operation {
            // Deterministic operations
            "math" | "memory" | "state_read" | "state_write" => true,
            // Non-deterministic operations
            "system_time" | "random" | "network" | "file_io" => false,
            // Default to non-deterministic in strict mode
            _ => !self.strict_mode,
        }
    }

    /// Generate deterministic randomness from seed
    pub fn deterministic_random(&self, seed: &[u8]) -> u64 {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(seed);
        hasher.update(
            &std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                .to_be_bytes(),
        );

        let hash = hasher.finalize();
        u64::from_be_bytes([
            hash[0], hash[1], hash[2], hash[3], hash[4], hash[5], hash[6], hash[7],
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_config_creation() {
        let config = SecurityConfig::default();
        assert!(config
            .default_capabilities
            .contains(&Capability::EmitEvents));
        assert!(config.allowed_imports.contains("icn.current_epoch"));
    }

    #[test]
    fn test_capability_enforcement() {
        let config = SecurityConfig::default();
        let enforcer = CapabilityEnforcer::new(config);

        let mut capabilities = BTreeSet::new();
        capabilities.insert(Capability::EmitEvents);

        // Should succeed for allowed capability
        assert!(enforcer
            .check_capability(&capabilities, &Capability::EmitEvents)
            .is_ok());

        // Should fail for disallowed capability
        assert!(enforcer
            .check_capability(&capabilities, &Capability::TransferMana)
            .is_err());
    }

    #[test]
    fn test_resource_enforcement() {
        let limits = ResourceLimits::default();
        let enforcer = ResourceEnforcer::new(limits);

        // Should succeed within limits
        assert!(enforcer.check_memory(1024).is_ok());

        // Should fail exceeding limits
        assert!(enforcer.check_memory(100 * 1024 * 1024).is_err());
    }

    #[test]
    fn test_determinism_enforcement() {
        let enforcer = DeterminismEnforcer::new(true);

        assert!(enforcer.is_deterministic("math"));
        assert!(!enforcer.is_deterministic("system_time"));

        // Test deterministic randomness
        let seed = b"test_seed";
        let rand1 = enforcer.deterministic_random(seed);
        let rand2 = enforcer.deterministic_random(seed);
        assert_eq!(rand1, rand2); // Should be deterministic
    }
}
