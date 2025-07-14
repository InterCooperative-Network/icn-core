use ark_bn254::Fr;
use ark_serialize::CanonicalSerialize;
use lru::LruCache;
use once_cell::sync::Lazy;
use sha2::{Digest, Sha256};
use std::num::NonZeroUsize;
use std::sync::Mutex;

use super::ZkError;

pub(crate) struct ProofCache;

static CACHE: Lazy<Mutex<LruCache<[u8; 32], bool>>> =
    Lazy::new(|| Mutex::new(LruCache::new(NonZeroUsize::new(128).unwrap())));

impl ProofCache {
    pub fn get_or_insert<F>(
        proof: &[u8],
        vk: &[u8],
        inputs: &[Fr],
        verify_fn: F,
    ) -> Result<bool, ZkError>
    where
        F: FnOnce() -> Result<bool, ZkError>,
    {
        let mut hasher = Sha256::new();
        hasher.update(proof);
        hasher.update(vk);
        for input in inputs {
            let mut buf = Vec::new();
            input
                .serialize_compressed(&mut buf)
                .map_err(|_| ZkError::InvalidProof)?;
            hasher.update(&buf);
        }
        let key: [u8; 32] = hasher.finalize().into();

        {
            let mut cache = CACHE.lock().expect("cache mutex poisoned");
            if let Some(v) = cache.get(&key) {
                return Ok(*v);
            }
        }

        let result = verify_fn()?;

        let mut cache = CACHE.lock().expect("cache mutex poisoned");
        cache.put(key, result);
        Ok(result)
    }

    #[cfg(test)]
    pub fn len() -> usize {
        CACHE.lock().unwrap().len()
    }
}
