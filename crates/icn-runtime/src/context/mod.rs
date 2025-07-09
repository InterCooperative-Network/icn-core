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
pub use runtime_context::RuntimeContext;
pub use signers::{Ed25519Signer, HsmKeyStore, Signer, StubSigner};
pub use stubs::{RuntimeStubDagStore, StubDagStore, StubMeshNetworkService};

// Conditional compilation helpers
#[cfg(feature = "async")]
use icn_dag::AsyncStorageService as DagStorageService;
#[cfg(not(feature = "async"))]
use icn_dag::StorageService as DagStorageService;

#[cfg(feature = "async")]
use tokio::sync::Mutex as DagStoreMutex;
#[cfg(not(feature = "async"))]
use std::sync::Mutex as DagStoreMutex;

// Re-export conditional types
pub use DagStorageService;
pub type DagStoreMutexType<T> = DagStoreMutex<T>; 