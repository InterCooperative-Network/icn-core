use once_cell::sync::Lazy;
use prometheus_client::metrics::{counter::Counter, gauge::Gauge};

/// Counts calls to `host_submit_mesh_job`.
pub static HOST_SUBMIT_MESH_JOB_CALLS: Lazy<Counter> = Lazy::new(Counter::default);

/// Counts calls to `host_get_pending_mesh_jobs`.
pub static HOST_GET_PENDING_MESH_JOBS_CALLS: Lazy<Counter> = Lazy::new(Counter::default);

/// Counts calls to `host_account_get_mana`.
pub static HOST_ACCOUNT_GET_MANA_CALLS: Lazy<Counter> = Lazy::new(Counter::default);

/// Counts calls to `host_account_spend_mana`.
pub static HOST_ACCOUNT_SPEND_MANA_CALLS: Lazy<Counter> = Lazy::new(Counter::default);

/// Counts mesh jobs submitted via the runtime.
pub static JOBS_SUBMITTED: Lazy<Counter> = Lazy::new(Counter::default);

/// Counts mesh jobs that completed successfully.
pub static JOBS_COMPLETED: Lazy<Counter> = Lazy::new(Counter::default);

/// Counts mesh jobs that failed.
pub static JOBS_FAILED: Lazy<Counter> = Lazy::new(Counter::default);

/// Tracks the number of jobs currently active (submitted but not finished).
pub static JOBS_ACTIVE_GAUGE: Lazy<Gauge<i64>> = Lazy::new(Gauge::default);

/// Counts receipts anchored to the DAG.
pub static RECEIPTS_ANCHORED: Lazy<Counter> = Lazy::new(Counter::default);
