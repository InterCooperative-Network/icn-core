// icn-ccl/src/package/resolver.rs
//! Dependency resolution for CCL packages

use std::collections::{HashMap, HashSet, VecDeque};
use super::{
    manifest::{Dependency, PackageManifest, VersionReq},
    registry::{Registry, RegistryError},
};

/// Resolved dependency with specific version
#[derive(Debug, Clone)]
pub struct ResolvedDependency {
    pub name: String,
    pub version: String,
    pub source: Option<String>,
    pub dependencies: Vec<ResolvedDependency>,
}

/// Dependency resolution errors
#[derive(Debug, thiserror::Error)]
pub enum ResolverError {
    #[error("Registry error: {0}")]
    Registry(#[from] RegistryError),
    
    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),
    
    #[error("Version conflict for package {package}: {version1} vs {version2}")]
    VersionConflict {
        package: String,
        version1: String,
        version2: String,
    },
    
    #[error("No compatible version found for {package} with requirement {requirement}")]
    NoCompatibleVersion {
        package: String,
        requirement: String,
    },
}

/// Dependency resolver
pub struct DependencyResolver {
    registry: Registry,
    resolved_cache: HashMap<String, ResolvedDependency>,
}

impl DependencyResolver {
    /// Create a new dependency resolver
    pub fn new(registry: Registry) -> Self {
        Self {
            registry,
            resolved_cache: HashMap::new(),
        }
    }

    /// Resolve all dependencies for a package manifest
    pub fn resolve(&mut self, manifest: &PackageManifest) -> Result<Vec<ResolvedDependency>, ResolverError> {
        let mut resolved = Vec::new();
        let mut visited = HashSet::new();
        let mut visiting = HashSet::new();

        // Resolve runtime dependencies
        for (name, dependency) in &manifest.dependencies {
            if !visited.contains(name) {
                let resolved_dep = self.resolve_dependency(dependency, &mut visited, &mut visiting)?;
                resolved.push(resolved_dep);
            }
        }

        Ok(resolved)
    }

    /// Resolve a single dependency recursively
    fn resolve_dependency(
        &mut self,
        dependency: &Dependency,
        visited: &mut HashSet<String>,
        visiting: &mut HashSet<String>,
    ) -> Result<ResolvedDependency, ResolverError> {
        let package_name = &dependency.name;

        // Check for circular dependency
        if visiting.contains(package_name) {
            return Err(ResolverError::CircularDependency(package_name.clone()));
        }

        // Check cache first
        if let Some(cached) = self.resolved_cache.get(package_name) {
            return Ok(cached.clone());
        }

        visiting.insert(package_name.clone());

        // Get available versions from registry
        let available_versions = self.registry.get_versions(package_name)?;
        
        // Find compatible version
        let compatible_version = self.find_compatible_version(&dependency.version, &available_versions)
            .ok_or_else(|| ResolverError::NoCompatibleVersion {
                package: package_name.clone(),
                requirement: dependency.version.requirement.clone(),
            })?;

        // Get package manifest for the resolved version
        let package_info = self.registry.get_package(package_name, Some(&compatible_version))?;
        
        // For now, assume packages don't have their own dependencies
        // TODO: Download and parse package manifest to get actual dependencies
        let sub_dependencies = Vec::new();

        let resolved = ResolvedDependency {
            name: package_name.clone(),
            version: compatible_version,
            source: dependency.source.clone(),
            dependencies: sub_dependencies,
        };

        visiting.remove(package_name);
        visited.insert(package_name.clone());
        self.resolved_cache.insert(package_name.clone(), resolved.clone());

        Ok(resolved)
    }

    /// Find a version that satisfies the version requirement
    fn find_compatible_version(&self, version_req: &VersionReq, available_versions: &[String]) -> Option<String> {
        // Simple implementation - just find first matching version
        // TODO: Implement proper semver resolution with preference for latest compatible
        
        for version in available_versions {
            if version_req.matches(version) {
                return Some(version.clone());
            }
        }
        
        None
    }

    /// Get the dependency tree as a flat list
    pub fn flatten_dependencies(&self, resolved: &[ResolvedDependency]) -> Vec<ResolvedDependency> {
        let mut flattened = Vec::new();
        let mut queue = VecDeque::new();
        let mut seen = HashSet::new();

        // Add initial dependencies to queue
        for dep in resolved {
            queue.push_back(dep);
        }

        while let Some(dep) = queue.pop_front() {
            let key = format!("{}@{}", dep.name, dep.version);
            
            if !seen.contains(&key) {
                seen.insert(key);
                flattened.push(dep.clone());
                
                // Add sub-dependencies to queue
                for sub_dep in &dep.dependencies {
                    queue.push_back(sub_dep);
                }
            }
        }

        flattened
    }

    /// Check for version conflicts in resolved dependencies
    pub fn check_conflicts(&self, resolved: &[ResolvedDependency]) -> Result<(), ResolverError> {
        let flattened = self.flatten_dependencies(resolved);
        let mut versions = HashMap::new();

        for dep in flattened {
            if let Some(existing_version) = versions.get(&dep.name) {
                if existing_version != &dep.version {
                    return Err(ResolverError::VersionConflict {
                        package: dep.name,
                        version1: existing_version.clone(),
                        version2: dep.version,
                    });
                }
            } else {
                versions.insert(dep.name, dep.version);
            }
        }

        Ok(())
    }
}