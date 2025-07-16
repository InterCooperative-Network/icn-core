//! Cooperative Registry for Federation Discovery
//!
//! This module provides a registry service for discovering and managing
//! cooperative profiles within the ICN federation.

use crate::cooperative_schemas::{CooperativeProfile, CooperativeType, TrustRelationship};
use icn_common::{Cid, CommonError, DagBlock, Did, SystemTimeProvider, TimeProvider};
use icn_dag::StorageService;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Registry for discovering cooperatives in the federation
pub struct CooperativeRegistry {
    dag_store: Arc<std::sync::Mutex<dyn StorageService<icn_common::DagBlock>>>,
    /// Local cache of cooperative profiles for fast lookup
    profile_cache: dashmap::DashMap<Did, CooperativeProfile>,
    /// Index of cooperatives by capability type
    capability_index: dashmap::DashMap<String, Vec<Did>>,
    /// Index of cooperatives by geographic location
    location_index: dashmap::DashMap<String, Vec<Did>>,
    /// Trust relationships between cooperatives
    trust_relationships: dashmap::DashMap<Did, Vec<TrustRelationship>>,
}

/// Search filters for cooperative discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CooperativeSearchFilter {
    /// Filter by cooperative type
    pub cooperative_type: Option<CooperativeType>,
    /// Filter by required capabilities
    pub required_capabilities: Vec<String>,
    /// Filter by geographic scope
    pub country: Option<String>,
    pub region: Option<String>,
    pub locality: Option<String>,
    /// Filter by minimum trust level from a specific cooperative
    pub trusted_by: Option<Did>,
    /// Filter by federation membership
    pub federation: Option<String>,
    /// Search query for name/description
    pub search_query: Option<String>,
    /// Maximum number of results
    pub limit: Option<usize>,
}

impl Default for CooperativeSearchFilter {
    fn default() -> Self {
        Self {
            cooperative_type: None,
            required_capabilities: Vec::new(),
            country: None,
            region: None,
            locality: None,
            trusted_by: None,
            federation: None,
            search_query: None,
            limit: Some(50),
        }
    }
}

/// Result of a cooperative search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CooperativeSearchResult {
    /// The cooperative profile
    pub profile: CooperativeProfile,
    /// Relevance score (0.0 to 1.0)
    pub relevance_score: f64,
    /// Trust level if filtered by trusted_by
    pub trust_level: Option<crate::cooperative_schemas::TrustLevel>,
    /// Matching capabilities
    pub matching_capabilities: Vec<String>,
}

impl CooperativeRegistry {
    /// Create a new cooperative registry
    pub fn new(dag_store: Arc<std::sync::Mutex<dyn StorageService<icn_common::DagBlock>>>) -> Self {
        Self {
            dag_store,
            profile_cache: dashmap::DashMap::new(),
            capability_index: dashmap::DashMap::new(),
            location_index: dashmap::DashMap::new(),
            trust_relationships: dashmap::DashMap::new(),
        }
    }

    /// Compute a deterministic CID for a cooperative profile based on DID
    fn compute_profile_cid(&self, did: &Did) -> Cid {
        let did_data = did.to_string().as_bytes().to_vec();
        icn_common::compute_merkle_cid(
            0x71, // CBOR codec
            &did_data,
            &[],
            0, // Use 0 for deterministic generation
            did,
            &None,
            &None,
        )
    }

    /// Register a new cooperative profile in the registry
    pub async fn register_cooperative(
        &self,
        profile: CooperativeProfile,
    ) -> Result<Cid, CommonError> {
        // Serialize the profile
        let profile_data = serde_json::to_vec(&profile)
            .map_err(|e| CommonError::SerializationError(e.to_string()))?;

        // Create a DAG block with deterministic CID
        let timestamp = SystemTimeProvider.unix_seconds();
        let cid = self.compute_profile_cid(&profile.did);
        
        let block = DagBlock {
            cid: cid.clone(),
            data: profile_data,
            links: vec![],
            timestamp,
            author_did: profile.did.clone(),
            signature: None,
            scope: None,
        };
        
        // Store in DAG
        let mut store = self.dag_store.lock().map_err(|_| CommonError::InternalError("Mutex poisoned".to_string()))?;
        store.put(&block)?;

        // Update local cache
        self.profile_cache.insert(profile.did.clone(), profile.clone());

        // Update capability index
        for capability in &profile.capabilities {
            let mut entry = self
                .capability_index
                .entry(capability.capability_type.clone())
                .or_insert_with(Vec::new);
            if !entry.contains(&profile.did) {
                entry.push(profile.did.clone());
            }
        }

        // Update location index
        let location_keys = self.generate_location_keys(&profile);
        for key in location_keys {
            let mut entry = self.location_index.entry(key).or_insert_with(Vec::new);
            if !entry.contains(&profile.did) {
                entry.push(profile.did.clone());
            }
        }

        Ok(cid)
    }

    /// Get a cooperative profile by DID
    pub async fn get_cooperative(&self, did: &Did) -> Result<Option<CooperativeProfile>, CommonError> {
        // Check cache first
        if let Some(profile) = self.profile_cache.get(did) {
            return Ok(Some(profile.clone()));
        }

        // Try to load from DAG using deterministic CID based on DID
        let cid = self.compute_profile_cid(did);
        let store = self.dag_store.lock().map_err(|_| CommonError::InternalError("Mutex poisoned".to_string()))?;
        
        if let Some(block) = store.get(&cid)? {
            if let Ok(profile) = serde_json::from_slice::<CooperativeProfile>(&block.data) {
                if profile.did == *did {
                    // Update cache
                    drop(store); // Release the lock before updating cache
                    self.profile_cache.insert(did.clone(), profile.clone());
                    return Ok(Some(profile));
                }
            }
        }

        Ok(None)
    }

    /// Search for cooperatives matching the given criteria
    pub async fn search_cooperatives(
        &self,
        filter: CooperativeSearchFilter,
    ) -> Result<Vec<CooperativeSearchResult>, CommonError> {
        let mut candidates = Vec::new();
        let mut processed_dids = std::collections::HashSet::new();

        // If specific capabilities are required, start with capability index
        if !filter.required_capabilities.is_empty() {
            for capability in &filter.required_capabilities {
                if let Some(capability_entry) = self.capability_index.get(capability) {
                    for did in capability_entry.iter() {
                        if !processed_dids.contains(did) {
                            candidates.push(did.clone());
                            processed_dids.insert(did.clone());
                        }
                    }
                }
            }
        } else {
            // If no specific capabilities, collect all cached profiles
            for entry in self.profile_cache.iter() {
                candidates.push(entry.key().clone());
            }
        }

        // Apply filters and calculate relevance scores
        let mut results = Vec::new();
        
        for did in candidates {
            if let Ok(Some(profile)) = self.get_cooperative(&did).await {
                if let Some(result) = self.evaluate_cooperative(&profile, &filter).await? {
                    results.push(result);
                }
            }
        }

        // Sort by relevance score (highest first)
        results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());

        // Apply limit
        if let Some(limit) = filter.limit {
            results.truncate(limit);
        }

        Ok(results)
    }

    /// Add a trust relationship between cooperatives
    pub async fn add_trust_relationship(
        &self,
        relationship: TrustRelationship,
    ) -> Result<Cid, CommonError> {
        // Store the trust relationship in DAG
        let trust_data = serde_json::to_vec(&relationship)
            .map_err(|e| CommonError::SerializationError(e.to_string()))?;

        // Create a DAG block for the trust relationship
        let timestamp = SystemTimeProvider.unix_seconds();
        let cid = icn_common::compute_merkle_cid(
            0x71, // CBOR codec
            &trust_data,
            &[],
            timestamp,
            &relationship.attestor,
            &None,
            &None,
        );
        
        let block = DagBlock {
            cid: cid.clone(),
            data: trust_data,
            links: vec![],
            timestamp,
            author_did: relationship.attestor.clone(),
            signature: None,
            scope: None,
        };
        
        // Store in DAG
        let mut store = self.dag_store.lock().map_err(|_| CommonError::InternalError("Mutex poisoned".to_string()))?;
        store.put(&block)?;

        // Update local trust index
        let mut entry = self
            .trust_relationships
            .entry(relationship.attestor.clone())
            .or_insert_with(Vec::new);
        entry.push(relationship);

        Ok(cid)
    }

    /// Get trust relationships for a specific cooperative
    pub fn get_trust_relationships(&self, did: &Did) -> Vec<TrustRelationship> {
        self.trust_relationships
            .get(did)
            .map(|entry| entry.clone())
            .unwrap_or_default()
    }

    /// Check trust level between two cooperatives
    pub fn get_trust_level(
        &self,
        attestor: &Did,
        subject: &Did,
        scope: &str,
    ) -> Option<crate::cooperative_schemas::TrustLevel> {
        if let Some(relationships) = self.trust_relationships.get(attestor) {
            for relationship in relationships.iter() {
                if relationship.subject == *subject
                    && relationship.is_valid()
                    && relationship.covers_scope(scope)
                {
                    return Some(relationship.trust_level.clone());
                }
            }
        }
        None
    }

    /// List all cooperatives offering a specific capability
    pub async fn find_capability_providers(
        &self,
        capability_type: &str,
    ) -> Result<Vec<CooperativeProfile>, CommonError> {
        let mut providers = Vec::new();

        if let Some(capability_entry) = self.capability_index.get(capability_type) {
            for did in capability_entry.iter() {
                if let Ok(Some(profile)) = self.get_cooperative(did).await {
                    if profile.has_capability(capability_type) {
                        providers.push(profile);
                    }
                }
            }
        }

        Ok(providers)
    }

    /// Get statistics about the cooperative registry
    pub fn get_registry_stats(&self) -> RegistryStats {
        let total_cooperatives = self.profile_cache.len();
        let total_capabilities = self.capability_index.len();
        let total_trust_relationships = self
            .trust_relationships
            .iter()
            .map(|entry| entry.len())
            .sum();

        // Count cooperatives by type
        let mut by_type = HashMap::new();
        for entry in self.profile_cache.iter() {
            let count = by_type
                .entry(entry.cooperative_type.clone())
                .or_insert(0);
            *count += 1;
        }

        RegistryStats {
            total_cooperatives,
            total_capabilities,
            total_trust_relationships,
            cooperatives_by_type: by_type,
        }
    }

    /// Load cooperatives from DAG storage into cache (initialization)
    pub async fn load_from_dag(&self) -> Result<usize, CommonError> {
        let mut loaded_count = 0;

        // Scan all blocks and try to parse as cooperative profiles
        let store = self.dag_store.lock().map_err(|_| CommonError::InternalError("Mutex poisoned".to_string()))?;
        let all_blocks = store.list_blocks()?;
        drop(store); // Release the lock before processing
        
        for block in all_blocks {
            if let Ok(profile) = serde_json::from_slice::<CooperativeProfile>(&block.data) {
                // Verify this is actually a cooperative profile
                self.profile_cache.insert(profile.did.clone(), profile.clone());
                
                // Update indices
                for capability in &profile.capabilities {
                    let mut entry = self
                        .capability_index
                        .entry(capability.capability_type.clone())
                        .or_insert_with(Vec::new);
                    if !entry.contains(&profile.did) {
                        entry.push(profile.did.clone());
                    }
                }

                let location_keys = self.generate_location_keys(&profile);
                for key in location_keys {
                    let mut entry = self.location_index.entry(key).or_insert_with(Vec::new);
                    if !entry.contains(&profile.did) {
                        entry.push(profile.did.clone());
                    }
                }

                loaded_count += 1;
            }
        }

        Ok(loaded_count)
    }

    // Private helper methods

    fn generate_location_keys(&self, profile: &CooperativeProfile) -> Vec<String> {
        let mut keys = Vec::new();
        
        if profile.geographic_scope.global {
            keys.push("global".to_string());
        }
        
        if let Some(ref country) = profile.geographic_scope.country {
            keys.push(format!("country:{}", country));
        }
        
        if let Some(ref region) = profile.geographic_scope.region {
            keys.push(format!("region:{}", region));
        }
        
        if let Some(ref locality) = profile.geographic_scope.locality {
            keys.push(format!("locality:{}", locality));
        }
        
        keys
    }

    async fn evaluate_cooperative(
        &self,
        profile: &CooperativeProfile,
        filter: &CooperativeSearchFilter,
    ) -> Result<Option<CooperativeSearchResult>, CommonError> {
        let mut score = 1.0;
        let mut matching_capabilities = Vec::new();

        // Filter by cooperative type
        if let Some(ref required_type) = filter.cooperative_type {
            if profile.cooperative_type != *required_type {
                return Ok(None);
            }
            score += 0.2;
        }

        // Filter by capabilities
        if !filter.required_capabilities.is_empty() {
            let mut capability_matches = 0;
            for required_cap in &filter.required_capabilities {
                if profile.has_capability(required_cap) {
                    capability_matches += 1;
                    matching_capabilities.push(required_cap.clone());
                }
            }
            
            if capability_matches == 0 {
                return Ok(None);
            }
            
            // Boost score based on capability match ratio
            let match_ratio = capability_matches as f64 / filter.required_capabilities.len() as f64;
            score += match_ratio * 0.5;
        }

        // Filter by location
        let location_match = self.check_location_match(profile, filter);
        if !location_match.matches {
            // If specific location is required but doesn't match, exclude
            if filter.country.is_some() || filter.region.is_some() || filter.locality.is_some() {
                return Ok(None);
            }
        } else if location_match.exact {
            score += 0.3;
        }

        // Filter by federation membership
        if let Some(ref federation) = filter.federation {
            if !profile.federation_memberships.contains(federation) {
                return Ok(None);
            }
            score += 0.1;
        }

        // Filter by search query
        if let Some(ref query) = filter.search_query {
            let query_lower = query.to_lowercase();
            let name_match = profile.name.to_lowercase().contains(&query_lower);
            let desc_match = profile.description.to_lowercase().contains(&query_lower);
            
            if !name_match && !desc_match {
                return Ok(None);
            }
            
            if name_match {
                score += 0.2;
            }
            if desc_match {
                score += 0.1;
            }
        }

        // Check trust level
        let trust_level = if let Some(ref trusted_by) = filter.trusted_by {
            self.get_trust_level(trusted_by, &profile.did, "*")
        } else {
            None
        };

        // Normalize score to 0.0-1.0 range
        score = (score / 2.0).min(1.0);

        Ok(Some(CooperativeSearchResult {
            profile: profile.clone(),
            relevance_score: score,
            trust_level,
            matching_capabilities,
        }))
    }

    fn check_location_match(
        &self,
        profile: &CooperativeProfile,
        filter: &CooperativeSearchFilter,
    ) -> LocationMatch {
        let mut matches = true;
        let mut exact = true;

        if profile.geographic_scope.global {
            return LocationMatch {
                matches: true,
                exact: false,
            };
        }

        if let Some(ref filter_country) = filter.country {
            if let Some(ref profile_country) = profile.geographic_scope.country {
                if profile_country != filter_country {
                    matches = false;
                }
            } else {
                exact = false;
            }
        }

        if let Some(ref filter_region) = filter.region {
            if let Some(ref profile_region) = profile.geographic_scope.region {
                if profile_region != filter_region {
                    matches = false;
                }
            } else {
                exact = false;
            }
        }

        if let Some(ref filter_locality) = filter.locality {
            if let Some(ref profile_locality) = profile.geographic_scope.locality {
                if profile_locality != filter_locality {
                    matches = false;
                }
            } else {
                exact = false;
            }
        }

        LocationMatch { matches, exact }
    }
}

struct LocationMatch {
    matches: bool,
    exact: bool,
}

/// Statistics about the cooperative registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStats {
    pub total_cooperatives: usize,
    pub total_capabilities: usize,
    pub total_trust_relationships: usize,
    pub cooperatives_by_type: HashMap<CooperativeType, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cooperative_schemas::{CooperativeCapability, GeographicScope};
    use icn_dag::InMemoryDagStore;

    #[tokio::test]
    async fn test_cooperative_registration_and_search() {
        let dag_store = Arc::new(std::sync::Mutex::new(InMemoryDagStore::new()));
        let registry = CooperativeRegistry::new(dag_store);

        // Create a test cooperative profile
        let did = Did::new("key", "test_coop");
        let mut profile = CooperativeProfile::new(
            did.clone(),
            "Test Worker Cooperative".to_string(),
            CooperativeType::Worker,
            "A worker-owned tech cooperative".to_string(),
        );

        profile.geographic_scope = GeographicScope {
            country: Some("US".to_string()),
            region: Some("California".to_string()),
            locality: Some("San Francisco".to_string()),
            global: false,
        };

        profile.add_capability(CooperativeCapability {
            capability_type: "web_development".to_string(),
            description: "Full-stack web development".to_string(),
            specifications: HashMap::new(),
            available: true,
            exchange_model: "time_bank".to_string(),
        });

        // Register the cooperative
        let cid = registry.register_cooperative(profile.clone()).await.unwrap();
        assert!(!cid.to_string().is_empty());

        // Test search by capability
        let filter = CooperativeSearchFilter {
            required_capabilities: vec!["web_development".to_string()],
            ..Default::default()
        };

        let results = registry.search_cooperatives(filter).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].profile.did, did);
        assert!(results[0].matching_capabilities.contains(&"web_development".to_string()));

        // Test search by location
        let filter = CooperativeSearchFilter {
            country: Some("US".to_string()),
            ..Default::default()
        };

        let results = registry.search_cooperatives(filter).await.unwrap();
        assert_eq!(results.len(), 1);

        // Test search by cooperative type
        let filter = CooperativeSearchFilter {
            cooperative_type: Some(CooperativeType::Worker),
            ..Default::default()
        };

        let results = registry.search_cooperatives(filter).await.unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_trust_relationships() {
        let dag_store = Arc::new(std::sync::Mutex::new(InMemoryDagStore::new()));
        let registry = CooperativeRegistry::new(dag_store);

        let coop_a = Did::new("key", "coop_a");
        let coop_b = Did::new("key", "coop_b");

        let trust = TrustRelationship::new(
            coop_a.clone(),
            coop_b.clone(),
            crate::cooperative_schemas::TrustLevel::Partial,
            vec!["mesh_computing".to_string()],
        );

        let cid = registry.add_trust_relationship(trust).await.unwrap();
        assert!(!cid.to_string().is_empty());

        let trust_level = registry.get_trust_level(&coop_a, &coop_b, "mesh_computing");
        assert_eq!(trust_level, Some(crate::cooperative_schemas::TrustLevel::Partial));

        let trust_level = registry.get_trust_level(&coop_a, &coop_b, "financial");
        assert_eq!(trust_level, None);
    }
} 