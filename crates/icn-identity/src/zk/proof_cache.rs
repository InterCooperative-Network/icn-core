use ark_bn254::Bn254;
use ark_groth16::PreparedVerifyingKey;
use ark_serialize::CanonicalSerialize;
use lru::LruCache;
use once_cell::sync::Lazy;
use sha2::{Digest, Sha256};
use std::num::NonZeroUsize;
use std::sync::Mutex;

use super::ZkError;

pub(crate) struct ProofCache;

static CACHE: Lazy<Mutex<LruCache<[u8; 32], bool>>> =
    Lazy::new(|| Mutex::new(LruCache::new(NonZeroUsize::new(64).unwrap())));

impl ProofCache {
    pub fn get_or_insert<F>(
        proof: &[u8],
        vk: &PreparedVerifyingKey<Bn254>,
        inputs: &[ark_bn254::Fr],
        verify_fn: F,
    ) -> Result<bool, ZkError>
    where
        F: FnOnce() -> Result<bool, ZkError>,
    {
        let key = Self::hash(proof, vk, inputs)?;
        {
            let mut cache = CACHE.lock().expect("cache mutex poisoned");
            if let Some(v) = cache.get(&key) {
                return Ok(*v);
            }
        }
        let res = verify_fn()?;
        let mut cache = CACHE.lock().expect("cache mutex poisoned");
        cache.put(key, res);
        Ok(res)
    }

    fn hash(
        proof: &[u8],
        vk: &PreparedVerifyingKey<Bn254>,
        inputs: &[ark_bn254::Fr],
    ) -> Result<[u8; 32], ZkError> {
        let mut hasher = Sha256::new();
        hasher.update(proof);

        let mut vk_bytes = Vec::new();
        vk.serialize_compressed(&mut vk_bytes)
            .map_err(|_| ZkError::InvalidProof)?;
        hasher.update(&vk_bytes);

        for fr in inputs {
            let mut buf = Vec::new();
            fr.serialize_compressed(&mut buf)
                .map_err(|_| ZkError::InvalidProof)?;
            hasher.update(&buf);
        }

        Ok(hasher.finalize().into())
    }

    #[cfg(test)]
    pub fn len() -> usize {
        CACHE.lock().unwrap().len()
    }

    #[cfg(test)]
    pub fn clear() {
        CACHE.lock().unwrap().clear();
    }
}
