//! # ICN Core Traits
//! 
//! Shared traits and interfaces for ICN Core components to break circular dependencies.
//! This crate contains common abstractions that multiple ICN crates need to implement
//! or depend on, without creating circular dependency chains.

pub mod network;
pub mod reputation;
pub mod governance;
pub mod economics;
pub mod mesh;
pub mod error;

// Re-export main traits for convenience
pub use network::{NetworkService, NetworkServiceProvider, PeerId, NetworkStats, NetworkEvent, BasicNetworkService};
pub use reputation::{ReputationStore, ReputationProvider, ReputationEvent};
pub use governance::{GovernanceProvider, ProposalProvider, VotingProvider};
pub use economics::{EconomicProvider, ManaProvider, ResourceProvider};
pub use mesh::{MeshProvider, JobProvider, ExecutorProvider};
pub use error::CoreTraitsError;