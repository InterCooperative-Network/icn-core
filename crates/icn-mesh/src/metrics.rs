use once_cell::sync::Lazy;
use prometheus_client::metrics::{
    counter::Counter,
    gauge::Gauge,
    histogram::{exponential_buckets, Histogram},
};

/// Counts calls to `select_executor`.
pub static SELECT_EXECUTOR_CALLS: Lazy<Counter> = Lazy::new(Counter::default);

/// Counts calls to `schedule_mesh_job`.
pub static SCHEDULE_MESH_JOB_CALLS: Lazy<Counter> = Lazy::new(Counter::default);

/// Tracks the number of jobs currently waiting in the runtime queue.
pub static PENDING_JOBS_GAUGE: Lazy<Gauge<i64>> = Lazy::new(Gauge::default);

/// Records the time from job assignment to receipt processing in seconds.
pub static JOB_PROCESS_TIME: Lazy<Histogram> =
    Lazy::new(|| Histogram::new(exponential_buckets(1.0, 2.0, 10)));

/// Counts the total number of jobs submitted to the mesh.
pub static JOBS_SUBMITTED_TOTAL: Lazy<Counter> = Lazy::new(Counter::default);

/// Counts the total number of bids received across all jobs.
pub static BIDS_RECEIVED_TOTAL: Lazy<Counter> = Lazy::new(Counter::default);

/// Counts the total number of jobs that have been completed successfully.
pub static JOBS_COMPLETED_TOTAL: Lazy<Counter> = Lazy::new(Counter::default);

/// Counts the total number of jobs that have failed.
pub static JOBS_FAILED_TOTAL: Lazy<Counter> = Lazy::new(Counter::default);

/// Counts the total number of jobs that have been assigned to executors.
pub static JOBS_ASSIGNED_TOTAL: Lazy<Counter> = Lazy::new(Counter::default);

/// Tracks the number of jobs currently in the bidding phase.
pub static JOBS_BIDDING_GAUGE: Lazy<Gauge<i64>> = Lazy::new(Gauge::default);

/// Tracks the number of jobs currently being executed.
pub static JOBS_EXECUTING_GAUGE: Lazy<Gauge<i64>> = Lazy::new(Gauge::default);

/// Records the time from job submission to assignment in seconds.
pub static JOB_ASSIGNMENT_TIME: Lazy<Histogram> =
    Lazy::new(|| Histogram::new(exponential_buckets(1.0, 2.0, 10)));

/// Records the time from job submission to completion in seconds.
pub static JOB_COMPLETION_TIME: Lazy<Histogram> =
    Lazy::new(|| Histogram::new(exponential_buckets(1.0, 2.0, 15)));
