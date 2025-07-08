use once_cell::sync::Lazy;
use prometheus_client::metrics::{counter::Counter, gauge::Gauge, histogram::{Histogram, exponential_buckets}};

/// Counts calls to `select_executor`.
pub static SELECT_EXECUTOR_CALLS: Lazy<Counter> = Lazy::new(Counter::default);

/// Counts calls to `schedule_mesh_job`.
pub static SCHEDULE_MESH_JOB_CALLS: Lazy<Counter> = Lazy::new(Counter::default);

/// Tracks the number of jobs currently waiting in the runtime queue.
pub static PENDING_JOBS_GAUGE: Lazy<Gauge<i64>> = Lazy::new(Gauge::default);

/// Records the time from job assignment to receipt processing in seconds.
pub static JOB_PROCESS_TIME: Lazy<Histogram> = Lazy::new(|| Histogram::new(exponential_buckets(0.1, 2.0, 10)));
