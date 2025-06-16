//! ICN Node Library
//! This library exposes functionality to create and run ICN nodes

#![allow(special_module_name)]
pub mod config;
pub mod node;
pub use node::{app_router, app_router_with_options};
