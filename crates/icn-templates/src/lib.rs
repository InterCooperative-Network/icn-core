//! Pre-built CCL templates for cooperative governance.
//!
//! Contracts are provided as plain source text so projects can compile or modify
//! them as needed. Each template reflects a common governance pattern.

/// Simple majority voting procedure in CCL.
pub const SIMPLE_VOTING: &str = include_str!("../templates/simple_voting.ccl");

/// Basic treasury rule example in CCL.
pub const TREASURY_RULES: &str = include_str!("../templates/treasury_rules.ccl");

/// Rotating steward governance template.
pub const ROTATING_STEWARDS: &str = include_str!("../templates/rotating_stewards.ccl");

/// Rotating council governance template.
pub const ROTATING_COUNCIL: &str = include_str!("../templates/rotating_council.ccl");

/// Rotating assembly governance template.
pub const ROTATING_ASSEMBLY: &str = include_str!("../templates/rotating_assembly.ccl");
