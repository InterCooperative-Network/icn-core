//! Runtime context module for the ICN runtime.
//!
//! This module contains all the types and implementations related to the runtime context,
//! including error handling, mana management, signers, network services, and more.

pub mod advanced_ccl_wasm;
pub mod compile_checks;
pub mod comprehensive_coordinator;
pub mod cross_component_coordinator;
pub mod dag_store_factory;
pub mod enhanced_dag_sync;
pub mod errors;
pub mod federation_integration;
pub mod host_environment;
pub mod mana;
pub mod mesh_network;
pub mod realtime_ccl_integration;
pub mod resource_ledger;
pub mod runtime_context;
pub mod runtime_factory;
pub mod service_config;
pub mod signers;
pub mod smart_p2p_routing;
pub mod stubs;

#[cfg(test)]
pub mod dag_storage_tests;

// Re-export important types for convenience
pub use advanced_ccl_wasm::{
    AdvancedCclWasmBackend, CclExecutionConfig, CclExecutionResult, CclPerformanceMetrics,
    OptimizationLevel,
};
pub use compile_checks::ProductionReady;
pub use comprehensive_coordinator::{
    ComprehensiveCoordinationConfig, ComprehensiveCoordinationStats, ComprehensiveCoordinator,
    CoordinationEvent, OptimizationExecutionResult, SystemHealthStatus,
};
pub use cross_component_coordinator::{
    CrossComponentCoordinator, DagOperation, DagOperationResult, HealthStatus,
    IntegrationMetricsSummary, PerformanceMetrics, Priority, SystemStatus,
};
pub use dag_store_factory::{
    DagStoreBackend, DagStoreConfig, DagStoreFactory, DagStoreOptions,
};
pub use enhanced_dag_sync::{
    EnhancedDagSync, NetworkConditions, PropagationPriority, PropagationStrategy, SyncHealth,
    SyncResult,
};
pub use errors::HostAbiError;
pub use federation_integration::{
    FederationAwareJobSelection, FederationBidInfo, FederationIntegration,
};
pub use host_environment::{ConcreteHostEnvironment, HostEnvironment};
pub use mana::{LedgerBackend, ManaRepository, SimpleManaLedger};
pub use mesh_network::{
    BidId, DefaultMeshNetworkService, JobAssignmentNotice, LocalMeshSubmitReceiptMessage,
    MeshJobStateChange, MeshNetworkService, PeerStatistics, PingResult, SelectionPolicy,
    PROPOSAL_COST_MANA, VOTE_COST_MANA,
};
pub use realtime_ccl_integration::{
    ActiveProposal, CclIntegrationCoordinator, GovernanceEvent, GovernanceEventType,
    ProposalStatus, VoteTracker,
};
pub use resource_ledger::{
    record_resource_event, ResourceAction, ResourceLedger, ResourceLedgerEntry,
};
pub use runtime_context::{
    CastVotePayload, CloseProposalResult, CreateProposalPayload, EnvironmentType,
    MeshNetworkServiceType, ParameterUpdate, RuntimeContext, RuntimeContextBuilder,
    RuntimeContextParams, MANA_MAX_CAPACITY_KEY,
};
pub use runtime_factory::{
    RuntimeContextFactory, RuntimeCreationConfig, RuntimeCreationConfigBuilder, RuntimeEnvironment,
};
pub use service_config::{ServiceConfig, ServiceConfigBuilder, ServiceEnvironment};
pub use signers::{Ed25519Signer, HsmKeyStore, Signer, StubSigner};
pub use smart_p2p_routing::{
    MessagePriority, PeerRouteInfo, RoutePath, RoutingMetrics, RoutingStrategy, SmartP2pRouter,
};
pub use stubs::{RuntimeStubDagStore, StubDagStore, StubMeshNetworkService};

// Conditional compilation helpers for DAG storage service
pub type DagStorageService = dyn icn_dag::AsyncStorageService<icn_common::DagBlock> + Send;
pub type DagStoreMutexType<T> = tokio::sync::Mutex<T>;
