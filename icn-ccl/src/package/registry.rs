// icn-ccl/src/package/registry.rs
//! CCL package registry for sharing governance patterns and modules

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::manifest::{Author, Metadata};

/// Package information in the registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub authors: Vec<Author>,
    pub license: Option<String>,
    pub repository: Option<String>,
    pub download_url: String,
    pub checksum: String,  // SHA-256 of package contents
    pub published_at: String,  // ISO 8601 timestamp
    pub metadata: Option<Metadata>,
}

/// Registry API client
pub struct Registry {
    base_url: String,
    client: reqwest::blocking::Client,
}

impl Registry {
    /// Create a new registry client
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::blocking::Client::new(),
        }
    }

    /// Get default ICN registry
    pub fn default() -> Self {
        Self::new("https://registry.icn.org".to_string())
    }

    /// Search for packages
    pub fn search(&self, query: &str) -> Result<Vec<PackageInfo>, RegistryError> {
        let url = format!("{}/api/v1/packages/search?q={}", self.base_url, query);
        
        let response = self.client
            .get(&url)
            .send()
            .map_err(|e| RegistryError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(RegistryError::ApiError(format!(
                "Search failed with status: {}", 
                response.status()
            )));
        }

        let search_result: SearchResult = response
            .json()
            .map_err(|e| RegistryError::ParseError(e.to_string()))?;

        Ok(search_result.packages)
    }

    /// Get package information
    pub fn get_package(&self, name: &str, version: Option<&str>) -> Result<PackageInfo, RegistryError> {
        let url = if let Some(version) = version {
            format!("{}/api/v1/packages/{}/{}", self.base_url, name, version)
        } else {
            format!("{}/api/v1/packages/{}", self.base_url, name)
        };

        let response = self.client
            .get(&url)
            .send()
            .map_err(|e| RegistryError::NetworkError(e.to_string()))?;

        if response.status().as_u16() == 404 {
            return Err(RegistryError::PackageNotFound(name.to_string()));
        }

        if !response.status().is_success() {
            return Err(RegistryError::ApiError(format!(
                "Get package failed with status: {}", 
                response.status()
            )));
        }

        let package_info: PackageInfo = response
            .json()
            .map_err(|e| RegistryError::ParseError(e.to_string()))?;

        Ok(package_info)
    }

    /// List all versions of a package
    pub fn get_versions(&self, name: &str) -> Result<Vec<String>, RegistryError> {
        let url = format!("{}/api/v1/packages/{}/versions", self.base_url, name);

        let response = self.client
            .get(&url)
            .send()
            .map_err(|e| RegistryError::NetworkError(e.to_string()))?;

        if response.status().as_u16() == 404 {
            return Err(RegistryError::PackageNotFound(name.to_string()));
        }

        if !response.status().is_success() {
            return Err(RegistryError::ApiError(format!(
                "Get versions failed with status: {}", 
                response.status()
            )));
        }

        let versions_result: VersionsResult = response
            .json()
            .map_err(|e| RegistryError::ParseError(e.to_string()))?;

        Ok(versions_result.versions)
    }

    /// Download package
    pub fn download_package(&self, name: &str, version: &str) -> Result<Vec<u8>, RegistryError> {
        let package_info = self.get_package(name, Some(version))?;
        
        let response = self.client
            .get(&package_info.download_url)
            .send()
            .map_err(|e| RegistryError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(RegistryError::ApiError(format!(
                "Download failed with status: {}", 
                response.status()
            )));
        }

        let bytes = response
            .bytes()
            .map_err(|e| RegistryError::NetworkError(e.to_string()))?;

        // Verify checksum
        let actual_checksum = sha2::Sha256::digest(&bytes);
        let expected_checksum = hex::decode(&package_info.checksum)
            .map_err(|e| RegistryError::ParseError(format!("Invalid checksum: {}", e)))?;

        if actual_checksum.as_slice() != expected_checksum.as_slice() {
            return Err(RegistryError::ChecksumMismatch);
        }

        Ok(bytes.to_vec())
    }

    /// Publish a package (requires authentication)
    pub fn publish_package(&self, _package_data: &[u8], _auth_token: &str) -> Result<(), RegistryError> {
        // TODO: Implement package publishing
        Err(RegistryError::NotImplemented("Package publishing not yet implemented".to_string()))
    }
}

/// Registry API response for search
#[derive(Debug, Deserialize)]
struct SearchResult {
    packages: Vec<PackageInfo>,
    total: u32,
}

/// Registry API response for versions
#[derive(Debug, Deserialize)]
struct VersionsResult {
    versions: Vec<String>,
}

/// Registry errors
#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("API error: {0}")]
    ApiError(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Package not found: {0}")]
    PackageNotFound(String),
    
    #[error("Checksum mismatch")]
    ChecksumMismatch,
    
    #[error("Not implemented: {0}")]
    NotImplemented(String),
}