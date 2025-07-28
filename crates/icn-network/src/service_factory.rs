//! Network service factory for ICN
//!
//! This module provides a factory for creating appropriate network services
//! based on environment, configuration, and available features.

#[cfg(feature = "libp2p")]
use crate::libp2p_service::{Libp2pNetworkService, NetworkConfig};
use crate::{MeshNetworkError, NetworkService, StubNetworkService};
use icn_common::TimeProvider;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

/// Network service environment types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkEnvironment {
    /// Production environment - requires real networking
    Production,
    /// Development environment - prefers real networking but allows fallbacks
    Development,
    /// Testing environment - may use stub services
    Testing,
    /// Benchmarking environment - optimized for performance testing
    Benchmarking,
}

/// Network service creation options
#[derive(Clone)]
pub struct NetworkServiceOptions {
    /// Target environment
    pub environment: NetworkEnvironment,
    /// Network configuration
    pub config: Option<NetworkServiceConfig>,
    /// Whether to allow fallback to stub services
    pub allow_fallback: bool,
    /// Time provider for network operations
    pub time_provider: Option<Arc<dyn TimeProvider>>,
    /// Custom network identifier for isolation
    pub network_id: Option<String>,
}

impl Default for NetworkServiceOptions {
    fn default() -> Self {
        Self {
            environment: NetworkEnvironment::Development,
            config: None,
            allow_fallback: true,
            time_provider: None,
            network_id: None,
        }
    }
}

/// Unified network service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkServiceConfig {
    /// Listen addresses
    pub listen_addresses: Vec<String>,
    /// Bootstrap peers
    pub bootstrap_peers: Vec<BootstrapPeer>,
    /// Maximum number of peers
    pub max_peers: usize,
    /// Maximum peers per IP
    pub max_peers_per_ip: usize,
    /// Connection timeout in milliseconds
    pub connection_timeout_ms: u64,
    /// Request timeout in milliseconds
    pub request_timeout_ms: u64,
    /// Heartbeat interval in milliseconds
    pub heartbeat_interval_ms: u64,
    /// Bootstrap interval in seconds
    pub bootstrap_interval_secs: u64,
    /// Peer discovery interval in seconds
    pub peer_discovery_interval_secs: u64,
    /// Enable mDNS discovery
    pub enable_mdns: bool,
    /// Kademlia replication factor
    pub kademlia_replication_factor: usize,
    /// Custom network protocol ID
    pub protocol_id: Option<String>,
}

impl Default for NetworkServiceConfig {
    fn default() -> Self {
        Self {
            listen_addresses: vec!["/ip4/0.0.0.0/tcp/0".to_string()],
            bootstrap_peers: Vec::new(),
            max_peers: 100,
            max_peers_per_ip: 5,
            connection_timeout_ms: 30000,
            request_timeout_ms: 10000,
            heartbeat_interval_ms: 15000,
            bootstrap_interval_secs: 300,
            peer_discovery_interval_secs: 60,
            enable_mdns: false,
            kademlia_replication_factor: 20,
            protocol_id: None,
        }
    }
}

/// Bootstrap peer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapPeer {
    /// Peer ID
    pub peer_id: String,
    /// Multiaddress
    pub address: String,
    /// Optional priority weight
    pub weight: Option<u32>,
    /// Whether this peer is trusted
    pub trusted: bool,
}

/// Result of network service creation
#[derive(Debug)]
pub enum NetworkServiceCreationResult {
    /// Successfully created libp2p service
    Libp2p(Arc<dyn NetworkService>),
    /// Fallback to stub service
    Stub(Arc<dyn NetworkService>),
    /// Creation failed
    Failed(MeshNetworkError),
}

/// Factory for creating network services
pub struct NetworkServiceFactory;

impl NetworkServiceFactory {
    /// Create a network service based on options
    pub async fn create(options: NetworkServiceOptions) -> NetworkServiceCreationResult {
        match options.environment {
            NetworkEnvironment::Production => Self::create_production_service(options).await,
            NetworkEnvironment::Development => Self::create_development_service(options).await,
            NetworkEnvironment::Testing => Self::create_testing_service(options).await,
            NetworkEnvironment::Benchmarking => Self::create_benchmarking_service(options).await,
        }
    }

    /// Create production network service (must be real libp2p)
    async fn create_production_service(
        options: NetworkServiceOptions,
    ) -> NetworkServiceCreationResult {
        #[cfg(feature = "libp2p")]
        {
            let config = options.config.clone().unwrap_or_default();
            let libp2p_config = Self::convert_to_libp2p_config(config, &options);

            match Libp2pNetworkService::new(libp2p_config).await {
                Ok(service) => {
                    log::info!("✅ Production libp2p network service created successfully");
                    NetworkServiceCreationResult::Libp2p(Arc::new(service))
                }
                Err(e) => {
                    log::error!("❌ Failed to create production libp2p service: {}", e);
                    if options.allow_fallback {
                        log::warn!("🔄 Production fallback to stub service (not recommended)");
                        NetworkServiceCreationResult::Stub(Arc::new(StubNetworkService::default()))
                    } else {
                        NetworkServiceCreationResult::Failed(e)
                    }
                }
            }
        }

        #[cfg(not(feature = "libp2p"))]
        {
            let error = MeshNetworkError::SetupError(
                "Production environment requires libp2p feature to be enabled".to_string(),
            );

            if options.allow_fallback {
                log::error!("❌ libp2p not available in production, using stub service");
                NetworkServiceCreationResult::Stub(Arc::new(StubNetworkService::default()))
            } else {
                NetworkServiceCreationResult::Failed(error)
            }
        }
    }

    /// Create development network service (prefers real, allows fallback)
    async fn create_development_service(
        options: NetworkServiceOptions,
    ) -> NetworkServiceCreationResult {
        #[cfg(feature = "libp2p")]
        {
            let config = options.config.clone().unwrap_or_else(|| {
                let mut cfg = NetworkServiceConfig::default();
                cfg.enable_mdns = true; // Enable mDNS for local development
                cfg.max_peers = 50; // Smaller peer limit for development
                
                // Add default development bootstrap peers if none provided
                if cfg.bootstrap_peers.is_empty() {
                    cfg.bootstrap_peers = Self::get_default_bootstrap_peers_for_env(NetworkEnvironment::Development);
                }
                
                cfg
            });

            let libp2p_config = Self::convert_to_libp2p_config(config, &options);

            match Libp2pNetworkService::new(libp2p_config).await {
                Ok(service) => {
                    log::info!("✅ Development libp2p network service created");
                    NetworkServiceCreationResult::Libp2p(Arc::new(service))
                }
                Err(e) => {
                    log::warn!("⚠️ Failed to create libp2p service for development: {}", e);
                    log::info!("🔄 Falling back to stub service for development");
                    NetworkServiceCreationResult::Stub(Arc::new(StubNetworkService::default()))
                }
            }
        }

        #[cfg(not(feature = "libp2p"))]
        {
            log::info!("🔄 Development using stub service (libp2p not available)");
            NetworkServiceCreationResult::Stub(Arc::new(StubNetworkService::default()))
        }
    }

    /// Create testing network service (prefers stub for speed)
    async fn create_testing_service(
        _options: NetworkServiceOptions,
    ) -> NetworkServiceCreationResult {
        // For testing, we usually want fast, deterministic stub services
        log::info!("🧪 Creating stub network service for testing");
        NetworkServiceCreationResult::Stub(Arc::new(StubNetworkService::default()))
    }

    /// Create benchmarking network service (optimized for performance)
    async fn create_benchmarking_service(
        options: NetworkServiceOptions,
    ) -> NetworkServiceCreationResult {
        #[cfg(feature = "libp2p")]
        {
            let config = options.config.clone().unwrap_or_else(|| {
                let mut cfg = NetworkServiceConfig::default();
                cfg.max_peers = 1000; // Higher peer limit for benchmarking
                cfg.connection_timeout_ms = 5000; // Faster timeouts
                cfg.request_timeout_ms = 2000;
                cfg.heartbeat_interval_ms = 30000; // Less frequent heartbeats
                cfg.enable_mdns = false; // Disable mDNS for cleaner benchmarks
                cfg
            });

            let libp2p_config = Self::convert_to_libp2p_config(config, &options);

            match Libp2pNetworkService::new(libp2p_config).await {
                Ok(service) => {
                    log::info!("✅ Benchmarking libp2p network service created");
                    NetworkServiceCreationResult::Libp2p(Arc::new(service))
                }
                Err(e) => {
                    log::error!("❌ Failed to create libp2p service for benchmarking: {}", e);
                    NetworkServiceCreationResult::Failed(e)
                }
            }
        }

        #[cfg(not(feature = "libp2p"))]
        {
            NetworkServiceCreationResult::Failed(MeshNetworkError::SetupError(
                "Benchmarking requires libp2p feature to be enabled".to_string(),
            ))
        }
    }

    /// Convert unified config to libp2p config
    #[cfg(feature = "libp2p")]
    fn convert_to_libp2p_config(
        config: NetworkServiceConfig,
        _options: &NetworkServiceOptions,
    ) -> NetworkConfig {
        use libp2p::{Multiaddr, PeerId};
        use std::str::FromStr;

        // Parse listen addresses
        let listen_addresses = config
            .listen_addresses
            .iter()
            .filter_map(|addr_str| addr_str.parse::<Multiaddr>().ok())
            .collect();

        // Parse bootstrap peers - only include those with valid peer IDs
        let mut bootstrap_peers = Vec::new();
        let mut discovery_addresses = Vec::new();
        
        for peer in &config.bootstrap_peers {
            if let Ok(multiaddr) = peer.address.parse::<Multiaddr>() {
                if !peer.peer_id.is_empty() {
                    // Explicit peer ID provided - validate it
                    if let Ok(peer_id) = PeerId::from_str(&peer.peer_id) {
                        bootstrap_peers.push((peer_id, multiaddr.clone()));
                        log::debug!("Added bootstrap peer with known ID {} at {}", peer_id, multiaddr);
                    } else {
                        log::warn!("Invalid peer ID for bootstrap peer: {}", peer.peer_id);
                    }
                } else {
                    // No peer ID provided - try to extract from multiaddr
                    match Self::extract_peer_id_from_multiaddr(&multiaddr) {
                        Some(peer_id) => {
                            bootstrap_peers.push((peer_id, multiaddr.clone()));
                            log::debug!("Extracted peer ID {} from multiaddr {}", peer_id, multiaddr);
                        }
                        None => {
                            // No peer ID available - add to discovery addresses instead
                            // These will be dialed for initial connection but NOT added to Kademlia
                            discovery_addresses.push(multiaddr.clone());
                            log::debug!(
                                "Added discovery address {} - will dial but not add to DHT until peer ID is known",
                                multiaddr
                            );
                        }
                    }
                }
            } else {
                log::warn!("Invalid bootstrap peer multiaddr: {}", peer.address);
            }
        }
        
        if bootstrap_peers.is_empty() && discovery_addresses.is_empty() && !config.bootstrap_peers.is_empty() {
            log::warn!("No valid bootstrap peers or discovery addresses could be parsed from configuration");
        } else {
            if !bootstrap_peers.is_empty() {
                log::info!("Successfully configured {} bootstrap peer(s) with known IDs", bootstrap_peers.len());
            }
            if !discovery_addresses.is_empty() {
                log::info!("Successfully configured {} discovery address(es) for initial connection", discovery_addresses.len());
            }
        }

        NetworkConfig {
            listen_addresses,
            bootstrap_peers,
            discovery_addresses, // New field for addresses without known peer IDs
            max_peers: config.max_peers,
            max_peers_per_ip: config.max_peers_per_ip,
            connection_timeout: Duration::from_millis(config.connection_timeout_ms),
            request_timeout: Duration::from_millis(config.request_timeout_ms),
            heartbeat_interval: Duration::from_millis(config.heartbeat_interval_ms),
            bootstrap_interval: Duration::from_secs(config.bootstrap_interval_secs),
            peer_discovery_interval: Duration::from_secs(config.peer_discovery_interval_secs),
            enable_mdns: config.enable_mdns,
            kademlia_replication_factor: config.kademlia_replication_factor,
        }
    }

    /// Create a service with automatic environment detection
    pub async fn create_auto() -> NetworkServiceCreationResult {
        let environment = Self::detect_environment();
        let options = NetworkServiceOptions {
            environment,
            ..Default::default()
        };
        Self::create(options).await
    }

    /// Detect appropriate environment based on compilation features and runtime context
    fn detect_environment() -> NetworkEnvironment {
        #[cfg(debug_assertions)]
        {
            // In debug builds, prefer development environment
            NetworkEnvironment::Development
        }

        #[cfg(not(debug_assertions))]
        {
            // In release builds, prefer production environment
            NetworkEnvironment::Production
        }
    }

    /// Create a service for testing with custom configuration
    pub async fn create_for_testing() -> Arc<dyn NetworkService> {
        match Self::create_testing_service(NetworkServiceOptions::default()).await {
            NetworkServiceCreationResult::Stub(service) => service,
            NetworkServiceCreationResult::Libp2p(service) => service,
            NetworkServiceCreationResult::Failed(_) => Arc::new(StubNetworkService::default()),
        }
    }

    /// Create a service for production with strict requirements
    pub async fn create_for_production(
        config: NetworkServiceConfig,
    ) -> Result<Arc<dyn NetworkService>, MeshNetworkError> {
        let options = NetworkServiceOptions {
            environment: NetworkEnvironment::Production,
            config: Some(config),
            allow_fallback: false,
            time_provider: None,
            network_id: None,
        };

        match Self::create(options).await {
            NetworkServiceCreationResult::Libp2p(service) => Ok(service),
            NetworkServiceCreationResult::Failed(error) => Err(error),
            NetworkServiceCreationResult::Stub(_) => Err(MeshNetworkError::SetupError(
                "Stub service not allowed in production".to_string(),
            )),
        }
    }

    /// Extract peer ID from multiaddr if present
    #[cfg(feature = "libp2p")]
    fn extract_peer_id_from_multiaddr(multiaddr: &libp2p::Multiaddr) -> Option<libp2p::PeerId> {
        use libp2p::multiaddr::Protocol;
        
        for protocol in multiaddr.iter() {
            if let Protocol::P2p(peer_id) = protocol {
                return Some(peer_id);
            }
        }
        None
    }

    /// Get default bootstrap peers for specific environments
    fn get_default_bootstrap_peers_for_env(env: NetworkEnvironment) -> Vec<BootstrapPeer> {
        match env {
            NetworkEnvironment::Production => {
                // In production, no default bootstrap peers - they must be explicitly configured
                Vec::new()
            }
            NetworkEnvironment::Development => {
                // For development, provide some well-known development bootstrap addresses
                vec![
                    BootstrapPeer {
                        peer_id: String::new(), // Will be discovered
                        address: "/ip4/127.0.0.1/tcp/4001".to_string(),
                        weight: Some(1),
                        trusted: true,
                    }
                ]
            }
            NetworkEnvironment::Testing => {
                // For testing, no bootstrap peers needed typically
                Vec::new()
            }
            NetworkEnvironment::Benchmarking => {
                // For benchmarking, no bootstrap peers needed typically
                Vec::new()
            }
        }
    }
}

/// Builder for network service options
pub struct NetworkServiceOptionsBuilder {
    options: NetworkServiceOptions,
}

impl NetworkServiceOptionsBuilder {
    pub fn new() -> Self {
        Self {
            options: NetworkServiceOptions::default(),
        }
    }

    pub fn environment(mut self, env: NetworkEnvironment) -> Self {
        self.options.environment = env;
        self
    }

    pub fn config(mut self, config: NetworkServiceConfig) -> Self {
        self.options.config = Some(config);
        self
    }

    pub fn allow_fallback(mut self, allow: bool) -> Self {
        self.options.allow_fallback = allow;
        self
    }

    pub fn time_provider(mut self, provider: Arc<dyn TimeProvider>) -> Self {
        self.options.time_provider = Some(provider);
        self
    }

    pub fn network_id(mut self, id: String) -> Self {
        self.options.network_id = Some(id);
        self
    }

    pub fn build(self) -> NetworkServiceOptions {
        self.options
    }

    pub async fn create(self) -> NetworkServiceCreationResult {
        NetworkServiceFactory::create(self.build()).await
    }
}

impl Default for NetworkServiceOptionsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience functions for common network service creation patterns
pub mod presets {
    use super::*;

    /// Create a local development network service
    pub async fn local_development() -> Arc<dyn NetworkService> {
        let config = NetworkServiceConfig {
            listen_addresses: vec!["/ip4/127.0.0.1/tcp/0".to_string()],
            enable_mdns: true,
            max_peers: 10,
            ..Default::default()
        };

        let options = NetworkServiceOptionsBuilder::new()
            .environment(NetworkEnvironment::Development)
            .config(config)
            .allow_fallback(true)
            .build();

        match NetworkServiceFactory::create(options).await {
            NetworkServiceCreationResult::Libp2p(service) => service,
            NetworkServiceCreationResult::Stub(service) => service,
            NetworkServiceCreationResult::Failed(_) => Arc::new(StubNetworkService::default()),
        }
    }

    /// Create a production-ready network service
    pub async fn production(
        listen_addr: &str,
        bootstrap_peers: Vec<BootstrapPeer>,
    ) -> Result<Arc<dyn NetworkService>, MeshNetworkError> {
        let config = NetworkServiceConfig {
            listen_addresses: vec![listen_addr.to_string()],
            bootstrap_peers,
            enable_mdns: false,
            max_peers: 1000,
            ..Default::default()
        };

        NetworkServiceFactory::create_for_production(config).await
    }

    /// Create a fast testing service
    pub async fn testing() -> Arc<dyn NetworkService> {
        NetworkServiceFactory::create_for_testing().await
    }
}
