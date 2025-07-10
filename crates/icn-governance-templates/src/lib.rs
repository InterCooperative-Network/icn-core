//! Reusable governance templates for Cooperative Contract Language (CCL).
//!
//! Each helper returns the source code of a template contract. Consumers can copy
//! the returned string or load the template files from this crate's `templates`
//! directory and modify them as needed.

/// Returns the CCL source for a basic voting workflow.
pub fn voting_logic_template() -> &'static str {
    include_str!("../templates/voting_logic.ccl")
}

/// Returns the CCL source for a simple treasury management workflow.
pub fn treasury_rules_template() -> &'static str {
    include_str!("../templates/treasury_rules.ccl")
}
