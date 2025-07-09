//! This module provides executor-side functionality for running mesh jobs.

use crate::context::RuntimeContext;
use crate::metrics::{WASM_MEMORY_GROWTH_DENIED, WASM_TABLE_GROWTH_DENIED};
use crate::{host_account_get_mana, host_get_reputation};
use icn_ccl::ContractMetadata;
use icn_common::{Cid, CommonError, Did};
use icn_identity::{
    ExecutionReceipt as IdentityExecutionReceipt,
    SignatureBytes, /* Removed , generate_ed25519_keypair */
    SigningKey,
};
#[cfg(test)]
use icn_mesh::JobSpec; /* ... other mesh types ... */
use icn_mesh::{ActualMeshJob, JobKind};
use log::{error, info, warn}; // Added warn, error
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use wasmparser::{Parser, Payload};
use wasmtime::{Caller, Config, Linker, Module, ResourceLimiter, Store};

/// Trait for a job executor.
#[async_trait::async_trait]
pub trait JobExecutor: Send + Sync {
    /// Executes the given job and returns an ExecutionReceipt.
    async fn execute_job(
        &self,
        job: &ActualMeshJob,
    ) -> Result<IdentityExecutionReceipt, CommonError>;
}

/// Security limits for WASM execution
#[derive(Clone, Debug)]
pub struct WasmSecurityLimits {
    /// Maximum execution time in seconds
    pub max_execution_time_secs: u64,
    /// Maximum linear memory pages (64KB each)
    pub max_memory_pages: u32,
    /// Maximum fuel consumption
    pub max_fuel: u64,
    /// Maximum stack depth
    pub max_stack_depth: u32,
    /// Maximum number of globals
    pub max_globals: u32,
    /// Maximum number of functions
    pub max_functions: u32,
    /// Maximum number of tables
    pub max_tables: u32,
    /// Maximum table size
    pub max_table_size: u32,
}

impl Default for WasmSecurityLimits {
    fn default() -> Self {
        Self {
            max_execution_time_secs: 30, // 30 second timeout
            max_memory_pages: 160,       // 10 MB (160 * 64KB)
            max_fuel: 1_000_000,         // 1 million instructions
            max_stack_depth: 1024,       // Reasonable stack depth
            max_globals: 100,            // Limited globals
            max_functions: 1000,         // Limited functions
            max_tables: 10,              // Limited tables
            max_table_size: 10000,       // Limited table size
        }
    }
}

/// Resource limiter for WASM execution
pub struct ICNResourceLimiter {
    #[allow(dead_code)]
    timeout: Duration,
    #[allow(dead_code)]
    memory_consumed: u64,
    #[allow(dead_code)]
    table_elements: u64,
    max_memory_bytes: usize,
}

impl ICNResourceLimiter {
    pub fn new(max_memory: usize, timeout: Duration) -> Self {
        Self {
            timeout,
            memory_consumed: 0,
            table_elements: 0,
            max_memory_bytes: max_memory,
        }
    }
}

impl ResourceLimiter for ICNResourceLimiter {
    fn memory_growing(
        &mut self,
        _current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> anyhow::Result<bool> {
        // Check if the desired memory growth exceeds our limits
        if desired > self.max_memory_bytes {
            warn!(
                "WASM memory limit exceeded: {} bytes > {} bytes",
                desired, self.max_memory_bytes
            );
            WASM_MEMORY_GROWTH_DENIED.inc();
            return Ok(false); // Deny the growth
        }
        Ok(true)
    }

    fn table_growing(
        &mut self,
        _current: u32,
        desired: u32,
        _maximum: Option<u32>,
    ) -> anyhow::Result<bool> {
        // Limit table growth (simple check)
        if desired > 1000 {
            warn!("WASM table size limit exceeded: {} > {}", desired, 1000);
            WASM_TABLE_GROWTH_DENIED.inc();
            return Ok(false); // Deny the growth
        }
        Ok(true)
    }
}

/// WASM module validator for security analysis
pub struct WasmModuleValidator {
    limits: WasmSecurityLimits,
}

impl WasmModuleValidator {
    pub fn new(limits: WasmSecurityLimits) -> Self {
        Self { limits }
    }

    /// Validates a WASM module against configured security limits
    pub fn validate(&self, wasm_bytes: &[u8]) -> Result<(), CommonError> {
        let mut function_count = 0;
        let mut global_count = 0;
        let mut table_count = 0;

        // Parse the WASM module
        let parser = Parser::new(0);

        // Fix: parse_all returns an iterator, handle errors properly
        for payload_result in parser.parse_all(wasm_bytes) {
            let payload = payload_result
                .map_err(|e| CommonError::InternalError(format!("WASM validation error: {}", e)))?;

            match payload {
                Payload::FunctionSection(reader) => {
                    function_count = reader.count();
                }
                Payload::GlobalSection(reader) => {
                    global_count = reader.count();
                }
                Payload::TableSection(reader) => {
                    table_count = reader.count();
                    // Fix: Use iterator instead of read() method
                    for table_result in reader {
                        let table =
                            table_result.map_err(|e| CommonError::DeserError(format!("{e}")))?;
                        let max = table.ty.maximum.unwrap_or(table.ty.initial);
                        if max > self.limits.max_table_size {
                            return Err(CommonError::PolicyDenied(
                                "WASM table size exceeds limit".to_string(),
                            ));
                        }
                    }
                }
                Payload::MemorySection(reader) => {
                    // Fix: Use iterator instead of read() method
                    for mem_result in reader {
                        let mem =
                            mem_result.map_err(|e| CommonError::DeserError(format!("{e}")))?;
                        let max = mem.maximum.unwrap_or(mem.initial);
                        if max > self.limits.max_memory_pages as u64 {
                            return Err(CommonError::PolicyDenied(
                                "WASM memory pages exceed limit".to_string(),
                            ));
                        }
                    }
                }
                _ => {}
            }
        }

        if function_count > self.limits.max_functions {
            return Err(CommonError::PolicyDenied(
                "Too many functions in WASM module".to_string(),
            ));
        }

        if global_count > self.limits.max_globals {
            return Err(CommonError::PolicyDenied(
                "Too many globals in WASM module".to_string(),
            ));
        }

        if table_count > self.limits.max_tables {
            return Err(CommonError::PolicyDenied(
                "Too many tables in WASM module".to_string(),
            ));
        }

        // Check total module size
        if wasm_bytes.len() > 50 * 1024 * 1024 {
            return Err(CommonError::PolicyDenied(
                "WASM module too large".to_string(),
            ));
        }

        Ok(())
    }
}

/// A simple executor that can handle basic predefined tasks like echo or hashing.
pub struct SimpleExecutor {
    node_did: Did,
    signing_key: SigningKey,
    ctx: Option<std::sync::Arc<RuntimeContext>>,
}

impl SimpleExecutor {
    pub fn new(node_did: Did, signing_key: SigningKey) -> Self {
        Self {
            node_did,
            signing_key,
            ctx: None,
        }
    }

    /// Create a `SimpleExecutor` that can also execute CCL WASM jobs using the
    /// provided runtime context.
    pub fn with_context(
        node_did: Did,
        signing_key: SigningKey,
        ctx: std::sync::Arc<RuntimeContext>,
    ) -> Self {
        Self {
            node_did,
            signing_key,
            ctx: Some(ctx),
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

        let result_bytes = match &job.spec.kind {
            JobKind::Echo { payload } => {
                info!("[SimpleExecutor] Executing echo job: {:?}", job.id);
                format!("Echo: {}", payload).into_bytes()
            }
            JobKind::CclWasm => {
                let ctx = self.ctx.as_ref().ok_or_else(|| {
                    CommonError::InternalError(
                        "SimpleExecutor missing context for CCL WASM job".into(),
                    )
                })?;

                // Fetch metadata block from the DAG store
                let meta_bytes = {
                    let store = ctx.dag_store.lock()
                        .map_err(|e| CommonError::InternalError(format!("Failed to lock DAG store: {}", e)))?;
                    store
                        .get(&job.manifest_cid)
                        .map_err(|e| CommonError::InternalError(e.to_string()))?
                        .ok_or_else(|| CommonError::ResourceNotFound("Metadata not found".into()))?
                        .data
                };

                // Parse and validate metadata
                let meta: ContractMetadata = serde_json::from_slice(&meta_bytes)
                    .map_err(|e| CommonError::DeserError(format!("{e}")))?;
                let wasm_cid = icn_common::parse_cid_from_string(&meta.cid)
                    .map_err(|e| CommonError::DeserError(format!("{e}")))?;

                // Ensure the referenced WASM module exists
                {
                    let store = ctx.dag_store.lock()
                        .map_err(|e| CommonError::InternalError(format!("Failed to lock DAG store: {}", e)))?;
                    store
                        .get(&wasm_cid)
                        .map_err(|e| CommonError::InternalError(e.to_string()))?
                        .ok_or_else(|| {
                            CommonError::ResourceNotFound(
                                "Referenced WASM module not found".to_string(),
                            )
                        })?;
                }

                let signer = std::sync::Arc::new(crate::context::StubSigner::new_with_keys(
                    self.signing_key.clone(),
                    self.signing_key.verifying_key(),
                )) as std::sync::Arc<dyn crate::context::Signer>;

                let wasm_exec =
                    WasmExecutor::new(ctx.clone(), signer, WasmExecutorConfig::default());
                let mut wasm_job = job.clone();
                wasm_job.manifest_cid = wasm_cid;
                let receipt = wasm_exec.execute_job(&wasm_job).await?;
                return Ok(receipt);
            }
            JobKind::GenericPlaceholder => {
                info!("[SimpleExecutor] Executing hashing job: {:?}", job.id);

                // Retrieve the manifest bytes from the DAG store
                let ctx = self.ctx.as_ref().ok_or_else(|| {
                    CommonError::InternalError(
                        "SimpleExecutor missing context for hashing job".into(),
                    )
                })?;

                let manifest_bytes = {
                    let store = ctx.dag_store.lock()
                        .map_err(|e| CommonError::InternalError(format!("Failed to lock DAG store: {}", e)))?;
                    store
                        .get(&job.manifest_cid)
                        .map_err(|e| CommonError::InternalError(e.to_string()))?
                }
                .ok_or_else(|| CommonError::ResourceNotFound("Manifest not found".into()))?
                .data;

                // Compute SHA-256 of the manifest bytes
                use sha2::{Digest, Sha256};
                Sha256::digest(&manifest_bytes).to_vec()
            }
        };

        let result_cid = Cid::new_v1_sha256(0x55, &result_bytes);
        let cpu_ms = start_time.elapsed().unwrap_or_default().as_millis() as u64;

        let unsigned_receipt = IdentityExecutionReceipt {
            job_id: job.id.clone().into(),
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
/// Configuration options for [`WasmExecutor`].
#[derive(Clone)]
pub struct WasmExecutorConfig {
    /// Maximum bytes of linear memory a guest may allocate.
    pub max_memory: usize,
    /// Instruction fuel allotted to each execution.
    pub fuel: u64,
    /// Security limits for WASM execution
    pub security_limits: WasmSecurityLimits,
}

impl Default for WasmExecutorConfig {
    fn default() -> Self {
        Self {
            max_memory: 10 * 1024 * 1024, // 10 MiB
            fuel: 1_000_000,
            security_limits: WasmSecurityLimits::default(),
        }
    }
}

pub struct WasmExecutor {
    ctx: std::sync::Arc<crate::context::RuntimeContext>,
    signer: std::sync::Arc<dyn crate::context::Signer>,
    engine: wasmtime::Engine,
    config: WasmExecutorConfig,
    validator: WasmModuleValidator,
}

impl WasmExecutor {
    /// Creates a new [`WasmExecutor`] bound to the given runtime context.
    pub fn new(
        ctx: std::sync::Arc<crate::context::RuntimeContext>,
        signer: std::sync::Arc<dyn crate::context::Signer>,
        config: WasmExecutorConfig,
    ) -> Self {
        let mut wasmtime_config = Config::new();
        wasmtime_config.consume_fuel(true);
        wasmtime_config.async_support(true);
        wasmtime_config.epoch_interruption(true); // Enable epoch-based interruption for timeouts

        // Security configurations
        wasmtime_config.max_wasm_stack(config.security_limits.max_stack_depth as usize * 1024); // Stack in bytes
        wasmtime_config.wasm_multi_memory(false); // Disable multi-memory for security
        wasmtime_config.wasm_threads(false); // Disable threads for security
        wasmtime_config.wasm_reference_types(false); // Disable reference types for simplicity
        wasmtime_config.wasm_bulk_memory(false); // Disable bulk memory operations for security
        wasmtime_config.wasm_simd(false); // Disable SIMD for security

        let engine = wasmtime::Engine::new(&wasmtime_config).expect("create engine");
        let validator = WasmModuleValidator::new(config.security_limits.clone());

        Self {
            ctx,
            signer,
            engine,
            config,
            validator,
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
        let execution_start = Instant::now();

        // Audit log the execution attempt
        info!(
            "WASM execution started: job_id={:?}, executor={}, max_time={}s",
            job.id,
            self.signer.did(),
            self.config.security_limits.max_execution_time_secs
        );

        let wasm_bytes = {
            let store = self
                .ctx
                .dag_store
                .lock()
                .map_err(|e| CommonError::InternalError(format!("Failed to lock DAG store: {}", e)))?;
            let block = store
                .get(&job.manifest_cid)
                .map_err(|e| CommonError::InternalError(e.to_string()))?
                .ok_or_else(|| CommonError::ResourceNotFound("WASM module not found".into()))?;
            block.data
        };

        // Security validation of the WASM module
        self.validator.validate(&wasm_bytes)?;

        // Create store with resource limiter
        let mut store = Store::new(&self.engine, self.ctx.clone());

        // Configure timeout and resource limits
        let timeout_duration =
            Duration::from_secs(self.config.security_limits.max_execution_time_secs);
        let max_mem = self.config.max_memory;
        store.limiter(move |_| {
            Box::leak(Box::new(ICNResourceLimiter::new(max_mem, timeout_duration)))
                as &'static mut dyn ResourceLimiter
        });

        store
            .set_fuel(self.config.fuel)
            .map_err(|e| CommonError::InternalError(format!("Failed to set fuel: {}", e)))?;

        // Set epoch deadline for wall-clock timeout
        self.engine.increment_epoch();
        store.set_epoch_deadline(1);

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

        let ctx_rep = self.ctx.clone();
        linker
            .func_wrap("icn", "host_get_reputation", move || -> i64 {
                let handle = tokio::runtime::Handle::current();
                handle
                    .block_on(async {
                        host_get_reputation(&ctx_rep, &ctx_rep.current_identity).await
                    })
                    .unwrap_or(0)
            })
            .map_err(|e| CommonError::InternalError(e.to_string()))?;

        linker
            .func_wrap(
                "icn",
                "host_submit_mesh_job",
                move |caller: Caller<'_, Arc<RuntimeContext>>, ptr: u32, len: u32| {
                    crate::wasm_host_submit_mesh_job(caller, ptr, len, 0, 0)
                },
            )
            .map_err(|e| CommonError::InternalError(e.to_string()))?;

        linker
            .func_wrap(
                "icn",
                "host_anchor_receipt",
                move |caller: Caller<'_, Arc<RuntimeContext>>, ptr: u32, len: u32| {
                    crate::wasm_host_anchor_receipt(caller, ptr, len, 0, 0)
                },
            )
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

        // Execute with timeout handling
        let result = async {
            // Start a timer for timeout
            let timeout_future = tokio::time::sleep(timeout_duration);
            let execution_future = async {
                func.call(&mut store, ())
                    .map_err(|e| CommonError::InternalError(e.to_string()))
            };

            tokio::select! {
                result = execution_future => result,
                _ = timeout_future => {
                    error!("WASM execution timeout: job_id={:?}, duration={:?}", job.id, timeout_duration);
                    Err(CommonError::InternalError("WASM execution timeout".into()))
                }
            }
        }.await?;

        let execution_duration = execution_start.elapsed();
        let cpu_ms = start_time.elapsed().unwrap_or_default().as_millis() as u64;

        // Audit log the execution completion
        info!(
            "WASM job {} completed in {:?}: {:?}",
            job.id, execution_duration, result
        );

        let result_bytes = result.to_le_bytes();
        let result_cid = Cid::new_v1_sha256(0x55, &result_bytes);

        let executor_did = self.signer.did();
        let mut msg = Vec::new();
        msg.extend_from_slice(job.id.to_string().as_bytes());
        msg.extend_from_slice(executor_did.to_string().as_bytes());
        msg.extend_from_slice(result_cid.to_string().as_bytes());
        msg.extend_from_slice(&cpu_ms.to_le_bytes());
        msg.push(true as u8);
        let sig = self
            .signer
            .sign(&msg)
            .map_err(|e| CommonError::InternalError(format!("{:?}", e)))?;
        let receipt = IdentityExecutionReceipt {
            job_id: job.id.clone().into(),
            executor_did,
            result_cid: result_cid.clone(),
            cpu_ms,
            success: true,
            sig: SignatureBytes(sig),
        };

        // Final audit log
        info!(
            "WASM execution receipt created: job_id={:?}, result_cid={}, success=true",
            job.id, result_cid
        );

        Ok(receipt)
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
        Cid::new_v1_sha256(0x55, s.as_bytes())
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
            id: icn_mesh::JobId::from(job_id.clone()),
            manifest_cid,
            spec: JobSpec {
                kind: JobKind::Echo {
                    payload: "Hello Echo Test".to_string(),
                },
                ..Default::default()
            },
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
            id: icn_mesh::JobId::from(dummy_cid_for_executor_test("job1")),
            manifest_cid: dummy_cid_for_executor_test("manifest1"),
            spec: JobSpec {
                kind: JobKind::Echo {
                    payload: "hello".to_string(),
                },
                ..Default::default()
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

        assert_eq!(receipt.job_id, job.id.into());
        assert_eq!(receipt.executor_did, node_did);
        assert!(!receipt.sig.0.is_empty());
        assert!(receipt.verify_against_key(&node_pk).is_ok());
        info!(
            "Echo job receipt (test_execute_job_echo_success): {:?}",
            receipt
        );
    }

    #[tokio::test]
    async fn test_generic_placeholder_hash_deterministic() {
        use icn_common::{compute_merkle_cid, DagBlock};
        use sha2::{Digest, Sha256};

        let ctx = RuntimeContext::new_with_stubs_and_mana("did:key:zHashTest", 0).unwrap();

        let manifest_data = b"manifest";
        let ts = 0u64;
        let author = Did::new("key", "tester");
        let sig_opt = None;
        let cid = compute_merkle_cid(0x71, manifest_data, &[], ts, &author, &sig_opt, &None);
        let block = DagBlock {
            cid: cid.clone(),
            data: manifest_data.to_vec(),
            links: vec![],
            timestamp: ts,
            author_did: author,
            signature: sig_opt,
            scope: None,
        };
        {
            let mut store = ctx.dag_store.lock().await;
            store.put(&block).await.unwrap();
        }

        let (sk, vk) = generate_keys_for_test();
        let node_did = Did::from_str(&did_key_from_verifying_key(&vk)).unwrap();

        let job = ActualMeshJob {
            id: icn_mesh::JobId::from(dummy_cid_for_executor_test("hash_job")),
            manifest_cid: cid.clone(),
            spec: JobSpec::default(),
            creator_did: node_did.clone(),
            cost_mana: 0,
            max_execution_wait_ms: None,
            signature: SignatureBytes(vec![]),
        };

        let exec = SimpleExecutor::with_context(node_did.clone(), sk, ctx);
        let receipt = exec.execute_job(&job).await.unwrap();

        let expected_bytes = Sha256::digest(manifest_data);
        let expected_cid = Cid::new_v1_sha256(0x55, &expected_bytes);

        assert_eq!(receipt.result_cid, expected_cid);
        assert_eq!(receipt.executor_did, node_did);
        assert!(receipt.verify_against_key(&vk).is_ok());
    }

    #[test]
    fn test_wasm_security_limits_default() {
        let limits = WasmSecurityLimits::default();
        assert_eq!(limits.max_execution_time_secs, 30);
        assert_eq!(limits.max_memory_pages, 160);
        assert_eq!(limits.max_fuel, 1_000_000);
    }

    #[test]
    fn test_wasm_module_validator_validates_size() {
        let limits = WasmSecurityLimits::default();
        let validator = WasmModuleValidator::new(limits);

        // Test with oversized module
        let oversized_wasm = vec![0u8; 51 * 1024 * 1024]; // 51MB
        assert!(validator.validate(&oversized_wasm).is_err());
    }
}
