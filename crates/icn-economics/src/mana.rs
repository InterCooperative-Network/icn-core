//! Mana System Implementation
//! 
//! This module implements the regenerative mana system as described in Economic_Incentive_Protocol.md section 3.
//! Mana represents potential computational work and serves as an anti-spam mechanism and fairness enforcer.

use crate::ManaLedger;
use icn_common::{CommonError, Did};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Maximum mana capacity constants
pub const BASE_MANA_CAP: u64 = 10_000;
pub const MIN_MANA_BALANCE: u64 = 10;
pub const REGEN_EPOCH_SECONDS: u64 = 3600; // 1 hour

/// Default emergency modulation factor during network attacks
pub const EMERGENCY_MODULATION_FACTOR: f64 = 0.25;

/// Organization type weights (κ_org)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum OrganizationType {
    /// Worker cooperatives - economic production baseline
    Cooperative,
    /// Civic infrastructure with high governance multipliers
    Community,
    /// Coordination premium for cross-org bridges
    Federation,
    /// Global baseline with bootstrap advantages
    DefaultIcnFederation,
    /// Limited until organizational affiliation
    Unaffiliated,
}

impl OrganizationType {
    /// Get the organizational weight (κ_org) for mana regeneration
    pub fn weight(&self) -> f64 {
        match self {
            OrganizationType::Cooperative => 1.00,
            OrganizationType::Community => 0.95,
            OrganizationType::Federation => 1.25,
            OrganizationType::DefaultIcnFederation => 1.10,
            OrganizationType::Unaffiliated => 0.70,
        }
    }
}

/// Hardware resource metrics for compute score calculation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HardwareMetrics {
    /// Number of CPU cores available
    pub cpu_cores: u32,
    /// Available memory in MB
    pub memory_mb: u32,
    /// Available storage in GB
    pub storage_gb: u32,
    /// Network bandwidth in Mbps
    pub bandwidth_mbps: u32,
    /// GPU compute units (0 if no GPU)
    pub gpu_units: u32,
    /// Uptime percentage (0.0 to 1.0)
    pub uptime_percentage: f64,
    /// Job success rate (0.0 to 1.0)
    pub job_success_rate: f64,
}

impl Default for HardwareMetrics {
    fn default() -> Self {
        HardwareMetrics {
            cpu_cores: 4,
            memory_mb: 8192,
            storage_gb: 500,
            bandwidth_mbps: 100,
            gpu_units: 0,
            uptime_percentage: 0.95,
            job_success_rate: 0.90,
        }
    }
}

/// Individual mana account state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ManaAccount {
    /// Current mana balance
    pub balance: u64,
    /// Maximum mana capacity
    pub max_capacity: u64,
    /// Last regeneration timestamp
    pub last_regen: u64,
    /// Hardware metrics for compute score
    pub hardware_metrics: HardwareMetrics,
    /// Organization type
    pub organization_type: OrganizationType,
    /// Trust multiplier β (0.5-2.0)
    pub trust_multiplier: f64,
    /// Recent participation factor η (0.25-1.5)
    pub participation_factor: f64,
    /// Governance engagement γ (0.5-1.5)
    pub governance_engagement: f64,
    /// Federation bonus (0.0-0.5)
    pub federation_bonus: f64,
}

impl Default for ManaAccount {
    fn default() -> Self {
        ManaAccount {
            balance: BASE_MANA_CAP / 4, // Start with 25% capacity
            max_capacity: BASE_MANA_CAP,
            last_regen: 0,
            hardware_metrics: HardwareMetrics::default(),
            organization_type: OrganizationType::Unaffiliated,
            trust_multiplier: 1.0,
            participation_factor: 1.0,
            governance_engagement: 1.0,
            federation_bonus: 0.0,
        }
    }
}

/// Trait for providing hardware metrics for compute score calculation
pub trait HardwareMetricsProvider: Send + Sync {
    /// Get current hardware metrics for the given DID
    fn get_hardware_metrics(&self, did: &Did) -> Result<HardwareMetrics, CommonError>;
}

/// Trait for providing organizational information
pub trait OrganizationProvider: Send + Sync {
    /// Get organization type for the given DID
    fn get_organization_type(&self, did: &Did) -> Result<OrganizationType, CommonError>;
    
    /// Get federation bonus for the given DID
    fn get_federation_bonus(&self, did: &Did) -> Result<f64, CommonError>;
}

/// Trait for providing trust and participation scores
pub trait TrustProvider: Send + Sync {
    /// Get trust multiplier β (0.5-2.0)
    fn get_trust_multiplier(&self, did: &Did) -> Result<f64, CommonError>;
    
    /// Get recent participation factor η (0.25-1.5)
    fn get_participation_factor(&self, did: &Did) -> Result<f64, CommonError>;
    
    /// Get governance engagement γ (0.5-1.5)
    fn get_governance_engagement(&self, did: &Did) -> Result<f64, CommonError>;
}

/// Trait for detecting network emergencies
pub trait EmergencyDetector: Send + Sync {
    /// Detect if the network is under attack or experiencing instability
    fn is_emergency(&self) -> bool;
    
    /// Get the current emergency modulation factor
    fn emergency_factor(&self) -> f64 {
        if self.is_emergency() {
            EMERGENCY_MODULATION_FACTOR
        } else {
            1.0
        }
    }
}

/// Network health factor provider
pub trait NetworkHealthProvider: Send + Sync {
    /// Get current network health factor (typically 0.5-1.5)
    fn network_health_factor(&self) -> f64;
}

/// Comprehensive mana ledger with regeneration capabilities
pub struct RegenerativeManaLedger<T, H, O, TR, E, N> {
    /// Underlying mana ledger
    ledger: T,
    /// Hardware metrics provider
    hardware_provider: H,
    /// Organization information provider
    organization_provider: O,
    /// Trust and participation provider
    trust_provider: TR,
    /// Emergency detector
    emergency_detector: E,
    /// Network health provider
    network_health_provider: N,
    /// Cache of mana accounts
    accounts: std::sync::RwLock<HashMap<Did, ManaAccount>>,
    /// Network average for compute score normalization
    network_average: std::sync::RwLock<HardwareMetrics>,
}

impl<T, H, O, TR, E, N> RegenerativeManaLedger<T, H, O, TR, E, N>
where
    T: ManaLedger,
    H: HardwareMetricsProvider,
    O: OrganizationProvider,
    TR: TrustProvider,
    E: EmergencyDetector,
    N: NetworkHealthProvider,
{
    /// Create a new regenerative mana ledger
    pub fn new(
        ledger: T,
        hardware_provider: H,
        organization_provider: O,
        trust_provider: TR,
        emergency_detector: E,
        network_health_provider: N,
    ) -> Self {
        RegenerativeManaLedger {
            ledger,
            hardware_provider,
            organization_provider,
            trust_provider,
            emergency_detector,
            network_health_provider,
            accounts: std::sync::RwLock::new(HashMap::new()),
            network_average: std::sync::RwLock::new(HardwareMetrics::default()),
        }
    }

    /// Get access to the hardware provider for testing
    pub fn hardware_provider(&mut self) -> &mut H {
        &mut self.hardware_provider
    }

    /// Get access to the organization provider for testing
    pub fn organization_provider(&mut self) -> &mut O {
        &mut self.organization_provider
    }

    /// Get access to the trust provider for testing
    pub fn trust_provider(&mut self) -> &mut TR {
        &mut self.trust_provider
    }

    /// Get access to the emergency detector for testing
    pub fn emergency_detector(&mut self) -> &mut E {
        &mut self.emergency_detector
    }

    /// Get access to the network health provider for testing
    pub fn network_health_provider(&mut self) -> &mut N {
        &mut self.network_health_provider
    }

    /// Get current timestamp (using system time for now)
    fn current_time(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Calculate compute score (σ) normalized 0-1
    pub fn calculate_compute_score(&self, metrics: &HardwareMetrics, network_avg: &HardwareMetrics) -> f64 {
        let weighted_sum = 
            (metrics.cpu_cores as f64) * 0.25 +
            (metrics.memory_mb as f64 / 1024.0) * 0.20 +  // Convert MB to GB
            (metrics.storage_gb as f64) * 0.15 +
            (metrics.bandwidth_mbps as f64) * 0.15 +
            (metrics.gpu_units as f64) * 0.10 +
            metrics.uptime_percentage * 0.10 +
            metrics.job_success_rate * 0.05;

        let network_weighted_sum = 
            (network_avg.cpu_cores as f64) * 0.25 +
            (network_avg.memory_mb as f64 / 1024.0) * 0.20 +
            (network_avg.storage_gb as f64) * 0.15 +
            (network_avg.bandwidth_mbps as f64) * 0.15 +
            (network_avg.gpu_units as f64) * 0.10 +
            network_avg.uptime_percentage * 0.10 +
            network_avg.job_success_rate * 0.05;

        if network_weighted_sum > 0.0 {
            (weighted_sum / network_weighted_sum).min(2.0).max(0.1)
        } else {
            1.0 // Default to 1.0 if network average is invalid
        }
    }

    /// Calculate regeneration rate: R(t) = κ_org × σ × β × η × network_health_factor
    pub fn calculate_regeneration_rate(&self, account: &ManaAccount) -> f64 {
        let network_avg = self.network_average.read().unwrap();
        let sigma = self.calculate_compute_score(&account.hardware_metrics, &network_avg);
        let kappa_org = account.organization_type.weight();
        let beta = account.trust_multiplier;
        let eta = account.participation_factor;
        let network_health = self.network_health_provider.network_health_factor();
        let emergency_factor = self.emergency_detector.emergency_factor();

        kappa_org * sigma * beta * eta * network_health * emergency_factor
    }

    /// Calculate maximum mana capacity
    pub fn calculate_max_capacity(&self, account: &ManaAccount) -> u64 {
        let network_avg = self.network_average.read().unwrap();
        let sigma = self.calculate_compute_score(&account.hardware_metrics, &network_avg);
        let kappa_org = account.organization_type.weight();
        let gamma = account.governance_engagement;
        let federation_bonus = account.federation_bonus;

        let capacity = (BASE_MANA_CAP as f64) * kappa_org * sigma * gamma * (1.0 + federation_bonus);
        capacity.round() as u64
    }

    /// Get or create a mana account for the given DID
    fn get_account(&self, did: &Did) -> Result<ManaAccount, CommonError> {
        {
            let accounts = self.accounts.read().unwrap();
            if let Some(account) = accounts.get(did) {
                return Ok(account.clone());
            }
        }

        // Create new account
        let mut account = ManaAccount::default();
        
        // Fetch current metrics
        if let Ok(hardware) = self.hardware_provider.get_hardware_metrics(did) {
            account.hardware_metrics = hardware;
        }
        
        if let Ok(org_type) = self.organization_provider.get_organization_type(did) {
            account.organization_type = org_type;
        }
        
        if let Ok(trust) = self.trust_provider.get_trust_multiplier(did) {
            account.trust_multiplier = trust.max(0.5).min(2.0);
        }
        
        if let Ok(participation) = self.trust_provider.get_participation_factor(did) {
            account.participation_factor = participation.max(0.25).min(1.5);
        }
        
        if let Ok(governance) = self.trust_provider.get_governance_engagement(did) {
            account.governance_engagement = governance.max(0.5).min(1.5);
        }
        
        if let Ok(bonus) = self.organization_provider.get_federation_bonus(did) {
            account.federation_bonus = bonus.max(0.0).min(0.5);
        }

        account.max_capacity = self.calculate_max_capacity(&account);
        account.last_regen = self.current_time();

        {
            let mut accounts = self.accounts.write().unwrap();
            accounts.insert(did.clone(), account.clone());
        }

        Ok(account)
    }

    /// Update account in cache
    fn update_account(&self, did: &Did, account: ManaAccount) {
        let mut accounts = self.accounts.write().unwrap();
        accounts.insert(did.clone(), account);
    }

    /// Regenerate mana for a given DID based on time elapsed
    pub fn regenerate_mana(&self, did: &Did) -> Result<u64, CommonError> {
        let mut account = self.get_account(did)?;
        let current_time = self.current_time();
        
        if current_time <= account.last_regen {
            return Ok(account.balance); // No time has passed
        }

        let time_elapsed = current_time - account.last_regen;
        let regeneration_rate = self.calculate_regeneration_rate(&account);
        
        // Calculate mana to regenerate based on time elapsed
        let mana_per_hour = regeneration_rate * 100.0; // Base regeneration rate
        let hours_elapsed = time_elapsed as f64 / REGEN_EPOCH_SECONDS as f64;
        let mana_to_add = (mana_per_hour * hours_elapsed).round() as u64;

        // Apply regeneration, capping at max capacity
        let new_balance = (account.balance + mana_to_add).min(account.max_capacity);
        
        // Update account
        account.balance = new_balance;
        account.last_regen = current_time;
        self.update_account(did, account);

        // Update underlying ledger
        self.ledger.set_balance(did, new_balance)?;

        Ok(new_balance)
    }

    /// Regenerate mana for all known accounts
    pub fn regenerate_all_mana(&self) -> Result<Vec<(Did, u64)>, CommonError> {
        let dids: Vec<Did> = {
            let accounts = self.accounts.read().unwrap();
            accounts.keys().cloned().collect()
        };

        let mut results = Vec::new();
        for did in dids {
            match self.regenerate_mana(&did) {
                Ok(balance) => results.push((did, balance)),
                Err(_) => continue, // Skip failed regenerations
            }
        }

        Ok(results)
    }

    /// Update network average metrics for compute score normalization
    pub fn update_network_average(&self, average: HardwareMetrics) {
        let mut network_avg = self.network_average.write().unwrap();
        *network_avg = average;
    }

    /// Get account information for a DID
    pub fn get_account_info(&self, did: &Did) -> Result<ManaAccount, CommonError> {
        self.get_account(did)
    }

    /// Update account metrics (called periodically to refresh from providers)
    pub fn update_account_metrics(&self, did: &Did) -> Result<(), CommonError> {
        let mut account = self.get_account(did)?;
        
        // Update from providers
        if let Ok(hardware) = self.hardware_provider.get_hardware_metrics(did) {
            account.hardware_metrics = hardware;
        }
        
        if let Ok(org_type) = self.organization_provider.get_organization_type(did) {
            account.organization_type = org_type;
        }
        
        if let Ok(trust) = self.trust_provider.get_trust_multiplier(did) {
            account.trust_multiplier = trust.max(0.5).min(2.0);
        }
        
        if let Ok(participation) = self.trust_provider.get_participation_factor(did) {
            account.participation_factor = participation.max(0.25).min(1.5);
        }
        
        if let Ok(governance) = self.trust_provider.get_governance_engagement(did) {
            account.governance_engagement = governance.max(0.5).min(1.5);
        }
        
        if let Ok(bonus) = self.organization_provider.get_federation_bonus(did) {
            account.federation_bonus = bonus.max(0.0).min(0.5);
        }

        // Recalculate max capacity
        account.max_capacity = self.calculate_max_capacity(&account);
        
        // Cap current balance to new max capacity
        if account.balance > account.max_capacity {
            account.balance = account.max_capacity;
            self.ledger.set_balance(did, account.balance)?;
        }

        self.update_account(did, account);
        Ok(())
    }
}

// Implement ManaLedger trait for RegenerativeManaLedger
impl<T, H, O, TR, E, N> ManaLedger for RegenerativeManaLedger<T, H, O, TR, E, N>
where
    T: ManaLedger,
    H: HardwareMetricsProvider,
    O: OrganizationProvider,
    TR: TrustProvider,
    E: EmergencyDetector,
    N: NetworkHealthProvider,
{
    fn get_balance(&self, did: &Did) -> u64 {
        // Trigger regeneration before returning balance
        if let Ok(balance) = self.regenerate_mana(did) {
            balance
        } else {
            self.ledger.get_balance(did)
        }
    }

    fn set_balance(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
        let account = self.get_account(did)?;
        if amount > account.max_capacity {
            return Err(CommonError::InvalidInputError(
                format!("Amount {} exceeds max capacity {}", amount, account.max_capacity)
            ));
        }

        let mut updated_account = account;
        updated_account.balance = amount;
        self.update_account(did, updated_account);
        
        self.ledger.set_balance(did, amount)
    }

    fn spend(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
        // Regenerate mana first
        let current_balance = self.regenerate_mana(did)?;
        
        if current_balance < amount {
            return Err(CommonError::InsufficientFunds(format!(
                "Insufficient mana: {} < {}", current_balance, amount
            )));
        }

        let new_balance = current_balance - amount;
        self.set_balance(did, new_balance)?;
        
        Ok(())
    }

    fn credit(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
        let account = self.get_account(did)?;
        let current_balance = self.get_balance(did); // This triggers regeneration
        let new_balance = (current_balance + amount).min(account.max_capacity);
        
        self.set_balance(did, new_balance)?;
        
        Ok(())
    }

    fn credit_all(&self, amount: u64) -> Result<(), CommonError> {
        let dids: Vec<Did> = {
            let accounts = self.accounts.read().unwrap();
            accounts.keys().cloned().collect()
        };

        for did in dids {
            let _ = self.credit(&did, amount); // Continue on errors
        }

        Ok(())
    }

    fn all_accounts(&self) -> Vec<Did> {
        let accounts = self.accounts.read().unwrap();
        accounts.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::{FixedTimeProvider, FixedSystemInfoProvider, Did};

    // Mock implementations for testing
    
    struct MockManaLedger {
        balances: std::sync::RwLock<HashMap<Did, u64>>,
    }

    impl Clone for MockManaLedger {
        fn clone(&self) -> Self {
            let balances = self.balances.read().unwrap().clone();
            Self {
                balances: std::sync::RwLock::new(balances),
            }
        }
    }
    
    impl MockManaLedger {
        fn new() -> Self {
            MockManaLedger {
                balances: std::sync::RwLock::new(HashMap::new()),
            }
        }
    }
    
    impl ManaLedger for MockManaLedger {
        fn get_balance(&self, did: &Did) -> u64 {
            let balances = self.balances.read().unwrap();
            balances.get(did).cloned().unwrap_or(0)
        }

        fn set_balance(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
            let mut balances = self.balances.write().unwrap();
            balances.insert(did.clone(), amount);
            Ok(())
        }

        fn spend(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
            let current = self.get_balance(did);
            if current < amount {
                return Err(CommonError::InsufficientFunds(format!(
                    "Insufficient balance: {} < {}", current, amount
                )));
            }
            self.set_balance(did, current - amount)
        }

        fn credit(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
            let current = self.get_balance(did);
            self.set_balance(did, current + amount)
        }

        fn all_accounts(&self) -> Vec<Did> {
            let balances = self.balances.read().unwrap();
            balances.keys().cloned().collect()
        }
    }

    #[derive(Clone)]
    struct MockHardwareProvider;
    
    impl HardwareMetricsProvider for MockHardwareProvider {
        fn get_hardware_metrics(&self, _did: &Did) -> Result<HardwareMetrics, CommonError> {
            Ok(HardwareMetrics::default())
        }
    }
    
    #[derive(Clone)]
    struct MockOrganizationProvider;
    
    impl OrganizationProvider for MockOrganizationProvider {
        fn get_organization_type(&self, _did: &Did) -> Result<OrganizationType, CommonError> {
            Ok(OrganizationType::Cooperative)
        }
        
        fn get_federation_bonus(&self, _did: &Did) -> Result<f64, CommonError> {
            Ok(0.1)
        }
    }
    
    #[derive(Clone)]
    struct MockTrustProvider;
    
    impl TrustProvider for MockTrustProvider {
        fn get_trust_multiplier(&self, _did: &Did) -> Result<f64, CommonError> {
            Ok(1.5)
        }
        
        fn get_participation_factor(&self, _did: &Did) -> Result<f64, CommonError> {
            Ok(1.2)
        }
        
        fn get_governance_engagement(&self, _did: &Did) -> Result<f64, CommonError> {
            Ok(1.3)
        }
    }
    
    #[derive(Clone)]
    struct MockEmergencyDetector;
    
    impl EmergencyDetector for MockEmergencyDetector {
        fn is_emergency(&self) -> bool {
            false
        }
    }
    
    #[derive(Clone)]
    struct MockNetworkHealthProvider;
    
    impl NetworkHealthProvider for MockNetworkHealthProvider {
        fn network_health_factor(&self) -> f64 {
            1.0
        }
    }

    #[test]
    fn test_organization_weights() {
        assert_eq!(OrganizationType::Cooperative.weight(), 1.00);
        assert_eq!(OrganizationType::Community.weight(), 0.95);
        assert_eq!(OrganizationType::Federation.weight(), 1.25);
        assert_eq!(OrganizationType::DefaultIcnFederation.weight(), 1.10);
        assert_eq!(OrganizationType::Unaffiliated.weight(), 0.70);
    }

    #[test]
    fn test_compute_score_calculation() {
        let ledger = MockManaLedger::new();
        let regen_ledger = RegenerativeManaLedger::new(
            ledger,
            MockHardwareProvider,
            MockOrganizationProvider,
            MockTrustProvider,
            MockEmergencyDetector,
            MockNetworkHealthProvider,
        );

        let metrics = HardwareMetrics {
            cpu_cores: 8,
            memory_mb: 16384,
            storage_gb: 1000,
            bandwidth_mbps: 200,
            gpu_units: 2,
            uptime_percentage: 0.98,
            job_success_rate: 0.95,
        };

        let network_avg = HardwareMetrics::default();
        let score = regen_ledger.calculate_compute_score(&metrics, &network_avg);
        
        // Should be > 1.0 since metrics are better than average
        assert!(score > 1.0);
        assert!(score <= 2.0); // Capped at 2.0
    }

    #[test]
    fn test_mana_regeneration() {
        let ledger = MockManaLedger::new();
        let regen_ledger = RegenerativeManaLedger::new(
            ledger,
            MockHardwareProvider,
            MockOrganizationProvider,
            MockTrustProvider,
            MockEmergencyDetector,
            MockNetworkHealthProvider,
        );

        let did = Did::new("test", "alice");
        
        // Get initial balance (should create account)
        let initial_balance = regen_ledger.get_balance(&did);
        assert_eq!(initial_balance, BASE_MANA_CAP / 4); // Default 25% capacity

        // Test spending
        regen_ledger.spend(&did, 100).unwrap();
        assert_eq!(regen_ledger.get_balance(&did), initial_balance - 100);

        // Test credit
        regen_ledger.credit(&did, 50).unwrap();
        assert_eq!(regen_ledger.get_balance(&did), initial_balance - 50);
    }

    #[test]
    fn test_organization_weights() {
        let weights = [
            (OrganizationType::Cooperative, 1.00),
            (OrganizationType::Community, 0.95),
            (OrganizationType::Federation, 1.25),
            (OrganizationType::DefaultIcnFederation, 1.10),
            (OrganizationType::Unaffiliated, 0.70),
        ];

        for (org_type, expected) in weights.iter() {
            let actual = org_type.weight();
            assert!((actual - expected).abs() < 0.01, 
                "Organization {:?} weight should be {}, got {}", 
                org_type, expected, actual);
        }
    }

    #[test]
    fn test_compute_score_calculation() {
        let metrics = HardwareMetrics {
            cpu_cores: 16,
            memory_mb: 32000,
            storage_gb: 1000,
            bandwidth_mbps: 1000,
            gpu_units: 2,
            uptime_percentage: 0.95,
            job_success_rate: 0.90,
        };

        let network_avg = HardwareMetrics {
            cpu_cores: 8,
            memory_mb: 16000,
            storage_gb: 500,
            bandwidth_mbps: 500,
            gpu_units: 1,
            uptime_percentage: 0.85,
            job_success_rate: 0.80,
        };

        let ledger = MockManaLedger::new();
        let regen_ledger = RegenerativeManaLedger::new(
            ledger,
            MockHardwareProvider,
            MockOrganizationProvider,
            MockTrustProvider,
            MockEmergencyDetector,
            MockNetworkHealthProvider,
        );

        let score = regen_ledger.calculate_compute_score(&metrics, &network_avg);
        
        // Score should be greater than 1.0 since all metrics are above average
        assert!(score > 1.0, "Compute score should be > 1.0 for above-average hardware, got {}", score);
        
        // Test edge case - metrics equal to network average should give score of 1.0
        let avg_score = regen_ledger.calculate_compute_score(&network_avg, &network_avg);
        assert!((avg_score - 1.0).abs() < 0.01, "Average hardware should give score ~1.0, got {}", avg_score);
    }

    #[test]
    fn test_regeneration_rate_calculation() {
        let hardware_metrics = HardwareMetrics {
            cpu_cores: 16,
            memory_mb: 32000,
            storage_gb: 1000,
            bandwidth_mbps: 1000,
            gpu_units: 2,
            uptime_percentage: 0.95,
            job_success_rate: 0.90,
        };

        let network_avg = HardwareMetrics {
            cpu_cores: 8,
            memory_mb: 16000,
            storage_gb: 500,
            bandwidth_mbps: 500,
            gpu_units: 1,
            uptime_percentage: 0.85,
            job_success_rate: 0.80,
        };

        // Test cooperative organization with good metrics
        let rate = calculate_regeneration_rate(
            &OrganizationType::Cooperative,
            &hardware_metrics,
            &network_avg,
            1.5, // trust multiplier
            1.3, // participation factor  
            1.2, // governance engagement
            1.0, // network health
            false, // not emergency
        );

        assert!(rate > 0.0, "Regeneration rate should be positive");
        
        // Test emergency conditions reduce rate
        let emergency_rate = calculate_regeneration_rate(
            &OrganizationType::Cooperative,
            &hardware_metrics,
            &network_avg,
            1.5,
            1.3,
            1.2,
            1.0,
            true, // emergency
        );

        assert!(emergency_rate < rate, "Emergency should reduce regeneration rate");
        assert!((emergency_rate / rate - EMERGENCY_MODULATION_FACTOR).abs() < 0.01, 
            "Emergency rate should be {} of normal rate", EMERGENCY_MODULATION_FACTOR);
    }
}
