//! Storage traits for zero-knowledge credential proofs.
#![forbid(unsafe_code)]

use icn_common::{Cid, CommonError, ZkCredentialProof};
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::sync::Mutex;

/// Trait for archiving and retrieving [`ZkCredentialProof`]s.
pub trait ProofStore: Send + Sync {
    /// Persist the supplied proof and return its content ID.
    fn put(&self, proof: &ZkCredentialProof) -> Result<Cid, CommonError>;

    /// Fetch a proof by its content ID.
    fn get(&self, cid: &Cid) -> Result<Option<ZkCredentialProof>, CommonError>;

    /// List all stored proofs.
    fn list(&self) -> Result<Vec<ZkCredentialProof>, CommonError>;
}

impl std::fmt::Debug for dyn ProofStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ProofStore")
    }
}

/// Simple file-based proof store using JSON Lines format.
#[derive(Debug)]
pub struct FileProofStore {
    path: PathBuf,
    lock: Mutex<()>,
}

impl FileProofStore {
    /// Create a new store writing to the given path.
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            lock: Mutex::new(()),
        }
    }
}

impl ProofStore for FileProofStore {
    fn put(&self, proof: &ZkCredentialProof) -> Result<Cid, CommonError> {
        let json = serde_json::to_vec(proof)
            .map_err(|e| CommonError::SerializationError(e.to_string()))?;
        let cid = Cid::new_v1_sha256(0x55, &json);
        let _g = self.lock.lock().unwrap();
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| CommonError::IoError(e.to_string()))?;
        let mut w = BufWriter::new(file);
        w.write_all(&json)
            .and_then(|_| w.write_all(b"\n"))
            .map_err(|e| CommonError::IoError(e.to_string()))?;
        w.flush()
            .map_err(|e| CommonError::IoError(e.to_string()))?;
        Ok(cid)
    }

    fn get(&self, cid: &Cid) -> Result<Option<ZkCredentialProof>, CommonError> {
        if !self.path.exists() {
            return Ok(None);
        }
        let _g = self.lock.lock().unwrap();
        let file = OpenOptions::new()
            .read(true)
            .open(&self.path)
            .map_err(|e| CommonError::IoError(e.to_string()))?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line.map_err(|e| CommonError::IoError(e.to_string()))?;
            let bytes = line.as_bytes();
            let current_cid = Cid::new_v1_sha256(0x55, bytes);
            if &current_cid == cid {
                let proof: ZkCredentialProof = serde_json::from_slice(bytes)
                    .map_err(|e| CommonError::DeserializationError(e.to_string()))?;
                return Ok(Some(proof));
            }
        }
        Ok(None)
    }

    fn list(&self) -> Result<Vec<ZkCredentialProof>, CommonError> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let _g = self.lock.lock().unwrap();
        let file = OpenOptions::new()
            .read(true)
            .open(&self.path)
            .map_err(|e| CommonError::IoError(e.to_string()))?;
        let reader = BufReader::new(file);
        let mut proofs = Vec::new();
        for line in reader.lines() {
            let line = line.map_err(|e| CommonError::IoError(e.to_string()))?;
            let proof: ZkCredentialProof = serde_json::from_str(&line)
                .map_err(|e| CommonError::DeserializationError(e.to_string()))?;
            proofs.push(proof);
        }
        Ok(proofs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::{Did, ZkProofType};
    use tempfile::tempdir;

    #[test]
    fn file_round_trip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("proofs.jsonl");
        let store = FileProofStore::new(path.clone());

        let proof = ZkCredentialProof {
            issuer: Did::new("key", "iss"),
            holder: Did::new("key", "holder"),
            claim_type: "age_over_18".into(),
            proof: vec![1, 2, 3],
            schema: Cid::new_v1_sha256(0x55, b"schema"),
            vk_cid: None,
            disclosed_fields: vec![],
            challenge: None,
            backend: ZkProofType::Groth16,
            verification_key: None,
            public_inputs: None,
        };

        let cid = store.put(&proof).unwrap();
        let fetched = store.get(&cid).unwrap().expect("stored");
        assert_eq!(fetched.issuer, proof.issuer);
        let all = store.list().unwrap();
        assert_eq!(all.len(), 1);
    }
}
