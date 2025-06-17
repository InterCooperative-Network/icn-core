//! This module provides executor-side functionality for running mesh jobs.

use icn_common::{Cid, CommonError, Did};
use icn_identity::{
    ExecutionReceipt as IdentityExecutionReceipt,
    SignatureBytes, /* Removed , generate_ed25519_keypair */
    SigningKey,
};
use icn_mesh::{ActualMeshJob, JobSpec /* ... other mesh types ... */};
use log::info; // Removed error
use std::time::SystemTime;

/// Trait for a job executor.
#[async_trait::async_trait]
pub trait JobExecutor: Send + Sync {
    /// Executes the given job and returns an ExecutionReceipt.
    async fn execute_job(
        &self,
        job: &ActualMeshJob,
    ) -> Result<IdentityExecutionReceipt, CommonError>;
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
        Self {
            node_did,
            signing_key,
        }
    }
}

#[async_trait::async_trait]
impl JobExecutor for SimpleExecutor {
    async fn execute_job(
        &self,
        job: &ActualMeshJob,
    ) -> Result<IdentityExecutionReceipt, CommonError> {
        info!(
            "[SimpleExecutor] Received job for execution: Job ID {:?}, Manifest CID: {:?}",
            job.id, job.manifest_cid
        );
        let start_time = SystemTime::now();

        let result_bytes = match &job.spec {
            JobSpec::Echo { payload } => {
                info!("[SimpleExecutor] Executing echo job: {:?}", job.id);
                format!("Echo: {}", payload).into_bytes()
            }
            JobSpec::GenericPlaceholder => {
                info!(
                    "[SimpleExecutor] Executing hash job (placeholder): {:?}",
                    job.id
                );
                let timestamp = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_millis();
                format!(
                    "job_id:{:?}-manifest:{:?}-timestamp:{}",
                    job.id, job.manifest_cid, timestamp
                )
                .into_bytes()
            }
        };

        let result_cid = Cid::new_v1_dummy(0x55, 0x12, &result_bytes);
        let cpu_ms = start_time.elapsed().unwrap_or_default().as_millis() as u64;

        let unsigned_receipt = IdentityExecutionReceipt {
            job_id: job.id.clone(),
            executor_did: self.node_did.clone(),
            result_cid,
            cpu_ms,
            success: true,
            sig: SignatureBytes(vec![]),
        };

        unsigned_receipt
            .sign_with_key(&self.signing_key)
            .map_err(|e| {
                CommonError::InternalError(format!("Failed to sign execution receipt: {}", e))
            })
    }
}

/// A WASM-based executor that loads WASM modules from the DAG store and
/// exposes host functions from the [`RuntimeContext`] to the guest module.
pub struct WasmExecutor {
    ctx: std::sync::Arc<crate::context::RuntimeContext>,
    node_did: Did,
    signing_key: SigningKey,
    engine: wasmtime::Engine,
}

impl WasmExecutor {
    /// Creates a new [`WasmExecutor`] bound to the given runtime context.
    pub fn new(
        ctx: std::sync::Arc<crate::context::RuntimeContext>,
        node_did: Did,
        signing_key: SigningKey,
    ) -> Self {
        Self {
            ctx,
            node_did,
            signing_key,
            engine: wasmtime::Engine::default(),
        }
    }

    /// Executes the given job using [`execute_job`] and immediately anchors the
    /// resulting receipt via the associated [`RuntimeContext`].
    pub async fn execute_and_anchor_job(
        &self,
        job: &ActualMeshJob,
    ) -> Result<Cid, crate::context::HostAbiError> {
        let receipt = self
            .execute_job(job)
            .await
            .map_err(crate::context::HostAbiError::Common)?;
        self.ctx.anchor_receipt(&receipt).await
    }
}

#[async_trait::async_trait]
impl JobExecutor for WasmExecutor {
    async fn execute_job(
        &self,
        job: &ActualMeshJob,
    ) -> Result<IdentityExecutionReceipt, CommonError> {
        use crate::host_account_get_mana;
        use wasmtime::{Linker, Module, Store};

        // Load WASM bytes from the DAG store
        let wasm_bytes = {
            let store = self.ctx.dag_store.lock().await;
            store
                .get(&job.manifest_cid)
                .map_err(|e| CommonError::StorageError(format!("{e}")))?
        }
        .ok_or_else(|| CommonError::ResourceNotFound("WASM module not found".into()))?
        .data;

        let mut store = Store::new(&self.engine, ());
        let mut linker = Linker::new(&self.engine);

        let ctx_clone = self.ctx.clone();
        linker
            .func_wrap("icn", "host_account_get_mana", move || -> i64 {
                let handle = tokio::runtime::Handle::current();
                let account = ctx_clone.current_identity.to_string();
                handle
                    .block_on(async { host_account_get_mana(&ctx_clone, &account).await })
                    .unwrap_or(0) as i64
            })
            .map_err(|e| CommonError::InternalError(e.to_string()))?;

        let module = Module::new(&self.engine, &wasm_bytes)
            .map_err(|e| CommonError::DeserError(e.to_string()))?;
        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| CommonError::InternalError(e.to_string()))?;
        let func = instance
            .get_typed_func::<(), i64>(&mut store, "run")
            .map_err(|e| CommonError::InternalError(e.to_string()))?;

        let start_time = SystemTime::now();
        let result = func
            .call(&mut store, ())
            .map_err(|e| CommonError::InternalError(e.to_string()))?;
        let cpu_ms = start_time.elapsed().unwrap_or_default().as_millis() as u64;

        let result_bytes = result.to_le_bytes();
        let result_cid = Cid::new_v1_dummy(0x55, 0x12, &result_bytes);

        let unsigned_receipt = IdentityExecutionReceipt {
            job_id: job.id.clone(),
            executor_did: self.node_did.clone(),
            result_cid,
            cpu_ms,
            success: true,
            sig: SignatureBytes(vec![]),
        };

        unsigned_receipt
            .sign_with_key(&self.signing_key)
            .map_err(|e| {
                CommonError::InternalError(format!("Failed to sign execution receipt: {}", e))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::Cid; // generate_cid is not public, ICN_CORE_VERSION was unused
    use icn_identity::{
        did_key_from_verifying_key, generate_ed25519_keypair as generate_keys_for_test,
    }; // Kept these
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
            spec: JobSpec::Echo {
                payload: "Hello Echo Test".to_string(),
            }, // Corrected JobSpec usage
            creator_did: Did::from_str("did:example:jobcreator").unwrap(),
            cost_mana: 10,
            max_execution_wait_ms: None,
            signature: SignatureBytes(vec![]),
        };

        let result = executor.execute_job(&job).await;
        assert!(result.is_ok());
        let receipt = result.unwrap();

        assert_eq!(receipt.job_id, job_id);
        assert_eq!(receipt.executor_did, node_did);
        assert!(!receipt.sig.0.is_empty());
        assert!(receipt.verify_against_key(&verifying_key).is_ok());
        info!(
            "Echo job receipt (test_simple_executor_echo_job): {:?}",
            receipt
        );
    }

    #[tokio::test]
    async fn test_execute_job_echo_success() {
        let (node_sk, node_pk) = generate_keys_for_test();
        let node_did_string = did_key_from_verifying_key(&node_pk);
        let node_did = Did::from_str(&node_did_string).unwrap();

        let job = ActualMeshJob {
            id: dummy_cid_for_executor_test("job1"),
            manifest_cid: dummy_cid_for_executor_test("manifest1"),
            spec: JobSpec::Echo {
                payload: "hello".to_string(),
            },
            creator_did: Did::from_str("did:example:jobcreator").unwrap(),
            cost_mana: 10,
            max_execution_wait_ms: None,
            signature: SignatureBytes(vec![]),
        };

        // SimpleExecutor::new expects SigningKey, not Arc<SigningKey> as per current definition
        let executor = SimpleExecutor::new(node_did.clone(), node_sk.clone()); // Clone sk if it will be used again
        let result = executor.execute_job(&job).await;
        assert!(result.is_ok());
        let receipt = result.unwrap();

        assert_eq!(receipt.job_id, job.id);
        assert_eq!(receipt.executor_did, node_did);
        assert!(!receipt.sig.0.is_empty());
        assert!(receipt.verify_against_key(&node_pk).is_ok());
        info!(
            "Echo job receipt (test_execute_job_echo_success): {:?}",
            receipt
        );
    }
}
