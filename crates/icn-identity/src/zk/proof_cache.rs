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
        mut verify_fn: F,
    ) -> Result<bool, ZkError>
    where
        F: FnMut() -> Result<bool, ZkError>,
    {
        let mut hasher = Sha256::new();
        hasher.update(proof);

        let mut vk_bytes = Vec::new();
        vk.serialize_compressed(&mut vk_bytes)
            .map_err(|_| ZkError::InvalidProof)?;
        hasher.update(&vk_bytes);

        for inp in inputs {
            let mut buf = Vec::new();
            inp.serialize_compressed(&mut buf)
                .map_err(|_| ZkError::InvalidProof)?;
            hasher.update(&buf);
        }

        let key: [u8; 32] = hasher.finalize().into();

        {
            let mut cache = CACHE.lock().expect("cache mutex poisoned");
            if let Some(val) = cache.get(&key) {
                return Ok(*val);
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
