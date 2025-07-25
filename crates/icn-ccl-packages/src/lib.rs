//! ICN CCL Package Manager
//!
//! This crate provides package management for CCL modules:
//! - Install and manage CCL governance pattern libraries
//! - Dependency resolution for CCL contracts
//! - Registry for sharing reusable governance components
//! - Version management and compatibility checking

pub mod registry;
pub mod package;
pub mod installer;
pub mod resolver;

pub use package::{Package, PackageManager};
pub use registry::PackageRegistry;