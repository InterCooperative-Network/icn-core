// icn-ccl/src/package_manager.rs
//! CCL Package Manager
//!
//! This module provides package management for CCL contracts and governance patterns:
//! - Package discovery and installation
//! - Dependency resolution
//! - Version management
//! - Governance pattern templates
//! - Module sharing and reuse

use crate::{CclError, ContractMetadata};
use icn_common::{Cid, Did, NodeScope};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Package information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: Did,
    pub license: String,
    pub keywords: Vec<String>,
    pub dependencies: HashMap<String, String>, // package_name -> version_requirement
    pub governance_patterns: Vec<GovernancePattern>,
    pub created_at: u64,
    pub updated_at: u64,
    pub download_count: u64,
    pub cid: Option<Cid>,
}

/// Governance pattern template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernancePattern {
    pub name: String,
    pub pattern_type: GovernancePatternType,
    pub description: String,
    pub parameters: HashMap<String, ParameterSpec>,
    pub source_code: String,
    pub examples: Vec<String>,
}

/// Types of governance patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GovernancePatternType {
    SimpleVoting,
    LiquidDemocracy,
    QuadraticVoting,
    RankedChoice,
    MultiStageProposal,
    BudgetAllocation,
    DividendDistribution,
    ConflictResolution,
    MembershipManagement,
    Custom(String),
}

/// Parameter specification for patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSpec {
    pub param_type: String,
    pub description: String,
    pub default_value: Option<String>,
    pub required: bool,
    pub validation: Option<String>,
}

/// Package registry interface
pub trait PackageRegistry: Send + Sync {
    fn search_packages(&self, query: &str) -> Result<Vec<PackageInfo>, CclError>;
    fn get_package(&self, name: &str, version: &str) -> Result<Option<Package>, CclError>;
    fn publish_package(&self, package: &Package) -> Result<(), CclError>;
    fn list_governance_patterns(&self) -> Result<Vec<GovernancePattern>, CclError>;
    fn get_governance_pattern(&self, name: &str) -> Result<Option<GovernancePattern>, CclError>;
}

/// Complete package with code and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub info: PackageInfo,
    pub source_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compiled_wasm: Option<Vec<u8>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<ContractMetadata>,
}

/// Local package manager
pub struct CclPackageManager {
    registry: Arc<dyn PackageRegistry>,
    local_cache: PathBuf,
    installed_packages: HashMap<String, Package>,
    governance_patterns: HashMap<String, GovernancePattern>,
}

impl CclPackageManager {
    /// Create a new package manager
    pub fn new<P: AsRef<Path>>(
        registry: Arc<dyn PackageRegistry>,
        cache_path: P,
    ) -> Result<Self, CclError> {
        let cache_path = cache_path.as_ref().to_path_buf();

        // Ensure cache directory exists
        std::fs::create_dir_all(&cache_path)
            .map_err(|e| CclError::IoError(format!("Failed to create cache directory: {}", e)))?;

        let mut manager = Self {
            registry,
            local_cache: cache_path,
            installed_packages: HashMap::new(),
            governance_patterns: HashMap::new(),
        };

        // Load default governance patterns
        manager.load_default_patterns()?;

        Ok(manager)
    }

    /// Search for packages in the registry
    pub fn search(&self, query: &str) -> Result<Vec<PackageInfo>, CclError> {
        self.registry.search_packages(query)
    }

    /// Install a package and its dependencies
    pub fn install(&mut self, name: &str, version: &str) -> Result<(), CclError> {
        // Check if already installed
        if self.installed_packages.contains_key(name) {
            return Ok(());
        }

        // Get package from registry
        let package = self
            .registry
            .get_package(name, version)?
            .ok_or_else(|| {
                CclError::CompilationError(format!("Package {}@{} not found", name, version))
            })?;

        // Install dependencies first
        for (dep_name, dep_version) in &package.info.dependencies {
            self.install(dep_name, dep_version)?;
        }

        // Cache package locally
        self.cache_package(&package)?;

        // Add to installed packages
        self.installed_packages.insert(name.to_string(), package);

        Ok(())
    }

    /// Uninstall a package
    pub fn uninstall(&mut self, name: &str) -> Result<(), CclError> {
        self.installed_packages.remove(name);

        // Remove from cache
        let package_path = self.local_cache.join(format!("{}.ccl", name));
        if package_path.exists() {
            std::fs::remove_file(package_path)
                .map_err(|e| CclError::IoError(format!("Failed to remove package cache: {}", e)))?;
        }

        Ok(())
    }

    /// Get an installed package
    pub fn get_package(&self, name: &str) -> Option<&Package> {
        self.installed_packages.get(name)
    }

    /// List all installed packages
    pub fn list_installed(&self) -> Vec<&PackageInfo> {
        self.installed_packages
            .values()
            .map(|p| &p.info)
            .collect()
    }

    /// Create a new package from source
    pub fn create_package(
        &self,
        name: String,
        version: String,
        description: String,
        author: Did,
        source_code: String,
        governance_patterns: Vec<GovernancePattern>,
    ) -> Result<Package, CclError> {
        // Compile the source to validate it
        let (wasm, metadata) = crate::compile_ccl_source_to_wasm(&source_code)?;

        let info = PackageInfo {
            name,
            version,
            description,
            author,
            license: "Apache-2.0".to_string(), // Default license
            keywords: Vec::new(),
            dependencies: HashMap::new(),
            governance_patterns,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            updated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            download_count: 0,
            cid: None,
        };

        Ok(Package {
            info,
            source_code,
            compiled_wasm: Some(wasm),
            metadata: Some(metadata),
        })
    }

    /// Publish a package to the registry
    pub fn publish(&self, package: &Package) -> Result<(), CclError> {
        self.registry.publish_package(package)
    }

    /// Get a governance pattern by name
    pub fn get_governance_pattern(&self, name: &str) -> Option<&GovernancePattern> {
        self.governance_patterns.get(name)
    }

    /// List all available governance patterns
    pub fn list_governance_patterns(&self) -> Vec<&GovernancePattern> {
        self.governance_patterns.values().collect()
    }

    /// Generate code from a governance pattern template
    pub fn generate_from_pattern(
        &self,
        pattern_name: &str,
        parameters: &HashMap<String, String>,
    ) -> Result<String, CclError> {
        let pattern = self
            .governance_patterns
            .get(pattern_name)
            .ok_or_else(|| {
                CclError::CompilationError(format!("Governance pattern '{}' not found", pattern_name))
            })?;

        let mut code = pattern.source_code.clone();

        // Replace parameter placeholders
        for (param_name, param_value) in parameters {
            let placeholder = format!("{{{}}}", param_name);
            code = code.replace(&placeholder, param_value);
        }

        // Validate the generated code
        let _ = crate::parser::parse_ccl_source(&code)?;

        Ok(code)
    }

    /// Cache a package locally
    fn cache_package(&self, package: &Package) -> Result<(), CclError> {
        let package_path = self
            .local_cache
            .join(format!("{}_{}.json", package.info.name, package.info.version));

        let package_json = serde_json::to_string_pretty(package)
            .map_err(|e| CclError::SerializationError(format!("Failed to serialize package: {}", e)))?;

        std::fs::write(package_path, package_json)
            .map_err(|e| CclError::IoError(format!("Failed to cache package: {}", e)))?;

        Ok(())
    }

    /// Load default governance patterns
    fn load_default_patterns(&mut self) -> Result<(), CclError> {
        // Simple voting pattern
        let simple_voting = GovernancePattern {
            name: "simple_voting".to_string(),
            pattern_type: GovernancePatternType::SimpleVoting,
            description: "Basic yes/no voting with simple majority".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert(
                    "quorum".to_string(),
                    ParameterSpec {
                        param_type: "u32".to_string(),
                        description: "Minimum number of votes required".to_string(),
                        default_value: Some("3".to_string()),
                        required: false,
                        validation: Some("value > 0".to_string()),
                    },
                );
                params.insert(
                    "threshold".to_string(),
                    ParameterSpec {
                        param_type: "f32".to_string(),
                        description: "Fraction of yes votes required (0.0-1.0)".to_string(),
                        default_value: Some("0.5".to_string()),
                        required: false,
                        validation: Some("value >= 0.0 && value <= 1.0".to_string()),
                    },
                );
                params
            },
            source_code: r#"
fn simple_vote(proposal_id: string, voter: string, vote: bool) -> bool {
    let proposal = get_proposal(proposal_id);
    if proposal.status != "voting" {
        return false;
    }
    
    cast_vote(proposal_id, voter, vote);
    
    let vote_count = count_votes(proposal_id);
    if vote_count.total >= {quorum} {
        let yes_ratio = vote_count.yes as f32 / vote_count.total as f32;
        if yes_ratio >= {threshold} {
            proposal.status = "accepted";
        } else {
            proposal.status = "rejected";
        }
        return true;
    }
    
    false
}
"#
            .to_string(),
            examples: vec![
                r#"simple_vote("prop_001", "alice", true)"#.to_string(),
                r#"simple_vote("prop_002", "bob", false)"#.to_string(),
            ],
        };

        // Liquid democracy pattern
        let liquid_democracy = GovernancePattern {
            name: "liquid_democracy".to_string(),
            pattern_type: GovernancePatternType::LiquidDemocracy,
            description: "Liquid democracy with delegation support".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert(
                    "max_delegation_depth".to_string(),
                    ParameterSpec {
                        param_type: "u32".to_string(),
                        description: "Maximum delegation chain length".to_string(),
                        default_value: Some("5".to_string()),
                        required: false,
                        validation: Some("value > 0 && value <= 10".to_string()),
                    },
                );
                params
            },
            source_code: r#"
fn delegate_vote(delegator: string, delegate: string) -> bool {
    if delegator == delegate {
        return false;
    }
    
    // Check for cycles
    let depth = 0;
    let current = delegate;
    while depth < {max_delegation_depth} {
        let next_delegate = get_delegation(current);
        if next_delegate == delegator {
            return false; // Cycle detected
        }
        if next_delegate.is_empty() {
            break;
        }
        current = next_delegate;
        depth = depth + 1;
    }
    
    set_delegation(delegator, delegate);
    true
}

fn vote_with_delegation(proposal_id: string, voter: string, vote: bool) -> bool {
    let final_voter = resolve_delegation(voter);
    cast_vote(proposal_id, final_voter, vote);
    true
}
"#
            .to_string(),
            examples: vec![
                r#"delegate_vote("alice", "bob")"#.to_string(),
                r#"vote_with_delegation("prop_001", "alice", true)"#.to_string(),
            ],
        };

        // Quadratic voting pattern
        let quadratic_voting = GovernancePattern {
            name: "quadratic_voting".to_string(),
            pattern_type: GovernancePatternType::QuadraticVoting,
            description: "Quadratic voting with voice credits".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert(
                    "initial_credits".to_string(),
                    ParameterSpec {
                        param_type: "u32".to_string(),
                        description: "Initial voice credits per voter".to_string(),
                        default_value: Some("100".to_string()),
                        required: false,
                        validation: Some("value > 0".to_string()),
                    },
                );
                params
            },
            source_code: r#"
fn quadratic_vote(proposal_id: string, voter: string, vote_strength: u32) -> bool {
    let credits_needed = vote_strength * vote_strength;
    let available_credits = get_voice_credits(voter);
    
    if credits_needed > available_credits {
        return false;
    }
    
    spend_voice_credits(voter, credits_needed);
    cast_weighted_vote(proposal_id, voter, vote_strength);
    true
}

fn initialize_voter_credits(voter: string) -> bool {
    set_voice_credits(voter, {initial_credits});
    true
}
"#
            .to_string(),
            examples: vec![
                r#"initialize_voter_credits("alice")"#.to_string(),
                r#"quadratic_vote("prop_001", "alice", 5)"#.to_string(),
            ],
        };

        // Budget allocation pattern
        let budget_allocation = GovernancePattern {
            name: "budget_allocation".to_string(),
            pattern_type: GovernancePatternType::BudgetAllocation,
            description: "Democratic budget allocation with proposals".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert(
                    "total_budget".to_string(),
                    ParameterSpec {
                        param_type: "u64".to_string(),
                        description: "Total budget to allocate".to_string(),
                        default_value: Some("10000".to_string()),
                        required: true,
                        validation: Some("value > 0".to_string()),
                    },
                );
                params
            },
            source_code: r#"
fn propose_budget_allocation(
    proposer: string,
    recipient: string, 
    amount: u64,
    purpose: string
) -> string {
    let proposal_id = generate_id();
    create_budget_proposal(proposal_id, proposer, recipient, amount, purpose);
    proposal_id
}

fn approve_budget_allocation(proposal_id: string) -> bool {
    let proposal = get_budget_proposal(proposal_id);
    let remaining_budget = get_remaining_budget();
    
    if proposal.amount > remaining_budget {
        return false;
    }
    
    allocate_budget(proposal.recipient, proposal.amount);
    proposal.status = "approved";
    true
}
"#
            .to_string(),
            examples: vec![
                r#"propose_budget_allocation("alice", "dev_team", 5000, "Development work")"#.to_string(),
                r#"approve_budget_allocation("budget_001")"#.to_string(),
            ],
        };

        // Add patterns to manager
        self.governance_patterns
            .insert(simple_voting.name.clone(), simple_voting);
        self.governance_patterns
            .insert(liquid_democracy.name.clone(), liquid_democracy);
        self.governance_patterns
            .insert(quadratic_voting.name.clone(), quadratic_voting);
        self.governance_patterns
            .insert(budget_allocation.name.clone(), budget_allocation);

        Ok(())
    }
}

/// In-memory package registry for testing and local development
pub struct InMemoryRegistry {
    packages: HashMap<String, HashMap<String, Package>>, // name -> version -> package
    patterns: HashMap<String, GovernancePattern>,
}

impl InMemoryRegistry {
    pub fn new() -> Self {
        Self {
            packages: HashMap::new(),
            patterns: HashMap::new(),
        }
    }
}

impl Default for InMemoryRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PackageRegistry for InMemoryRegistry {
    fn search_packages(&self, query: &str) -> Result<Vec<PackageInfo>, CclError> {
        let mut results = Vec::new();

        for (_, versions) in &self.packages {
            for (_, package) in versions {
                if package.info.name.contains(query)
                    || package.info.description.contains(query)
                    || package.info.keywords.iter().any(|k| k.contains(query))
                {
                    results.push(package.info.clone());
                }
            }
        }

        // Sort by download count (descending)
        results.sort_by(|a, b| b.download_count.cmp(&a.download_count));

        Ok(results)
    }

    fn get_package(&self, name: &str, version: &str) -> Result<Option<Package>, CclError> {
        Ok(self
            .packages
            .get(name)
            .and_then(|versions| versions.get(version))
            .cloned())
    }

    fn publish_package(&self, _package: &Package) -> Result<(), CclError> {
        // In a real implementation, this would be mutable
        // For now, just return success
        Ok(())
    }

    fn list_governance_patterns(&self) -> Result<Vec<GovernancePattern>, CclError> {
        Ok(self.patterns.values().cloned().collect())
    }

    fn get_governance_pattern(&self, name: &str) -> Result<Option<GovernancePattern>, CclError> {
        Ok(self.patterns.get(name).cloned())
    }
}

/// Create a package manager with in-memory registry for testing
pub fn create_test_package_manager<P: AsRef<Path>>(
    cache_path: P,
) -> Result<CclPackageManager, CclError> {
    let registry = Arc::new(InMemoryRegistry::new());
    CclPackageManager::new(registry, cache_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_package_manager_creation() {
        let temp_dir = tempdir().unwrap();
        let manager = create_test_package_manager(temp_dir.path());
        assert!(manager.is_ok());
    }

    #[test]
    fn test_governance_pattern_generation() {
        let temp_dir = tempdir().unwrap();
        let manager = create_test_package_manager(temp_dir.path()).unwrap();

        let mut params = HashMap::new();
        params.insert("quorum".to_string(), "5".to_string());
        params.insert("threshold".to_string(), "0.6".to_string());

        let result = manager.generate_from_pattern("simple_voting", &params);
        assert!(result.is_ok());

        let code = result.unwrap();
        assert!(code.contains("5"));
        assert!(code.contains("0.6"));
    }

    #[test]
    fn test_pattern_listing() {
        let temp_dir = tempdir().unwrap();
        let manager = create_test_package_manager(temp_dir.path()).unwrap();

        let patterns = manager.list_governance_patterns();
        assert!(!patterns.is_empty());

        let pattern_names: Vec<&str> = patterns.iter().map(|p| p.name.as_str()).collect();
        assert!(pattern_names.contains(&"simple_voting"));
        assert!(pattern_names.contains(&"liquid_democracy"));
        assert!(pattern_names.contains(&"quadratic_voting"));
    }
}