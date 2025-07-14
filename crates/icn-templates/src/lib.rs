//! Pre-built CCL templates for cooperative governance.
//!
//! Contracts are provided as plain source text so projects can compile or modify
//! them as needed. Each template reflects a common governance pattern.

/// Simple majority voting procedure in CCL.
pub const SIMPLE_VOTING: &str = include_str!("../templates/simple_voting.ccl");

/// Basic treasury rule example in CCL.
pub const TREASURY_RULES: &str = include_str!("../templates/treasury_rules.ccl");

/// Federation membership proof contract.
pub const FEDERATION_MEMBERSHIP_PROOF: &str =
    include_str!("../templates/federation_membership_proof.ccl");
