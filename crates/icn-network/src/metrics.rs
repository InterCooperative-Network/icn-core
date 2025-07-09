use once_cell::sync::Lazy;
use prometheus_client::metrics::{counter::Counter, gauge::Gauge};
use std::sync::atomic::AtomicU64;

/// Last observed ping round-trip time in milliseconds.
pub static PING_LAST_RTT_MS: Lazy<Gauge<f64, AtomicU64>> = Lazy::new(Gauge::default);

/// Minimum observed ping round-trip time in milliseconds.
pub static PING_MIN_RTT_MS: Lazy<Gauge<f64, AtomicU64>> = Lazy::new(Gauge::default);

/// Maximum observed ping round-trip time in milliseconds.
pub static PING_MAX_RTT_MS: Lazy<Gauge<f64, AtomicU64>> = Lazy::new(Gauge::default);

/// Average ping round-trip time in milliseconds.
pub static PING_AVG_RTT_MS: Lazy<Gauge<f64, AtomicU64>> = Lazy::new(Gauge::default);

/// Current number of connected peers.
pub static PEER_COUNT_GAUGE: Lazy<Gauge<i64>> = Lazy::new(Gauge::default);

/// Current number of peers in the Kademlia routing table.
pub static KADEMLIA_PEERS_GAUGE: Lazy<Gauge<i64>> = Lazy::new(Gauge::default);

/// Total bytes sent over the network.
pub static BYTES_SENT_TOTAL: Lazy<Counter> = Lazy::new(Counter::default);

/// Total bytes received over the network.
pub static BYTES_RECEIVED_TOTAL: Lazy<Counter> = Lazy::new(Counter::default);

/// Total messages sent over the network.
pub static MESSAGES_SENT_TOTAL: Lazy<Counter> = Lazy::new(Counter::default);

/// Total messages received over the network.
pub static MESSAGES_RECEIVED_TOTAL: Lazy<Counter> = Lazy::new(Counter::default);
