use once_cell::sync::Lazy;
use prometheus_client::metrics::counter::Counter;

/// Counts calls to `record_execution` across reputation stores.
pub static EXECUTION_RECORDS: Lazy<Counter> = Lazy::new(Counter::default);

/// Counts calls to `record_proof_attempt` across reputation stores.
pub static PROOF_ATTEMPTS: Lazy<Counter> = Lazy::new(Counter::default);
