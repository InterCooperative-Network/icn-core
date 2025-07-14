use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Stored circuit parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitRecord {
    pub slug: String,
    pub version: String,
    pub proving_key: Vec<u8>,
    pub verification_key: Vec<u8>,
}

/// Simple in-memory circuit registry.
#[derive(Default, Clone)]
pub struct CircuitRegistry {
    inner: Arc<RwLock<BTreeMap<String, BTreeMap<String, CircuitRecord>>>>,
}

impl CircuitRegistry {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }

    pub async fn register(
        &self,
        slug: String,
        version: String,
        proving_key: Vec<u8>,
        verification_key: Vec<u8>,
    ) {
        let mut map = self.inner.write().await;
        let versions = map.entry(slug.clone()).or_insert_with(BTreeMap::new);
        versions.insert(
            version.clone(),
            CircuitRecord {
                slug,
                version,
                proving_key,
                verification_key,
            },
        );
    }

    pub async fn get(&self, slug: &str, version: &str) -> Option<CircuitRecord> {
        let map = self.inner.read().await;
        map.get(slug).and_then(|v| v.get(version).cloned())
    }

    pub async fn list_versions(&self, slug: &str) -> Vec<String> {
        let map = self.inner.read().await;
        map.get(slug)
            .map(|v| v.keys().cloned().collect())
            .unwrap_or_default()
    }
}
