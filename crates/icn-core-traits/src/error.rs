//! Error types for ICN Core Traits

use icn_common::CommonError;
use thiserror::Error;

/// Errors that can occur when working with core traits
#[derive(Error, Debug)]
pub enum CoreTraitsError {
    /// Network service is unavailable or failed
    #[error("Network service error: {0}")]
    NetworkService(String),
    
    /// Reputation service is unavailable or failed
    #[error("Reputation service error: {0}")]
    ReputationService(String),
    
    /// Governance service is unavailable or failed
    #[error("Governance service error: {0}")]
    GovernanceService(String),
    
    /// Economic service is unavailable or failed
    #[error("Economic service error: {0}")]
    EconomicService(String),
    
    /// Mesh service is unavailable or failed
    #[error("Mesh service error: {0}")]
    MeshService(String),
    
    /// Service provider is not available
    #[error("Service provider not available: {service_type}")]
    ServiceNotAvailable { service_type: String },
    
    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    /// Common error wrapper
    #[error("Common error: {0}")]
    Common(#[from] CommonError),
}

impl CoreTraitsError {
    /// Create a network service error
    pub fn network_service<S: Into<String>>(msg: S) -> Self {
        Self::NetworkService(msg.into())
    }
    
    /// Create a reputation service error
    pub fn reputation_service<S: Into<String>>(msg: S) -> Self {
        Self::ReputationService(msg.into())
    }
    
    /// Create a governance service error
    pub fn governance_service<S: Into<String>>(msg: S) -> Self {
        Self::GovernanceService(msg.into())
    }
    
    /// Create an economic service error
    pub fn economic_service<S: Into<String>>(msg: S) -> Self {
        Self::EconomicService(msg.into())
    }
    
    /// Create a mesh service error
    pub fn mesh_service<S: Into<String>>(msg: S) -> Self {
        Self::MeshService(msg.into())
    }
    
    /// Create a service not available error
    pub fn service_not_available<S: Into<String>>(service_type: S) -> Self {
        Self::ServiceNotAvailable {
            service_type: service_type.into(),
        }
    }
    
    /// Create a configuration error
    pub fn configuration<S: Into<String>>(msg: S) -> Self {
        Self::Configuration(msg.into())
    }
}