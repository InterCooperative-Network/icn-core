//! # ICN Core Traits
//!
//! Shared traits and interfaces for ICN Core components to break circular dependencies.
//! This crate contains common abstractions that multiple ICN crates need to implement
//! or depend on, without creating circular dependency chains.

pub mod economics;
pub mod error;
pub mod governance;
pub mod mesh;
pub mod network;
pub mod reputation;

// Re-export main traits for convenience
pub use economics::{EconomicProvider, ManaProvider, ResourceProvider};
pub use error::CoreTraitsError;
pub use governance::{GovernanceProvider, ProposalProvider, VotingProvider};
pub use mesh::{ExecutorProvider, JobProvider, MeshProvider};
pub use network::{
    BasicNetworkService, NetworkEvent, NetworkService, NetworkServiceProvider, NetworkStats, PeerId,
};
pub use reputation::{ReputationEvent, ReputationProvider, ReputationStore};
