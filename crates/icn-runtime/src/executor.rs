#![doc = "Defines a simple job executor for the ICN runtime."]

use icn_common::{Cid, CommonError, Did};
use icn_identity::ExecutionReceipt;
use icn_mesh::{ActualMeshJob, JobSpec};
use serde_json::Value;
use std::time::Instant;

// For now, using types from icn-identity directly for keys.
// This might be abstracted later if the executor needs its own identity management.
use icn_identity::{SigningKey, VerifyingKey, SignatureBytes, did_key_from_verifying_key};

/// Trait for a job executor.
#[async_trait::async_trait]
pub trait JobExecutor: Send + Sync {
    /// Executes the given job and returns an ExecutionReceipt.
    async fn execute_job(&self, job: &ActualMeshJob) -> Result<ExecutionReceipt, CommonError>;
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

    // Example internal function for a specific job type
    fn handle_echo_job(&self, job_spec: &JobSpec, job_id: &Cid, input_payload: Option<&Value>) -> Result<Value, CommonError> {
        info!("[SimpleExecutor] Executing echo job: {:?}", job_id);
        // In a real echo job, input_payload would be the data to echo.
        // For now, let's just echo back the job_spec or a part of it, or a fixed message.
        let output = input_payload.cloned().unwrap_or_else(|| Value::String("Echo successful!".to_string()));
        Ok(output)
    }

    // Example internal function for another job type (e.g., simple hash)
    fn handle_hash_job(&self, job_spec: &JobSpec, job_id: &Cid, input_payload: Option<&Value>) -> Result<Value, CommonError> {
        info!("[SimpleExecutor] Executing hash job: {:?}", job_id);
        let data_to_hash = input_payload
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| "default_string_to_hash");
        
        // Using a simple hash like SHA256 (requires a crypto lib like sha2)
        // For now, let's simulate this by just creating a CID of the input string.
        let hashed_output_cid = icn_common::cid::generate_cid(data_to_hash.as_bytes())?;
        Ok(Value::String(hashed_output_cid.to_string()))
    }
}

#[async_trait::async_trait]
impl JobExecutor for SimpleExecutor {
    async fn execute_job(&self, job: &ActualMeshJob) -> Result<ExecutionReceipt, CommonError> {
        info!("[SimpleExecutor] Received job for execution: Job ID {:?}, Manifest CID: {:?}", job.id, job.manifest_cid);
        let start_time = Instant::now();

        // TODO: Parse job.spec to determine job type and parameters.
        // For now, we'll use a placeholder logic or assume a default job type.
        // Example: determine job type from manifest_cid or a field in JobSpec.
        // Let's assume JobSpec might contain a "task_type" field and "payload".

        // Simulate parsing job_spec (assuming it's JSON for this example)
        // let job_spec_value: Value = serde_json::from_str(&job.spec.to_string()) // Assuming JobSpec can be stringified to JSON
        //     .map_err(|e| CommonError::InvalidParameters(format!("Failed to parse JobSpec: {}", e)))?;
        // let task_type = job_spec_value.get("task_type").and_then(|v| v.as_str()).unwrap_or("echo");
        // let input_payload = job_spec_value.get("payload");
        
        // Hardcoding to echo for now
        let task_type = "echo";
        let input_payload : Option<Value> = None; // Or some default from job.spec if parsed

        let result_value = match task_type {
            "echo" => self.handle_echo_job(&job.spec, &job.id, input_payload.as_ref()),
            "hash" => self.handle_hash_job(&job.spec, &job.id, input_payload.as_ref()),
            _ => Err(CommonError::NotImplemented(format!("Job task_type '{}' not supported by SimpleExecutor", task_type)))
        }?; 

        let result_bytes = serde_json::to_vec(&result_value)
            .map_err(|e| CommonError::SerializationError(format!("Failed to serialize job result: {}", e)))?;
        let result_cid = icn_common::cid::generate_cid(&result_bytes)?;
        let cpu_ms = start_time.elapsed().as_millis() as u64;

        let receipt = ExecutionReceipt {
            job_id: job.id.clone(),
            executor_did: self.node_did.clone(), 
            result_cid,
            cpu_ms,
            sig: SignatureBytes(vec![]), // Placeholder, will be signed next
        };

        let signed_receipt = receipt.sign_with_key(&self.signing_key)?; // Using the method from icn-identity
        info!("[SimpleExecutor] Job {:?} executed. Receipt CID: {:?}, CPU ms: {}", job.id, signed_receipt.result_cid, signed_receipt.cpu_ms);
        Ok(signed_receipt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::generate_cid; // For test CIDs
    use icn_identity::generate_ed25519_keypair;
    use icn_mesh::JobSpec; // Assuming JobSpec is Default or simple to construct
    use std::str::FromStr; // For Did::from_str

    #[tokio::test]
    async fn test_simple_executor_echo_job() {
        let (signing_key, verifying_key) = generate_ed25519_keypair();
        let node_did_string = did_key_from_verifying_key(&verifying_key);
        let node_did = Did::from_string(&node_did_string).unwrap();

        let executor = SimpleExecutor::new(node_did.clone(), signing_key);

        let job_id = generate_cid(b"test_echo_job_id").unwrap();
        let manifest_cid = generate_cid(b"test_echo_manifest").unwrap();
        
        // Create a ActualMeshJob. For an echo, spec might be simple or define input.
        // For now, assuming JobSpec is Default-able and SimpleExecutor doesn't rely on specific fields yet.
        let job = ActualMeshJob {
            id: job_id.clone(),
            manifest_cid,
            spec: JobSpec { /* ... */ }, // Assuming JobSpec can be defaulted or is simple
            creator_did: Did::from_string("did:example:jobcreator").unwrap(),
            cost_mana: 10,
            signature: SignatureBytes(vec![]) // Not relevant for executor test itself
        };

        let result = executor.execute_job(&job).await;
        assert!(result.is_ok());
        let receipt = result.unwrap();

        assert_eq!(receipt.job_id, job_id);
        assert_eq!(receipt.executor_did, node_did);
        assert!(receipt.cpu_ms >= 0);
        assert!(!receipt.sig.0.is_empty());

        // Verify signature (optional here, but good practice)
        assert!(receipt.verify_against_key(&verifying_key).is_ok());

        // Further checks: inspect receipt.result_cid to see if it reflects echoed data.
        // This requires knowing how SimpleExecutor structures its echo output and generates the CID.
        // For now, we checked that a receipt is generated and signed.
        info!("Echo job receipt: {:?}", receipt);
    }
    
    // TODO: Add test for hash_job if it gets more complex than current simulation
    // TODO: Add test for unsupported job type
} 