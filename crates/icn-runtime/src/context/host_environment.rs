//! Host environment types for WASM interaction.

use super::errors::HostAbiError;
use super::runtime_context::RuntimeContext;
use std::sync::Arc;

/// Host environment trait for WASM interaction.
pub trait HostEnvironment: Send + Sync + std::fmt::Debug {
    fn env_submit_mesh_job(
        &self,
        ctx: &Arc<RuntimeContext>,
        job_data_ptr: u32,
        job_data_len: u32,
    ) -> Result<u32, HostAbiError>;
    fn env_account_get_mana(
        &self,
        ctx: &Arc<RuntimeContext>,
        account_did_ptr: u32,
        account_did_len: u32,
    ) -> Result<u64, HostAbiError>;
    fn env_account_spend_mana(
        &self,
        ctx: &Arc<RuntimeContext>,
        account_did_ptr: u32,
        account_did_len: u32,
        amount: u64,
    ) -> Result<(), HostAbiError>;
}

/// Concrete implementation of HostEnvironment.
pub struct ConcreteHostEnvironment {
    memory: Vec<u8>,
}

impl std::fmt::Debug for ConcreteHostEnvironment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ConcreteHostEnvironment {{ memory_size: {} }}", self.memory.len())
    }
}

impl ConcreteHostEnvironment {
    pub fn new() -> Self {
        Self {
            memory: Vec::new(),
        }
    }

    pub fn set_memory(&mut self, data: Vec<u8>) {
        self.memory = data;
    }
}

impl HostEnvironment for ConcreteHostEnvironment {
    fn env_submit_mesh_job(
        &self,
        ctx: &Arc<RuntimeContext>,
        job_data_ptr: u32,
        job_data_len: u32,
    ) -> Result<u32, HostAbiError> {
        // Extract job data from memory
        let start = job_data_ptr as usize;
        let end = start + job_data_len as usize;
        
        if end > self.memory.len() {
            return Err(HostAbiError::InvalidParameters(
                "Job data pointer/length exceeds memory bounds".to_string(),
            ));
        }
        
        let job_data = &self.memory[start..end];
        let job_json = std::str::from_utf8(job_data).map_err(|_| {
            HostAbiError::InvalidParameters("Invalid UTF-8 in job data".to_string())
        })?;
        
        // Call the actual host function (this would need to be async in real implementation)
        // For now, return a placeholder job ID
        let _job_result = tokio::runtime::Handle::current()
            .block_on(crate::host_submit_mesh_job(ctx, job_json));
        
        // Return placeholder job ID
        Ok(42)
    }

    fn env_account_get_mana(
        &self,
        ctx: &Arc<RuntimeContext>,
        account_did_ptr: u32,
        account_did_len: u32,
    ) -> Result<u64, HostAbiError> {
        // Extract DID from memory
        let start = account_did_ptr as usize;
        let end = start + account_did_len as usize;
        
        if end > self.memory.len() {
            return Err(HostAbiError::InvalidParameters(
                "Account DID pointer/length exceeds memory bounds".to_string(),
            ));
        }
        
        let did_data = &self.memory[start..end];
        let did_str = std::str::from_utf8(did_data).map_err(|_| {
            HostAbiError::InvalidParameters("Invalid UTF-8 in DID data".to_string())
        })?;
        
        // Call the actual host function
        tokio::runtime::Handle::current()
            .block_on(crate::host_account_get_mana(ctx, did_str))
    }

    fn env_account_spend_mana(
        &self,
        ctx: &Arc<RuntimeContext>,
        account_did_ptr: u32,
        account_did_len: u32,
        amount: u64,
    ) -> Result<(), HostAbiError> {
        // Extract DID from memory
        let start = account_did_ptr as usize;
        let end = start + account_did_len as usize;
        
        if end > self.memory.len() {
            return Err(HostAbiError::InvalidParameters(
                "Account DID pointer/length exceeds memory bounds".to_string(),
            ));
        }
        
        let did_data = &self.memory[start..end];
        let did_str = std::str::from_utf8(did_data).map_err(|_| {
            HostAbiError::InvalidParameters("Invalid UTF-8 in DID data".to_string())
        })?;
        
        // Call the actual host function
        tokio::runtime::Handle::current()
            .block_on(crate::host_account_spend_mana(ctx, did_str, amount))
    }
} 