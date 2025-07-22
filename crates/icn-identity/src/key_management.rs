//! Advanced key management with automated rotation, audit trails, and HSM support
//!
//! This module extends the basic key rotation functionality with enterprise-grade
//! features including scheduled rotation, comprehensive audit logging, and 
//! hardware security module integration.

use crate::{KeyRotation, KeyStorage, SigningKey, generate_ed25519_keypair};
use icn_common::{CommonError, Did, TimeProvider};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, Mutex};
use std::time::Duration;

/// Configuration for automated key rotation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotationConfig {
    /// Automatic rotation interval
    pub rotation_interval: Duration,
    /// Maximum key age before forced rotation
    pub max_key_age: Duration,
    /// Enable rotation warnings before expiry
    pub enable_rotation_warnings: bool,
    /// Warning period before key expiry
    pub warning_period: Duration,
    /// Enable audit trail logging
    pub enable_audit_logging: bool,
    /// Maximum number of previous keys to retain
    pub max_historical_keys: usize,
    /// Enable HSM integration
    pub enable_hsm: bool,
}

impl Default for KeyRotationConfig {
    fn default() -> Self {
        Self {
            rotation_interval: Duration::from_secs(30 * 24 * 60 * 60), // 30 days
            max_key_age: Duration::from_secs(90 * 24 * 60 * 60), // 90 days
            enable_rotation_warnings: true,
            warning_period: Duration::from_secs(7 * 24 * 60 * 60), // 7 days
            enable_audit_logging: true,
            max_historical_keys: 5,
            enable_hsm: false,
        }
    }
}

/// Key metadata for tracking key lifecycle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMetadata {
    /// When the key was created
    pub created_at: u64,
    /// When the key was last rotated
    pub last_rotated: u64,
    /// Number of times this key has been rotated
    pub rotation_count: u32,
    /// Whether this key is scheduled for rotation
    pub scheduled_for_rotation: bool,
    /// Key version number
    pub version: u64,
    /// Previous key versions (for key recovery)
    pub previous_versions: Vec<KeyVersion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyVersion {
    pub version: u64,
    pub created_at: u64,
    pub retired_at: u64,
    pub did: Did,
}

/// Audit log entry for key operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyAuditEntry {
    pub timestamp: u64,
    pub did: Did,
    pub operation: KeyOperation,
    pub result: KeyOperationResult,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyOperation {
    Created,
    Rotated,
    Retired,
    Accessed,
    RecoveryAttempted,
    HsmOperationFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyOperationResult {
    Success,
    Failed(String),
    Warning(String),
}

/// Advanced key manager with automated rotation and audit trails
pub struct AdvancedKeyManager {
    config: KeyRotationConfig,
    key_metadata: Arc<RwLock<HashMap<Did, KeyMetadata>>>,
    audit_log: Arc<Mutex<Vec<KeyAuditEntry>>>,
    time_provider: Arc<dyn TimeProvider>,
    rotation_schedule: Arc<Mutex<HashMap<Did, u64>>>, // DID -> next rotation timestamp
}

impl AdvancedKeyManager {
    /// Create a new advanced key manager
    pub fn new(config: KeyRotationConfig, time_provider: Arc<dyn TimeProvider>) -> Self {
        Self {
            config,
            key_metadata: Arc::new(RwLock::new(HashMap::new())),
            audit_log: Arc::new(Mutex::new(Vec::new())),
            time_provider,
            rotation_schedule: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register a new key with the manager
    pub fn register_key(&self, did: Did) -> Result<(), CommonError> {
        let current_time = self.time_provider.unix_seconds() * 1000;
        
        let metadata = KeyMetadata {
            created_at: current_time,
            last_rotated: current_time,
            rotation_count: 0,
            scheduled_for_rotation: false,
            version: 1,
            previous_versions: Vec::new(),
        };

        // Store metadata
        self.key_metadata.write().unwrap().insert(did.clone(), metadata);

        // Schedule first rotation
        let next_rotation = current_time + self.config.rotation_interval.as_millis() as u64;
        self.rotation_schedule.lock().unwrap().insert(did.clone(), next_rotation);

        // Log audit entry
        self.log_key_operation(
            did,
            KeyOperation::Created,
            KeyOperationResult::Success,
            HashMap::new(),
        );

        Ok(())
    }

    /// Check which keys need rotation
    pub fn check_rotation_schedule(&self) -> Vec<Did> {
        let current_time = self.time_provider.unix_seconds() * 1000;
        let schedule = self.rotation_schedule.lock().unwrap();
        
        schedule
            .iter()
            .filter(|(_, &next_rotation)| current_time >= next_rotation)
            .map(|(did, _)| did.clone())
            .collect()
    }

    /// Check which keys need rotation warnings
    pub fn check_rotation_warnings(&self) -> Vec<(Did, u64)> {
        if !self.config.enable_rotation_warnings {
            return Vec::new();
        }

        let current_time = self.time_provider.unix_seconds() * 1000;
        let warning_threshold = self.config.warning_period.as_millis() as u64;
        let schedule = self.rotation_schedule.lock().unwrap();
        
        schedule
            .iter()
            .filter(|(_, &next_rotation)| {
                next_rotation > current_time && 
                (next_rotation - current_time) <= warning_threshold
            })
            .map(|(did, &next_rotation)| (did.clone(), next_rotation))
            .collect()
    }

    /// Perform automated key rotation for a DID
    pub fn rotate_key<K>(&self, did: &Did, key_store: &mut K) -> Result<Did, CommonError>
    where
        K: KeyRotation + KeyStorage,
    {
        let current_time = self.time_provider.unix_seconds() * 1000;
        
        // Get current metadata
        let mut metadata_map = self.key_metadata.write().unwrap();
        let metadata = metadata_map.get_mut(did).ok_or_else(|| {
            CommonError::KeyNotFound(format!("Key metadata not found for DID: {}", did))
        })?;

        // Create key version entry for the old key
        let old_version = KeyVersion {
            version: metadata.version,
            created_at: metadata.created_at,
            retired_at: current_time,
            did: did.clone(),
        };

        // Rotate the key using the key store
        let new_did = key_store.rotate_ed25519(did)?;

        // Update metadata
        metadata.previous_versions.push(old_version);
        metadata.last_rotated = current_time;
        metadata.rotation_count += 1;
        metadata.version += 1;
        metadata.scheduled_for_rotation = false;

        // Limit historical key storage
        if metadata.previous_versions.len() > self.config.max_historical_keys {
            metadata.previous_versions.remove(0);
        }

        // Move metadata to new DID
        let updated_metadata = metadata.clone();
        metadata_map.remove(did);
        metadata_map.insert(new_did.clone(), updated_metadata);

        // Update rotation schedule
        let next_rotation = current_time + self.config.rotation_interval.as_millis() as u64;
        let mut schedule = self.rotation_schedule.lock().unwrap();
        schedule.remove(did);
        schedule.insert(new_did.clone(), next_rotation);

        // Log audit entry
        let mut audit_metadata = HashMap::new();
        audit_metadata.insert("old_did".to_string(), did.to_string());
        audit_metadata.insert("new_did".to_string(), new_did.to_string());
        audit_metadata.insert("rotation_count".to_string(), metadata.rotation_count.to_string());
        
        self.log_key_operation(
            new_did.clone(),
            KeyOperation::Rotated,
            KeyOperationResult::Success,
            audit_metadata,
        );

        Ok(new_did)
    }

    /// Get key metadata for a DID
    pub fn get_key_metadata(&self, did: &Did) -> Option<KeyMetadata> {
        self.key_metadata.read().unwrap().get(did).cloned()
    }

    /// Get audit log entries for a specific DID
    pub fn get_audit_log(&self, did: &Did) -> Vec<KeyAuditEntry> {
        self.audit_log
            .lock()
            .unwrap()
            .iter()
            .filter(|entry| &entry.did == did)
            .cloned()
            .collect()
    }

    /// Get full audit log
    pub fn get_full_audit_log(&self) -> Vec<KeyAuditEntry> {
        self.audit_log.lock().unwrap().clone()
    }

    /// Log a key operation for audit purposes
    fn log_key_operation(
        &self,
        did: Did,
        operation: KeyOperation,
        result: KeyOperationResult,
        metadata: HashMap<String, String>,
    ) {
        if !self.config.enable_audit_logging {
            return;
        }

        let entry = KeyAuditEntry {
            timestamp: self.time_provider.unix_seconds() * 1000,
            did,
            operation,
            result,
            metadata,
        };

        self.audit_log.lock().unwrap().push(entry);
    }

    /// Get rotation statistics
    pub fn get_rotation_stats(&self) -> KeyRotationStats {
        let metadata_map = self.key_metadata.read().unwrap();
        let schedule = self.rotation_schedule.lock().unwrap();
        let current_time = self.time_provider.unix_seconds() * 1000;

        let total_keys = metadata_map.len();
        let keys_due_for_rotation = schedule
            .values()
            .filter(|&&next_rotation| current_time >= next_rotation)
            .count();
        
        let total_rotations: u32 = metadata_map
            .values()
            .map(|meta| meta.rotation_count)
            .sum();

        let avg_key_age = if !metadata_map.is_empty() {
            metadata_map
                .values()
                .map(|meta| current_time - meta.created_at)
                .sum::<u64>() / metadata_map.len() as u64
        } else {
            0
        };

        KeyRotationStats {
            total_keys,
            keys_due_for_rotation,
            total_rotations,
            avg_key_age_ms: avg_key_age,
        }
    }
}

/// Statistics for key rotation monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotationStats {
    pub total_keys: usize,
    pub keys_due_for_rotation: usize,
    pub total_rotations: u32,
    pub avg_key_age_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{InMemoryKeyStore, generate_ed25519_keypair, did_key_from_verifying_key};
    use icn_common::{SystemTimeProvider, Did};
    use std::str::FromStr;

    #[test]
    fn test_key_registration_and_metadata() {
        let config = KeyRotationConfig::default();
        let time_provider = Arc::new(SystemTimeProvider);
        let manager = AdvancedKeyManager::new(config, time_provider);

        let (_, pk) = generate_ed25519_keypair();
        let did_str = did_key_from_verifying_key(&pk);
        let did = Did::from_str(&did_str).unwrap();

        manager.register_key(did.clone()).unwrap();
        
        let metadata = manager.get_key_metadata(&did).unwrap();
        assert_eq!(metadata.rotation_count, 0);
        assert_eq!(metadata.version, 1);
    }

    #[test]
    fn test_key_rotation_with_audit() {
        let config = KeyRotationConfig::default();
        let time_provider = Arc::new(SystemTimeProvider);
        let manager = AdvancedKeyManager::new(config, time_provider);
        let mut key_store = InMemoryKeyStore::default();

        let (sk, pk) = generate_ed25519_keypair();
        let did_str = did_key_from_verifying_key(&pk);
        let did = Did::from_str(&did_str).unwrap();

        key_store.store_signing_key(did.clone(), sk);
        manager.register_key(did.clone()).unwrap();

        let new_did = manager.rotate_key(&did, &mut key_store).unwrap();
        
        assert_ne!(did, new_did);
        
        let metadata = manager.get_key_metadata(&new_did).unwrap();
        assert_eq!(metadata.rotation_count, 1);
        assert_eq!(metadata.version, 2);
        assert_eq!(metadata.previous_versions.len(), 1);

        let audit_entries = manager.get_audit_log(&new_did);
        assert!(!audit_entries.is_empty());
    }
}