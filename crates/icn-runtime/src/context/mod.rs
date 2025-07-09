//! Runtime context module for the ICN runtime.
//!
//! This module contains all the types and implementations related to the runtime context,
//! including error handling, mana management, signers, network services, and more.

pub mod errors;
pub mod host_environment;
pub mod mana;
pub mod mesh_network;
pub mod runtime_context;
pub mod signers;
pub mod stubs;

// Re-export important types for convenience
pub use errors::HostAbiError;
pub use host_environment::{ConcreteHostEnvironment, HostEnvironment};
pub use mana::{LedgerBackend, ManaRepository, SimpleManaLedger};
pub use mesh_network::{
    DefaultMeshNetworkService, JobAssignmentNotice, LocalMeshSubmitReceiptMessage,
    MeshJobStateChange, MeshNetworkService, SelectionPolicy, BidId,
    PROPOSAL_COST_MANA, VOTE_COST_MANA,
};
pub use runtime_context::{
    RuntimeContext, MeshNetworkServiceType, CreateProposalPayload, CastVotePayload, CloseProposalResult,
};
pub use signers::{Ed25519Signer, HsmKeyStore, Signer, StubSigner};
pub use stubs::{RuntimeStubDagStore, StubDagStore, StubMeshNetworkService};

// Conditional compilation helpers for DAG storage service
pub type DagStorageService = dyn icn_dag::StorageService<icn_common::DagBlock> + Send + Sync;
pub type DagStoreMutexType<T> = tokio::sync::Mutex<T>; 