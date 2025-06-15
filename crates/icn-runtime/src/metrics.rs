use once_cell::sync::Lazy;
use prometheus_client::metrics::counter::Counter;

/// Counts calls to `host_submit_mesh_job`.
pub static HOST_SUBMIT_MESH_JOB_CALLS: Lazy<Counter> = Lazy::new(Counter::default);

/// Counts calls to `host_get_pending_mesh_jobs`.
pub static HOST_GET_PENDING_MESH_JOBS_CALLS: Lazy<Counter> = Lazy::new(Counter::default);

/// Counts calls to `host_account_get_mana`.
pub static HOST_ACCOUNT_GET_MANA_CALLS: Lazy<Counter> = Lazy::new(Counter::default);

/// Counts calls to `host_account_spend_mana`.
pub static HOST_ACCOUNT_SPEND_MANA_CALLS: Lazy<Counter> = Lazy::new(Counter::default);
