use once_cell::sync::Lazy;
use prometheus_client::metrics::gauge::Gauge;
use std::sync::atomic::AtomicU64;

/// Last observed ping round-trip time in milliseconds.
pub static PING_LAST_RTT_MS: Lazy<Gauge<f64, AtomicU64>> = Lazy::new(Gauge::default);

/// Minimum observed ping round-trip time in milliseconds.
pub static PING_MIN_RTT_MS: Lazy<Gauge<f64, AtomicU64>> = Lazy::new(Gauge::default);

/// Maximum observed ping round-trip time in milliseconds.
pub static PING_MAX_RTT_MS: Lazy<Gauge<f64, AtomicU64>> = Lazy::new(Gauge::default);

/// Average ping round-trip time in milliseconds.
pub static PING_AVG_RTT_MS: Lazy<Gauge<f64, AtomicU64>> = Lazy::new(Gauge::default);
