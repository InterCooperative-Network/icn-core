use crate::package::Package;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub download_url: String,
    pub checksum: String,
    pub published_at: String,
}

pub struct PackageRegistry {
    client: Client,
    registry_url: String,
    local_cache: HashMap<String, RegistryEntry>,
}

impl PackageRegistry {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            registry_url: "https://registry.icn.coop/packages".to_string(), // Hypothetical registry
            local_cache: HashMap::new(),
        }
    }
    
    /// Search for a package in the registry
    pub async fn search_package(&mut self, name: &str) -> Result<Option<RegistryEntry>, String> {
        // Check local cache first
        if let Some(entry) = self.local_cache.get(name) {
            return Ok(Some(entry.clone()));
        }
        
        // For now, return mock data since we don't have a real registry
        let mock_packages = self.get_mock_packages();
        
        if let Some(package) = mock_packages.get(name) {
            self.local_cache.insert(name.to_string(), package.clone());
            Ok(Some(package.clone()))
        } else {
            Ok(None)
        }
    }
    
    /// Download a package from the registry
    pub async fn download_package(&self, name: &str, version: &str) -> Result<Vec<u8>, String> {
        // In a real implementation, this would download from the registry
        // For now, create a mock package
        self.create_mock_package(name, version)
    }
    
    /// Publish a package to the registry
    pub async fn publish_package(&self, package: &Package, package_data: Vec<u8>) -> Result<(), String> {
        println!("📤 Publishing {} v{} to registry...", package.name, package.version);
        
        // In a real implementation, this would upload to the registry
        // For now, just simulate success
        println!("✅ Published successfully!");
        
        Ok(())
    }
    
    /// List available packages
    pub async fn list_packages(&self) -> Result<Vec<RegistryEntry>, String> {
        let mock_packages = self.get_mock_packages();
        Ok(mock_packages.into_values().collect())
    }
    
    fn get_mock_packages(&self) -> HashMap<String, RegistryEntry> {
        let mut packages = HashMap::new();
        
        packages.insert("governance-core".to_string(), RegistryEntry {
            name: "governance-core".to_string(),
            version: "1.0.0".to_string(),
            description: "Core governance primitives for cooperatives".to_string(),
            author: "ICN Team".to_string(),
            download_url: "https://registry.icn.coop/packages/governance-core/1.0.0".to_string(),
            checksum: "abc123".to_string(),
            published_at: "2024-01-01T00:00:00Z".to_string(),
        });
        
        packages.insert("liquid-democracy".to_string(), RegistryEntry {
            name: "liquid-democracy".to_string(),
            version: "0.9.0".to_string(),
            description: "Liquid democracy and delegation patterns".to_string(),
            author: "ICN Team".to_string(),
            download_url: "https://registry.icn.coop/packages/liquid-democracy/0.9.0".to_string(),
            checksum: "def456".to_string(),
            published_at: "2024-01-15T00:00:00Z".to_string(),
        });
        
        packages.insert("quadratic-voting".to_string(), RegistryEntry {
            name: "quadratic-voting".to_string(),
            version: "0.8.0".to_string(),
            description: "Quadratic voting implementation for fair decision making".to_string(),
            author: "ICN Team".to_string(),
            download_url: "https://registry.icn.coop/packages/quadratic-voting/0.8.0".to_string(),
            checksum: "ghi789".to_string(),
            published_at: "2024-01-20T00:00:00Z".to_string(),
        });
        
        packages.insert("economic-primitives".to_string(), RegistryEntry {
            name: "economic-primitives".to_string(),
            version: "1.1.0".to_string(),
            description: "Budget management and surplus distribution tools".to_string(),
            author: "ICN Team".to_string(),
            download_url: "https://registry.icn.coop/packages/economic-primitives/1.1.0".to_string(),
            checksum: "jkl012".to_string(),
            published_at: "2024-02-01T00:00:00Z".to_string(),
        });
        
        packages.insert("cooperative-templates".to_string(), RegistryEntry {
            name: "cooperative-templates".to_string(),
            version: "2.0.0".to_string(),
            description: "Ready-to-use cooperative governance templates".to_string(),
            author: "ICN Community".to_string(),
            download_url: "https://registry.icn.coop/packages/cooperative-templates/2.0.0".to_string(),
            checksum: "mno345".to_string(),
            published_at: "2024-02-15T00:00:00Z".to_string(),
        });
        
        packages
    }
    
    fn create_mock_package(&self, name: &str, _version: &str) -> Result<Vec<u8>, String> {
        // Create a simple tar.gz package with mock content
        let content = match name {
            "governance-core" => include_str!("../templates/governance-core.ccl"),
            "liquid-democracy" => include_str!("../templates/liquid-democracy.ccl"),
            "quadratic-voting" => include_str!("../templates/quadratic-voting.ccl"),
            "economic-primitives" => include_str!("../templates/economic-primitives.ccl"),
            "cooperative-templates" => include_str!("../templates/cooperative-templates.ccl"),
            _ => "// Mock package content\nfn main() -> Integer { return 42; }",
        };
        
        // In a real implementation, this would create a proper tar.gz archive
        Ok(content.as_bytes().to_vec())
    }
}