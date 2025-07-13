use ark_bn254::Bn254;
use ark_groth16::{Groth16, PreparedVerifyingKey, VerifyingKey};
use ark_serialize::CanonicalDeserialize;
use ark_snark::SNARK;
use lru::LruCache;
use once_cell::sync::Lazy;
use std::num::NonZeroUsize;
use std::sync::Mutex;

use super::ZkError;

static PREPARED_VK_CACHE: Lazy<Mutex<LruCache<Vec<u8>, PreparedVerifyingKey<Bn254>>>> =
    Lazy::new(|| Mutex::new(LruCache::new(NonZeroUsize::new(16).expect("non-zero"))));

pub struct PreparedVkCache;

impl PreparedVkCache {
    pub fn get_or_insert(bytes: &[u8]) -> Result<PreparedVerifyingKey<Bn254>, ZkError> {
        let mut cache = PREPARED_VK_CACHE.lock().expect("cache lock");
        if let Some(pvk) = cache.get(bytes) {
            return Ok(pvk.clone());
        }

        let vk = VerifyingKey::<Bn254>::deserialize_compressed(bytes)
            .map_err(|_| ZkError::InvalidProof)?;
        let pvk = Groth16::<Bn254>::process_vk(&vk).map_err(|_| ZkError::VerificationFailed)?;
        cache.put(bytes.to_vec(), pvk.clone());
        Ok(pvk)
    }
}
