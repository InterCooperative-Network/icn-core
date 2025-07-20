use once_cell::sync::Lazy;
use prometheus_client::encoding::text::encode;
use prometheus_client::metrics::{counter::Counter, gauge::Gauge, histogram::Histogram};
use prometheus_client::registry::Registry;
use std::sync::atomic::AtomicU64;

/// Additional system-level metrics for ICN nodes
pub mod system {
    use super::*;

    /// Node startup timestamp (Unix seconds)
    pub static NODE_START_TIME: Lazy<Gauge<f64, AtomicU64>> = Lazy::new(Gauge::default);

    /// Current system memory usage in bytes
    pub static SYSTEM_MEMORY_USAGE: Lazy<Gauge<f64, AtomicU64>> = Lazy::new(Gauge::default);

    /// Current system CPU usage percentage
    pub static SYSTEM_CPU_USAGE: Lazy<Gauge<f64, AtomicU64>> = Lazy::new(Gauge::default);

    /// Number of active HTTP connections
    pub static HTTP_ACTIVE_CONNECTIONS: Lazy<Gauge<i64>> = Lazy::new(Gauge::default);

    /// HTTP request duration histogram
    pub static HTTP_REQUEST_DURATION: Lazy<Histogram> = Lazy::new(|| {
        Histogram::new([0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0, 60.0].into_iter())
    });

    /// HTTP request counter by status code
    pub static HTTP_REQUESTS_TOTAL: Lazy<Counter> = Lazy::new(Counter::default);

    /// HTTP errors counter
    pub static HTTP_ERRORS_TOTAL: Lazy<Counter> = Lazy::new(Counter::default);

    /// Rate limit violations counter
    pub static RATE_LIMIT_VIOLATIONS: Lazy<Counter> = Lazy::new(Counter::default);

    /// Authentication failures counter
    pub static AUTH_FAILURES: Lazy<Counter> = Lazy::new(Counter::default);
}

/// ICN-specific application metrics
pub mod application {
    use super::*;

    /// Total number of federation members
    pub static FEDERATION_MEMBERS: Lazy<Gauge<i64>> = Lazy::new(Gauge::default);

    /// Active consensus rounds
    pub static ACTIVE_CONSENSUS_ROUNDS: Lazy<Gauge<i64>> = Lazy::new(Gauge::default);

    /// Total mana in circulation
    pub static TOTAL_MANA_SUPPLY: Lazy<Gauge<f64, AtomicU64>> = Lazy::new(Gauge::default);

    /// Average mana regeneration rate
    pub static MANA_REGENERATION_RATE: Lazy<Gauge<f64, AtomicU64>> = Lazy::new(Gauge::default);

    /// Number of active DIDs
    pub static ACTIVE_DIDS: Lazy<Gauge<i64>> = Lazy::new(Gauge::default);

    /// DAG block height (latest block)
    pub static DAG_BLOCK_HEIGHT: Lazy<Gauge<i64>> = Lazy::new(Gauge::default);

    /// DAG storage size in bytes
    pub static DAG_STORAGE_SIZE: Lazy<Gauge<f64, AtomicU64>> = Lazy::new(Gauge::default);

    /// CCL contracts deployed
    pub static CCL_CONTRACTS_DEPLOYED: Lazy<Counter> = Lazy::new(Counter::default);

    /// CCL contract execution time
    pub static CCL_EXECUTION_TIME: Lazy<Histogram> =
        Lazy::new(|| Histogram::new([0.001, 0.01, 0.1, 1.0, 10.0, 100.0].into_iter()));
}

/// Registers common ICN metrics (job execution, governance actions, network health)
/// to the provided registry with improved organization and additional metrics.
pub fn register_core_metrics(registry: &mut Registry) {
    // Core metrics from various crates
    register_dag_metrics(registry);
    register_governance_metrics(registry);
    register_identity_metrics(registry);
    register_reputation_metrics(registry);
    register_network_metrics(registry);

    // Runtime-specific metrics (conditionally compiled)
    #[cfg(feature = "runtime-metrics")]
    {
        register_economics_metrics(registry);
        register_mesh_metrics(registry);
        register_runtime_metrics(registry);
    }

    // System-level metrics
    register_system_metrics(registry);

    // Application-level metrics
    register_application_metrics(registry);
}

/// Register DAG-related metrics
fn register_dag_metrics(registry: &mut Registry) {
    use icn_dag::metrics::{DAG_GET_CALLS, DAG_PUT_CALLS};

    registry.register(
        "dag_put_calls_total",
        "Number of DAG put operations",
        DAG_PUT_CALLS.clone(),
    );
    registry.register(
        "dag_get_calls_total",
        "Number of DAG get operations",
        DAG_GET_CALLS.clone(),
    );
}

/// Register governance-related metrics
fn register_governance_metrics(registry: &mut Registry) {
    use icn_governance::metrics::{CAST_VOTE_CALLS, EXECUTE_PROPOSAL_CALLS, SUBMIT_PROPOSAL_CALLS};

    registry.register(
        "governance_proposals_submitted_total",
        "Number of governance proposals submitted",
        SUBMIT_PROPOSAL_CALLS.clone(),
    );
    registry.register(
        "governance_votes_cast_total",
        "Number of votes cast on proposals",
        CAST_VOTE_CALLS.clone(),
    );
    registry.register(
        "governance_proposals_executed_total",
        "Number of proposals executed",
        EXECUTE_PROPOSAL_CALLS.clone(),
    );
}

/// Register identity-related metrics
fn register_identity_metrics(registry: &mut Registry) {
    use icn_identity::metrics::{CREDENTIALS_ISSUED, PROOFS_VERIFIED, PROOF_VERIFICATION_FAILURES};

    registry.register(
        "identity_credentials_issued_total",
        "Number of credentials issued",
        CREDENTIALS_ISSUED.clone(),
    );
    registry.register(
        "identity_proofs_verified_total",
        "Number of successful proof verifications",
        PROOFS_VERIFIED.clone(),
    );
    registry.register(
        "identity_proof_verification_failures_total",
        "Number of failed proof verifications",
        PROOF_VERIFICATION_FAILURES.clone(),
    );
}

/// Register reputation-related metrics
fn register_reputation_metrics(registry: &mut Registry) {
    use icn_reputation::metrics::{EXECUTION_RECORDS, PROOF_ATTEMPTS};

    registry.register(
        "reputation_execution_records_total",
        "Number of execution records processed",
        EXECUTION_RECORDS.clone(),
    );
    registry.register(
        "reputation_proof_attempts_total",
        "Number of reputation proof attempts",
        PROOF_ATTEMPTS.clone(),
    );
}

/// Register network-related metrics
fn register_network_metrics(registry: &mut Registry) {
    use icn_network::metrics::{
        BYTES_RECEIVED_TOTAL, BYTES_SENT_TOTAL, KADEMLIA_PEERS_GAUGE, MESSAGES_RECEIVED_TOTAL,
        MESSAGES_SENT_TOTAL, PEER_COUNT_GAUGE, PING_AVG_RTT_MS, PING_LAST_RTT_MS, PING_MAX_RTT_MS,
        PING_MIN_RTT_MS,
    };

    registry.register(
        "network_peer_count",
        "Number of connected peers",
        PEER_COUNT_GAUGE.clone(),
    );
    registry.register(
        "network_kademlia_peers",
        "Number of peers in Kademlia routing table",
        KADEMLIA_PEERS_GAUGE.clone(),
    );
    registry.register(
        "network_bytes_sent_total",
        "Total bytes sent over network",
        BYTES_SENT_TOTAL.clone(),
    );
    registry.register(
        "network_bytes_received_total",
        "Total bytes received over network",
        BYTES_RECEIVED_TOTAL.clone(),
    );
    registry.register(
        "network_messages_sent_total",
        "Total messages sent over network",
        MESSAGES_SENT_TOTAL.clone(),
    );
    registry.register(
        "network_messages_received_total",
        "Total messages received over network",
        MESSAGES_RECEIVED_TOTAL.clone(),
    );
    registry.register(
        "network_ping_rtt_last_ms",
        "Last measured ping round-trip time in milliseconds",
        PING_LAST_RTT_MS.clone(),
    );
    registry.register(
        "network_ping_rtt_min_ms",
        "Minimum ping round-trip time in milliseconds",
        PING_MIN_RTT_MS.clone(),
    );
    registry.register(
        "network_ping_rtt_max_ms",
        "Maximum ping round-trip time in milliseconds",
        PING_MAX_RTT_MS.clone(),
    );
    registry.register(
        "network_ping_rtt_avg_ms",
        "Average ping round-trip time in milliseconds",
        PING_AVG_RTT_MS.clone(),
    );
}

/// Register economics-related metrics
#[cfg(feature = "runtime-metrics")]
fn register_economics_metrics(registry: &mut Registry) {
    use icn_economics::metrics::{CREDIT_MANA_CALLS, GET_BALANCE_CALLS, SPEND_MANA_CALLS};

    registry.register(
        "economics_mana_balance_queries_total",
        "Number of mana balance queries",
        GET_BALANCE_CALLS.clone(),
    );
    registry.register(
        "economics_mana_spending_operations_total",
        "Number of mana spending operations",
        SPEND_MANA_CALLS.clone(),
    );
    registry.register(
        "economics_mana_credit_operations_total",
        "Number of mana credit operations",
        CREDIT_MANA_CALLS.clone(),
    );
}

/// Register mesh-related metrics
#[cfg(feature = "runtime-metrics")]
fn register_mesh_metrics(registry: &mut Registry) {
    use icn_mesh::metrics::{
        BIDS_RECEIVED_TOTAL, JOBS_ASSIGNED_TOTAL, JOBS_BIDDING_GAUGE, JOBS_COMPLETED_TOTAL,
        JOBS_EXECUTING_GAUGE, JOBS_FAILED_TOTAL, JOBS_SUBMITTED_TOTAL, JOB_ASSIGNMENT_TIME,
        JOB_COMPLETION_TIME, JOB_PROCESS_TIME, PENDING_JOBS_GAUGE, SCHEDULE_MESH_JOB_CALLS,
        SELECT_EXECUTOR_CALLS,
    };

    registry.register(
        "mesh_executor_selection_calls_total",
        "Number of executor selection operations",
        SELECT_EXECUTOR_CALLS.clone(),
    );
    registry.register(
        "mesh_job_scheduling_calls_total",
        "Number of job scheduling operations",
        SCHEDULE_MESH_JOB_CALLS.clone(),
    );
    registry.register(
        "mesh_jobs_pending",
        "Number of jobs currently pending",
        PENDING_JOBS_GAUGE.clone(),
    );
    registry.register(
        "mesh_jobs_submitted_total",
        "Total number of jobs submitted",
        JOBS_SUBMITTED_TOTAL.clone(),
    );
    registry.register(
        "mesh_jobs_completed_total",
        "Total number of jobs completed successfully",
        JOBS_COMPLETED_TOTAL.clone(),
    );
    registry.register(
        "mesh_jobs_failed_total",
        "Total number of jobs that failed",
        JOBS_FAILED_TOTAL.clone(),
    );
    registry.register(
        "mesh_jobs_assigned_total",
        "Total number of jobs assigned to executors",
        JOBS_ASSIGNED_TOTAL.clone(),
    );
    registry.register(
        "mesh_bids_received_total",
        "Total number of bids received",
        BIDS_RECEIVED_TOTAL.clone(),
    );
    registry.register(
        "mesh_jobs_bidding",
        "Number of jobs currently in bidding phase",
        JOBS_BIDDING_GAUGE.clone(),
    );
    registry.register(
        "mesh_jobs_executing",
        "Number of jobs currently executing",
        JOBS_EXECUTING_GAUGE.clone(),
    );
    registry.register(
        "mesh_job_assignment_duration_seconds",
        "Time from job submission to assignment",
        JOB_ASSIGNMENT_TIME.clone(),
    );
    registry.register(
        "mesh_job_execution_duration_seconds",
        "Time from job submission to completion",
        JOB_COMPLETION_TIME.clone(),
    );
    registry.register(
        "mesh_job_processing_duration_seconds",
        "Time from job assignment to receipt processing",
        JOB_PROCESS_TIME.clone(),
    );
}

/// Register runtime-related metrics
#[cfg(feature = "runtime-metrics")]
fn register_runtime_metrics(registry: &mut Registry) {
    // TODO: Move this to icn-runtime to avoid circular dependency
    // The runtime should register its own metrics directly
    // use icn_runtime::metrics::{
    //     HOST_ACCOUNT_CREDIT_MANA_CALLS, HOST_ACCOUNT_GET_MANA_CALLS, HOST_ACCOUNT_SPEND_MANA_CALLS,
    //     HOST_ANCHOR_RECEIPT_CALLS, HOST_GET_PENDING_MESH_JOBS_CALLS, HOST_SUBMIT_MESH_JOB_CALLS,
    //     MANA_ACCOUNTS_GAUGE, RECEIPTS_ANCHORED, WASM_MEMORY_GROWTH_DENIED,
    //     WASM_TABLE_GROWTH_DENIED,
    // };

    // TODO: Temporarily commented out to fix circular dependency
    // These metrics should be registered directly by icn-runtime
    /*
    registry.register(
        "runtime_host_submit_mesh_job_calls_total",
        "Number of host_submit_mesh_job calls",
        HOST_SUBMIT_MESH_JOB_CALLS.clone(),
    );
    registry.register(
        "runtime_host_get_pending_mesh_jobs_calls_total",
        "Number of host_get_pending_mesh_jobs calls",
        HOST_GET_PENDING_MESH_JOBS_CALLS.clone(),
    );
    registry.register(
        "runtime_host_account_get_mana_calls_total",
        "Number of host_account_get_mana calls",
        HOST_ACCOUNT_GET_MANA_CALLS.clone(),
    );
    registry.register(
        "runtime_host_account_spend_mana_calls_total",
        "Number of host_account_spend_mana calls",
        HOST_ACCOUNT_SPEND_MANA_CALLS.clone(),
    );
    registry.register(
        "runtime_host_account_credit_mana_calls_total",
        "Number of host_account_credit_mana calls",
        HOST_ACCOUNT_CREDIT_MANA_CALLS.clone(),
    );
    registry.register(
        "runtime_host_anchor_receipt_calls_total",
        "Number of host_anchor_receipt calls",
        HOST_ANCHOR_RECEIPT_CALLS.clone(),
    );
    registry.register(
        "runtime_receipts_anchored_total",
        "Number of receipts anchored to the DAG",
        RECEIPTS_ANCHORED.clone(),
    );
    */
    /*
    registry.register(
        "runtime_wasm_memory_growth_denied_total",
        "Number of denied WASM memory growth attempts",
        WASM_MEMORY_GROWTH_DENIED.clone(),
    );
    registry.register(
        "runtime_wasm_table_growth_denied_total",
        "Number of denied WASM table growth attempts",
        WASM_TABLE_GROWTH_DENIED.clone(),
    );
    registry.register(
        "mana_accounts",
        "Number of accounts in the mana ledger",
        MANA_ACCOUNTS_GAUGE.clone(),
    );
    */
}

/// Register system-level metrics
fn register_system_metrics(registry: &mut Registry) {
    registry.register(
        "node_start_time_seconds",
        "Unix timestamp when the node started",
        system::NODE_START_TIME.clone(),
    );
    registry.register(
        "system_memory_usage_bytes",
        "Current system memory usage in bytes",
        system::SYSTEM_MEMORY_USAGE.clone(),
    );
    registry.register(
        "system_cpu_usage_percent",
        "Current system CPU usage percentage",
        system::SYSTEM_CPU_USAGE.clone(),
    );
    registry.register(
        "http_active_connections",
        "Number of active HTTP connections",
        system::HTTP_ACTIVE_CONNECTIONS.clone(),
    );
    registry.register(
        "http_request_duration_seconds",
        "HTTP request duration in seconds",
        system::HTTP_REQUEST_DURATION.clone(),
    );
    registry.register(
        "http_requests_total",
        "Total number of HTTP requests",
        system::HTTP_REQUESTS_TOTAL.clone(),
    );
    registry.register(
        "http_errors_total",
        "Total number of HTTP errors",
        system::HTTP_ERRORS_TOTAL.clone(),
    );
    registry.register(
        "rate_limit_violations_total",
        "Total number of rate limit violations",
        system::RATE_LIMIT_VIOLATIONS.clone(),
    );
    registry.register(
        "auth_failures_total",
        "Total number of authentication failures",
        system::AUTH_FAILURES.clone(),
    );
}

/// Register application-level metrics
fn register_application_metrics(registry: &mut Registry) {
    registry.register(
        "federation_members_total",
        "Total number of federation members",
        application::FEDERATION_MEMBERS.clone(),
    );
    registry.register(
        "consensus_rounds_active",
        "Number of active consensus rounds",
        application::ACTIVE_CONSENSUS_ROUNDS.clone(),
    );
    registry.register(
        "mana_supply_total",
        "Total mana in circulation",
        application::TOTAL_MANA_SUPPLY.clone(),
    );
    registry.register(
        "mana_regeneration_rate",
        "Average mana regeneration rate",
        application::MANA_REGENERATION_RATE.clone(),
    );
    registry.register(
        "active_dids_total",
        "Number of active DIDs in the network",
        application::ACTIVE_DIDS.clone(),
    );
    registry.register(
        "dag_block_height",
        "Current DAG block height",
        application::DAG_BLOCK_HEIGHT.clone(),
    );
    registry.register(
        "dag_storage_size_bytes",
        "Total DAG storage size in bytes",
        application::DAG_STORAGE_SIZE.clone(),
    );
    registry.register(
        "ccl_contracts_deployed_total",
        "Total number of CCL contracts deployed",
        application::CCL_CONTRACTS_DEPLOYED.clone(),
    );
    registry.register(
        "ccl_execution_duration_seconds",
        "CCL contract execution time in seconds",
        application::CCL_EXECUTION_TIME.clone(),
    );
}

/// Returns Prometheus metrics in text format for the registered core metrics.
pub fn collect_metrics() -> String {
    let mut registry = Registry::default();
    register_core_metrics(&mut registry);
    let mut buffer = String::new();
    encode(&mut buffer, &registry).unwrap();
    buffer
}

/// Helper function to update system metrics (to be called periodically)
pub fn update_system_metrics() {
    use std::time::{SystemTime, UNIX_EPOCH};

    // Update node start time if not already set
    if system::NODE_START_TIME.get() == 0.0 {
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        system::NODE_START_TIME.set(start_time as f64);
    }

    // Update system memory usage
    #[cfg(target_os = "linux")]
    {
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    if let Some(kb_str) = line.split_whitespace().nth(1) {
                        if let Ok(kb) = kb_str.parse::<u64>() {
                            system::SYSTEM_MEMORY_USAGE.set((kb * 1024) as f64);
                        }
                    }
                    break;
                }
            }
        }
    }

    // Update CPU usage (basic implementation)
    #[cfg(target_os = "linux")]
    {
        if let Ok(stat) = std::fs::read_to_string("/proc/self/stat") {
            let fields: Vec<&str> = stat.split_whitespace().collect();
            if fields.len() > 15 {
                if let (Ok(utime), Ok(stime)) =
                    (fields[13].parse::<u64>(), fields[14].parse::<u64>())
                {
                    // This is a simplified CPU usage calculation
                    // In production, you'd want to calculate this over time intervals
                    let total_time = utime + stime;
                    system::SYSTEM_CPU_USAGE.set(total_time as f64);
                }
            }
        }
    }
}

#[cfg(feature = "axum-endpoint")]
pub async fn metrics_handler() -> impl axum::response::IntoResponse {
    update_system_metrics();
    let text = collect_metrics();
    (axum::http::StatusCode::OK, text)
}
