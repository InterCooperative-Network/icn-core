use once_cell::sync::Lazy;
use prometheus_client::metrics::counter::Counter;

/// Counts calls to `select_executor`.
pub static SELECT_EXECUTOR_CALLS: Lazy<Counter> = Lazy::new(Counter::default);

/// Counts calls to `schedule_mesh_job`.
pub static SCHEDULE_MESH_JOB_CALLS: Lazy<Counter> = Lazy::new(Counter::default);
