#![doc = include_str!("../README.md")]
#![allow(
    clippy::uninlined_format_args,
    clippy::clone_on_copy,
    clippy::field_reassign_with_default,
    unused_imports,
    clippy::useless_conversion,
    clippy::needless_borrows_for_generic_args,
    dead_code,
    irrefutable_let_patterns
)]

//! # ICN Node Crate
//! This crate provides the main binary for running a long-lived InterCooperative Network (ICN) daemon.
//! It integrates various core components to operate a functional ICN node, handling initialization,
//! lifecycle, configuration, service hosting, and persistence.

use icn_api::governance_trait::{
    CastVoteRequest as ApiCastVoteRequest, SubmitProposalRequest as ApiSubmitProposalRequest,
};
use icn_common::{parse_cid_from_string, Cid, Did, NodeInfo, NodeStatus, ICN_CORE_VERSION};
use icn_identity::{
    did_key_from_verifying_key, generate_ed25519_keypair,
    ExecutionReceipt as IdentityExecutionReceipt, SignatureBytes,
};
use icn_mesh::ActualMeshJob;
use icn_runtime::context::{
    RuntimeContext, StubDagStore as RuntimeStubDagStore, StubMeshNetworkService,
    StubSigner as RuntimeStubSigner,
};
use icn_runtime::{host_anchor_receipt, host_submit_mesh_job, ReputationUpdater};

use axum::{
    extract::{Path as AxumPath, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use clap::Parser;
#[cfg(feature = "enable-libp2p")]
use log::warn;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

#[cfg(feature = "enable-libp2p")]
use libp2p::{Multiaddr, PeerId as Libp2pPeerId};

// --- CLI Arguments ---

#[derive(Parser, Debug)]
#[clap(author, version = ICN_CORE_VERSION, about = "ICN Node HTTP Server", long_about = None)]
struct Cli {
    #[clap(
        long,
        value_enum,
        default_value = "memory",
        help = "Storage backend type"
    )]
    storage_backend: StorageBackendType,

    #[clap(
        long,
        default_value = "./icn_data/node_store",
        help = "Path for file-based storage (if 'file' backend is chosen)"
    )]
    storage_path: PathBuf,

    #[clap(
        long,
        default_value = "127.0.0.1:7845",
        help = "Listen address for the HTTP server"
    )]
    http_listen_addr: String,

    #[clap(
        long,
        help = "Optional fixed DID for the node (e.g., did:key:zExample...)"
    )]
    node_did: Option<String>,

    #[clap(
        long,
        help = "Optional fixed Ed25519 private key (bs58 encoded string) for the node DID. If not provided and node_did is, it implies did:key or resolvable DID. If neither, a new key is generated."
    )]
    node_private_key_bs58: Option<String>,

    #[clap(
        long,
        help = "Human-readable name for this node (for logging and identification)"
    )]
    node_name: Option<String>,

    #[clap(
        long,
        default_value = "/ip4/0.0.0.0/tcp/0",
        help = "Libp2p listen address for P2P networking"
    )]
    p2p_listen_addr: String,

    #[clap(
        long,
        help = "Bootstrap peer multiaddrs for P2P discovery (format: /ip4/1.2.3.4/tcp/port/p2p/PeerID)",
        value_delimiter = ','
    )]
    bootstrap_peers: Option<Vec<String>>,

    #[clap(
        long,
        action,
        help = "Enable real libp2p networking (requires with-libp2p feature)"
    )]
    enable_p2p: bool,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum StorageBackendType {
    Memory,
    File,
}

// --- Supporting Types ---

#[derive(Deserialize)]
struct DagBlock {
    data: Vec<u8>,
}

// --- Application State ---
#[derive(Clone)]
struct AppState {
    runtime_context: Arc<RuntimeContext>,
    node_name: String,
    node_version: String,
}

// --- Public App Constructor (for tests or embedding) ---
pub async fn app_router() -> Router {
    // Renamed to app_router and made async for RT init
    // Generate a new identity for this test/embedded instance
    let (sk, pk) = generate_ed25519_keypair();
    let node_did_string = did_key_from_verifying_key(&pk);
    let node_did = Did::from_str(&node_did_string).expect("Failed to create test node DID");
    info!("Test/Embedded Node DID: {}", node_did);

    let signer = Arc::new(RuntimeStubSigner::new_with_keys(sk, pk)); // Use the generated keys
    let dag_store_for_rt = Arc::new(RuntimeStubDagStore::new());
    let mesh_network_service = Arc::new(StubMeshNetworkService::new());
    // Note: GovernanceModule is initialized inside RuntimeContext::new_with_stubs or similar

    let rt_ctx = RuntimeContext::new(
        node_did.clone(),
        mesh_network_service,
        signer,
        dag_store_for_rt,
        // GovernanceModule will be default in RuntimeContext::new
    );

    // Initialize the test node with some mana for testing
    rt_ctx
        .credit_mana(&node_did, 1000)
        .await
        .expect("Failed to initialize test node with mana");
    info!("✅ Test node initialized with 1000 mana");

    rt_ctx.clone().spawn_mesh_job_manager().await; // Start the job manager

    let app_state = AppState {
        runtime_context: rt_ctx.clone(),
        node_name: "ICN Test/Embedded Node".to_string(),
        node_version: ICN_CORE_VERSION.to_string(),
    };

    Router::new()
        .route("/info", get(info_handler))
        .route("/status", get(status_handler))
        .route("/dag/put", post(dag_put_handler)) // These will use RT context's DAG store
        .route("/dag/get", post(dag_get_handler)) // These will use RT context's DAG store
        .route("/governance/submit", post(gov_submit_handler)) // Uses RT context's Gov mod
        .route("/governance/vote", post(gov_vote_handler)) // Uses RT context's Gov mod
        .route("/governance/proposals", get(gov_list_proposals_handler)) // Uses RT context's Gov mod
        .route(
            "/governance/proposal/:proposal_id",
            get(gov_get_proposal_handler),
        ) // Uses RT context's Gov mod
        .route("/mesh/submit", post(mesh_submit_job_handler)) // Job submission
        .route("/mesh/jobs", get(mesh_list_jobs_handler)) // List all jobs
        .route("/mesh/jobs/:job_id", get(mesh_get_job_status_handler)) // Get specific job status
        .route("/mesh/receipts", post(mesh_submit_receipt_handler)) // Submit execution receipt
        .with_state(app_state)
}

// --- Main Application Logic ---
#[tokio::main]
async fn main() {
    env_logger::init(); // Initialize logger
    let cli = Cli::parse();

    // --- Initialize Node Identity ---
    let (node_sk, node_pk) = generate_ed25519_keypair(); // Generate fresh for now
    let node_did_string = did_key_from_verifying_key(&node_pk);
    let node_did = Did::from_str(&node_did_string).expect("Failed to create node DID");

    let node_name = cli
        .node_name
        .clone()
        .unwrap_or_else(|| "ICN Node".to_string());
    info!("Starting {} with DID: {}", node_name, node_did);

    // --- Create RuntimeContext with Networking ---
    let rt_ctx = if cli.enable_p2p {
        #[cfg(feature = "enable-libp2p")]
        {
            info!(
                "Enabling libp2p networking with P2P listen address: {}",
                cli.p2p_listen_addr
            );

            // Parse bootstrap peers if provided
            let bootstrap_peers = if let Some(peer_strings) = &cli.bootstrap_peers {
                let mut parsed_peers = Vec::new();
                for peer_str in peer_strings {
                    match peer_str.parse::<Multiaddr>() {
                        Ok(multiaddr) => {
                            // Extract PeerID from multiaddr if present
                            if let Some(libp2p::core::multiaddr::Protocol::P2p(peer_id)) =
                                multiaddr.iter().last()
                            {
                                if let Ok(peer_id) = peer_id.try_into() {
                                    parsed_peers.push((peer_id, multiaddr));
                                    info!("Added bootstrap peer: {}", peer_str);
                                } else {
                                    warn!("Failed to parse PeerID from multiaddr: {}", peer_str);
                                }
                            } else {
                                warn!("Multiaddr missing PeerID component: {}", peer_str);
                            }
                        }
                        Err(e) => {
                            error!(
                                "Failed to parse bootstrap peer multiaddr '{}': {}",
                                peer_str, e
                            );
                            std::process::exit(1);
                        }
                    }
                }
                if parsed_peers.is_empty() {
                    None
                } else {
                    Some(parsed_peers)
                }
            } else {
                None
            };

            let listen_addr = cli
                .p2p_listen_addr
                .parse::<Multiaddr>()
                .expect("Invalid p2p listen multiaddr");
            let listen_addrs = vec![listen_addr];

            match RuntimeContext::new_with_real_libp2p(
                &node_did_string,
                listen_addrs,
                bootstrap_peers,
            )
            .await
            {
                Ok(ctx) => {
                    info!("✅ RuntimeContext created with real libp2p networking");

                    // Get libp2p service info for logging
                    if let Ok(libp2p_service) = ctx.get_libp2p_service() {
                        info!("📟 Local Peer ID: {}", libp2p_service.local_peer_id());
                    }

                    ctx
                }
                Err(e) => {
                    error!("Failed to create RuntimeContext with libp2p: {}", e);
                    std::process::exit(1);
                }
            }
        }
        #[cfg(not(feature = "enable-libp2p"))]
        {
            error!("--enable-p2p flag requires the 'with-libp2p' feature to be compiled");
            error!("Please recompile with: cargo build --features with-libp2p");
            std::process::exit(1);
        }
    } else {
        info!("Using stub networking (P2P disabled)");
        let signer = Arc::new(RuntimeStubSigner::new_with_keys(node_sk, node_pk));
        let dag_store_for_rt = Arc::new(RuntimeStubDagStore::new());
        let mesh_network_service = Arc::new(StubMeshNetworkService::new());

        RuntimeContext::new(
            node_did.clone(),
            mesh_network_service,
            signer,
            dag_store_for_rt,
        )
    };

    // Start the job manager
    rt_ctx.clone().spawn_mesh_job_manager().await;
    info!("ICN RuntimeContext initialized and JobManager spawned.");

    // --- Create AppState for Axum ---
    let app_state = AppState {
        runtime_context: rt_ctx.clone(),
        node_name: node_name.clone(),
        node_version: ICN_CORE_VERSION.to_string(),
    };

    // --- Define HTTP Routes ---
    let router = Router::new()
        .route("/info", get(info_handler))
        .route("/status", get(status_handler))
        .route("/dag/put", post(dag_put_handler))
        .route("/dag/get", post(dag_get_handler))
        .route("/governance/submit", post(gov_submit_handler))
        .route("/governance/vote", post(gov_vote_handler))
        .route("/governance/proposals", get(gov_list_proposals_handler))
        .route(
            "/governance/proposal/:proposal_id",
            get(gov_get_proposal_handler),
        )
        .route("/mesh/submit", post(mesh_submit_job_handler))
        .route("/mesh/jobs", get(mesh_list_jobs_handler))
        .route("/mesh/jobs/:job_id", get(mesh_get_job_status_handler))
        .route("/mesh/receipts", post(mesh_submit_receipt_handler))
        .with_state(app_state.clone());

    let addr: SocketAddr = cli
        .http_listen_addr
        .parse()
        .expect("Invalid HTTP listen address");
    info!("🌐 {} HTTP server listening on {}", node_name, addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, router.into_make_service())
        .await
        .unwrap();

    if cli.enable_p2p {
        #[cfg(feature = "enable-libp2p")]
        {
            if let Err(e) = rt_ctx.shutdown_network().await {
                error!("Network shutdown failed: {}", e);
            }
        }
    }
}

// --- Utility Functions for HTTP Responses ---

#[derive(Serialize)]
struct JsonErrorResponse {
    error: String,
}

fn map_rust_error_to_json_response<E: std::fmt::Display>(
    err: E,
    status_code: StatusCode,
) -> (StatusCode, Json<JsonErrorResponse>) {
    (
        status_code,
        Json(JsonErrorResponse {
            error: err.to_string(),
        }),
    )
}

// --- HTTP Handlers ---

// GET /info – Node version, name, status message.
async fn info_handler(State(state): State<AppState>) -> impl IntoResponse {
    let info = NodeInfo {
        name: state.node_name.clone(),
        version: state.node_version.clone(),
        status_message: "ICN Node is operational".to_string(), // TODO: Get status from RuntimeContext?
    };
    (StatusCode::OK, Json(info))
}

// GET /status – Node status.
async fn status_handler(State(state): State<AppState>) -> impl IntoResponse {
    // TODO: Fetch more dynamic status from RuntimeContext if available (e.g., peer count from NetworkService)
    let peer_count = 0; // Placeholder
                        // let current_block_height = state.runtime_context.dag_store.get_latest_block_height().await.unwrap_or(0); // Example
    let current_block_height = 0; // Placeholder
    let status = NodeStatus {
        is_online: true, // Basic check
        peer_count,
        current_block_height,
        version: state.node_version.clone(),
    };
    (StatusCode::OK, Json(status))
}

// POST /dag/put – Store a DAG block. (Body: block JSON)
async fn dag_put_handler(
    State(state): State<AppState>,
    Json(block): Json<DagBlock>,
) -> impl IntoResponse {
    // Use RuntimeContext's dag_store now
    match state.runtime_context.dag_store.put(&block.data).await {
        // Assuming block.data is Vec<u8>
        Ok(cid) => (StatusCode::CREATED, Json(cid)).into_response(),
        Err(e) => map_rust_error_to_json_response(
            format!("DAG put error: {}", e),
            StatusCode::INTERNAL_SERVER_ERROR,
        )
        .into_response(),
    }
}

// POST /dag/get – Retrieve a DAG block. (Body: CID JSON)
async fn dag_get_handler(
    State(state): State<AppState>,
    Json(cid_request): Json<CidRequest>, // Assuming a struct for JSON like {"cid": "..."}
) -> impl IntoResponse {
    // TODO: Parse cid_request.cid into a real Cid. For now, using a placeholder.
    let cid_to_get = Cid::new_v1_dummy(0, 0, cid_request.cid.as_bytes()); // Placeholder for Cid::from_str
    match state.runtime_context.dag_store.get(&cid_to_get).await {
        Ok(Some(data)) => (StatusCode::OK, Json(data)).into_response(), // Assuming data is Vec<u8>
        Ok(None) => map_rust_error_to_json_response("Block not found", StatusCode::NOT_FOUND)
            .into_response(),
        Err(e) => map_rust_error_to_json_response(
            format!("DAG get error: {}", e),
            StatusCode::INTERNAL_SERVER_ERROR,
        )
        .into_response(),
    }
}

#[derive(Deserialize)]
struct CidRequest {
    cid: String,
}

// POST /governance/submit – Submit a proposal. (Body: SubmitProposalRequest JSON)
async fn gov_submit_handler(
    State(state): State<AppState>,
    Json(request): Json<ApiSubmitProposalRequest>,
) -> impl IntoResponse {
    debug!("Received /governance/submit request: {:?}", request);

    let (ptype_str, payload_bytes) = match request.proposal.clone() {
        icn_api::governance_trait::ProposalInputType::SystemParameterChange { param, value } => (
            "SystemParameterChange".to_string(),
            serde_json::to_vec(&(param, value)).unwrap(),
        ),
        icn_api::governance_trait::ProposalInputType::MemberAdmission { did } => {
            ("MemberAdmission".to_string(), did.into_bytes())
        }
        icn_api::governance_trait::ProposalInputType::SoftwareUpgrade { version } => {
            ("SoftwareUpgrade".to_string(), version.into_bytes())
        }
        icn_api::governance_trait::ProposalInputType::GenericText { text } => {
            ("GenericText".to_string(), text.into_bytes())
        }
    };

    let payload = icn_runtime::context::CreateProposalPayload {
        proposal_type_str: ptype_str,
        type_specific_payload: payload_bytes,
        description: request.description,
        duration_secs: request.duration_secs,
    };

    let payload_json = match serde_json::to_string(&payload) {
        Ok(j) => j,
        Err(e) => {
            return map_rust_error_to_json_response(
                format!("Failed to serialize proposal payload: {}", e),
                StatusCode::BAD_REQUEST,
            )
            .into_response();
        }
    };

    match icn_runtime::host_create_governance_proposal(&state.runtime_context, &payload_json).await
    {
        Ok(id_str) => (
            StatusCode::CREATED,
            Json(icn_governance::ProposalId(id_str)),
        )
            .into_response(),
        Err(e) => map_rust_error_to_json_response(
            format!("Governance submit error: {}", e),
            StatusCode::BAD_REQUEST,
        )
        .into_response(),
    }
}

// POST /governance/vote – Cast a vote. (Body: CastVoteRequest JSON)
async fn gov_vote_handler(
    State(state): State<AppState>,
    Json(request): Json<ApiCastVoteRequest>,
) -> impl IntoResponse {
    debug!("Received /governance/vote request: {:?}", request);

    let payload = icn_runtime::context::CastVotePayload {
        proposal_id_str: request.proposal_id,
        vote_option_str: request.vote_option,
    };

    let payload_json = match serde_json::to_string(&payload) {
        Ok(j) => j,
        Err(e) => {
            return map_rust_error_to_json_response(
                format!("Failed to serialize vote payload: {}", e),
                StatusCode::BAD_REQUEST,
            )
            .into_response();
        }
    };

    match icn_runtime::host_cast_governance_vote(&state.runtime_context, &payload_json).await {
        Ok(_) => (StatusCode::OK, Json("Vote cast successfully".to_string())).into_response(),
        Err(e) => map_rust_error_to_json_response(
            format!("Governance vote error: {}", e),
            StatusCode::BAD_REQUEST,
        )
        .into_response(),
    }
}

// GET /governance/proposals
async fn gov_list_proposals_handler(State(state): State<AppState>) -> impl IntoResponse {
    debug!("Received /governance/proposals request");
    let gov_mod = state.runtime_context.governance_module.lock().await;
    match gov_mod.list_proposals() {
        Ok(props) => (StatusCode::OK, Json(props)).into_response(),
        Err(e) => map_rust_error_to_json_response(
            format!("Governance list error: {}", e),
            StatusCode::BAD_REQUEST,
        )
        .into_response(),
    }
}

// GET /governance/proposal/:proposal_id
async fn gov_get_proposal_handler(
    State(state): State<AppState>,
    AxumPath(proposal_id_str): AxumPath<String>,
) -> impl IntoResponse {
    debug!("Received /governance/proposal/{} request", proposal_id_str);
    let gov_mod = state.runtime_context.governance_module.lock().await;
    let pid = icn_governance::ProposalId(proposal_id_str);
    match gov_mod.get_proposal(&pid) {
        Ok(Some(prop)) => (StatusCode::OK, Json(prop)).into_response(),
        Ok(None) => map_rust_error_to_json_response("Proposal not found", StatusCode::NOT_FOUND)
            .into_response(),
        Err(e) => map_rust_error_to_json_response(
            format!("Governance get error: {}", e),
            StatusCode::BAD_REQUEST,
        )
        .into_response(),
    }
}

// --- Mesh Job Endpoints ---

/// Request body for submitting a mesh job.
/// Note: `id` and `creator_did` will be overridden by the runtime.
/// Signature also will be handled by the job submission logic after this initial DTO.
#[derive(Debug, Serialize, Deserialize)] // Added Serialize and Deserialize
pub struct SubmitJobRequest {
    pub manifest_cid: String, // String to be parsed into Cid
    // pub spec: JobSpec, // JobSpec is currently {}, so not very useful as JSON.
    // Let's assume spec comes as a JSON Value or stringified JSON for now.
    pub spec_json: serde_json::Value, // Expecting JobSpec as a JSON value
    pub cost_mana: u64,
}

async fn mesh_submit_job_handler(
    State(state): State<AppState>,
    Json(request): Json<SubmitJobRequest>,
) -> impl IntoResponse {
    info!("[Node] Received mesh_submit_job request: {:?}", request);

    let manifest_cid = Cid::new_v1_dummy(0, 0, request.manifest_cid.as_bytes()); // Placeholder for Cid::from_str

    let job_spec = match serde_json::from_value::<icn_mesh::JobSpec>(request.spec_json.clone()) {
        Ok(spec) => spec,
        Err(e) => {
            return map_rust_error_to_json_response(
                format!("Failed to parse job spec: {}", e),
                StatusCode::BAD_REQUEST,
            )
            .into_response()
        }
    };

    // This temporary job is just to satisfy the current host_submit_mesh_job ABI,
    // which expects a fully formed ActualMeshJob. The real job ID and potentially
    // other fields will be determined by the runtime.
    let temp_job_for_serialization = ActualMeshJob {
        // icn_mesh::ActualMeshJob
        id: Cid::new_v1_dummy(0, 0, b"placeholder_default_cid"), // Will be replaced by host_submit_mesh_job
        manifest_cid,
        spec: job_spec,
        creator_did: state.runtime_context.current_identity.clone(), // Use node's DID as creator
        cost_mana: request.cost_mana,
        signature: SignatureBytes(vec![]), // Will be ignored and re-signed if host_submit_mesh_job handles it, or added before.
    };

    let job_json = match serde_json::to_string(&temp_job_for_serialization) {
        Ok(json) => json,
        Err(e) => {
            return map_rust_error_to_json_response(
                format!("Failed to serialize job for submission: {}", e),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into_response()
        }
    };

    match host_submit_mesh_job(&state.runtime_context, &job_json).await {
        Ok(actual_job_id_cid) => {
            info!(
                "[NODE] Job submitted via runtime, Actual Job ID: {}",
                actual_job_id_cid
            );
            (
                StatusCode::ACCEPTED,
                Json(serde_json::json!({ "job_id": actual_job_id_cid.to_string() })),
            )
                .into_response()
        }
        Err(e) => {
            error!("[NODE] Error submitting job via runtime: {:?}", e);
            map_rust_error_to_json_response(
                format!("Mesh job submission failed: {}", e),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into_response()
        }
    }
}

// GET /mesh/jobs - List all jobs with their current status
async fn mesh_list_jobs_handler(State(state): State<AppState>) -> impl IntoResponse {
    info!("[Node] Received mesh_list_jobs request");

    let job_states = state.runtime_context.job_states.lock().await;
    let jobs: Vec<serde_json::Value> = job_states
        .iter()
        .map(|(job_id, job_state)| {
            serde_json::json!({
                "job_id": job_id.to_string(),
                "status": match job_state {
                    icn_mesh::JobState::Pending => serde_json::json!("pending"),
                    icn_mesh::JobState::Assigned { executor } => {
                        serde_json::json!({
                            "status": "assigned",
                            "executor": executor.to_string()
                        })
                    },
                    icn_mesh::JobState::Completed { receipt } => {
                        serde_json::json!({
                            "status": "completed",
                            "executor": receipt.executor_did.to_string(),
                            "result_cid": receipt.result_cid.to_string(),
                            "cpu_ms": receipt.cpu_ms
                        })
                    },
                    icn_mesh::JobState::Failed { reason } => {
                        serde_json::json!({
                            "status": "failed",
                            "reason": reason
                        })
                    }
                }
            })
        })
        .collect();

    (StatusCode::OK, Json(serde_json::json!({ "jobs": jobs }))).into_response()
}

// GET /mesh/jobs/:job_id - Get specific job status
async fn mesh_get_job_status_handler(
    State(state): State<AppState>,
    AxumPath(job_id_str): AxumPath<String>,
) -> impl IntoResponse {
    info!(
        "[Node] Received mesh_get_job_status request for job: {}",
        job_id_str
    );

    // Parse job_id from string
    let job_id = match parse_cid_from_string(&job_id_str) {
        Ok(cid) => {
            info!("[Node] Parsed job_id as CID: {:?}", cid);
            cid
        }
        Err(e) => {
            error!("[Node] Failed to parse job_id '{}': {}", job_id_str, e);
            return map_rust_error_to_json_response(
                format!("Invalid job ID format: {}", e),
                StatusCode::BAD_REQUEST,
            )
            .into_response();
        }
    };

    let job_states = state.runtime_context.job_states.lock().await;
    info!(
        "[Node] Looking for job_id {:?} in {} stored jobs",
        job_id,
        job_states.len()
    );

    // Debug: List all stored job IDs
    for stored_job_id in job_states.keys() {
        info!("[Node] Stored job ID: {:?}", stored_job_id);
    }

    match job_states.get(&job_id) {
        Some(job_state) => {
            let response = serde_json::json!({
                "job_id": job_id.to_string(),
                "status": match job_state {
                    icn_mesh::JobState::Pending => serde_json::json!("pending"),
                    icn_mesh::JobState::Assigned { executor } => {
                        serde_json::json!({
                            "status": "assigned",
                            "executor": executor.to_string()
                        })
                    },
                    icn_mesh::JobState::Completed { receipt } => {
                        serde_json::json!({
                            "status": "completed",
                            "executor": receipt.executor_did.to_string(),
                            "result_cid": receipt.result_cid.to_string(),
                            "cpu_ms": receipt.cpu_ms,
                            "signature_length": receipt.sig.0.len()
                        })
                    },
                    icn_mesh::JobState::Failed { reason } => {
                        serde_json::json!({
                            "status": "failed",
                            "reason": reason
                        })
                    }
                }
            });
            (StatusCode::OK, Json(response)).into_response()
        }
        None => {
            map_rust_error_to_json_response("Job not found", StatusCode::NOT_FOUND).into_response()
        }
    }
}

/// Request body for submitting an execution receipt
#[derive(Debug, Serialize, Deserialize)]
pub struct SubmitReceiptRequest {
    pub job_id: String,
    pub executor_did: String,
    pub result_cid: String,
    pub cpu_ms: u64,
    pub signature_hex: String, // Hex-encoded signature bytes
}

// POST /mesh/receipts - Submit execution receipt (typically called by executors)
async fn mesh_submit_receipt_handler(
    State(state): State<AppState>,
    Json(request): Json<SubmitReceiptRequest>,
) -> impl IntoResponse {
    info!("[Node] Received mesh_submit_receipt request: {:?}", request);

    // Parse request fields
    let job_id = match parse_cid_from_string(&request.job_id) {
        Ok(cid) => cid,
        Err(e) => {
            return map_rust_error_to_json_response(
                format!("Invalid job ID format: {}", e),
                StatusCode::BAD_REQUEST,
            )
            .into_response();
        }
    };

    let executor_did = match Did::from_str(&request.executor_did) {
        Ok(did) => did,
        Err(e) => {
            return map_rust_error_to_json_response(
                format!("Invalid executor DID format: {}", e),
                StatusCode::BAD_REQUEST,
            )
            .into_response();
        }
    };

    let result_cid = match parse_cid_from_string(&request.result_cid) {
        Ok(cid) => cid,
        Err(e) => {
            return map_rust_error_to_json_response(
                format!("Invalid result CID format: {}", e),
                StatusCode::BAD_REQUEST,
            )
            .into_response();
        }
    };

    let signature_bytes = match hex::decode(&request.signature_hex) {
        Ok(bytes) => SignatureBytes(bytes),
        Err(e) => {
            return map_rust_error_to_json_response(
                format!("Invalid signature hex format: {}", e),
                StatusCode::BAD_REQUEST,
            )
            .into_response();
        }
    };

    // Create execution receipt
    let execution_receipt = IdentityExecutionReceipt {
        job_id,
        executor_did,
        result_cid,
        cpu_ms: request.cpu_ms,
        sig: signature_bytes,
    };

    let receipt_json = match serde_json::to_string(&execution_receipt) {
        Ok(json) => json,
        Err(e) => {
            return map_rust_error_to_json_response(
                format!("Failed to serialize receipt: {}", e),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into_response();
        }
    };

    // Submit receipt via Host ABI
    let reputation_updater = ReputationUpdater::new();
    match host_anchor_receipt(&state.runtime_context, &receipt_json, &reputation_updater).await {
        Ok(anchored_cid) => {
            info!(
                "[Node] Receipt anchored via runtime, Anchored CID: {}",
                anchored_cid
            );
            (
                StatusCode::ACCEPTED,
                Json(serde_json::json!({
                    "anchored_cid": anchored_cid.to_string(),
                    "receipt_job_id": execution_receipt.job_id.to_string()
                })),
            )
                .into_response()
        }
        Err(e) => {
            error!("[Node] Error anchoring receipt via runtime: {:?}", e);
            map_rust_error_to_json_response(
                format!("Receipt anchoring failed: {}", e),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into_response()
        }
    }
}

// --- Test module (can be expanded later) ---
#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt; // for `oneshot` and `ready`

    // Helper to create a test router with a fresh RuntimeContext
    async fn test_app() -> Router {
        app_router().await
    }

    #[tokio::test]
    async fn info_endpoint_works() {
        let app = test_app().await;
        let response = app
            .oneshot(Request::builder().uri("/info").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        // TODO: Deserialize and check content if needed
    }

    #[tokio::test]
    async fn mesh_submit_job_endpoint_basic() {
        let app = test_app().await;

        let job_req = SubmitJobRequest {
            manifest_cid: Cid::new_v1_dummy(0x55, 0x14, b"test_manifest").to_string(),
            spec_json: serde_json::json!({ "Echo": { "payload": "hello" } }),
            cost_mana: 50,
        };
        let job_req_json = serde_json::to_string(&job_req).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/mesh/submit")
                    .header("content-type", "application/json")
                    .body(Body::from(job_req_json))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::ACCEPTED);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(body_json.get("job_id").is_some());
        info!("Mesh submit response: {:?}", body_json);
    }

    #[tokio::test]
    async fn complete_http_to_mesh_pipeline() {
        let app = test_app().await;

        // Step 1: Submit a job via HTTP
        let job_req = SubmitJobRequest {
            manifest_cid: Cid::new_v1_dummy(0x55, 0x14, b"pipeline_test_manifest").to_string(),
            spec_json: serde_json::json!({ "Echo": { "payload": "HTTP pipeline test" } }),
            cost_mana: 100,
        };
        let job_req_json = serde_json::to_string(&job_req).unwrap();

        let submit_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/mesh/submit")
                    .header("content-type", "application/json")
                    .body(Body::from(job_req_json))
                    .unwrap(),
            )
            .await
            .unwrap();

        let status = submit_response.status();
        if status != StatusCode::ACCEPTED {
            let error_body = axum::body::to_bytes(submit_response.into_body(), usize::MAX)
                .await
                .unwrap();
            let error_text = String::from_utf8_lossy(&error_body);
            panic!(
                "Job submission failed with status {}: {}",
                status, error_text
            );
        }

        let submit_body = axum::body::to_bytes(submit_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let submit_json: serde_json::Value = serde_json::from_slice(&submit_body).unwrap();
        let job_id = submit_json
            .get("job_id")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        info!(
            "✅ Job submitted via HTTP, Job ID from response: {}",
            job_id
        );

        // Debug: Let's also check what jobs are actually in the system
        let debug_list_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/mesh/jobs")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let debug_list_body = axum::body::to_bytes(debug_list_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let debug_list_json: serde_json::Value = serde_json::from_slice(&debug_list_body).unwrap();
        info!("🔍 Debug - All jobs in system: {:?}", debug_list_json);

        // Step 2: Check job status immediately (should be pending)
        let status_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(&format!("/mesh/jobs/{}", job_id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let status_code = status_response.status();
        if status_code != StatusCode::OK {
            let error_body = axum::body::to_bytes(status_response.into_body(), usize::MAX)
                .await
                .unwrap();
            let error_text = String::from_utf8_lossy(&error_body);
            panic!(
                "Job status check failed with status {}: {}",
                status_code, error_text
            );
        }

        let status_body = axum::body::to_bytes(status_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let status_json: serde_json::Value = serde_json::from_slice(&status_body).unwrap();
        info!("✅ Job status response: {:?}", status_json);
        assert_eq!(status_json.get("job_id").unwrap().as_str().unwrap(), job_id);

        // Step 3: List all jobs (should include our job)
        let list_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/mesh/jobs")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(list_response.status(), StatusCode::OK);
        let list_body = axum::body::to_bytes(list_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let list_json: serde_json::Value = serde_json::from_slice(&list_body).unwrap();
        info!("✅ Jobs list response: {:?}", list_json);

        let jobs = list_json.get("jobs").unwrap().as_array().unwrap();
        assert!(!jobs.is_empty());
        let found_job = jobs
            .iter()
            .find(|job| job.get("job_id").unwrap().as_str().unwrap() == job_id);
        assert!(found_job.is_some());

        // Step 4: Validate core HTTP → mesh job pipeline is working
        // (Receipt submission with real cryptographic verification is tested in the runtime layer)

        info!("🎉 Complete HTTP-to-mesh pipeline test passed!");
    }

    #[tokio::test]
    async fn test_simple_job_submission_and_listing() {
        let app = test_app().await;

        // Step 1: Submit a job
        let job_req = SubmitJobRequest {
            manifest_cid: Cid::new_v1_dummy(0x55, 0x14, b"simple_test").to_string(),
            spec_json: serde_json::json!({ "Echo": { "payload": "simple test" } }),
            cost_mana: 50,
        };
        let job_req_json = serde_json::to_string(&job_req).unwrap();

        let submit_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/mesh/submit")
                    .header("content-type", "application/json")
                    .body(Body::from(job_req_json))
                    .unwrap(),
            )
            .await
            .unwrap();

        println!("Submit response status: {}", submit_response.status());
        let submit_body = axum::body::to_bytes(submit_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let submit_text = String::from_utf8_lossy(&submit_body);
        println!("Submit response body: {}", submit_text);

        // Step 2: List all jobs
        let list_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/mesh/jobs")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        println!("List response status: {}", list_response.status());
        let list_body = axum::body::to_bytes(list_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let list_text = String::from_utf8_lossy(&list_body);
        println!("List response body: {}", list_text);
    }

    // TODO: Add more tests for DAG and Governance endpoints with new AppState structure.
    // Example for dag_put, assuming DagBlock can be easily created.
    // #[tokio::test]
    // async fn dag_put_get_cycle() {
    //     let app = test_app().await;
    //     let test_data = b"hello dag world";
    //     let block_cid = icn_common::cid::generate_cid(test_data).unwrap();
    //     let dag_block = DagBlock { cid: block_cid.clone(), data: test_data.to_vec(), links: vec![] };
    //     let dag_block_json = serde_json::to_string(&dag_block).unwrap();

    //     // PUT
    //     let response_put = app.clone()
    //         .oneshot(
    //             Request::builder()
    //                 .method("POST")
    //                 .uri("/dag/put")
    //                 .header("content-type", "application/json")
    //                 .body(Body::from(dag_block_json))
    //                 .unwrap(),
    //         )
    //         .await
    //         .unwrap();
    //     assert_eq!(response_put.status(), StatusCode::CREATED);

    //     // GET
    //     let cid_req = CidRequest { cid: block_cid.to_string() };
    //     let cid_req_json = serde_json::to_string(&cid_req).unwrap();
    //     let response_get = app
    //         .oneshot(
    //             Request::builder()
    //                 .method("POST")
    //                 .uri("/dag/get")
    //                 .header("content-type", "application/json")
    //                 .body(Body::from(cid_req_json))
    //                 .unwrap(),
    //         )
    //         .await
    //         .unwrap();
    //     assert_eq!(response_get.status(), StatusCode::OK);
    //     let body_get = axum::body::to_bytes(response_get.into_body(), usize::MAX).await.unwrap();
    //     let fetched_block: DagBlock = serde_json::from_slice(&body_get).unwrap();
    //     assert_eq!(fetched_block.cid, block_cid);
    //     assert_eq!(fetched_block.data, test_data.to_vec());
    // }
}
