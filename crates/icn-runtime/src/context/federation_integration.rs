//! Federation management integration for the ICN runtime.
//!
//! This module integrates the FederationManager with the runtime context to provide
//! real federation management capabilities and replace hardcoded federations.

use super::{HostAbiError, MeshNetworkServiceType};
use icn_common::{CommonError, Did, TimeProvider};
use icn_identity::{FederationManager, FederationInfo, FederationMembershipService, MembershipStatus};
use icn_reputation::ReputationStore;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use log::{debug, info, warn, error};

/// Integration layer for federation management within the runtime context
pub struct FederationIntegration {
    /// Core federation manager
    federation_manager: Arc<FederationManager>,
    /// Network service for federation synchronization
    network_service: Arc<MeshNetworkServiceType>,
    /// Current node identity
    node_identity: Did,
    /// Reputation store for federation trust scoring
    reputation_store: Arc<dyn ReputationStore>,
    /// Time provider for federation lifecycle management
    time_provider: Arc<dyn TimeProvider>,
    /// Cache of known federations
    federation_cache: Arc<RwLock<HashMap<String, FederationInfo>>>,
    /// Federation discovery state
    discovery_state: Arc<RwLock<FederationDiscoveryState>>,
}

/// State of federation discovery operations
#[derive(Debug, Default)]
struct FederationDiscoveryState {
    /// Last discovery attempt timestamp
    last_discovery: Option<u64>,
    /// Discovered peer federations
    peer_federations: HashMap<Did, Vec<String>>,
    /// Failed discovery attempts
    failed_attempts: HashMap<String, u64>,
}

impl FederationIntegration {
    /// Create a new federation integration instance
    pub fn new(
        federation_manager: Arc<FederationManager>,
        network_service: Arc<MeshNetworkServiceType>,
        node_identity: Did,
        reputation_store: Arc<dyn ReputationStore>,
        time_provider: Arc<dyn TimeProvider>,
    ) -> Self {
        Self {
            federation_manager,
            network_service,
            node_identity,
            reputation_store,
            time_provider,
            federation_cache: Arc::new(RwLock::new(HashMap::new())),
            discovery_state: Arc::new(RwLock::new(FederationDiscoveryState::default())),
        }
    }

    /// Get all known federations (cached + dynamically discovered)
    pub async fn get_all_federations(&self) -> Result<Vec<FederationInfo>, HostAbiError> {
        let mut federations = Vec::new();

        // Get federations from manager
        let managed_federations = self.federation_manager.list_federations().await
            .map_err(|e| HostAbiError::InternalError(format!("Failed to list managed federations: {}", e)))?;
        federations.extend(managed_federations);

        // Get cached discovered federations
        let cache = self.federation_cache.read().await;
        for federation in cache.values() {
            // Avoid duplicates
            if !federations.iter().any(|f| f.federation_id == federation.federation_id) {
                federations.push(federation.clone());
            }
        }

        Ok(federations)
    }

    /// Discover federations from connected peers
    pub async fn discover_federations(&self) -> Result<Vec<FederationInfo>, HostAbiError> {
        let now = self.time_provider.unix_seconds();
        
        // Check if we've done discovery recently (within last 5 minutes)
        {
            let state = self.discovery_state.read().await;
            if let Some(last_discovery) = state.last_discovery {
                if now - last_discovery < 300 {
                    debug!("Skipping federation discovery - recent attempt at {}", last_discovery);
                    return self.get_cached_federations().await;
                }
            }
        }

        info!("Starting federation discovery from network peers");

        // Update discovery timestamp
        {
            let mut state = self.discovery_state.write().await;
            state.last_discovery = Some(now);
        }

        // Use network service to discover federations
        let discovered = match &*self.network_service {
            MeshNetworkServiceType::Default(service) => {
                service.discover_federations().await
                    .map_err(|e| HostAbiError::NetworkError(format!("Federation discovery failed: {}", e)))?
            }
            MeshNetworkServiceType::Stub(service) => {
                service.discover_federations().await
                    .map_err(|e| HostAbiError::NetworkError(format!("Federation discovery failed: {}", e)))?
            }
        };

        // Update cache with discovered federations
        {
            let mut cache = self.federation_cache.write().await;
            for federation in &discovered {
                cache.insert(federation.federation_id.clone(), federation.clone());
            }
        }

        info!("Discovered {} federations from network", discovered.len());
        Ok(discovered)
    }

    /// Get cached federations without network discovery
    async fn get_cached_federations(&self) -> Result<Vec<FederationInfo>, HostAbiError> {
        let cache = self.federation_cache.read().await;
        Ok(cache.values().cloned().collect())
    }

    /// Join a federation
    pub async fn join_federation(&self, federation_id: &str) -> Result<(), HostAbiError> {
        // First check if we already know about this federation
        let federation_info = {
            let cache = self.federation_cache.read().await;
            cache.get(federation_id).cloned()
        };

        let federation_info = if let Some(info) = federation_info {
            info
        } else {
            // Try to discover the federation
            self.discover_federations().await?;
            let cache = self.federation_cache.read().await;
            cache.get(federation_id).cloned()
                .ok_or_else(|| HostAbiError::InternalError(
                    format!("Federation {} not found after discovery", federation_id)
                ))?
        };

        // Use federation manager to join
        self.federation_manager.request_membership(
            federation_id.to_string(),
            self.node_identity.clone(),
        ).await.map_err(|e| HostAbiError::InternalError(
            format!("Failed to request federation membership: {}", e)
        ))?;

        info!("Successfully requested membership in federation: {}", federation_id);
        Ok(())
    }

    /// Leave a federation
    pub async fn leave_federation(&self, federation_id: &str) -> Result<(), HostAbiError> {
        self.federation_manager.leave_federation(
            federation_id.to_string(),
            self.node_identity.clone(),
        ).await.map_err(|e| HostAbiError::InternalError(
            format!("Failed to leave federation: {}", e)
        ))?;

        info!("Successfully left federation: {}", federation_id);
        Ok(())
    }

    /// Get our membership status in a federation
    pub async fn get_membership_status(&self, federation_id: &str) -> Result<Option<MembershipStatus>, HostAbiError> {
        self.federation_manager.get_membership_status(
            federation_id,
            &self.node_identity,
        ).await.map_err(|e| HostAbiError::InternalError(
            format!("Failed to get membership status: {}", e)
        ))
    }

    /// Get federations we are members of
    pub async fn get_our_federations(&self) -> Result<Vec<FederationInfo>, HostAbiError> {
        let all_federations = self.get_all_federations().await?;
        let mut our_federations = Vec::new();

        for federation in all_federations {
            if let Ok(Some(status)) = self.get_membership_status(&federation.federation_id).await {
                if status == MembershipStatus::Active {
                    our_federations.push(federation);
                }
            }
        }

        Ok(our_federations)
    }

    /// Check if we can execute a job based on federation requirements
    pub async fn check_federation_eligibility(
        &self,
        allowed_federations: &[String],
    ) -> Result<bool, HostAbiError> {
        if allowed_federations.is_empty() {
            return Ok(true); // No federation requirements
        }

        let our_federations = self.get_our_federations().await?;
        let our_federation_ids: Vec<&str> = our_federations
            .iter()
            .map(|f| f.federation_id.as_str())
            .collect();

        // Check if we're in any of the allowed federations
        for allowed in allowed_federations {
            if our_federation_ids.contains(&allowed.as_str()) {
                return Ok(true);
            }
        }

        debug!(
            "Node not eligible for job - not member of any allowed federations: {:?}",
            allowed_federations
        );
        Ok(false)
    }

    /// Get federation-based trust score for a peer
    pub async fn get_federation_trust_score(&self, peer_did: &Did) -> Result<f64, HostAbiError> {
        let our_federations = self.get_our_federations().await?;
        let all_federations = self.get_all_federations().await?;

        let mut trust_score = 0.0;
        let mut shared_federations = 0;

        // Check each federation we're both in
        for our_federation in &our_federations {
            for federation in &all_federations {
                if federation.federation_id == our_federation.federation_id {
                    // Check if peer is also in this federation
                    if let Ok(Some(peer_status)) = self.federation_manager.get_membership_status(
                        &federation.federation_id,
                        peer_did,
                    ).await {
                        if peer_status == MembershipStatus::Active {
                            shared_federations += 1;
                            // Add trust based on federation type and reputation
                            let federation_weight = match federation.federation_type {
                                icn_identity::FederationType::Governance => 2.0,
                                icn_identity::FederationType::Compute => 1.5,
                                icn_identity::FederationType::Economic => 1.8,
                                _ => 1.0,
                            };
                            trust_score += federation_weight;
                        }
                    }
                }
            }
        }

        // Normalize trust score
        if shared_federations > 0 {
            trust_score /= shared_federations as f64;
            // Cap at 1.0 and add base reputation
            trust_score = (trust_score * 0.7).min(1.0);
            let reputation_score = self.reputation_store.get_reputation(peer_did) as f64 / 100.0;
            trust_score += reputation_score * 0.3;
        }

        Ok(trust_score.min(1.0))
    }

    /// Synchronize federation state with the network
    pub async fn sync_federations(&self) -> Result<(), HostAbiError> {
        info!("Starting federation synchronization");

        // Discover new federations
        let discovered = self.discover_federations().await?;
        
        // Update our federation memberships
        for federation in discovered {
            // Check if we should maintain membership
            if let Ok(Some(status)) = self.get_membership_status(&federation.federation_id).await {
                if status == MembershipStatus::Active {
                    debug!("Maintaining membership in federation: {}", federation.federation_id);
                    // Could add periodic membership renewal here
                }
            }
        }

        info!("Federation synchronization completed");
        Ok(())
    }

    /// Get federation capabilities for job matching
    pub async fn get_federation_capabilities(&self) -> Result<Vec<String>, HostAbiError> {
        let our_federations = self.get_our_federations().await?;
        let mut capabilities = Vec::new();

        for federation in our_federations {
            // Add federation-specific capabilities
            match federation.federation_type {
                icn_identity::FederationType::Compute => {
                    capabilities.push("compute_federation".to_string());
                    capabilities.push("distributed_processing".to_string());
                }
                icn_identity::FederationType::Data => {
                    capabilities.push("data_federation".to_string());
                    capabilities.push("data_sharing".to_string());
                }
                icn_identity::FederationType::Governance => {
                    capabilities.push("governance_federation".to_string());
                    capabilities.push("proposal_voting".to_string());
                }
                icn_identity::FederationType::Economic => {
                    capabilities.push("economic_federation".to_string());
                    capabilities.push("resource_trading".to_string());
                }
                _ => {
                    capabilities.push("general_federation".to_string());
                }
            }

            // Add federation ID as capability
            capabilities.push(format!("federation:{}", federation.federation_id));
        }

        capabilities.sort();
        capabilities.dedup();
        Ok(capabilities)
    }
}

/// Federation-aware job selection helper
pub struct FederationAwareJobSelection {
    federation_integration: Arc<FederationIntegration>,
}

impl FederationAwareJobSelection {
    pub fn new(federation_integration: Arc<FederationIntegration>) -> Self {
        Self {
            federation_integration,
        }
    }

    /// Check if we should bid on a job based on federation requirements
    pub async fn should_bid_on_job(
        &self,
        job_allowed_federations: &[String],
        job_required_capabilities: &[String],
    ) -> Result<bool, HostAbiError> {
        // Check federation eligibility
        if !self.federation_integration
            .check_federation_eligibility(job_allowed_federations)
            .await? {
            return Ok(false);
        }

        // Check federation capabilities
        if !job_required_capabilities.is_empty() {
            let our_capabilities = self.federation_integration
                .get_federation_capabilities()
                .await?;

            for required_capability in job_required_capabilities {
                if !our_capabilities.contains(required_capability) {
                    debug!(
                        "Missing required capability: {} (have: {:?})",
                        required_capability, our_capabilities
                    );
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    /// Get enhanced bid information including federation context
    pub async fn get_federation_bid_info(&self) -> Result<FederationBidInfo, HostAbiError> {
        let federations = self.federation_integration.get_our_federations().await?;
        let capabilities = self.federation_integration.get_federation_capabilities().await?;

        Ok(FederationBidInfo {
            federation_ids: federations.iter().map(|f| f.federation_id.clone()).collect(),
            federation_capabilities: capabilities,
            trust_level: self.calculate_trust_level(&federations).await,
        })
    }

    async fn calculate_trust_level(&self, federations: &[FederationInfo]) -> f64 {
        let mut trust_level = 0.0;
        let mut weight_sum = 0.0;

        for federation in federations {
            let weight = match federation.federation_type {
                icn_identity::FederationType::Governance => 3.0,
                icn_identity::FederationType::Economic => 2.5,
                icn_identity::FederationType::Compute => 2.0,
                _ => 1.0,
            };

            trust_level += weight * 0.8; // Base trust for federation membership
            weight_sum += weight;
        }

        if weight_sum > 0.0 {
            trust_level / weight_sum
        } else {
            0.0
        }
    }
}

/// Information about federation context for job bidding
#[derive(Debug, Clone)]
pub struct FederationBidInfo {
    pub federation_ids: Vec<String>,
    pub federation_capabilities: Vec<String>,
    pub trust_level: f64,
}