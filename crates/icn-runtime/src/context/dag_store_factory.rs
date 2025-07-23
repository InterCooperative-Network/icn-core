//! Factory for creating appropriate DAG storage backends based on environment and configuration.
//!
//! This module ensures that:
//! - Production contexts never accidentally use stub implementations
//! - Development contexts can use either real or stub implementations based on configuration
//! - Testing contexts default to stubs but can use real backends when needed
//! - Backend selection is based on available features and user preferences

use super::dag_store_wrapper::{DagStoreType, DagStoreWrapper};
use super::stubs::StubDagStore;
use super::{DagStorageService, DagStoreMutexType};
use icn_common::CommonError;
use icn_dag::compat::CompatAsyncStore;
use std::path::PathBuf;
use std::sync::Arc;

/// Configuration for DAG storage backend selection
#[derive(Debug, Clone)]
pub struct DagStoreConfig {
    /// Storage backend type
    pub backend: DagStoreBackend,
    /// Path to the storage directory/file
    pub storage_path: PathBuf,
    /// Additional configuration options
    pub options: DagStoreOptions,
}

/// Available DAG storage backends
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DagStoreBackend {
    /// Persistent Sled database (default for production)
    #[cfg(feature = "persist-sled")]
    Sled,
    /// RocksDB backend (high performance)
    #[cfg(feature = "persist-rocksdb")]
    RocksDB,
    /// SQLite backend (single file)
    #[cfg(feature = "persist-sqlite")]
    SQLite,
    /// PostgreSQL backend (multi-node)
    #[cfg(feature = "persist-postgres")]
    PostgreSQL,
    /// In-memory stub (testing only)
    Stub,
}

/// Additional configuration options for DAG stores
#[derive(Debug, Clone)]
pub struct DagStoreOptions {
    /// Enable compression for stored data
    pub enable_compression: bool,
    /// Cache size limit (if supported by backend)
    pub cache_size_mb: Option<usize>,
    /// Connection pool size (for database backends)
    pub pool_size: Option<usize>,
}

impl Default for DagStoreOptions {
    fn default() -> Self {
        Self {
            enable_compression: true,
            cache_size_mb: Some(128),
            pool_size: Some(10),
        }
    }
}

impl DagStoreConfig {
    /// Create a production configuration with Sled backend (default)
    pub fn production(storage_path: PathBuf) -> Result<Self, CommonError> {
        // Default to Sled for production as it's included in default features
        #[cfg(feature = "persist-sled")]
        {
            Ok(Self {
                backend: DagStoreBackend::Sled,
                storage_path,
                options: DagStoreOptions::default(),
            })
        }

        #[cfg(not(feature = "persist-sled"))]
        {
            // If Sled is not available, try RocksDB
            #[cfg(feature = "persist-rocksdb")]
            {
                Ok(Self {
                    backend: DagStoreBackend::RocksDB,
                    storage_path,
                    options: DagStoreOptions::default(),
                })
            }

            #[cfg(not(feature = "persist-rocksdb"))]
            {
                // If neither Sled nor RocksDB are available, try SQLite
                #[cfg(feature = "persist-sqlite")]
                {
                    Ok(Self {
                        backend: DagStoreBackend::SQLite,
                        storage_path,
                        options: DagStoreOptions::default(),
                    })
                }

                #[cfg(not(feature = "persist-sqlite"))]
                {
                    Err(CommonError::ConfigError(
                        "No persistent DAG storage backend available. Enable one of: persist-sled, persist-rocksdb, persist-sqlite".to_string()
                    ))
                }
            }
        }
    }

    /// Create a development configuration (allows stub fallback)
    pub fn development(storage_path: Option<PathBuf>) -> Self {
        if let Some(path) = storage_path {
            // If path is provided, use production config
            Self::production(path).unwrap_or_else(|_| Self::testing())
        } else {
            // No path provided, use stub for development convenience
            Self::testing()
        }
    }

    /// Create a testing configuration (stub backend)
    pub fn testing() -> Self {
        Self {
            backend: DagStoreBackend::Stub,
            storage_path: PathBuf::from("/tmp/stub"),
            options: DagStoreOptions::default(),
        }
    }

    /// Validate that this configuration is appropriate for production
    pub fn validate_for_production(&self) -> Result<(), CommonError> {
        match self.backend {
            DagStoreBackend::Stub => Err(CommonError::InternalError(
                "❌ PRODUCTION ERROR: Stub DAG store cannot be used in production contexts. Use a persistent backend like Sled, RocksDB, SQLite, or PostgreSQL.".to_string()
            )),
            _ => Ok(())
        }
    }
}

/// Factory for creating DAG storage backends
pub struct DagStoreFactory;

impl DagStoreFactory {
    /// Create a DAG store based on configuration
    pub fn create(config: &DagStoreConfig) -> Result<DagStoreWrapper, CommonError> {
        match config.backend {
            #[cfg(feature = "persist-sled")]
            DagStoreBackend::Sled => {
                let store = icn_dag::sled_store::SledDagStore::new(config.storage_path.clone())?;
                let compat_store = CompatAsyncStore::new(store);
                let wrapped_store = Arc::new(DagStoreMutexType::new(compat_store));
                Ok(DagStoreWrapper::production(wrapped_store, DagStoreType::Sled))
            }

            #[cfg(feature = "persist-rocksdb")]
            DagStoreBackend::RocksDB => {
                let store = icn_dag::rocksdb_store::RocksDagStore::new(config.storage_path.clone())?;
                let compat_store = CompatAsyncStore::new(store);
                let wrapped_store = Arc::new(DagStoreMutexType::new(compat_store));
                Ok(DagStoreWrapper::production(wrapped_store, DagStoreType::RocksDB))
            }

            #[cfg(feature = "persist-sqlite")]
            DagStoreBackend::SQLite => {
                let store = icn_dag::sqlite_store::SqliteDagStore::new(config.storage_path.clone())?;
                let wrapped_store = Arc::new(DagStoreMutexType::new(store));
                Ok(DagStoreWrapper::production(wrapped_store, DagStoreType::SQLite))
            }

            #[cfg(feature = "persist-postgres")]
            DagStoreBackend::PostgreSQL => {
                // For PostgreSQL, the storage_path is used as connection string or config path
                let config_str = config.storage_path.to_string_lossy().to_string();
                let store = icn_dag::postgres_store::PostgresDagStore::new(&config_str)?;
                let wrapped_store = Arc::new(DagStoreMutexType::new(store));
                Ok(DagStoreWrapper::production(wrapped_store, DagStoreType::PostgreSQL))
            }

            DagStoreBackend::Stub => {
                let store = StubDagStore::new();
                let wrapped_store = Arc::new(DagStoreMutexType::new(store));
                Ok(DagStoreWrapper::stub(wrapped_store))
            }

            // Handle disabled features
            #[cfg(not(feature = "persist-sled"))]
            DagStoreBackend::Sled => Err(CommonError::ConfigError(
                "Sled backend not available. Enable 'persist-sled' feature.".to_string()
            )),

            #[cfg(not(feature = "persist-rocksdb"))]
            DagStoreBackend::RocksDB => Err(CommonError::ConfigError(
                "RocksDB backend not available. Enable 'persist-rocksdb' feature.".to_string()
            )),

            #[cfg(not(feature = "persist-sqlite"))]
            DagStoreBackend::SQLite => Err(CommonError::ConfigError(
                "SQLite backend not available. Enable 'persist-sqlite' feature.".to_string()
            )),

            #[cfg(not(feature = "persist-postgres"))]
            DagStoreBackend::PostgreSQL => Err(CommonError::ConfigError(
                "PostgreSQL backend not available. Enable 'persist-postgres' feature.".to_string()
            )),
        }
    }

    /// Create a production DAG store with default settings
    pub fn create_production(storage_path: PathBuf) -> Result<DagStoreWrapper, CommonError> {
        let config = DagStoreConfig::production(storage_path)?;
        config.validate_for_production()?;
        Self::create(&config)
    }

    /// Create a development DAG store with fallback to stub
    pub fn create_development(storage_path: Option<PathBuf>) -> Result<DagStoreWrapper, CommonError> {
        let config = DagStoreConfig::development(storage_path);
        Self::create(&config)
    }

    /// Create a testing DAG store (always stub)
    pub fn create_testing() -> DagStoreWrapper {
        let config = DagStoreConfig::testing();
        Self::create(&config).expect("Stub DAG store creation should never fail")
    }

    /// List available backends based on enabled features
    pub fn available_backends() -> Vec<DagStoreBackend> {
        let mut backends = vec![DagStoreBackend::Stub];

        #[cfg(feature = "persist-sled")]
        backends.push(DagStoreBackend::Sled);

        #[cfg(feature = "persist-rocksdb")]
        backends.push(DagStoreBackend::RocksDB);

        #[cfg(feature = "persist-sqlite")]
        backends.push(DagStoreBackend::SQLite);

        #[cfg(feature = "persist-postgres")]
        backends.push(DagStoreBackend::PostgreSQL);

        backends
    }

    /// Get the recommended production backend based on available features
    pub fn recommended_production_backend() -> Result<DagStoreBackend, CommonError> {
        // Preference order: Sled (stable, fast) > RocksDB (high performance) > SQLite (simple) > PostgreSQL (distributed)
        #[cfg(feature = "persist-sled")]
        return Ok(DagStoreBackend::Sled);

        #[cfg(feature = "persist-rocksdb")]
        return Ok(DagStoreBackend::RocksDB);

        #[cfg(feature = "persist-sqlite")]
        return Ok(DagStoreBackend::SQLite);

        #[cfg(feature = "persist-postgres")]
        return Ok(DagStoreBackend::PostgreSQL);

        Err(CommonError::ConfigError(
            "No persistent DAG storage backend available for production. Enable at least one of: persist-sled, persist-rocksdb, persist-sqlite, persist-postgres".to_string()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_available_backends() {
        let backends = DagStoreFactory::available_backends();
        assert!(!backends.is_empty());
        assert!(backends.contains(&DagStoreBackend::Stub));
    }

    #[test]
    fn test_testing_config() {
        let config = DagStoreConfig::testing();
        assert_eq!(config.backend, DagStoreBackend::Stub);
        
        let store_wrapper = DagStoreFactory::create_testing();
        assert!(store_wrapper.is_stub());
        assert!(!store_wrapper.is_production_ready());
        assert!(store_wrapper.inner().try_lock().is_ok());
    }

    #[test]
    fn test_production_config_validation() {
        let temp_dir = tempdir().unwrap();
        
        // Valid production config
        if let Ok(config) = DagStoreConfig::production(temp_dir.path().to_path_buf()) {
            assert!(config.validate_for_production().is_ok());
        }
        
        // Invalid production config (stub)
        let stub_config = DagStoreConfig::testing();
        assert!(stub_config.validate_for_production().is_err());
    }

    #[cfg(feature = "persist-sled")]
    #[test]
    fn test_sled_store_creation() {
        let temp_dir = tempdir().unwrap();
        let config = DagStoreConfig {
            backend: DagStoreBackend::Sled,
            storage_path: temp_dir.path().to_path_buf(),
            options: DagStoreOptions::default(),
        };
        
        let store_wrapper = DagStoreFactory::create(&config);
        assert!(store_wrapper.is_ok());
        let wrapper = store_wrapper.unwrap();
        assert!(wrapper.is_production_ready());
        assert!(!wrapper.is_stub());
        assert_eq!(wrapper.get_type(), DagStoreType::Sled);
    }
}