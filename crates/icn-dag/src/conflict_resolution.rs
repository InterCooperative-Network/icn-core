//! DAG Conflict Resolution Protocol
//!
//! This module implements robust conflict resolution for federation/DAG synchronization,
//! handling fork detection, resolution strategies, and maintaining consistency across nodes.

use crate::{StorageService, DagBlockMetadata, metadata_from_block};
use icn_common::{Cid, CommonError, DagBlock, Did};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

/// Represents a detected conflict in the DAG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagConflict {
    /// Unique identifier for this conflict
    pub conflict_id: String,
    /// The blocks that are in conflict
    pub conflicting_blocks: Vec<Cid>,
    /// The common ancestor where the fork began
    pub fork_point: Option<Cid>,
    /// Timestamp when conflict was detected
    pub detected_at: u64,
    /// Type of conflict
    pub conflict_type: ConflictType,
    /// Nodes that contributed to each side of the conflict
    pub node_positions: HashMap<Did, ConflictPosition>,
    /// Current resolution status
    pub resolution_status: ResolutionStatus,
}

/// Types of DAG conflicts that can occur
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConflictType {
    /// Two blocks claim to be roots simultaneously
    RootConflict,
    /// Different nodes have different views of the same chain position
    ChainFork,
    /// Blocks reference different versions of the same content
    ContentFork,
    /// Circular dependencies detected
    CyclicDependency,
    /// Missing critical blocks create inconsistency
    MissingBlocks,
}

/// Position a node takes in a conflict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictPosition {
    /// Which side of the conflict this node supports
    pub preferred_branch: Cid,
    /// Node's confidence in their position (0.0 - 1.0)
    pub confidence: f64,
    /// Timestamp when node declared this position
    pub declared_at: u64,
    /// Evidence supporting this position
    pub evidence: Vec<ConflictEvidence>,
}

/// Evidence that supports a particular conflict resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictEvidence {
    /// Block has earlier timestamp
    EarlierTimestamp { block_cid: Cid, timestamp: u64 },
    /// Block has higher reputation author
    HigherReputationAuthor { block_cid: Cid, author: Did, reputation: u64 },
    /// Block has more subsequent references
    MoreReferences { block_cid: Cid, reference_count: usize },
    /// Block is part of longer chain
    LongerChain { block_cid: Cid, chain_length: usize },
    /// Block validated by more nodes
    MoreValidations { block_cid: Cid, validator_count: usize },
}

/// Current status of conflict resolution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ResolutionStatus {
    /// Conflict detected but resolution not started
    Detected,
    /// Gathering information from federation nodes
    GatheringEvidence,
    /// Analyzing evidence and determining resolution
    Analyzing,
    /// Resolution determined, propagating decision
    ResolutionFound { winner: Cid },
    /// Resolution complete and applied
    Resolved { winner: Cid, applied_at: u64 },
    /// Resolution failed, manual intervention needed
    Failed { reason: String },
}

/// Configuration for conflict resolution behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolutionConfig {
    /// Maximum time to wait for evidence gathering (seconds)
    pub evidence_timeout: u64,
    /// Minimum nodes that must participate in resolution
    pub min_participants: usize,
    /// Maximum conflicts to track simultaneously
    pub max_concurrent_conflicts: usize,
    /// Enable automatic resolution (vs manual only)
    pub auto_resolve: bool,
    /// Resolution strategy to use
    pub resolution_strategy: ResolutionStrategy,
}

impl Default for ConflictResolutionConfig {
    fn default() -> Self {
        Self {
            evidence_timeout: 300, // 5 minutes
            min_participants: 3,
            max_concurrent_conflicts: 10,
            auto_resolve: true,
            resolution_strategy: ResolutionStrategy::MultiCriteria,
        }
    }
}

/// Strategies for resolving DAG conflicts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResolutionStrategy {
    /// First block wins (by timestamp)
    FirstWins,
    /// Block with highest reputation author wins
    ReputationBased,
    /// Block with most references wins
    PopularityBased,
    /// Longest chain wins
    LongestChain,
    /// Multi-criteria decision (combines multiple factors)
    MultiCriteria,
    /// Federation consensus vote
    FederationVote,
}

/// Manages conflict detection and resolution for DAG synchronization
pub struct ConflictResolver<S: StorageService<DagBlock>> {
    store: S,
    config: ConflictResolutionConfig,
    active_conflicts: HashMap<String, DagConflict>,
    resolution_history: VecDeque<DagConflict>,
    node_identity: Did,
}

impl<S: StorageService<DagBlock>> ConflictResolver<S> {
    /// Create a new conflict resolver
    pub fn new(store: S, config: ConflictResolutionConfig, node_identity: Did) -> Self {
        Self {
            store,
            config,
            active_conflicts: HashMap::new(),
            resolution_history: VecDeque::new(),
            node_identity,
        }
    }

    /// Detect conflicts in the current DAG state
    pub fn detect_conflicts(&mut self) -> Result<Vec<DagConflict>, CommonError> {
        let blocks = self.store.list_blocks()?;
        let mut new_conflicts = Vec::new();

        // Build DAG structure for analysis
        let dag_structure = self.build_dag_structure(&blocks)?;

        // Check for root conflicts
        if let Some(conflict) = self.detect_root_conflicts(&dag_structure)? {
            new_conflicts.push(conflict);
        }

        // Check for chain forks
        new_conflicts.extend(self.detect_chain_forks(&dag_structure)?);

        // Check for cyclic dependencies
        if let Some(conflict) = self.detect_cycles(&dag_structure)? {
            new_conflicts.push(conflict);
        }

        // Check for missing critical blocks
        new_conflicts.extend(self.detect_missing_blocks(&dag_structure)?);

        // Add new conflicts to active tracking
        for conflict in new_conflicts.iter() {
            self.active_conflicts.insert(conflict.conflict_id.clone(), conflict.clone());
        }

        // Limit concurrent conflicts
        self.prune_old_conflicts();

        Ok(new_conflicts)
    }

    /// Resolve a specific conflict using the configured strategy
    pub fn resolve_conflict(&mut self, conflict_id: &str) -> Result<ResolutionStatus, CommonError> {
        // Clone the conflict to avoid borrowing issues
        let conflict = self.active_conflicts.get(conflict_id)
            .ok_or_else(|| CommonError::ResourceNotFound(format!("Conflict {} not found", conflict_id)))?
            .clone();

        if !self.config.auto_resolve {
            return Ok(ResolutionStatus::Failed {
                reason: "Automatic resolution disabled".to_string(),
            });
        }

        // Update status to analyzing
        if let Some(c) = self.active_conflicts.get_mut(conflict_id) {
            c.resolution_status = ResolutionStatus::Analyzing;
        }

        // Apply resolution strategy
        let winner = match self.config.resolution_strategy {
            ResolutionStrategy::FirstWins => self.resolve_by_timestamp(&conflict)?,
            ResolutionStrategy::ReputationBased => self.resolve_by_reputation(&conflict)?,
            ResolutionStrategy::PopularityBased => self.resolve_by_popularity(&conflict)?,
            ResolutionStrategy::LongestChain => self.resolve_by_chain_length(&conflict)?,
            ResolutionStrategy::MultiCriteria => self.resolve_by_multiple_criteria(&conflict)?,
            ResolutionStrategy::FederationVote => {
                return Ok(ResolutionStatus::Failed {
                    reason: "Federation vote not implemented yet".to_string(),
                });
            }
        };

        // Apply the resolution
        self.apply_resolution(conflict_id, &winner)?;

        Ok(ResolutionStatus::Resolved {
            winner,
            applied_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        })
    }

    /// Build a structured representation of the DAG for analysis
    fn build_dag_structure(&self, blocks: &[DagBlock]) -> Result<DagStructure, CommonError> {
        let mut structure = DagStructure::new();

        for block in blocks {
            let metadata = metadata_from_block(block);
            structure.add_block(block.cid.clone(), metadata);

            // Add edges for links
            for link in &block.links {
                structure.add_edge(block.cid.clone(), link.cid.clone());
            }
        }

        Ok(structure)
    }

    /// Detect conflicts where multiple blocks claim to be roots
    fn detect_root_conflicts(&self, dag: &DagStructure) -> Result<Option<DagConflict>, CommonError> {
        let roots = dag.find_roots();
        
        if roots.len() > 1 {
            let conflict_id = format!("root_conflict_{}", 
                SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());

            Ok(Some(DagConflict {
                conflict_id,
                conflicting_blocks: roots,
                fork_point: None,
                detected_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                conflict_type: ConflictType::RootConflict,
                node_positions: HashMap::new(),
                resolution_status: ResolutionStatus::Detected,
            }))
        } else {
            Ok(None)
        }
    }

    /// Detect chain forks where the same logical position has different blocks
    fn detect_chain_forks(&self, dag: &DagStructure) -> Result<Vec<DagConflict>, CommonError> {
        let mut conflicts = Vec::new();
        let mut visited = HashSet::new();

        // Find nodes with multiple children (potential fork points)
        for (block_cid, children) in &dag.children {
            if children.len() > 1 && !visited.contains(block_cid) {
                let conflict_id = format!("fork_{}_{}",
                    block_cid.to_string()[..8].to_lowercase(),
                    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());

                conflicts.push(DagConflict {
                    conflict_id,
                    conflicting_blocks: children.clone(),
                    fork_point: Some(block_cid.clone()),
                    detected_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    conflict_type: ConflictType::ChainFork,
                    node_positions: HashMap::new(),
                    resolution_status: ResolutionStatus::Detected,
                });

                visited.insert(block_cid.clone());
            }
        }

        Ok(conflicts)
    }

    /// Detect cycles in the DAG (which shouldn't exist in a proper DAG)
    fn detect_cycles(&self, dag: &DagStructure) -> Result<Option<DagConflict>, CommonError> {
        if let Some(cycle) = dag.find_cycle() {
            let conflict_id = format!("cycle_{}", 
                SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());

            Ok(Some(DagConflict {
                conflict_id,
                conflicting_blocks: cycle,
                fork_point: None,
                detected_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                conflict_type: ConflictType::CyclicDependency,
                node_positions: HashMap::new(),
                resolution_status: ResolutionStatus::Detected,
            }))
        } else {
            Ok(None)
        }
    }

    /// Detect missing blocks that break DAG consistency
    fn detect_missing_blocks(&self, dag: &DagStructure) -> Result<Vec<DagConflict>, CommonError> {
        let mut conflicts = Vec::new();
        let missing = dag.find_missing_references();

        if !missing.is_empty() {
            let conflict_id = format!("missing_blocks_{}", 
                SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());

            conflicts.push(DagConflict {
                conflict_id,
                conflicting_blocks: missing,
                fork_point: None,
                detected_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                conflict_type: ConflictType::MissingBlocks,
                node_positions: HashMap::new(),
                resolution_status: ResolutionStatus::Detected,
            });
        }

        Ok(conflicts)
    }

    /// Resolve conflict by choosing the block with earliest timestamp
    fn resolve_by_timestamp(&self, conflict: &DagConflict) -> Result<Cid, CommonError> {
        let mut earliest_block = None;
        let mut earliest_time = u64::MAX;

        for block_cid in &conflict.conflicting_blocks {
            if let Ok(Some(block)) = self.store.get(block_cid) {
                if block.timestamp < earliest_time {
                    earliest_time = block.timestamp;
                    earliest_block = Some(block_cid.clone());
                }
            }
        }

        earliest_block.ok_or_else(|| CommonError::ResourceNotFound("No valid blocks found".to_string()))
    }

    /// Resolve conflict by choosing block from highest reputation author
    fn resolve_by_reputation(&self, conflict: &DagConflict) -> Result<Cid, CommonError> {
        // For now, use timestamp as proxy for reputation
        // In full implementation, this would query the reputation system
        self.resolve_by_timestamp(conflict)
    }

    /// Resolve conflict by choosing the most referenced block
    fn resolve_by_popularity(&self, conflict: &DagConflict) -> Result<Cid, CommonError> {
        let blocks = self.store.list_blocks()?;
        let mut reference_counts = HashMap::new();

        // Count references to each conflicting block
        for block_cid in &conflict.conflicting_blocks {
            reference_counts.insert(block_cid.clone(), 0);
        }

        for block in &blocks {
            for link in &block.links {
                if let Some(count) = reference_counts.get_mut(&link.cid) {
                    *count += 1;
                }
            }
        }

        // Find the most referenced block
        let most_popular = reference_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(cid, _)| cid);

        most_popular.ok_or_else(|| CommonError::ResourceNotFound("No references found".to_string()))
    }

    /// Resolve conflict by choosing block from longest chain
    fn resolve_by_chain_length(&self, conflict: &DagConflict) -> Result<Cid, CommonError> {
        let mut longest_chain = None;
        let mut max_length = 0;

        for block_cid in &conflict.conflicting_blocks {
            let chain_length = self.calculate_chain_length(block_cid)?;
            if chain_length > max_length {
                max_length = chain_length;
                longest_chain = Some(block_cid.clone());
            }
        }

        longest_chain.ok_or_else(|| CommonError::ResourceNotFound("No chains found".to_string()))
    }

    /// Resolve conflict using multiple criteria with weighted scoring
    fn resolve_by_multiple_criteria(&self, conflict: &DagConflict) -> Result<Cid, CommonError> {
        let mut scores = HashMap::new();

        for block_cid in &conflict.conflicting_blocks {
            let mut score = 0.0;

            // Timestamp factor (earlier is better)
            if let Ok(Some(block)) = self.store.get(block_cid) {
                let age_hours = (SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() - block.timestamp) / 3600;
                score += 1.0 / (1.0 + age_hours as f64 * 0.1); // Decay with age
            }

            // Chain length factor
            if let Ok(chain_length) = self.calculate_chain_length(block_cid) {
                score += chain_length as f64 * 0.1;
            }

            // Reference count factor
            let reference_count = self.count_references(block_cid)?;
            score += reference_count as f64 * 0.2;

            scores.insert(block_cid.clone(), score);
        }

        let winner = scores.into_iter()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(cid, _)| cid);

        winner.ok_or_else(|| CommonError::ResourceNotFound("No winner determined".to_string()))
    }

    /// Apply the conflict resolution by updating DAG state
    fn apply_resolution(&mut self, conflict_id: &str, winner: &Cid) -> Result<(), CommonError> {
        let conflict = self.active_conflicts.get_mut(conflict_id)
            .ok_or_else(|| CommonError::ResourceNotFound(format!("Conflict {} not found", conflict_id)))?;

        // Mark as resolved
        conflict.resolution_status = ResolutionStatus::Resolved {
            winner: winner.clone(),
            applied_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        };

        // Remove losing blocks (optional - might want to keep for audit)
        for block_cid in &conflict.conflicting_blocks {
            if block_cid != winner {
                // In a full implementation, we might move losing blocks to an archive
                // rather than deleting them completely
                println!("Conflict resolved: {} wins over {:?}", winner, block_cid);
            }
        }

        // Move to history
        let resolved_conflict = conflict.clone();
        self.resolution_history.push_back(resolved_conflict);
        self.active_conflicts.remove(conflict_id);

        // Limit history size
        while self.resolution_history.len() > 100 {
            self.resolution_history.pop_front();
        }

        Ok(())
    }

    /// Calculate the length of the chain starting from a given block
    fn calculate_chain_length(&self, start_cid: &Cid) -> Result<usize, CommonError> {
        let mut length = 0;
        let mut current = Some(start_cid.clone());
        let mut visited = HashSet::new();

        while let Some(cid) = current {
            if visited.contains(&cid) {
                break; // Cycle detected
            }
            visited.insert(cid.clone());

            if let Ok(Some(block)) = self.store.get(&cid) {
                length += 1;
                // Follow the first link (main chain)
                current = block.links.first().map(|link| link.cid.clone());
            } else {
                break;
            }
        }

        Ok(length)
    }

    /// Count how many blocks reference the given block
    fn count_references(&self, target_cid: &Cid) -> Result<usize, CommonError> {
        let blocks = self.store.list_blocks()?;
        let mut count = 0;

        for block in &blocks {
            for link in &block.links {
                if &link.cid == target_cid {
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    /// Remove old conflicts to limit memory usage
    fn prune_old_conflicts(&mut self) {
        if self.active_conflicts.len() > self.config.max_concurrent_conflicts {
            let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            
            // Remove conflicts older than timeout
            self.active_conflicts.retain(|_, conflict| {
                current_time - conflict.detected_at < self.config.evidence_timeout
            });
        }
    }

    /// Get all active conflicts
    pub fn get_active_conflicts(&self) -> &HashMap<String, DagConflict> {
        &self.active_conflicts
    }

    /// Get resolution history
    pub fn get_resolution_history(&self) -> &VecDeque<DagConflict> {
        &self.resolution_history
    }
}

/// Internal representation of DAG structure for analysis
struct DagStructure {
    blocks: HashMap<Cid, DagBlockMetadata>,
    parents: HashMap<Cid, Vec<Cid>>,
    children: HashMap<Cid, Vec<Cid>>,
}

impl DagStructure {
    fn new() -> Self {
        Self {
            blocks: HashMap::new(),
            parents: HashMap::new(),
            children: HashMap::new(),
        }
    }

    fn add_block(&mut self, cid: Cid, metadata: DagBlockMetadata) {
        self.blocks.insert(cid.clone(), metadata);
        self.parents.entry(cid.clone()).or_default();
        self.children.entry(cid).or_default();
    }

    fn add_edge(&mut self, from: Cid, to: Cid) {
        self.children.entry(from.clone()).or_default().push(to.clone());
        self.parents.entry(to).or_default().push(from);
    }

    fn find_roots(&self) -> Vec<Cid> {
        self.parents
            .iter()
            .filter(|(_, parents)| parents.is_empty())
            .map(|(cid, _)| cid.clone())
            .collect()
    }

    fn find_cycle(&self) -> Option<Vec<Cid>> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for cid in self.blocks.keys() {
            if !visited.contains(cid) {
                if let Some(cycle) = self.dfs_cycle_detection(cid, &mut visited, &mut rec_stack) {
                    return Some(cycle);
                }
            }
        }
        None
    }

    fn dfs_cycle_detection(
        &self,
        cid: &Cid,
        visited: &mut HashSet<Cid>,
        rec_stack: &mut HashSet<Cid>,
    ) -> Option<Vec<Cid>> {
        visited.insert(cid.clone());
        rec_stack.insert(cid.clone());

        if let Some(children) = self.children.get(cid) {
            for child in children {
                if !visited.contains(child) {
                    if let Some(cycle) = self.dfs_cycle_detection(child, visited, rec_stack) {
                        return Some(cycle);
                    }
                } else if rec_stack.contains(child) {
                    // Cycle detected
                    return Some(vec![cid.clone(), child.clone()]);
                }
            }
        }

        rec_stack.remove(cid);
        None
    }

    fn find_missing_references(&self) -> Vec<Cid> {
        let mut missing = Vec::new();
        let existing_cids: HashSet<_> = self.blocks.keys().cloned().collect();

        for children in self.children.values() {
            for child_cid in children {
                if !existing_cids.contains(child_cid) {
                    missing.push(child_cid.clone());
                }
            }
        }

        missing.sort_by(|a, b| a.to_string().cmp(&b.to_string()));
        missing.dedup();
        missing
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::InMemoryDagStore;
    use icn_common::{DagLink, compute_merkle_cid};

    fn create_test_block(id: &str, links: Vec<DagLink>) -> DagBlock {
        let data = format!("data for {}", id).into_bytes();
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let author = Did::new("key", "tester");
        let sig = None;
        let cid = compute_merkle_cid(0x71, &data, &links, timestamp, &author, &sig, &None);
        
        DagBlock {
            cid,
            data,
            links,
            timestamp,
            author_did: author,
            signature: sig,
            scope: None,
        }
    }

    #[test]
    fn test_conflict_resolver_creation() {
        let store = InMemoryDagStore::new();
        let config = ConflictResolutionConfig::default();
        let node_id = Did::new("key", "test_node");
        
        let resolver = ConflictResolver::new(store, config, node_id);
        assert_eq!(resolver.active_conflicts.len(), 0);
    }

    #[test]
    fn test_root_conflict_detection() {
        let mut store = InMemoryDagStore::new();
        let config = ConflictResolutionConfig::default();
        let node_id = Did::new("key", "test_node");
        
        // Create two root blocks (no parents)
        let block1 = create_test_block("root1", vec![]);
        let block2 = create_test_block("root2", vec![]);
        
        store.put(&block1).unwrap();
        store.put(&block2).unwrap();
        
        let mut resolver = ConflictResolver::new(store, config, node_id);
        let conflicts = resolver.detect_conflicts().unwrap();
        
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].conflict_type, ConflictType::RootConflict);
        assert_eq!(conflicts[0].conflicting_blocks.len(), 2);
    }

    #[test]
    fn test_chain_fork_detection() {
        let mut store = InMemoryDagStore::new();
        let config = ConflictResolutionConfig::default();
        let node_id = Did::new("key", "test_node");
        
        // Create a common ancestor
        let ancestor = create_test_block("ancestor", vec![]);
        store.put(&ancestor).unwrap();
        
        // Create two children of the ancestor (fork)
        let link = DagLink {
            cid: ancestor.cid.clone(),
            name: "parent".to_string(),
            size: 0,
        };
        
        let fork1 = create_test_block("fork1", vec![link.clone()]);
        let fork2 = create_test_block("fork2", vec![link]);
        
        store.put(&fork1).unwrap();
        store.put(&fork2).unwrap();
        
        let mut resolver = ConflictResolver::new(store, config, node_id);
        let conflicts = resolver.detect_conflicts().unwrap();
        
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].conflict_type, ConflictType::ChainFork);
        assert_eq!(conflicts[0].fork_point, Some(ancestor.cid));
    }

    #[test]
    fn test_conflict_resolution_by_timestamp() {
        let mut store = InMemoryDagStore::new();
        let config = ConflictResolutionConfig {
            resolution_strategy: ResolutionStrategy::FirstWins,
            ..Default::default()
        };
        let node_id = Did::new("key", "test_node");
        
        // Create blocks with different timestamps
        let mut block1 = create_test_block("early", vec![]);
        let mut block2 = create_test_block("late", vec![]);
        
        block1.timestamp = 1000;
        block2.timestamp = 2000;
        
        store.put(&block1).unwrap();
        store.put(&block2).unwrap();
        
        let mut resolver = ConflictResolver::new(store, config, node_id);
        let conflicts = resolver.detect_conflicts().unwrap();
        
        if !conflicts.is_empty() {
            let resolution = resolver.resolve_conflict(&conflicts[0].conflict_id).unwrap();
            if let ResolutionStatus::Resolved { winner, .. } = resolution {
                assert_eq!(winner, block1.cid); // Earlier block should win
            }
        }
    }
}