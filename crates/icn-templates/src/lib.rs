//! Pre-built CCL templates for cooperative governance.
//!
//! Contracts are provided as plain source text so projects can compile or modify
//! them as needed. Each template reflects a common governance pattern.

/// Simple majority voting procedure in CCL.
pub const SIMPLE_VOTING: &str = include_str!("../templates/simple_voting.ccl");

/// Basic treasury rule example in CCL.
pub const TREASURY_RULES: &str = include_str!("../templates/treasury_rules.ccl");

/// Rotating stewards governance template.
pub const ROTATING_STEWARDS: &str = include_str!("../templates/rotating_stewards.ccl");

/// Council voting governance template.
pub const COUNCIL_VOTE: &str = include_str!("../templates/council_vote.ccl");

/// General assembly governance template.
pub const GENERAL_ASSEMBLY: &str = include_str!("../templates/general_assembly.ccl");
