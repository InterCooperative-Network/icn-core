//! Runtime context module for the ICN runtime.
//!
//! This module contains all the types and implementations related to the runtime context,
//! including error handling, mana management, signers, network services, and more.

pub mod compile_checks;
pub mod errors;
pub mod host_environment;
pub mod mana;
pub mod mesh_network;
pub mod resource_ledger;
pub mod runtime_context;
pub mod service_config;
pub mod signers;
pub mod stubs;

// Re-export important types for convenience
pub use compile_checks::ProductionReady;
pub use errors::HostAbiError;
pub use host_environment::{ConcreteHostEnvironment, HostEnvironment};
pub use mana::{LedgerBackend, ManaRepository, SimpleManaLedger};
pub use mesh_network::{
    BidId, DefaultMeshNetworkService, JobAssignmentNotice, LocalMeshSubmitReceiptMessage,
    MeshJobStateChange, MeshNetworkService, SelectionPolicy, PROPOSAL_COST_MANA, VOTE_COST_MANA,
};
pub use resource_ledger::{
    record_resource_event, ResourceAction, ResourceLedger, ResourceLedgerEntry,
};
pub use runtime_context::{
    RuntimeContext, RuntimeContextParams, RuntimeContextBuilder, EnvironmentType, MeshNetworkServiceType, CreateProposalPayload, CastVotePayload, CloseProposalResult, ParameterUpdate,
    MANA_MAX_CAPACITY_KEY,
};
pub use service_config::{ServiceConfig, ServiceConfigBuilder, ServiceEnvironment};
pub use signers::{Ed25519Signer, HsmKeyStore, Signer, StubSigner};
pub use stubs::{RuntimeStubDagStore, StubDagStore, StubMeshNetworkService};

// Conditional compilation helpers for DAG storage service
pub type DagStorageService = dyn icn_dag::AsyncStorageService<icn_common::DagBlock> + Send;
pub type DagStoreMutexType<T> = tokio::sync::Mutex<T>;
