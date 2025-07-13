//! Runtime-wide constants used across the node and runtime crates.

/// Mana cost charged for verifying a zero-knowledge proof.
///
/// This mirrors the previous value defined in `icn-node` and is
/// re-exported for consumers of `icn-runtime`.
pub const ZK_VERIFY_COST_MANA: u64 = 2;

