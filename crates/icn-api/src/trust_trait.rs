//! Trust Graph API trait for querying trust relationships and paths

use async_trait::async_trait;
use icn_common::{CommonError, Did};
use icn_identity::{FederationId, TrustContext, TrustLevel};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Trust relationship information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustRelationshipInfo {
    /// Source DID (who extends trust)
    pub from: Did,
    /// Target DID (who receives trust)
    pub to: Did,
    /// Trust level
    pub trust_level: TrustLevel,
    /// Trust context
    pub context: TrustContext,
    /// Federation this trust belongs to
    pub federation: Option<FederationId>,
    /// When the trust was established
    pub created_at: u64,
    /// When the trust was last updated
    pub updated_at: u64,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Trust path between two entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustPath {
    /// Source DID
    pub from: Did,
    /// Target DID  
    pub to: Did,
    /// Path hops (intermediate DIDs)
    pub path: Vec<Did>,
    /// Effective trust level at destination
    pub effective_trust: TrustLevel,
    /// Trust contexts involved in the path
    pub contexts: HashSet<TrustContext>,
    /// Path length (number of hops)
    pub length: usize,
    /// Path weight/score
    pub weight: f64,
}

/// Trust score for an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustScore {
    /// Entity DID
    pub did: Did,
    /// Overall trust score (0.0-1.0)
    pub score: f64,
    /// Trust scores by context
    pub context_scores: HashMap<TrustContext, f64>,
    /// Number of incoming trust relationships
    pub incoming_trust_count: usize,
    /// Number of outgoing trust relationships
    pub outgoing_trust_count: usize,
    /// Federations this entity belongs to
    pub federations: HashSet<FederationId>,
    /// When scores were last calculated
    pub calculated_at: u64,
}

/// Trust graph statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustGraphStats {
    /// Total number of entities in the trust graph
    pub total_entities: usize,
    /// Total number of trust relationships
    pub total_relationships: usize,
    /// Trust relationships by context
    pub relationships_by_context: HashMap<TrustContext, usize>,
    /// Average trust score
    pub average_trust_score: f64,
    /// Trust score distribution
    pub trust_distribution: HashMap<String, usize>, // "high", "medium", "low", "none"
    /// Number of connected components
    pub connected_components: usize,
    /// Statistics by federation
    pub federation_stats: HashMap<FederationId, FederationTrustStats>,
    /// When stats were last calculated
    pub calculated_at: u64,
}

/// Trust statistics for a specific federation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationTrustStats {
    /// Number of members
    pub member_count: usize,
    /// Average internal trust score
    pub average_internal_trust: f64,
    /// Trust contexts actively used
    pub active_contexts: HashSet<TrustContext>,
    /// Number of cross-federation bridges
    pub bridge_count: usize,
}

/// Trust query filters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustQueryFilter {
    /// Filter by trust context
    pub context: Option<TrustContext>,
    /// Filter by minimum trust level
    pub min_trust_level: Option<TrustLevel>,
    /// Filter by federation
    pub federation: Option<FederationId>,
    /// Filter by date range (created after)
    pub created_after: Option<u64>,
    /// Filter by date range (created before)  
    pub created_before: Option<u64>,
    /// Include inherited trust
    pub include_inherited: bool,
    /// Include cross-federation trust
    pub include_cross_federation: bool,
}

impl Default for TrustQueryFilter {
    fn default() -> Self {
        Self {
            context: None,
            min_trust_level: None,
            federation: None,
            created_after: None,
            created_before: None,
            include_inherited: true,
            include_cross_federation: true,
        }
    }
}

/// Request for finding trust paths
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustPathRequest {
    /// Source DID
    pub from: Did,
    /// Target DID
    pub to: Did,
    /// Trust context to search within
    pub context: TrustContext,
    /// Maximum path length to search
    pub max_length: Option<usize>,
    /// Maximum number of paths to return
    pub max_paths: Option<usize>,
    /// Minimum trust level required
    pub min_trust_level: Option<TrustLevel>,
}

/// Request for updating trust relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustUpdateRequest {
    /// Source DID (who extends trust)
    pub from: Did,
    /// Target DID (who receives trust)
    pub to: Did,
    /// New trust level
    pub trust_level: TrustLevel,
    /// Trust context
    pub context: TrustContext,
    /// Federation context
    pub federation: Option<FederationId>,
    /// Additional metadata
    pub metadata: Option<HashMap<String, String>>,
}

/// Trust update event for real-time notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustUpdateEvent {
    /// Event type
    pub event_type: TrustEventType,
    /// Affected trust relationship
    pub relationship: TrustRelationshipInfo,
    /// Timestamp of the event
    pub timestamp: u64,
    /// Event source/initiator
    pub source: Did,
}

/// Types of trust events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrustEventType {
    /// New trust relationship created
    TrustCreated,
    /// Existing trust relationship updated
    TrustUpdated,
    /// Trust relationship removed
    TrustRemoved,
    /// Trust score recalculated
    ScoreRecalculated,
    /// Trust violation reported
    ViolationReported,
    /// Trust sanction applied
    SanctionApplied,
}

/// API trait for trust graph operations
#[async_trait]
pub trait TrustApi {
    /// Get trust relationship between two entities
    async fn get_trust_relationship(
        &self,
        from: &Did,
        to: &Did,
        context: &TrustContext,
    ) -> Result<Option<TrustRelationshipInfo>, CommonError>;

    /// Get all trust relationships for an entity
    async fn get_entity_trust_relationships(
        &self,
        entity: &Did,
        filter: Option<TrustQueryFilter>,
    ) -> Result<Vec<TrustRelationshipInfo>, CommonError>;

    /// Find trust paths between two entities
    async fn find_trust_paths(
        &self,
        request: TrustPathRequest,
    ) -> Result<Vec<TrustPath>, CommonError>;

    /// Get trust score for an entity
    async fn get_trust_score(&self, entity: &Did) -> Result<TrustScore, CommonError>;

    /// Get trust scores for multiple entities
    async fn get_trust_scores(&self, entities: &[Did]) -> Result<Vec<TrustScore>, CommonError>;

    /// Update trust relationship
    async fn update_trust_relationship(
        &self,
        request: TrustUpdateRequest,
    ) -> Result<(), CommonError>;

    /// Remove trust relationship
    async fn remove_trust_relationship(
        &self,
        from: &Did,
        to: &Did,
        context: &TrustContext,
    ) -> Result<(), CommonError>;

    /// Get trust graph statistics
    async fn get_trust_graph_stats(&self) -> Result<TrustGraphStats, CommonError>;

    /// Get federation trust statistics
    async fn get_federation_trust_stats(
        &self,
        federation: &FederationId,
    ) -> Result<FederationTrustStats, CommonError>;

    /// Subscribe to trust update events
    async fn subscribe_trust_updates(
        &self,
        filter: Option<TrustQueryFilter>,
    ) -> Result<(), CommonError>;

    /// Recalculate trust scores (triggers real-time updates)
    async fn recalculate_trust_scores(&self, entities: Option<Vec<Did>>)
        -> Result<(), CommonError>;

    /// Search entities by trust criteria
    async fn search_by_trust(
        &self,
        filter: TrustQueryFilter,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<TrustScore>, CommonError>;

    /// Validate trust for a specific operation
    async fn validate_trust_operation(
        &self,
        actor: &Did,
        target: &Did,
        context: &TrustContext,
        operation: &str,
    ) -> Result<bool, CommonError>;

    /// Get trust network neighbors for an entity
    async fn get_trust_network_neighbors(
        &self,
        entity: &Did,
        max_distance: Option<usize>,
        min_trust_level: Option<TrustLevel>,
    ) -> Result<Vec<(Did, TrustPath)>, CommonError>;
}
