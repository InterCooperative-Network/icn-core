use once_cell::sync::Lazy;
use prometheus_client::metrics::counter::Counter;

/// Counts credentials issued by `CredentialIssuer::issue`.
pub static CREDENTIALS_ISSUED: Lazy<Counter> = Lazy::new(Counter::default);

/// Counts successful zero-knowledge proof verifications.
pub static PROOFS_VERIFIED: Lazy<Counter> = Lazy::new(Counter::default);

/// Counts failed zero-knowledge proof verifications.
pub static PROOF_VERIFICATION_FAILURES: Lazy<Counter> = Lazy::new(Counter::default);
