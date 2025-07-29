//! Enhanced DID resolution system with support for multiple methods and LRU caching
//!
//! This module provides a comprehensive DID resolution system that can handle
//! multiple DID methods efficiently with LRU caching and invalidation mechanisms.

use crate::{DidResolver, KeyDidResolver, PeerDidResolver, WebDidResolver};
use icn_common::{CommonError, Did, TimeProvider};
use lru::LruCache;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex, RwLock};

/// Cache entry for DID resolution results with access tracking
#[derive(Debug, Clone)]
struct CacheEntry {
    verifying_key: ed25519_dalek::VerifyingKey,
    expires_at: u64,
    method_used: String,
    access_count: u64,
    last_accessed: u64,
}

/// Configuration for DID resolution with enhanced cache settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidResolutionConfig {
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
    /// Maximum cache size (number of entries)
    pub max_cache_size: usize,
    /// Timeout for web-based resolution in seconds
    pub web_timeout_seconds: u64,
    /// Enable fallback to other methods if primary fails
    pub enable_fallback: bool,
    /// Preferred method order for resolution
    pub method_preference: Vec<String>,
    /// Enable cache invalidation based on access patterns
    pub enable_intelligent_invalidation: bool,
    /// Minimum access count before entry is considered "hot"
    pub hot_entry_threshold: u64,
    /// Cache hit ratio threshold for triggering cache optimization
    pub cache_optimization_threshold: f64,
}

impl Default for DidResolutionConfig {
    fn default() -> Self {
        Self {
            cache_ttl_seconds: 3600, // 1 hour
            max_cache_size: 10000,
            web_timeout_seconds: 30,
            enable_fallback: true,
            method_preference: vec!["key".to_string(), "peer".to_string(), "web".to_string()],
            enable_intelligent_invalidation: true,
            hot_entry_threshold: 5,
            cache_optimization_threshold: 0.8,
        }
    }
}

/// Enhanced DID resolver with LRU caching and multiple method support
pub struct EnhancedDidResolver {
    config: DidResolutionConfig,
    lru_cache: Arc<Mutex<LruCache<String, CacheEntry>>>,
    time_provider: Arc<dyn TimeProvider>,

    // Method-specific resolvers
    key_resolver: KeyDidResolver,
    peer_resolver: PeerDidResolver,
    web_resolver: Arc<RwLock<WebDidResolver>>,

    // Statistics
    stats: Arc<RwLock<ResolutionStats>>,
    method_stats: Arc<RwLock<HashMap<String, MethodStats>>>,
}

/// Resolution statistics for monitoring and optimization
#[derive(Debug, Default, Clone)]
pub struct ResolutionStats {
    pub total_resolutions: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub errors: HashMap<String, u64>,
    pub avg_resolution_time_ms: f64,
}

#[derive(Debug, Default, Clone)]
pub struct MethodStats {
    pub successes: u64,
    pub failures: u64,
    pub average_time_ms: f64,
}

impl EnhancedDidResolver {
    /// Create a new enhanced DID resolver with LRU caching
    pub fn new(config: DidResolutionConfig, time_provider: Arc<dyn TimeProvider>) -> Self {
        let cache_size = NonZeroUsize::new(config.max_cache_size)
            .unwrap_or_else(|| NonZeroUsize::new(1000).unwrap());

        Self {
            config,
            lru_cache: Arc::new(Mutex::new(LruCache::new(cache_size))),
            time_provider,
            key_resolver: KeyDidResolver,
            peer_resolver: PeerDidResolver,
            web_resolver: Arc::new(RwLock::new(WebDidResolver::default())),
            stats: Arc::new(RwLock::new(ResolutionStats::default())),
            method_stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create with default configuration
    pub fn with_defaults(time_provider: Arc<dyn TimeProvider>) -> Self {
        Self::new(DidResolutionConfig::default(), time_provider)
    }

    /// Add a web DID key mapping
    pub fn add_web_did_key(&self, did: String, key: ed25519_dalek::VerifyingKey) {
        if let Ok(mut resolver) = self.web_resolver.write() {
            resolver.insert(did, key);
        }
    }

    /// Get resolution statistics
    pub fn get_stats(&self) -> ResolutionStats {
        self.stats.read().unwrap().clone()
    }

    /// Clear the resolution cache
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.lru_cache.lock() {
            cache.clear();
        }
    }

    /// Resolve DID with caching and fallback
    pub fn resolve_with_caching(
        &self,
        did: &Did,
    ) -> Result<ed25519_dalek::VerifyingKey, CommonError> {
        let start_time = self.time_provider.unix_seconds();
        let did_string = did.to_string();

        // Update total resolution counter
        if let Ok(mut stats) = self.stats.write() {
            stats.total_resolutions += 1;
        }

        // Check cache first
        if let Some(cached) = self.get_from_cache(&did_string) {
            if let Ok(mut stats) = self.stats.write() {
                stats.cache_hits += 1;
            }
            return Ok(cached.verifying_key);
        }

        // Cache miss - record it
        if let Ok(mut stats) = self.stats.write() {
            stats.cache_misses += 1;
        }

        // Attempt resolution with configured method preference
        let result = self.resolve_with_methods(did);

        // Record timing and results
        let end_time = self.time_provider.unix_seconds();
        let duration_ms = (end_time - start_time) as f64 * 1000.0;

        match &result {
            Ok(key) => {
                // Cache successful resolution
                self.cache_result(&did_string, *key, &did.method);

                // Update method stats
                self.update_method_stats(&did.method, true, duration_ms);
            }
            Err(error) => {
                // Record error
                self.record_error(error);
                self.update_method_stats(&did.method, false, duration_ms);
            }
        }

        result
    }

    /// Resolve using method preference order with fallbacks
    fn resolve_with_methods(&self, did: &Did) -> Result<ed25519_dalek::VerifyingKey, CommonError> {
        // First try the DID's native method
        if let Ok(key) = self.resolve_by_method(did, &did.method) {
            return Ok(key);
        }

        // If fallback is disabled, return the error
        if !self.config.enable_fallback {
            return self.resolve_by_method(did, &did.method);
        }

        // Try fallback methods in preference order
        for method in &self.config.method_preference {
            if method != &did.method {
                if let Ok(key) = self.resolve_by_method(did, method) {
                    // DID resolved using fallback method
                    return Ok(key);
                }
            }
        }

        Err(CommonError::IdentityError(format!(
            "Failed to resolve DID {did} with any available method"
        )))
    }

    /// Resolve using a specific method
    fn resolve_by_method(
        &self,
        did: &Did,
        method: &str,
    ) -> Result<ed25519_dalek::VerifyingKey, CommonError> {
        match method {
            "key" => self.key_resolver.resolve(did),
            "peer" => self.peer_resolver.resolve(did),
            "web" => {
                if let Ok(resolver) = self.web_resolver.read() {
                    resolver.resolve(did)
                } else {
                    Err(CommonError::IdentityError(
                        "Web resolver unavailable".to_string(),
                    ))
                }
            }
            _ => Err(CommonError::IdentityError(format!(
                "Unsupported DID method: {method}"
            ))),
        }
    }

    /// Get result from cache if valid
    fn get_from_cache(&self, did_string: &str) -> Option<CacheEntry> {
        let mut cache = self.lru_cache.lock().ok()?;
        let entry = cache.get(did_string)?.clone();

        // Check if expired
        let now = self.time_provider.unix_seconds();
        if now >= entry.expires_at {
            // Entry expired, remove it
            drop(cache);
            if let Ok(mut cache) = self.lru_cache.lock() {
                cache.pop(did_string);
            }
            return None;
        }

        Some(entry)
    }

    /// Cache a successful resolution result
    fn cache_result(&self, did_string: &str, key: ed25519_dalek::VerifyingKey, method: &str) {
        if let Ok(mut cache) = self.lru_cache.lock() {
            // LruCache automatically evicts oldest entry when capacity is exceeded
            // No manual eviction needed

            let expires_at = self.time_provider.unix_seconds() + self.config.cache_ttl_seconds;
            cache.put(
                did_string.to_string(),
                CacheEntry {
                    verifying_key: key,
                    expires_at,
                    method_used: method.to_string(),
                    access_count: 1,
                    last_accessed: self.time_provider.unix_seconds(),
                },
            );
        }
    }

    /// Evict oldest cache entry (simple FIFO for now)
    fn evict_oldest_entry(&self, cache: &mut HashMap<String, CacheEntry>) {
        let mut oldest_key = None;
        let mut oldest_time = u64::MAX;

        for (key, entry) in cache.iter() {
            if entry.expires_at < oldest_time {
                oldest_time = entry.expires_at;
                oldest_key = Some(key.clone());
            }
        }

        if let Some(key) = oldest_key {
            cache.remove(&key);
        }
    }

    /// Update method-specific statistics
    fn update_method_stats(&self, method: &str, success: bool, duration_ms: f64) {
        if let Ok(mut method_stats_map) = self.method_stats.write() {
            let method_stats = method_stats_map.entry(method.to_string()).or_default();

            if success {
                method_stats.successes += 1;
            } else {
                method_stats.failures += 1;
            }

            // Update average time (simple moving average)
            let total_operations = method_stats.successes + method_stats.failures;
            method_stats.average_time_ms =
                (method_stats.average_time_ms * (total_operations - 1) as f64 + duration_ms)
                    / total_operations as f64;
        }
    }

    /// Record error statistics
    fn record_error(&self, error: &CommonError) {
        if let Ok(mut stats) = self.stats.write() {
            let error_type = match error {
                CommonError::IdentityError(_) => "identity_error",
                CommonError::NetworkError(_) => "network_error",
                CommonError::TimeoutError(_) => "timeout_error",
                _ => "other_error",
            };

            *stats.errors.entry(error_type.to_string()).or_insert(0) += 1;
        }
    }

    /// Batch resolve multiple DIDs efficiently
    pub fn batch_resolve(
        &self,
        dids: &[Did],
    ) -> Vec<Result<ed25519_dalek::VerifyingKey, CommonError>> {
        dids.iter()
            .map(|did| self.resolve_with_caching(did))
            .collect()
    }

    /// Preload DIDs into cache (useful for known federation members)
    pub fn preload_cache(&self, did_key_pairs: Vec<(Did, ed25519_dalek::VerifyingKey)>) {
        for (did, key) in did_key_pairs {
            self.cache_result(&did.to_string(), key, &did.method);
        }
    }
}

impl DidResolver for EnhancedDidResolver {
    fn resolve(&self, did: &Did) -> Result<ed25519_dalek::VerifyingKey, CommonError> {
        self.resolve_with_caching(did)
    }
}

/// Builder for creating EnhancedDidResolver with custom configuration
pub struct EnhancedDidResolverBuilder {
    config: DidResolutionConfig,
    web_keys: HashMap<String, ed25519_dalek::VerifyingKey>,
}

impl EnhancedDidResolverBuilder {
    pub fn new() -> Self {
        Self {
            config: DidResolutionConfig::default(),
            web_keys: HashMap::new(),
        }
    }

    pub fn with_cache_ttl(mut self, ttl_seconds: u64) -> Self {
        self.config.cache_ttl_seconds = ttl_seconds;
        self
    }

    pub fn with_cache_size(mut self, max_size: usize) -> Self {
        self.config.max_cache_size = max_size;
        self
    }

    pub fn with_web_timeout(mut self, timeout_seconds: u64) -> Self {
        self.config.web_timeout_seconds = timeout_seconds;
        self
    }

    pub fn with_fallback(mut self, enable: bool) -> Self {
        self.config.enable_fallback = enable;
        self
    }

    pub fn with_method_preference(mut self, methods: Vec<String>) -> Self {
        self.config.method_preference = methods;
        self
    }

    pub fn add_web_key(mut self, did: String, key: ed25519_dalek::VerifyingKey) -> Self {
        self.web_keys.insert(did, key);
        self
    }

    pub fn build(self, time_provider: Arc<dyn TimeProvider>) -> EnhancedDidResolver {
        let resolver = EnhancedDidResolver::new(self.config, time_provider);

        // Add web keys
        for (did, key) in self.web_keys {
            resolver.add_web_did_key(did, key);
        }

        resolver
    }
}

impl Default for EnhancedDidResolverBuilder {
    fn default() -> Self {
        Self::new()
    }
}
