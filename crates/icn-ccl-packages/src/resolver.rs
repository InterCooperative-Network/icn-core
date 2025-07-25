use crate::package::Package;
use std::collections::{HashMap, HashSet, VecDeque};

pub struct DependencyResolver {
    known_packages: HashMap<String, Package>,
}

impl DependencyResolver {
    pub fn new() -> Self {
        Self {
            known_packages: HashMap::new(),
        }
    }
    
    pub fn add_package(&mut self, package: Package) {
        self.known_packages.insert(package.name.clone(), package);
    }
    
    /// Resolve dependencies for a package
    pub fn resolve_dependencies(&self, package_name: &str) -> Result<Vec<String>, String> {
        let mut resolved = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        
        queue.push_back(package_name.to_string());
        
        while let Some(current) = queue.pop_front() {
            if visited.contains(&current) {
                continue;
            }
            
            visited.insert(current.clone());
            
            if let Some(package) = self.known_packages.get(&current) {
                for (dep_name, _version) in &package.dependencies {
                    if !visited.contains(dep_name) {
                        queue.push_back(dep_name.clone());
                    }
                }
                
                if current != package_name {
                    resolved.push(current);
                }
            }
        }
        
        Ok(resolved)
    }
    
    /// Check for circular dependencies
    pub fn check_circular_dependencies(&self, package_name: &str) -> Result<(), String> {
        let mut visiting = HashSet::new();
        let mut visited = HashSet::new();
        
        self.dfs_check_cycle(package_name, &mut visiting, &mut visited)?;
        
        Ok(())
    }
    
    fn dfs_check_cycle(
        &self,
        package_name: &str,
        visiting: &mut HashSet<String>,
        visited: &mut HashSet<String>,
    ) -> Result<(), String> {
        if visiting.contains(package_name) {
            return Err(format!("Circular dependency detected involving: {}", package_name));
        }
        
        if visited.contains(package_name) {
            return Ok(());
        }
        
        visiting.insert(package_name.to_string());
        
        if let Some(package) = self.known_packages.get(package_name) {
            for (dep_name, _version) in &package.dependencies {
                self.dfs_check_cycle(dep_name, visiting, visited)?;
            }
        }
        
        visiting.remove(package_name);
        visited.insert(package_name.to_string());
        
        Ok(())
    }
    
    /// Check version compatibility
    pub fn check_version_compatibility(&self, package_name: &str, required_version: &str) -> bool {
        if let Some(package) = self.known_packages.get(package_name) {
            // Simple version comparison (in real implementation, use semver)
            return package.version == required_version || self.is_compatible_version(&package.version, required_version);
        }
        false
    }
    
    fn is_compatible_version(&self, available: &str, required: &str) -> bool {
        // Simple compatibility check - in real implementation, use proper semver
        let available_parts: Vec<&str> = available.split('.').collect();
        let required_parts: Vec<&str> = required.split('.').collect();
        
        if available_parts.len() >= 2 && required_parts.len() >= 2 {
            // Major version must match, minor version can be higher
            available_parts[0] == required_parts[0] && 
            available_parts[1] >= required_parts[1]
        } else {
            available == required
        }
    }
}