//! DAG Store Wrapper with synchronous type checking capabilities.
//!
//! This module provides a wrapper around DAG stores that allows for synchronous
//! type checking without requiring async mutex access. This prevents the need
//! for the problematic `tokio::task::block_in_place` + `Handle::current().block_on()`
//! pattern that can cause deadlocks and panics.

use super::{DagStorageService, DagStoreMutexType};
use icn_common::CommonError;
use std::sync::Arc;

/// Type information for DAG stores that can be checked synchronously
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DagStoreType {
    /// Stub store (in-memory HashMap) - for testing only
    Stub,
    /// Sled store (embedded database) - production ready
    Sled,
    /// RocksDB store (high performance) - production ready
    RocksDB,
    /// SQLite store (single file) - production ready
    SQLite,
    /// PostgreSQL store (distributed) - production ready
    PostgreSQL,
    /// Generic production store (unknown specific type)
    Production,
}

impl DagStoreType {
    /// Check if this store type is appropriate for production use
    pub fn is_production_ready(&self) -> bool {
        match self {
            DagStoreType::Stub => false,
            DagStoreType::Sled
            | DagStoreType::RocksDB
            | DagStoreType::SQLite
            | DagStoreType::PostgreSQL
            | DagStoreType::Production => true,
        }
    }

    /// Check if this is a stub store
    pub fn is_stub(&self) -> bool {
        matches!(self, DagStoreType::Stub)
    }
}

/// Wrapper around DAG store that provides synchronous type checking
pub struct DagStoreWrapper {
    /// The actual DAG store behind an async mutex
    pub store: Arc<DagStoreMutexType<DagStorageService>>,
    /// Type information that can be checked synchronously
    pub store_type: DagStoreType,
}

impl DagStoreWrapper {
    /// Create a new wrapper with explicit type information
    pub fn new(store: Arc<DagStoreMutexType<DagStorageService>>, store_type: DagStoreType) -> Self {
        Self { store, store_type }
    }

    /// Create a wrapper for a stub store
    pub fn stub(store: Arc<DagStoreMutexType<DagStorageService>>) -> Self {
        Self::new(store, DagStoreType::Stub)
    }

    /// Create a wrapper for a production store with specific type
    pub fn production(
        store: Arc<DagStoreMutexType<DagStorageService>>,
        store_type: DagStoreType,
    ) -> Self {
        Self::new(store, store_type)
    }

    /// Create a wrapper for a generic production store
    pub fn generic_production(store: Arc<DagStoreMutexType<DagStorageService>>) -> Self {
        Self::new(store, DagStoreType::Production)
    }

    /// Check if this store is production ready (synchronous)
    pub fn is_production_ready(&self) -> bool {
        self.store_type.is_production_ready()
    }

    /// Check if this store is a stub (synchronous)
    pub fn is_stub(&self) -> bool {
        self.store_type.is_stub()
    }

    /// Get the store type (synchronous)
    pub fn get_type(&self) -> DagStoreType {
        self.store_type
    }

    /// Validate that this store is appropriate for production use
    pub fn validate_for_production(&self) -> Result<(), CommonError> {
        if self.is_stub() {
            Err(CommonError::InternalError(
                "❌ PRODUCTION ERROR: Stub DAG store cannot be used in production. Use DagStoreFactory::create_production() to create a persistent store.".to_string()
            ))
        } else {
            Ok(())
        }
    }

    /// Get access to the underlying store
    pub fn inner(&self) -> &Arc<DagStoreMutexType<DagStorageService>> {
        &self.store
    }

    /// Clone the underlying store reference
    pub fn clone_inner(&self) -> Arc<DagStoreMutexType<DagStorageService>> {
        self.store.clone()
    }
}

impl Clone for DagStoreWrapper {
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
            store_type: self.store_type,
        }
    }
}

impl std::fmt::Debug for DagStoreWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DagStoreWrapper")
            .field("store_type", &self.store_type)
            .finish()
    }
} 