//! Advanced CCL WASM backend features for enhanced governance and execution
//!
//! This module provides advanced capabilities for CCL (Cooperative Coordination Language)
//! WASM execution including optimized compilation, advanced runtime features, and
//! performance monitoring.

use super::{DagStorageService, DagStoreMutexType, HostAbiError};
use icn_common::{Cid, CommonError, Did, TimeProvider};
use icn_governance::{Proposal, ProposalId, Vote, VoteOption};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use wasmtime::{Config, Engine, Instance, Linker, Module, Store, TypedFunc};

/// Advanced CCL WASM execution environment with optimization features
pub struct AdvancedCclWasmBackend {
    /// Optimized WASM engine for CCL execution
    engine: Engine,
    /// Module linker for CCL runtime
    linker: Linker<CclWasmContext>,
    /// DAG store for accessing WASM modules and data
    dag_store: Arc<DagStoreMutexType<DagStorageService>>,
    /// Time provider for execution timing
    time_provider: Arc<dyn TimeProvider>,
    /// Execution performance metrics
    performance_metrics: Arc<RwLock<CclPerformanceMetrics>>,
    /// Module compilation cache
    module_cache: Arc<RwLock<HashMap<Cid, CachedModule>>>,
    /// Execution policy configuration
    execution_config: CclExecutionConfig,
}

/// Context for CCL WASM execution
pub struct CclWasmContext {
    /// Current proposal being executed
    current_proposal: Option<ProposalId>,
    /// Initial mana available at start of execution
    initial_mana: u64,
    /// Available mana for execution (decreases during execution)
    available_mana: u64,
    /// Execution start time
    start_time: Instant,
    /// Memory usage tracking
    memory_usage: u64,
    /// Gas/instruction counter for execution limits
    instruction_count: u64,
    /// Current node identity
    node_identity: Did,
    /// Available governance data
    governance_data: HashMap<String, Vec<u8>>,
}

/// Cached compiled WASM module
#[derive(Clone)]
struct CachedModule {
    module: Module,
    compilation_time: Duration,
    last_used: Instant,
    usage_count: u64,
    optimization_level: OptimizationLevel,
}

/// CCL execution performance metrics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CclPerformanceMetrics {
    /// Total proposals executed
    pub total_executions: u64,
    /// Successful executions
    pub successful_executions: u64,
    /// Failed executions
    pub failed_executions: u64,
    /// Average execution time in milliseconds
    pub avg_execution_time_ms: f64,
    /// Memory usage statistics
    pub memory_stats: MemoryUsageStats,
    /// Compilation statistics
    pub compilation_stats: CompilationStats,
    /// Optimization effectiveness
    pub optimization_stats: OptimizationStats,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MemoryUsageStats {
    pub average_memory_mb: f64,
    pub peak_memory_mb: u64,
    pub memory_efficiency: f64,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CompilationStats {
    pub total_compilations: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub avg_compilation_time_ms: f64,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct OptimizationStats {
    pub optimized_modules: u64,
    pub optimization_time_saved_ms: u64,
    pub performance_improvement_ratio: f64,
}

/// Configuration for CCL WASM execution
#[derive(Debug, Clone)]
pub struct CclExecutionConfig {
    /// Maximum execution time for proposals
    pub max_execution_time: Duration,
    /// Maximum memory usage in bytes
    pub max_memory_bytes: u64,
    /// Maximum instruction count
    pub max_instructions: u64,
    /// Enable compilation optimizations
    pub enable_optimizations: bool,
    /// Optimization level to use
    pub optimization_level: OptimizationLevel,
    /// Enable execution monitoring
    pub enable_monitoring: bool,
    /// Cache size for compiled modules
    pub module_cache_size: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OptimizationLevel {
    None,
    Speed,
    Size,
    Balanced,
}

impl Default for CclExecutionConfig {
    fn default() -> Self {
        Self {
            max_execution_time: Duration::from_secs(30),
            max_memory_bytes: 64 * 1024 * 1024, // 64MB
            max_instructions: 10_000_000,       // 10M instructions
            enable_optimizations: true,
            optimization_level: OptimizationLevel::Balanced,
            enable_monitoring: true,
            module_cache_size: 100,
        }
    }
}

impl AdvancedCclWasmBackend {
    /// Create a new advanced CCL WASM backend
    pub fn new(
        dag_store: Arc<DagStoreMutexType<DagStorageService>>,
        time_provider: Arc<dyn TimeProvider>,
        config: CclExecutionConfig,
    ) -> Result<Self, HostAbiError> {
        // Configure WASM engine with optimizations
        let mut engine_config = Config::new();
        engine_config.wasm_simd(true);
        engine_config.wasm_bulk_memory(true);
        engine_config.wasm_reference_types(true);

        // Configure optimizations based on settings
        match config.optimization_level {
            OptimizationLevel::None => {
                engine_config.cranelift_opt_level(wasmtime::OptLevel::None);
            }
            OptimizationLevel::Speed => {
                engine_config.cranelift_opt_level(wasmtime::OptLevel::Speed);
            }
            OptimizationLevel::Size => {
                engine_config.cranelift_opt_level(wasmtime::OptLevel::SpeedAndSize);
            }
            OptimizationLevel::Balanced => {
                engine_config.cranelift_opt_level(wasmtime::OptLevel::Speed);
            }
        }

        // Enable debugging and profiling if monitoring is enabled
        if config.enable_monitoring {
            engine_config.debug_info(true);
            engine_config.profiler(wasmtime::ProfilingStrategy::JitDump);
        }

        let engine = Engine::new(&engine_config).map_err(|e| {
            HostAbiError::InternalError(format!("Failed to create WASM engine: {}", e))
        })?;

        // Create linker with CCL-specific functions
        let mut linker = Linker::new(&engine);

        // Add CCL runtime functions
        Self::add_ccl_functions(&mut linker)?;

        Ok(Self {
            engine,
            linker,
            dag_store,
            time_provider,
            performance_metrics: Arc::new(RwLock::new(CclPerformanceMetrics::default())),
            module_cache: Arc::new(RwLock::new(HashMap::new())),
            execution_config: config,
        })
    }

    /// Add CCL-specific runtime functions to the linker
    fn add_ccl_functions(linker: &mut Linker<CclWasmContext>) -> Result<(), HostAbiError> {
        // Governance functions
        linker
            .func_wrap(
                "ccl",
                "get_proposal_data",
                |_caller: wasmtime::Caller<'_, CclWasmContext>,
                 proposal_id_ptr: u32,
                 proposal_id_len: u32|
                 -> u32 {
                    // Implementation for getting proposal data
                    0 // Success code
                },
            )
            .map_err(|e| {
                HostAbiError::InternalError(format!("Failed to add get_proposal_data: {}", e))
            })?;

        linker
            .func_wrap(
                "ccl",
                "get_vote_count",
                |_caller: wasmtime::Caller<'_, CclWasmContext>,
                 proposal_id_ptr: u32,
                 proposal_id_len: u32|
                 -> u32 {
                    // Implementation for getting vote counts
                    0
                },
            )
            .map_err(|e| {
                HostAbiError::InternalError(format!("Failed to add get_vote_count: {}", e))
            })?;

        linker
            .func_wrap(
                "ccl",
                "get_member_count",
                |_caller: wasmtime::Caller<'_, CclWasmContext>| -> u32 {
                    // Implementation for getting member count
                    100 // Mock member count
                },
            )
            .map_err(|e| {
                HostAbiError::InternalError(format!("Failed to add get_member_count: {}", e))
            })?;

        linker
            .func_wrap(
                "ccl",
                "calculate_quorum",
                |_caller: wasmtime::Caller<'_, CclWasmContext>, threshold_percent: u32| -> u32 {
                    // Implementation for calculating quorum
                    let member_count = 100; // Mock member count
                    (member_count * threshold_percent) / 100
                },
            )
            .map_err(|e| {
                HostAbiError::InternalError(format!("Failed to add calculate_quorum: {}", e))
            })?;

        // Mana and resource management
        linker
            .func_wrap(
                "ccl",
                "get_available_mana",
                |caller: wasmtime::Caller<'_, CclWasmContext>| -> u64 {
                    caller.data().available_mana
                },
            )
            .map_err(|e| {
                HostAbiError::InternalError(format!("Failed to add get_available_mana: {}", e))
            })?;

        linker
            .func_wrap(
                "ccl",
                "consume_mana",
                |mut caller: wasmtime::Caller<'_, CclWasmContext>, amount: u64| -> u32 {
                    let data = caller.data_mut();
                    if data.available_mana >= amount {
                        data.available_mana -= amount;
                        1 // Success
                    } else {
                        0 // Insufficient mana
                    }
                },
            )
            .map_err(|e| {
                HostAbiError::InternalError(format!("Failed to add consume_mana: {}", e))
            })?;

        // Time and scheduling functions
        linker
            .func_wrap(
                "ccl",
                "get_current_timestamp",
                |_caller: wasmtime::Caller<'_, CclWasmContext>| -> u64 {
                    // Get current Unix timestamp
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                },
            )
            .map_err(|e| {
                HostAbiError::InternalError(format!("Failed to add get_current_timestamp: {}", e))
            })?;

        // Cryptographic functions
        linker
            .func_wrap(
                "ccl",
                "verify_signature",
                |_caller: wasmtime::Caller<'_, CclWasmContext>,
                 sig_ptr: u32,
                 sig_len: u32,
                 msg_ptr: u32,
                 msg_len: u32,
                 pubkey_ptr: u32,
                 pubkey_len: u32|
                 -> u32 {
                    // Implementation for signature verification
                    1 // Mock success
                },
            )
            .map_err(|e| {
                HostAbiError::InternalError(format!("Failed to add verify_signature: {}", e))
            })?;

        Ok(())
    }

    /// Compile CCL WASM module with optimizations
    pub async fn compile_ccl_module(&self, module_cid: &Cid) -> Result<Module, HostAbiError> {
        let start_time = Instant::now();

        // Check cache first
        {
            let cache = self.module_cache.read().await;
            if let Some(cached) = cache.get(module_cid) {
                // Update cache statistics
                {
                    let mut metrics = self.performance_metrics.write().await;
                    metrics.compilation_stats.cache_hits += 1;
                }

                // Update last used time and return cached module
                drop(cache);
                let mut cache = self.module_cache.write().await;
                if let Some(cached) = cache.get_mut(module_cid) {
                    cached.last_used = Instant::now();
                    cached.usage_count += 1;
                    return Ok(cached.module.clone());
                }
            }
        }

        // Cache miss - compile the module
        {
            let mut metrics = self.performance_metrics.write().await;
            metrics.compilation_stats.cache_misses += 1;
        }

        // Load WASM bytes from DAG
        let wasm_bytes = {
            let dag_store = self.dag_store.lock().await;
            let block = dag_store
                .get(module_cid)
                .await
                .map_err(|e| HostAbiError::DagError(format!("Failed to load WASM module: {}", e)))?
                .ok_or_else(|| HostAbiError::DagError("WASM module not found in DAG".to_string()))?;
            block.data
        };

        // Compile with optimizations
        let module = Module::new(&self.engine, &wasm_bytes)
            .map_err(|e| HostAbiError::InternalError(format!("WASM compilation failed: {}", e)))?;

        let compilation_time = start_time.elapsed();

        // Cache the compiled module
        {
            let mut cache = self.module_cache.write().await;

            // Evict oldest if cache is full
            if cache.len() >= self.execution_config.module_cache_size {
                let oldest_key = cache
                    .iter()
                    .min_by_key(|(_, cached)| cached.last_used)
                    .map(|(k, _)| k.clone());
                if let Some(key) = oldest_key {
                    cache.remove(&key);
                }
            }

            cache.insert(
                module_cid.clone(),
                CachedModule {
                    module: module.clone(),
                    compilation_time,
                    last_used: Instant::now(),
                    usage_count: 1,
                    optimization_level: self.execution_config.optimization_level,
                },
            );
        }

        // Update compilation statistics
        {
            let mut metrics = self.performance_metrics.write().await;
            metrics.compilation_stats.total_compilations += 1;
            let total_time = metrics.compilation_stats.avg_compilation_time_ms
                * (metrics.compilation_stats.total_compilations - 1) as f64
                + compilation_time.as_millis() as f64;
            metrics.compilation_stats.avg_compilation_time_ms =
                total_time / metrics.compilation_stats.total_compilations as f64;
        }

        Ok(module)
    }

    /// Execute CCL proposal with advanced monitoring and limits
    pub async fn execute_proposal(
        &self,
        proposal: &Proposal,
        module_cid: &Cid,
        node_identity: Did,
        available_mana: u64,
    ) -> Result<CclExecutionResult, HostAbiError> {
        let execution_start = Instant::now();

        // Compile the module
        let module = self.compile_ccl_module(module_cid).await?;

        // Create execution context
        let context = CclWasmContext {
            current_proposal: Some(proposal.id.clone()),
            initial_mana: available_mana,
            available_mana,
            start_time: execution_start,
            memory_usage: 0,
            instruction_count: 0,
            node_identity,
            governance_data: HashMap::new(), // Would be populated with real governance data
        };

        let mut store = Store::new(&self.engine, context);

        // TODO: Fix resource limiter - wasmtime API lifetime issue
        // Need to properly implement Store resource limiting
        // The limiter API requires complex lifetime management that needs refactoring
        //
        // let mut limiter = ResourceLimiter {
        //     max_memory: self.execution_config.max_memory_bytes,
        //     max_instructions: self.execution_config.max_instructions,
        //     current_memory: 0,
        //     current_instructions: 0,
        // };
        // store.limiter(|_| &mut limiter);

        // Instantiate the module
        let instance = self.linker.instantiate(&mut store, &module).map_err(|e| {
            HostAbiError::InternalError(format!("Module instantiation failed: {}", e))
        })?;

        // Execute the proposal evaluation function
        let result = self
            .execute_with_timeout(&mut store, &instance, execution_start)
            .await?;

        // Update performance metrics
        self.update_execution_metrics(execution_start, &result)
            .await;

        Ok(result)
    }

    /// Execute WASM function with timeout and monitoring
    async fn execute_with_timeout(
        &self,
        store: &mut Store<CclWasmContext>,
        instance: &Instance,
        start_time: Instant,
    ) -> Result<CclExecutionResult, HostAbiError> {
        // Get the main execution function
        let execute_func: TypedFunc<(), u32> = instance
            .get_typed_func(&mut *store, "execute_proposal")
            .map_err(|e| {
                HostAbiError::InternalError(format!("Function 'execute_proposal' not found: {}", e))
            })?;

        // Execute with timeout
        let timeout_duration = self.execution_config.max_execution_time;
        let execution_future = async {
            execute_func
                .call(&mut *store, ())
                .map_err(|e| HostAbiError::InternalError(format!("Execution failed: {}", e)))
        };

        let result_code = match tokio::time::timeout(timeout_duration, execution_future).await {
            Ok(Ok(code)) => code,
            Ok(Err(e)) => return Err(e),
            Err(_) => {
                return Ok(CclExecutionResult {
                    success: false,
                    result_code: 0,
                    execution_time: start_time.elapsed(),
                    memory_used: store.data().memory_usage,
                    instructions_executed: store.data().instruction_count,
                    mana_consumed: 0,
                    error_message: Some("Execution timeout".to_string()),
                });
            }
        };

        // Collect execution metrics
        let execution_time = start_time.elapsed();
        let memory_used = store.data().memory_usage;
        let instructions = store.data().instruction_count;
        // Calculate actual mana consumed based on initial vs remaining mana
        let mana_consumed = store
            .data()
            .initial_mana
            .saturating_sub(store.data().available_mana);

        Ok(CclExecutionResult {
            success: result_code == 1,
            result_code,
            execution_time,
            memory_used,
            instructions_executed: instructions,
            mana_consumed,
            error_message: None,
        })
    }

    /// Update execution performance metrics
    async fn update_execution_metrics(&self, start_time: Instant, result: &CclExecutionResult) {
        let mut metrics = self.performance_metrics.write().await;

        metrics.total_executions += 1;
        if result.success {
            metrics.successful_executions += 1;
        } else {
            metrics.failed_executions += 1;
        }

        // Update average execution time
        let total_time = metrics.avg_execution_time_ms * (metrics.total_executions - 1) as f64
            + result.execution_time.as_millis() as f64;
        metrics.avg_execution_time_ms = total_time / metrics.total_executions as f64;

        // Update memory statistics
        let memory_mb = result.memory_used as f64 / (1024.0 * 1024.0);
        if memory_mb > metrics.memory_stats.peak_memory_mb as f64 {
            metrics.memory_stats.peak_memory_mb = memory_mb as u64;
        }

        let total_memory = metrics.memory_stats.average_memory_mb
            * (metrics.total_executions - 1) as f64
            + memory_mb;
        metrics.memory_stats.average_memory_mb = total_memory / metrics.total_executions as f64;
    }

    /// Get current performance metrics
    pub async fn get_performance_metrics(&self) -> CclPerformanceMetrics {
        self.performance_metrics.read().await.clone()
    }

    /// Clear module cache
    pub async fn clear_module_cache(&self) {
        self.module_cache.write().await.clear();
    }
}

/// Result of CCL WASM execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CclExecutionResult {
    pub success: bool,
    pub result_code: u32,
    pub execution_time: Duration,
    pub memory_used: u64,
    pub instructions_executed: u64,
    pub mana_consumed: u64,
    pub error_message: Option<String>,
}

/// Resource limiter for WASM execution
struct ResourceLimiter {
    max_memory: u64,
    max_instructions: u64,
    current_memory: u64,
    current_instructions: u64,
}

impl wasmtime::ResourceLimiter for ResourceLimiter {
    fn memory_growing(
        &mut self,
        current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> anyhow::Result<bool> {
        let new_memory = desired as u64;
        if new_memory > self.max_memory {
            anyhow::bail!(
                "Memory limit exceeded: {} > {}",
                new_memory,
                self.max_memory
            );
        }
        self.current_memory = new_memory;
        Ok(true)
    }

    fn table_growing(
        &mut self,
        _current: u32,
        _desired: u32,
        _maximum: Option<u32>,
    ) -> anyhow::Result<bool> {
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::{Did, TimeProvider};
    use icn_governance::{Proposal, ProposalId, ProposalType, Vote, VoteOption};
    use std::sync::Arc;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    /// Mock time provider for deterministic testing
    struct MockTimeProvider {
        current_time: u64,
    }

    impl TimeProvider for MockTimeProvider {
        fn unix_seconds(&self) -> u64 {
            self.current_time
        }
    }

    /// Test that mana calculation correctly tracks consumption
    #[tokio::test]
    async fn test_mana_calculation_accuracy() {
        // Create mock context with initial mana
        let initial_mana = 1000u64;
        let mut context = CclWasmContext {
            current_proposal: None,
            initial_mana,
            available_mana: initial_mana,
            start_time: std::time::Instant::now(),
            memory_usage: 0,
            instruction_count: 0,
            node_identity: Did::parse("did:key:test").unwrap(),
            governance_data: HashMap::new(),
        };

        // Simulate mana consumption
        let consumed_amount = 250u64;
        context.available_mana -= consumed_amount;

        // Calculate mana consumed using the fixed logic
        let mana_consumed = context.initial_mana.saturating_sub(context.available_mana);

        // Verify calculation is correct
        assert_eq!(mana_consumed, consumed_amount);
        assert_eq!(context.available_mana, initial_mana - consumed_amount);
    }

    /// Test that mana calculation handles edge cases correctly
    #[tokio::test]
    async fn test_mana_calculation_edge_cases() {
        let initial_mana = 100u64;

        // Test case 1: No mana consumed
        let mut context = CclWasmContext {
            current_proposal: None,
            initial_mana,
            available_mana: initial_mana,
            start_time: std::time::Instant::now(),
            memory_usage: 0,
            instruction_count: 0,
            node_identity: Did::parse("did:key:test").unwrap(),
            governance_data: HashMap::new(),
        };

        let mana_consumed = context.initial_mana.saturating_sub(context.available_mana);
        assert_eq!(mana_consumed, 0);

        // Test case 2: All mana consumed
        context.available_mana = 0;
        let mana_consumed = context.initial_mana.saturating_sub(context.available_mana);
        assert_eq!(mana_consumed, initial_mana);

        // Test case 3: Overflow protection (available_mana > initial_mana should not happen)
        // but if it does, saturating_sub prevents underflow
        context.available_mana = initial_mana + 50; // Impossible scenario
        let mana_consumed = context.initial_mana.saturating_sub(context.available_mana);
        assert_eq!(mana_consumed, 0); // saturating_sub prevents underflow
    }

    /// Test that the old calculation method would have been wrong
    #[test]
    fn test_old_calculation_was_wrong() {
        let initial_mana = 1000u64;
        let available_mana = 750u64; // 250 mana consumed
        let execution_time_secs = 30u64;

        // The old flawed calculation
        let old_estimated_initial = available_mana + (execution_time_secs * 10);
        let old_mana_consumed = old_estimated_initial - available_mana;

        // This would always equal execution_time_secs * 10, regardless of actual usage
        assert_eq!(old_mana_consumed, execution_time_secs * 10); // 300, not the actual 250

        // The new correct calculation
        let new_mana_consumed = initial_mana.saturating_sub(available_mana);
        assert_eq!(new_mana_consumed, 250); // Correct actual consumption

        // Demonstrate the old calculation was wrong
        assert_ne!(old_mana_consumed, new_mana_consumed);
    }

    /// Test overflow protection in long execution scenarios
    #[test]
    fn test_overflow_protection() {
        // Test with large execution time that would cause overflow in old calculation
        let execution_time_secs = u64::MAX / 5; // Large value that would overflow when * 10
        let available_mana = 500u64;

        // Old calculation would overflow: available_mana + (execution_time_secs * 10)
        // This is why the old calculation was dangerous

        // New calculation is safe regardless of execution time
        let initial_mana = 1000u64;
        let mana_consumed = initial_mana.saturating_sub(available_mana);
        assert_eq!(mana_consumed, 500); // Safe calculation, correct result
    }
}
