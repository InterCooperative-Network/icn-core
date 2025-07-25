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

pub use manifest::{PackageManifest, Dependency, VersionReq};
pub use registry::{Registry, PackageInfo};
pub use resolver::DependencyResolver;