//! Integration tests for ICN Core production services
//!
//! This module contains comprehensive integration tests that validate
//! the complete production service implementations.

pub mod production_services;

// Re-export test functions for easy access
pub use production_services::*;