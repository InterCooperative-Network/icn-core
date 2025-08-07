//! Economic metering and mana consumption for CCL contracts

use crate::{CclRuntimeError, WasmCode};
use icn_common::Did;
use icn_economics::{ResourceLedger, TokenClass, FileResourceLedger};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

/// Mana amount type
pub type Mana = u64;

/// Economic metering configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeteringConfig {
    // Base costs
    pub deploy_base_cost: Mana,
    pub call_base_cost: Mana,
    pub storage_cost_per_byte: f64,
    pub compute_cost_per_mgas: Mana,
    
    // Gas conversion rates
    pub instructions_per_gas: u64,
    pub gas_per_mana: u64,
    
    // Economic parameters
    pub min_balance_for_deployment: Mana,
    pub refund_percentage: f64, // Percentage of unused mana to refund
}

impl Default for MeteringConfig {
    fn default() -> Self {
        Self {
            deploy_base_cost: 1000,
            call_base_cost: 10,
            storage_cost_per_byte: 0.001,
            compute_cost_per_mgas: 1,
            instructions_per_gas: 1000,
            gas_per_mana: 1,
            min_balance_for_deployment: 5000,
            refund_percentage: 0.8, // 80% refund for unused mana
        }
    }
}

/// Mana metering system for contract execution
pub struct ManaMetering {
    config: MeteringConfig,
    resource_ledger: Arc<RwLock<dyn ResourceLedger + Send + Sync>>,
    mana_token_class: TokenClass,
    usage_stats: Arc<RwLock<HashMap<Did, UsageStats>>>,
}

/// Usage statistics for economic analysis
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UsageStats {
    pub total_contracts_deployed: u64,
    pub total_function_calls: u64,
    pub total_mana_consumed: Mana,
    pub total_mana_refunded: Mana,
    pub avg_execution_cost: f64,
    pub last_activity: u64, // Timestamp
}

impl ManaMetering {
    /// Create a new mana metering system
    pub fn new() -> Self {
        Self::with_config(MeteringConfig::default())
    }
    
    /// Create with custom configuration
    pub fn with_config(config: MeteringConfig) -> Self {
        // Create mana token class with anti-speculation rules
        let mana_token_class = TokenClass {
            id: "mana".to_string(),
            name: "Computational Mana".to_string(),
            description: "Resource tokens for contract execution".to_string(),
            total_supply: 0, // Unlimited supply, minted as needed
            anti_speculation_rules: Some(icn_economics::AntiSpeculationRules {
                demurrage_rate: 0.0, // No demurrage for mana
                velocity_limit: None, // No velocity limits
                purpose_locks: vec!["computation".to_string()], // Only for computation
                grace_period_epochs: 0,
            }),
        };
        
        Self {
            config,
            resource_ledger: Arc::new(RwLock::new(FileResourceLedger::new())),
            mana_token_class,
            usage_stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Calculate deployment cost for a contract
    pub fn calculate_deploy_cost(&self, code: &WasmCode) -> Mana {
        let base_cost = self.config.deploy_base_cost;
        let storage_cost = (code.len() as f64 * self.config.storage_cost_per_byte) as Mana;
        
        // Add complexity cost based on WASM size and structure
        let complexity_cost = self.calculate_complexity_cost(code);
        
        base_cost + storage_cost + complexity_cost
    }
    
    /// Calculate function call cost
    pub fn calculate_call_cost(
        &self,
        function: &str,
        args: &[u8],
        estimated_compute: u64,
    ) -> Mana {
        let base_cost = self.config.call_base_cost;
        let arg_cost = (args.len() as f64 * 0.0001) as Mana;
        let compute_cost = (estimated_compute / 1_000_000) * self.config.compute_cost_per_mgas;
        
        // Add function-specific costs
        let function_cost = match function {
            "transfer" | "mint" | "burn" => 50, // Economic operations cost more
            "vote" | "propose" => 100,          // Governance operations
            _ => 0,                             // Default functions
        };
        
        base_cost + arg_cost + compute_cost + function_cost
    }
    
    /// Check if a DID has sufficient mana balance
    pub fn check_mana_balance(&self, did: &Did, required: Mana) -> Result<(), CclRuntimeError> {
        let ledger = self.resource_ledger.read();
        let balance = ledger.get_token_balance(did, &self.mana_token_class.id).unwrap_or(0);
        
        if balance < required {
            Err(CclRuntimeError::InsufficientMana {
                required,
                available: balance,
            })
        } else {
            Ok(())
        }
    }
    
    /// Charge mana for an operation
    pub fn charge_mana(&self, did: &Did, amount: Mana) -> Result<(), CclRuntimeError> {
        let mut ledger = self.resource_ledger.write();
        
        // Check balance first
        let current_balance = ledger.get_token_balance(did, &self.mana_token_class.id).unwrap_or(0);
        if current_balance < amount {
            return Err(CclRuntimeError::InsufficientMana {
                required: amount,
                available: current_balance,
            });
        }
        
        // Burn mana tokens (consumption)
        ledger.burn_tokens(did, &self.mana_token_class.id, amount)
            .map_err(|e| CclRuntimeError::ExecutionError(format!("Failed to charge mana: {}", e)))?;
        
        // Update usage statistics
        self.update_usage_stats(did, amount, 0);
        
        Ok(())
    }
    
    /// Refund unused mana
    pub fn refund_mana(&self, did: &Did, amount: Mana) -> Result<(), CclRuntimeError> {
        if amount == 0 {
            return Ok(());
        }
        
        let refund_amount = (amount as f64 * self.config.refund_percentage) as Mana;
        
        if refund_amount > 0 {
            let mut ledger = self.resource_ledger.write();
            
            // Mint refund tokens
            ledger.mint_tokens(did, &self.mana_token_class.id, refund_amount)
                .map_err(|e| CclRuntimeError::ExecutionError(format!("Failed to refund mana: {}", e)))?;
            
            // Update usage statistics
            self.update_usage_stats(did, 0, refund_amount);
        }
        
        Ok(())
    }
    
    /// Get mana balance for a DID
    pub fn get_mana_balance(&self, did: &Did) -> Result<Mana, CclRuntimeError> {
        let ledger = self.resource_ledger.read();
        Ok(ledger.get_token_balance(did, &self.mana_token_class.id).unwrap_or(0))
    }
    
    /// Mint initial mana for a new member
    pub fn mint_initial_mana(&self, did: &Did, amount: Mana) -> Result<(), CclRuntimeError> {
        let mut ledger = self.resource_ledger.write();
        
        ledger.mint_tokens(did, &self.mana_token_class.id, amount)
            .map_err(|e| CclRuntimeError::ExecutionError(format!("Failed to mint initial mana: {}", e)))?;
        
        Ok(())
    }
    
    /// Calculate complexity cost based on WASM analysis
    fn calculate_complexity_cost(&self, code: &WasmCode) -> Mana {
        // Basic complexity metrics
        let size_factor = (code.len() / 1024).max(1) as Mana; // Cost per KB
        
        // TODO: More sophisticated analysis:
        // - Number of functions
        // - Import count
        // - Loop complexity
        // - Memory usage patterns
        
        size_factor * 10 // Base complexity multiplier
    }
    
    /// Update usage statistics for a DID
    fn update_usage_stats(&self, did: &Did, consumed: Mana, refunded: Mana) {
        let mut stats = self.usage_stats.write();
        let user_stats = stats.entry(did.clone()).or_default();
        
        user_stats.total_mana_consumed += consumed;
        user_stats.total_mana_refunded += refunded;
        user_stats.last_activity = icn_common::current_timestamp();
        
        // Update average execution cost
        if user_stats.total_function_calls > 0 {
            user_stats.avg_execution_cost = 
                user_stats.total_mana_consumed as f64 / user_stats.total_function_calls as f64;
        }
    }
    
    /// Get usage statistics for a DID
    pub fn get_usage_stats(&self, did: &Did) -> Option<UsageStats> {
        self.usage_stats.read().get(did).cloned()
    }
    
    /// Get aggregate usage statistics
    pub fn get_aggregate_stats(&self) -> AggregateStats {
        let stats = self.usage_stats.read();
        let mut aggregate = AggregateStats::default();
        
        for user_stats in stats.values() {
            aggregate.total_users += 1;
            aggregate.total_mana_consumed += user_stats.total_mana_consumed;
            aggregate.total_function_calls += user_stats.total_function_calls;
            aggregate.total_contracts_deployed += user_stats.total_contracts_deployed;
        }
        
        if aggregate.total_function_calls > 0 {
            aggregate.avg_cost_per_call = 
                aggregate.total_mana_consumed as f64 / aggregate.total_function_calls as f64;
        }
        
        aggregate
    }
}

/// Aggregate usage statistics across all users
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AggregateStats {
    pub total_users: u64,
    pub total_mana_consumed: Mana,
    pub total_function_calls: u64,
    pub total_contracts_deployed: u64,
    pub avg_cost_per_call: f64,
}

/// Economic enforcement mechanisms
pub struct EconomicEnforcement {
    metering: Arc<ManaMetering>,
    config: MeteringConfig,
}

impl EconomicEnforcement {
    pub fn new(metering: Arc<ManaMetering>) -> Self {
        Self {
            config: metering.config.clone(),
            metering,
        }
    }
    
    /// Enforce minimum balance for contract deployment
    pub fn enforce_deploy_balance(&self, deployer: &Did) -> Result<(), CclRuntimeError> {
        let balance = self.metering.get_mana_balance(deployer)?;
        
        if balance < self.config.min_balance_for_deployment {
            return Err(CclRuntimeError::InsufficientMana {
                required: self.config.min_balance_for_deployment,
                available: balance,
            });
        }
        
        Ok(())
    }
    
    /// Calculate storage cost for data
    pub fn calculate_storage_cost(&self, bytes: u64) -> Mana {
        (bytes as f64 * self.config.storage_cost_per_byte) as Mana
    }
    
    /// Charge for storage usage
    pub fn charge_storage(&self, did: &Did, bytes: u64) -> Result<(), CclRuntimeError> {
        let cost = self.calculate_storage_cost(bytes);
        self.metering.charge_mana(did, cost)
    }
    
    /// Calculate compute cost based on gas consumption
    pub fn calculate_compute_cost(&self, gas_used: u64) -> Mana {
        let mgas = gas_used / 1_000_000;
        mgas * self.config.compute_cost_per_mgas
    }
    
    /// Charge for compute usage
    pub fn charge_compute(&self, did: &Did, gas_used: u64) -> Result<(), CclRuntimeError> {
        let cost = self.calculate_compute_cost(gas_used);
        self.metering.charge_mana(did, cost)
    }
    
    /// Apply economic penalties for violations
    pub fn apply_penalty(&self, did: &Did, violation_type: &str) -> Result<(), CclRuntimeError> {
        let penalty_amount = match violation_type {
            "resource_abuse" => 1000,
            "spam_calls" => 500,
            "invalid_data" => 100,
            _ => 50, // Default penalty
        };
        
        self.metering.charge_mana(did, penalty_amount)
    }
    
    /// Provide economic incentives for good behavior
    pub fn provide_incentive(&self, did: &Did, incentive_type: &str) -> Result<(), CclRuntimeError> {
        let reward_amount = match incentive_type {
            "efficient_contract" => 100,
            "community_contribution" => 500,
            "bug_report" => 200,
            _ => 50, // Default reward
        };
        
        self.metering.mint_initial_mana(did, reward_amount)
    }
}

/// Mana market mechanisms for price discovery
pub struct ManaMarket {
    metering: Arc<ManaMetering>,
    base_price: f64, // Base price in terms of other tokens
    demand_factor: f64,
    supply_factor: f64,
}

impl ManaMarket {
    pub fn new(metering: Arc<ManaMetering>) -> Self {
        Self {
            metering,
            base_price: 1.0,
            demand_factor: 1.0,
            supply_factor: 1.0,
        }
    }
    
    /// Calculate current mana price based on supply and demand
    pub fn calculate_price(&self) -> f64 {
        self.base_price * self.demand_factor / self.supply_factor
    }
    
    /// Update price factors based on network usage
    pub fn update_price_factors(&mut self) {
        let stats = self.metering.get_aggregate_stats();
        
        // Increase demand factor based on usage
        if stats.total_function_calls > 1000 {
            self.demand_factor = (stats.total_function_calls as f64 / 1000.0).sqrt();
        }
        
        // Supply factor could be adjusted based on governance decisions
        // or automatic monetary policy
    }
    
    /// Convert between mana and other token types
    pub fn convert_tokens(
        &self,
        from_token: &str,
        to_token: &str,
        amount: u64,
    ) -> Result<u64, CclRuntimeError> {
        // TODO: Implement actual token conversion
        // This would integrate with the economics protocol
        // to handle exchange rates between different token classes
        
        match (from_token, to_token) {
            ("credits", "mana") => Ok(amount * 10), // Example rate
            ("mana", "credits") => Ok(amount / 10),
            _ => Err(CclRuntimeError::ExecutionError(
                format!("Unsupported conversion: {} to {}", from_token, to_token)
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::Did;
    
    #[test]
    fn test_metering_creation() {
        let metering = ManaMetering::new();
        assert_eq!(metering.config.deploy_base_cost, 1000);
    }
    
    #[test]
    fn test_deploy_cost_calculation() {
        let metering = ManaMetering::new();
        let code = vec![0u8; 1000]; // 1KB of code
        let cost = metering.calculate_deploy_cost(&code);
        
        // Should include base cost + storage cost + complexity cost
        assert!(cost > 1000); // More than just base cost
    }
    
    #[test]
    fn test_call_cost_calculation() {
        let metering = ManaMetering::new();
        let cost = metering.calculate_call_cost("transfer", &vec![0u8; 100], 1_000_000);
        
        // Transfer function should cost more than default
        assert!(cost > 10); // More than base cost
    }
    
    #[tokio::test]
    async fn test_mana_operations() {
        let metering = ManaMetering::new();
        let did = Did::new("key", "test_user");
        
        // Mint initial mana
        assert!(metering.mint_initial_mana(&did, 1000).is_ok());
        
        // Check balance
        let balance = metering.get_mana_balance(&did).unwrap();
        assert_eq!(balance, 1000);
        
        // Charge mana
        assert!(metering.charge_mana(&did, 100).is_ok());
        
        // Check updated balance
        let new_balance = metering.get_mana_balance(&did).unwrap();
        assert_eq!(new_balance, 900);
        
        // Refund some mana
        assert!(metering.refund_mana(&did, 50).is_ok());
        
        // Check final balance (should include refund percentage)
        let final_balance = metering.get_mana_balance(&did).unwrap();
        assert!(final_balance > 900); // Should be more due to refund
    }
    
    #[test]
    fn test_insufficient_mana() {
        let metering = ManaMetering::new();
        let did = Did::new("key", "poor_user");
        
        // Try to charge mana without sufficient balance
        let result = metering.charge_mana(&did, 100);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            CclRuntimeError::InsufficientMana { required, available } => {
                assert_eq!(required, 100);
                assert_eq!(available, 0);
            }
            _ => panic!("Expected InsufficientMana error"),
        }
    }
    
    #[test]
    fn test_economic_enforcement() {
        let metering = Arc::new(ManaMetering::new());
        let enforcement = EconomicEnforcement::new(metering.clone());
        let did = Did::new("key", "test_user");
        
        // Test deployment balance enforcement
        let result = enforcement.enforce_deploy_balance(&did);
        assert!(result.is_err()); // Should fail without sufficient balance
        
        // Mint sufficient mana
        metering.mint_initial_mana(&did, 10000).unwrap();
        
        // Should now pass
        let result = enforcement.enforce_deploy_balance(&did);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_mana_market() {
        let metering = Arc::new(ManaMetering::new());
        let market = ManaMarket::new(metering);
        
        let price = market.calculate_price();
        assert_eq!(price, 1.0); // Initial base price
        
        // Test token conversion
        let converted = market.convert_tokens("credits", "mana", 100);
        assert!(converted.is_ok());
        assert_eq!(converted.unwrap(), 1000); // 10:1 conversion rate
    }
}