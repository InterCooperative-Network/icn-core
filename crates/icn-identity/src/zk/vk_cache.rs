use ark_bn254::Bn254;
use ark_groth16::{prepare_verifying_key, PreparedVerifyingKey, VerifyingKey};
use ark_serialize::CanonicalDeserialize;
use lru::LruCache;
use once_cell::sync::Lazy;
use std::num::NonZeroUsize;
use std::sync::Mutex;

use super::ZkError;

/// Cache of prepared verifying keys keyed by their serialized representation.
static PREPARED_VK_CACHE: Lazy<Mutex<LruCache<Vec<u8>, PreparedVerifyingKey<Bn254>>>> =
    Lazy::new(|| Mutex::new(LruCache::new(NonZeroUsize::new(32).unwrap())));

/// Convenience wrapper around the global prepared verifying key cache.
pub struct PreparedVkCache;

impl PreparedVkCache {
    /// Deserialize the verifying key and insert it into the cache if absent.
    pub fn get_or_insert(bytes: &[u8]) -> Result<PreparedVerifyingKey<Bn254>, ZkError> {
        let mut cache = PREPARED_VK_CACHE
            .lock()
            .expect("prepared vk cache mutex poisoned");
        if let Some(pvk) = cache.get(bytes) {
            return Ok(pvk.clone());
        }
        let vk = VerifyingKey::<Bn254>::deserialize_compressed(bytes)
            .map_err(|_| ZkError::InvalidProof)?;
        let pvk = prepare_verifying_key(&vk);
        cache.put(bytes.to_vec(), pvk.clone());
        Ok(pvk)
    }
}
