use once_cell::sync::Lazy;
use prometheus_client::metrics::counter::Counter;

/// Counts DAG block inserts.
pub static DAG_PUT_CALLS: Lazy<Counter> = Lazy::new(Counter::default);

/// Counts DAG block fetches.
pub static DAG_GET_CALLS: Lazy<Counter> = Lazy::new(Counter::default);
