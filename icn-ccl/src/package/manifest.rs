// icn-ccl/src/package/manifest.rs
//! Package manifest definition for CCL packages

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Version requirement for dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionReq {
    pub requirement: String,  // e.g., "^1.0.0", ">=2.0.0", "~1.2.3"
}

impl VersionReq {
    pub fn new(requirement: &str) -> Self {
        Self {
            requirement: requirement.to_string(),
        }
    }

    /// Check if a version satisfies this requirement
    pub fn matches(&self, version: &str) -> bool {
        // TODO: Implement proper semver matching
        // For now, just do exact match
        self.requirement == version || self.requirement == "*"
    }
}

/// Dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version: VersionReq,
    pub source: Option<String>,  // Registry URL, git repo, or local path
    pub features: Vec<String>,   // Optional features to enable
}

/// Author information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    pub name: String,
    pub email: Option<String>,
    pub did: Option<String>,  // Decentralized identifier
}

/// Package metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
    pub governance_patterns: Vec<String>,  // Types of governance this package implements
    pub mana_cost: Option<u64>,            // Estimated mana cost for deployment
}

/// CCL package manifest (package.ccl)
#[derive(Debug, Serialize, Deserialize)]
pub struct PackageManifest {
    pub package: PackageInfo,
    pub dependencies: HashMap<String, Dependency>,
    pub dev_dependencies: HashMap<String, Dependency>,
    pub metadata: Option<Metadata>,
    pub build: Option<BuildConfig>,
}

/// Package information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub authors: Vec<Author>,
    pub license: Option<String>,
    pub repository: Option<String>,
    pub homepage: Option<String>,
    pub documentation: Option<String>,
    pub readme: Option<String>,
}

/// Build configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    pub optimization_level: Option<String>,  // "none", "basic", "aggressive"
    pub target: Option<String>,              // "wasm32-unknown-unknown"
    pub features: Vec<String>,               // Build features to enable
}

impl PackageManifest {
    /// Create a new package manifest
    pub fn new(name: String, version: String, authors: Vec<Author>) -> Self {
        Self {
            package: PackageInfo {
                name,
                version,
                description: None,
                authors,
                license: None,
                repository: None,
                homepage: None,
                documentation: None,
                readme: None,
            },
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
            metadata: None,
            build: None,
        }
    }

    /// Add a dependency
    pub fn add_dependency(&mut self, name: String, version_req: VersionReq) {
        let dependency = Dependency {
            name: name.clone(),
            version: version_req,
            source: None,
            features: Vec::new(),
        };
        self.dependencies.insert(name, dependency);
    }

    /// Add a development dependency
    pub fn add_dev_dependency(&mut self, name: String, version_req: VersionReq) {
        let dependency = Dependency {
            name: name.clone(),
            version: version_req,
            source: None,
            features: Vec::new(),
        };
        self.dev_dependencies.insert(name, dependency);
    }

    /// Load manifest from TOML file
    pub fn from_toml(content: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(content)
    }

    /// Save manifest to TOML format
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    /// Get all dependencies (runtime + dev)
    pub fn all_dependencies(&self) -> impl Iterator<Item = (&String, &Dependency)> {
        self.dependencies.iter().chain(self.dev_dependencies.iter())
    }

    /// Check if package has dependency
    pub fn has_dependency(&self, name: &str) -> bool {
        self.dependencies.contains_key(name) || self.dev_dependencies.contains_key(name)
    }
}