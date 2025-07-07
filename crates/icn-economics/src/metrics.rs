use once_cell::sync::Lazy;
use prometheus_client::metrics::counter::Counter;

/// Counts calls to `get_balance`.
pub static GET_BALANCE_CALLS: Lazy<Counter> = Lazy::new(Counter::default);

/// Counts calls to `credit_mana`.
pub static CREDIT_MANA_CALLS: Lazy<Counter> = Lazy::new(Counter::default);

/// Counts calls to `spend_mana`.
pub static SPEND_MANA_CALLS: Lazy<Counter> = Lazy::new(Counter::default);
