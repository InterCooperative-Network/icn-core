// icn-ccl/src/package/mod.rs
//! CCL package manager for managing dependencies and governance patterns
//!
//! This module provides:
//! - Package definition and metadata
//! - Dependency resolution and management
//! - Registry integration for sharing CCL packages
//! - Installation and version management

pub mod manifest;
pub mod registry;
pub mod resolver;

pub use manifest::{Dependency, PackageManifest, VersionReq};
pub use registry::{PackageInfo, Registry};
pub use resolver::DependencyResolver;
