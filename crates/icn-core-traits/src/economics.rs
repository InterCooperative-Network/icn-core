//! Economic service traits and types

use crate::CoreTraitsError;
use async_trait::async_trait;
use icn_common::Did;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Economic provider trait
#[async_trait]
pub trait EconomicProvider: Send + Sync {
    /// Get economic health metrics
    async fn get_economic_health(&self) -> Result<EconomicHealth, CoreTraitsError>;
    
    /// Calculate pricing for a resource or service
    async fn calculate_pricing(
        &self,
        resource_type: &str,
        usage_amount: u64,
        context: HashMap<String, String>,
    ) -> Result<u64, CoreTraitsError>;
    
    /// Process economic transaction
    async fn process_transaction(
        &self,
        from: &Did,
        to: &Did,
        amount: u64,
        transaction_type: TransactionType,
    ) -> Result<String, CoreTraitsError>; // Returns transaction ID
    
    /// Get economic policy for a resource
    async fn get_economic_policy(&self, resource_type: &str) -> Result<EconomicPolicy, CoreTraitsError>;
}

/// Mana provider trait for mana-based economics
#[async_trait]
pub trait ManaProvider: Send + Sync {
    /// Get mana balance for a DID
    async fn get_mana_balance(&self, did: &Did) -> Result<u64, CoreTraitsError>;
    
    /// Spend mana for a DID
    async fn spend_mana(&self, did: &Did, amount: u64) -> Result<u64, CoreTraitsError>; // Returns remaining balance
    
    /// Add mana to a DID (e.g., from regeneration)
    async fn add_mana(&self, did: &Did, amount: u64) -> Result<u64, CoreTraitsError>; // Returns new balance
    
    /// Get mana regeneration rate for a DID
    async fn get_regeneration_rate(&self, did: &Did) -> Result<u64, CoreTraitsError>; // Mana per time unit
    
    /// Check if DID has sufficient mana for operation
    async fn has_sufficient_mana(&self, did: &Did, required: u64) -> Result<bool, CoreTraitsError>;
    
    /// Get mana transaction history
    async fn get_mana_history(&self, did: &Did, limit: usize) -> Result<Vec<ManaTransaction>, CoreTraitsError>;
}

/// Resource provider trait for resource management
#[async_trait]
pub trait ResourceProvider: Send + Sync {
    /// Allocate resources for a request
    async fn allocate_resources(
        &self,
        requester: &Did,
        resource_spec: ResourceSpec,
    ) -> Result<ResourceAllocation, CoreTraitsError>;
    
    /// Release allocated resources
    async fn release_resources(&self, allocation_id: &str) -> Result<(), CoreTraitsError>;
    
    /// Get available resources
    async fn get_available_resources(&self) -> Result<HashMap<String, u64>, CoreTraitsError>;
    
    /// Get resource utilization metrics
    async fn get_resource_utilization(&self) -> Result<ResourceUtilization, CoreTraitsError>;
    
    /// Check resource availability
    async fn check_resource_availability(&self, resource_spec: &ResourceSpec) -> Result<bool, CoreTraitsError>;
}

/// Economic health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicHealth {
    pub total_mana_in_circulation: u64,
    pub mana_velocity: f64,
    pub resource_utilization_rate: f64,
    pub transaction_volume: u64,
    pub economic_stability_index: f64,
}

/// Transaction types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    ManaTransfer,
    ResourcePayment,
    ReputationIncentive,
    GovernanceReward,
    SystemReward,
}

/// Economic policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicPolicy {
    pub resource_type: String,
    pub base_price: u64,
    pub pricing_model: PricingModel,
    pub regeneration_rate: u64,
    pub max_allocation: u64,
}

/// Pricing models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PricingModel {
    Fixed,
    Dynamic { demand_factor: f64 },
    AuctionBased,
    ReputationWeighted,
}

/// Mana transaction record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManaTransaction {
    pub transaction_id: String,
    pub did: Did,
    pub amount: i64, // Positive for additions, negative for spending
    pub transaction_type: TransactionType,
    pub timestamp: u64,
    pub context: HashMap<String, String>,
}

/// Resource specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSpec {
    pub resource_type: String,
    pub amount: u64,
    pub duration: Option<u64>, // Duration in seconds
    pub priority: ResourcePriority,
    pub requirements: HashMap<String, String>,
}

/// Resource priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourcePriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Resource allocation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    pub allocation_id: String,
    pub allocated_resources: HashMap<String, u64>,
    pub allocation_time: u64,
    pub expiry_time: Option<u64>,
    pub cost: u64,
}

/// Resource utilization metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUtilization {
    pub cpu_utilization: f64,
    pub memory_utilization: f64,
    pub storage_utilization: f64,
    pub network_utilization: f64,
    pub custom_resources: HashMap<String, f64>,
}