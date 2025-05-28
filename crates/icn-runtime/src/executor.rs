#![doc = "Defines a simple job executor for the ICN runtime."]

use icn_common::{Did, Cid, CommonError};
use icn_identity::{ExecutionReceipt as IdentityExecutionReceipt, SigningKey, SignatureBytes /* Removed , generate_ed25519_keypair */};
use icn_mesh::{JobId, ActualMeshJob, JobSpec, /* ... other mesh types ... */};
use log::{info}; // Removed error
use std::time::SystemTime;
use serde_json::Value; // Added for handle_*_job functions

/// Trait for a job executor.
#[async_trait::async_trait]
pub trait JobExecutor: Send + Sync {
    /// Executes the given job and returns an ExecutionReceipt.
    async fn execute_job(&self, job: &ActualMeshJob) -> Result<IdentityExecutionReceipt, CommonError>;
}

/// A simple executor that can handle basic predefined tasks like echo or hashing.
#[derive(Debug)]
pub struct SimpleExecutor {
    node_did: Did,
    signing_key: SigningKey,
    // VerifyingKey can be derived from SigningKey if needed for self-check or DID generation.
    // verifying_key: VerifyingKey, 
}

impl SimpleExecutor {
    pub fn new(node_did: Did, signing_key: SigningKey) -> Self {
        Self { node_did, signing_key }
    }

    fn handle_echo_job(&self, _job_spec: &JobSpec, job_id: &Cid, input_payload: Option<&Value>) -> Result<Value, CommonError> {
        info!("[SimpleExecutor] Executing echo job: {:?}", job_id);
        let output = input_payload.cloned().unwrap_or_else(|| Value::String("Echo successful!".to_string()));
        Ok(output)
    }

    fn handle_hash_job(&self, _job_spec: &JobSpec, job_id: &Cid, input_payload: Option<&Value>) -> Result<Value, CommonError> {
        info!("[SimpleExecutor] Executing hash job: {:?}", job_id);
        let data_to_hash = input_payload
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| "default_string_to_hash");
        
        // In a real scenario, you'd use a proper hash function and likely return bytes or a CID directly.
        // For this placeholder, we return a string representation of a dummy CID.
        let dummy_cid = Cid::new_v1_dummy(0x55, 0x12, data_to_hash.as_bytes());
        Ok(Value::String(dummy_cid.to_string()))
    }

    async fn execute_job_logic(&self, job_id: &JobId, job: &ActualMeshJob) -> Result<Vec<u8>, CommonError> {
        match &job.spec {
            JobSpec::Echo { payload } => {
                info!("[SimpleExecutor] Executing echo job: {:?}", job_id);
                let output_data = format!("Echo: {}", payload);
                Ok(output_data.into_bytes())
            }
            JobSpec::GenericPlaceholder => {
                info!("[SimpleExecutor] Executing hash job (placeholder): {:?}", job_id);
                let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis();
                let data_to_hash = format!("job_id:{:?}-manifest:{:?}-timestamp:{}", job.id, job.manifest_cid, timestamp);
                Ok(data_to_hash.into_bytes())
            }
        }
    }
}

#[async_trait::async_trait]
impl JobExecutor for SimpleExecutor {
    async fn execute_job(&self, job: &ActualMeshJob) -> Result<IdentityExecutionReceipt, CommonError> {
        info!("[SimpleExecutor] Received job for execution: Job ID {:?}, Manifest CID: {:?}", job.id, job.manifest_cid);
        let start_time = SystemTime::now();

        let result_bytes = match &job.spec {
            JobSpec::Echo { payload } => {
                info!("[SimpleExecutor] Executing echo job: {:?}", job.id);
                format!("Echo: {}", payload).into_bytes()
            }
            JobSpec::GenericPlaceholder => {
                info!("[SimpleExecutor] Executing hash job (placeholder): {:?}", job.id);
                let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis();
                format!("job_id:{:?}-manifest:{:?}-timestamp:{}", job.id, job.manifest_cid, timestamp).into_bytes()
            }
        };

        let result_cid = Cid::new_v1_dummy(0x55, 0x12, &result_bytes);
        let cpu_ms = start_time.elapsed().unwrap_or_default().as_millis() as u64;

        let unsigned_receipt = IdentityExecutionReceipt {
            job_id: job.id.clone(),
            executor_did: self.node_did.clone(), 
            result_cid,
            cpu_ms,
            sig: SignatureBytes(vec![]),
        };

        unsigned_receipt.sign_with_key(&self.signing_key)
            .map_err(|e| CommonError::InternalError(format!("Failed to sign execution receipt: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::Cid; // generate_cid is not public, ICN_CORE_VERSION was unused
    use icn_identity::{generate_ed25519_keypair as generate_keys_for_test, did_key_from_verifying_key}; // Kept these
    use std::str::FromStr; // Added For Did::from_str here
    // Removed unused: serde_json::json, std::convert::TryInto, std::sync::Arc

    fn dummy_cid_for_executor_test(s: &str) -> Cid {
        Cid::new_v1_dummy(0x55, 0x12, s.as_bytes())
    }

    #[tokio::test]
    async fn test_simple_executor_echo_job() {
        let (signing_key, verifying_key) = generate_keys_for_test();
        let node_did_string = did_key_from_verifying_key(&verifying_key);
        let node_did = Did::from_str(&node_did_string).unwrap();

        let executor = SimpleExecutor::new(node_did.clone(), signing_key.clone()); // Clone sk if needed later, or pass Arc

        let job_id = dummy_cid_for_executor_test("test_echo_job_id");
        let manifest_cid = dummy_cid_for_executor_test("test_echo_manifest");
        
        let job = ActualMeshJob {
            id: job_id.clone(),
            manifest_cid,
            spec: JobSpec::Echo { payload: "Hello Echo Test".to_string() }, // Corrected JobSpec usage
            creator_did: Did::from_str("did:example:jobcreator").unwrap(),
            cost_mana: 10,
            signature: SignatureBytes(vec![]) 
        };

        let result = executor.execute_job(&job).await;
        assert!(result.is_ok());
        let receipt = result.unwrap();

        assert_eq!(receipt.job_id, job_id);
        assert_eq!(receipt.executor_did, node_did);
        assert!(receipt.cpu_ms >= 0);
        assert!(!receipt.sig.0.is_empty());
        assert!(receipt.verify_against_key(&verifying_key).is_ok());
        info!("Echo job receipt (test_simple_executor_echo_job): {:?}", receipt);
    }

    #[tokio::test]
    async fn test_execute_job_echo_success() {
        let (node_sk, node_pk) = generate_keys_for_test();
        let node_did_string = did_key_from_verifying_key(&node_pk);
        let node_did = Did::from_str(&node_did_string).unwrap();

        let job = ActualMeshJob {
            id: dummy_cid_for_executor_test("job1"),
            manifest_cid: dummy_cid_for_executor_test("manifest1"),
            spec: JobSpec::Echo { payload: "hello".to_string() },
            creator_did: Did::from_str("did:example:jobcreator").unwrap(),
            cost_mana: 10,
            signature: SignatureBytes(vec![]), 
        };

        // SimpleExecutor::new expects SigningKey, not Arc<SigningKey> as per current definition
        let executor = SimpleExecutor::new(node_did.clone(), node_sk.clone()); // Clone sk if it will be used again
        let result = executor.execute_job(&job).await;
        assert!(result.is_ok());
        let receipt = result.unwrap();

        assert_eq!(receipt.job_id, job.id);
        assert_eq!(receipt.executor_did, node_did);
        assert!(receipt.cpu_ms >= 0);
        assert!(!receipt.sig.0.is_empty());
        assert!(receipt.verify_against_key(&node_pk).is_ok());
        info!("Echo job receipt (test_execute_job_echo_success): {:?}", receipt);
    }
} 