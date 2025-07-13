use std::sync::atomic::AtomicU64;

/// Global atomic storing the node start time in seconds since Unix epoch.
/// Used by components to report uptime metrics.
pub static NODE_START_TIME: AtomicU64 = AtomicU64::new(0);
