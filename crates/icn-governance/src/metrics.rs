use once_cell::sync::Lazy;
use prometheus_client::metrics::counter::Counter;

/// Counts proposal submissions.
pub static SUBMIT_PROPOSAL_CALLS: Lazy<Counter> = Lazy::new(Counter::default);

/// Counts votes cast.
pub static CAST_VOTE_CALLS: Lazy<Counter> = Lazy::new(Counter::default);

/// Counts executed proposals.
pub static EXECUTE_PROPOSAL_CALLS: Lazy<Counter> = Lazy::new(Counter::default);
