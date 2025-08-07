//! Default implementations for mana system providers

use crate::mana::{
    EmergencyDetector, HardwareMetrics, HardwareMetricsProvider, NetworkHealthProvider,
    OrganizationProvider, OrganizationType, TrustProvider,
};
use icn_common::{CommonError, Did, SystemInfoProvider};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

/// Default hardware metrics provider using SystemInfoProvider
pub struct DefaultHardwareMetricsProvider<S: SystemInfoProvider> {
    system_info: S,
    /// Cache of hardware metrics by DID
    metrics_cache: RwLock<HashMap<Did, HardwareMetrics>>,
}

impl<S: SystemInfoProvider> DefaultHardwareMetricsProvider<S> {
    pub fn new(system_info: S) -> Self {
        DefaultHardwareMetricsProvider {
            system_info,
            metrics_cache: RwLock::new(HashMap::new()),
        }
    }

    /// Update hardware metrics for a specific DID
    pub fn update_metrics(&self, did: &Did, metrics: HardwareMetrics) {
        let mut cache = self.metrics_cache.write().unwrap();
        cache.insert(did.clone(), metrics);
    }

    /// Get default metrics from system info
    fn get_default_metrics(&self) -> HardwareMetrics {
        HardwareMetrics {
            cpu_cores: self.system_info.cpu_cores(),
            memory_mb: self.system_info.memory_mb(),
            storage_gb: 500,         // Default assumption
            bandwidth_mbps: 100,     // Default assumption
            gpu_units: 0,            // Default assumption
            uptime_percentage: 0.95, // Default assumption
            job_success_rate: 0.90,  // Default assumption
        }
    }
}

impl<S: SystemInfoProvider> HardwareMetricsProvider for DefaultHardwareMetricsProvider<S> {
    fn get_hardware_metrics(&self, did: &Did) -> Result<HardwareMetrics, CommonError> {
        let cache = self.metrics_cache.read().unwrap();
        if let Some(metrics) = cache.get(did) {
            Ok(metrics.clone())
        } else {
            // Return default metrics based on system info
            Ok(self.get_default_metrics())
        }
    }
}

/// In-memory organization provider
#[derive(Debug)]
pub struct InMemoryOrganizationProvider {
    organizations: RwLock<HashMap<Did, OrganizationType>>,
    federation_bonuses: RwLock<HashMap<Did, f64>>,
}

impl InMemoryOrganizationProvider {
    pub fn new() -> Self {
        InMemoryOrganizationProvider {
            organizations: RwLock::new(HashMap::new()),
            federation_bonuses: RwLock::new(HashMap::new()),
        }
    }

    /// Set organization type for a DID
    pub fn set_organization_type(&self, did: Did, org_type: OrganizationType) {
        let mut orgs = self.organizations.write().unwrap();
        orgs.insert(did, org_type);
    }

    /// Set federation bonus for a DID
    pub fn set_federation_bonus(&self, did: Did, bonus: f64) {
        let mut bonuses = self.federation_bonuses.write().unwrap();
        bonuses.insert(did, bonus.clamp(0.0, 0.5));
    }
}

impl Clone for InMemoryOrganizationProvider {
    fn clone(&self) -> Self {
        let orgs = self.organizations.read().unwrap().clone();
        let bonuses = self.federation_bonuses.read().unwrap().clone();

        InMemoryOrganizationProvider {
            organizations: RwLock::new(orgs),
            federation_bonuses: RwLock::new(bonuses),
        }
    }
}

impl Default for InMemoryOrganizationProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl OrganizationProvider for InMemoryOrganizationProvider {
    fn get_organization_type(&self, did: &Did) -> Result<OrganizationType, CommonError> {
        let orgs = self.organizations.read().unwrap();
        Ok(orgs
            .get(did)
            .cloned()
            .unwrap_or(OrganizationType::Unaffiliated))
    }

    fn get_federation_bonus(&self, did: &Did) -> Result<f64, CommonError> {
        let bonuses = self.federation_bonuses.read().unwrap();
        Ok(bonuses.get(did).cloned().unwrap_or(0.0))
    }
}

/// In-memory trust provider
#[derive(Debug)]
pub struct InMemoryTrustProvider {
    trust_multipliers: RwLock<HashMap<Did, f64>>,
    participation_factors: RwLock<HashMap<Did, f64>>,
    governance_engagement: RwLock<HashMap<Did, f64>>,
}

impl InMemoryTrustProvider {
    pub fn new() -> Self {
        InMemoryTrustProvider {
            trust_multipliers: RwLock::new(HashMap::new()),
            participation_factors: RwLock::new(HashMap::new()),
            governance_engagement: RwLock::new(HashMap::new()),
        }
    }

    /// Set trust multiplier for a DID
    pub fn set_trust_multiplier(&self, did: Did, trust: f64) {
        let mut trust_map = self.trust_multipliers.write().unwrap();
        trust_map.insert(did, trust.clamp(0.5, 2.0));
    }

    /// Set participation factor for a DID
    pub fn set_participation_factor(&self, did: Did, participation: f64) {
        let mut participation_map = self.participation_factors.write().unwrap();
        participation_map.insert(did, participation.clamp(0.25, 1.5));
    }

    /// Set governance engagement for a DID
    pub fn set_governance_engagement(&self, did: Did, engagement: f64) {
        let mut engagement_map = self.governance_engagement.write().unwrap();
        engagement_map.insert(did, engagement.clamp(0.5, 1.5));
    }
}

impl Clone for InMemoryTrustProvider {
    fn clone(&self) -> Self {
        let trust = self.trust_multipliers.read().unwrap().clone();
        let participation = self.participation_factors.read().unwrap().clone();
        let governance = self.governance_engagement.read().unwrap().clone();

        InMemoryTrustProvider {
            trust_multipliers: RwLock::new(trust),
            participation_factors: RwLock::new(participation),
            governance_engagement: RwLock::new(governance),
        }
    }
}

impl Default for InMemoryTrustProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl TrustProvider for InMemoryTrustProvider {
    fn get_trust_multiplier(&self, did: &Did) -> Result<f64, CommonError> {
        let trust_map = self.trust_multipliers.read().unwrap();
        Ok(trust_map.get(did).cloned().unwrap_or(1.0))
    }

    fn get_participation_factor(&self, did: &Did) -> Result<f64, CommonError> {
        let participation_map = self.participation_factors.read().unwrap();
        Ok(participation_map.get(did).cloned().unwrap_or(1.0))
    }

    fn get_governance_engagement(&self, did: &Did) -> Result<f64, CommonError> {
        let engagement_map = self.governance_engagement.read().unwrap();
        Ok(engagement_map.get(did).cloned().unwrap_or(1.0))
    }
}

/// Simple emergency detector based on configurable state
#[derive(Debug)]
pub struct SimpleEmergencyDetector {
    emergency_state: RwLock<bool>,
    emergency_factor: f64,
}

impl SimpleEmergencyDetector {
    pub fn new() -> Self {
        SimpleEmergencyDetector {
            emergency_state: RwLock::new(false),
            emergency_factor: crate::mana::EMERGENCY_MODULATION_FACTOR,
        }
    }

    pub fn with_factor(emergency_factor: f64) -> Self {
        SimpleEmergencyDetector {
            emergency_state: RwLock::new(false),
            emergency_factor,
        }
    }

    /// Set emergency state
    pub fn set_emergency(&self, is_emergency: bool) {
        let mut state = self.emergency_state.write().unwrap();
        *state = is_emergency;
    }
}

impl Clone for SimpleEmergencyDetector {
    fn clone(&self) -> Self {
        let state = *self.emergency_state.read().unwrap();
        SimpleEmergencyDetector {
            emergency_state: RwLock::new(state),
            emergency_factor: self.emergency_factor,
        }
    }
}

impl Default for SimpleEmergencyDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl EmergencyDetector for SimpleEmergencyDetector {
    fn is_emergency(&self) -> bool {
        *self.emergency_state.read().unwrap()
    }

    fn emergency_factor(&self) -> f64 {
        if self.is_emergency() {
            self.emergency_factor
        } else {
            1.0
        }
    }
}

/// Simple network health provider with configurable factor
#[derive(Debug)]
pub struct SimpleNetworkHealthProvider {
    health_factor: RwLock<f64>,
}

impl SimpleNetworkHealthProvider {
    pub fn new(initial_factor: f64) -> Self {
        SimpleNetworkHealthProvider {
            health_factor: RwLock::new(initial_factor.clamp(0.1, 2.0)),
        }
    }

    /// Set network health factor
    pub fn set_health_factor(&self, factor: f64) {
        let mut health = self.health_factor.write().unwrap();
        *health = factor.clamp(0.1, 2.0);
    }
}

impl Clone for SimpleNetworkHealthProvider {
    fn clone(&self) -> Self {
        let factor = *self.health_factor.read().unwrap();
        SimpleNetworkHealthProvider {
            health_factor: RwLock::new(factor),
        }
    }
}

impl Default for SimpleNetworkHealthProvider {
    fn default() -> Self {
        Self::new(1.0)
    }
}

impl NetworkHealthProvider for SimpleNetworkHealthProvider {
    fn network_health_factor(&self) -> f64 {
        *self.health_factor.read().unwrap()
    }
}

/// Composite provider that integrates all mana system providers
pub struct ComprehensiveManaProvider<H, O, T, E, N>
where
    H: HardwareMetricsProvider,
    O: OrganizationProvider,
    T: TrustProvider,
    E: EmergencyDetector,
    N: NetworkHealthProvider,
{
    pub hardware_provider: H,
    pub organization_provider: O,
    pub trust_provider: T,
    pub emergency_detector: E,
    pub network_health_provider: N,
}

impl<H, O, T, E, N> ComprehensiveManaProvider<H, O, T, E, N>
where
    H: HardwareMetricsProvider,
    O: OrganizationProvider,
    T: TrustProvider,
    E: EmergencyDetector,
    N: NetworkHealthProvider,
{
    pub fn new(
        hardware_provider: H,
        organization_provider: O,
        trust_provider: T,
        emergency_detector: E,
        network_health_provider: N,
    ) -> Self {
        ComprehensiveManaProvider {
            hardware_provider,
            organization_provider,
            trust_provider,
            emergency_detector,
            network_health_provider,
        }
    }
}

/// Configuration for mana system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManaSystemConfig {
    /// Base mana capacity
    pub base_mana_cap: u64,
    /// Minimum mana balance required
    pub min_mana_balance: u64,
    /// Regeneration epoch in seconds
    pub regen_epoch_seconds: u64,
    /// Emergency modulation factor
    pub emergency_modulation_factor: f64,
    /// Whether to enable emergency modulation
    pub enable_emergency_modulation: bool,
}

impl Default for ManaSystemConfig {
    fn default() -> Self {
        ManaSystemConfig {
            base_mana_cap: crate::mana::BASE_MANA_CAP,
            min_mana_balance: crate::mana::MIN_MANA_BALANCE,
            regen_epoch_seconds: crate::mana::REGEN_EPOCH_SECONDS,
            emergency_modulation_factor: crate::mana::EMERGENCY_MODULATION_FACTOR,
            enable_emergency_modulation: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::{Did, FixedSystemInfoProvider};

    #[test]
    fn test_default_hardware_provider() {
        let system_info = FixedSystemInfoProvider::new(4, 8192);
        let provider = DefaultHardwareMetricsProvider::new(system_info);

        let did = Did::new("test", "alice");
        let metrics = provider.get_hardware_metrics(&did).unwrap();

        assert_eq!(metrics.cpu_cores, 4);
        assert_eq!(metrics.memory_mb, 8192);
        assert_eq!(metrics.storage_gb, 500); // Default
    }

    #[test]
    fn test_in_memory_organization_provider() {
        let provider = InMemoryOrganizationProvider::new();
        let did = Did::new("test", "alice");

        // Default should be Unaffiliated
        assert_eq!(
            provider.get_organization_type(&did).unwrap(),
            OrganizationType::Unaffiliated
        );

        // Set and retrieve
        provider.set_organization_type(did.clone(), OrganizationType::Cooperative);
        assert_eq!(
            provider.get_organization_type(&did).unwrap(),
            OrganizationType::Cooperative
        );

        // Test federation bonus
        assert_eq!(provider.get_federation_bonus(&did).unwrap(), 0.0);
        provider.set_federation_bonus(did.clone(), 0.3);
        assert_eq!(provider.get_federation_bonus(&did).unwrap(), 0.3);
    }

    #[test]
    fn test_in_memory_trust_provider() {
        let provider = InMemoryTrustProvider::new();
        let did = Did::new("test", "alice");

        // Defaults should be 1.0
        assert_eq!(provider.get_trust_multiplier(&did).unwrap(), 1.0);
        assert_eq!(provider.get_participation_factor(&did).unwrap(), 1.0);
        assert_eq!(provider.get_governance_engagement(&did).unwrap(), 1.0);

        // Set and retrieve with bounds checking
        provider.set_trust_multiplier(did.clone(), 2.5); // Should be capped at 2.0
        assert_eq!(provider.get_trust_multiplier(&did).unwrap(), 2.0);

        provider.set_participation_factor(did.clone(), 0.1); // Should be capped at 0.25
        assert_eq!(provider.get_participation_factor(&did).unwrap(), 0.25);
    }

    #[test]
    fn test_simple_emergency_detector() {
        let detector = SimpleEmergencyDetector::new();

        // Default should be no emergency
        assert!(!detector.is_emergency());
        assert_eq!(detector.emergency_factor(), 1.0);

        // Set emergency
        detector.set_emergency(true);
        assert!(detector.is_emergency());
        assert_eq!(
            detector.emergency_factor(),
            crate::mana::EMERGENCY_MODULATION_FACTOR
        );
    }

    #[test]
    fn test_simple_network_health_provider() {
        let provider = SimpleNetworkHealthProvider::new(1.2);
        assert_eq!(provider.network_health_factor(), 1.2);

        // Test bounds
        provider.set_health_factor(3.0); // Should be capped at 2.0
        assert_eq!(provider.network_health_factor(), 2.0);

        provider.set_health_factor(0.05); // Should be capped at 0.1
        assert_eq!(provider.network_health_factor(), 0.1);
    }
}
