#![doc = include_str!("../README.md")]

//! # ICN Node Crate
//! This crate provides the main binary for running a long-lived InterCooperative Network (ICN) daemon.
//! It integrates various core components to operate a functional ICN node, handling initialization,
//! lifecycle, configuration, service hosting, and persistence.

use icn_common::{NodeInfo, Did, Cid, ICN_CORE_VERSION, CommonError};
use icn_identity::{generate_ed25519_keypair, did_key_from_verifying_key, SigningKey, VerifyingKey, SignatureBytes, EdSignature, ExecutionReceipt as IdentityExecutionReceipt};
use icn_runtime::context::{RuntimeContext, HostAbiError, StubSigner as RuntimeStubSigner, StubMeshNetworkService, StubDagStore as RuntimeStubDagStore, MeshNetworkService, Signer as RuntimeSigner};
use icn_runtime::{
    host_submit_mesh_job,
};
use icn_mesh::{ActualMeshJob};
use icn_governance::{ProposalId, VoteOption};
use icn_api::governance_trait::{self as governance_api, ProposalInputType as ApiProposalInputType, CastVoteRequest as ApiCastVoteRequest, SubmitProposalRequest as ApiSubmitProposalRequest};

use std::sync::Arc;
use tokio::sync::Mutex;
use std::net::SocketAddr;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use axum::{
    extract::{Path as AxumPath, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use clap::Parser;
use env_logger;
use log::{info, warn, error, debug};
use std::str::FromStr;

#[cfg(feature = "enable-libp2p")]
use libp2p::{Multiaddr, PeerId as Libp2pPeerId};

// --- CLI Arguments --- 

#[derive(Parser, Debug)]
#[clap(author, version = ICN_CORE_VERSION, about = "ICN Node HTTP Server", long_about = None)]
struct Cli {
    #[clap(long, value_enum, default_value = "memory", help = "Storage backend type")]
    storage_backend: StorageBackendType,

    #[clap(long, default_value = "./icn_data/node_store", help = "Path for file-based storage (if 'file' backend is chosen)")]
    storage_path: PathBuf,

    #[clap(long, default_value = "127.0.0.1:7845", help = "Listen address for the HTTP server")]
    http_listen_addr: String,

    #[clap(long, help = "Optional fixed DID for the node (e.g., did:key:zExample...)")]
    node_did: Option<String>,

    #[clap(long, help = "Optional fixed Ed25519 private key (bs58 encoded string) for the node DID. If not provided and node_did is, it implies did:key or resolvable DID. If neither, a new key is generated.")]
    node_private_key_bs58: Option<String>,

    #[clap(long, help = "Human-readable name for this node (for logging and identification)")]
    node_name: Option<String>,

    #[clap(long, default_value = "/ip4/0.0.0.0/tcp/0", help = "Libp2p listen address for P2P networking")]
    p2p_listen_addr: String,

    #[clap(long, help = "Bootstrap peer multiaddrs for P2P discovery (format: /ip4/1.2.3.4/tcp/port/p2p/PeerID)", value_delimiter = ',')]
    bootstrap_peers: Option<Vec<String>>,

    #[clap(long, action, help = "Enable real libp2p networking (requires with-libp2p feature)")]
    enable_p2p: bool,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum StorageBackendType {
    Memory,
    File,
}

// --- Supporting Types ---

#[derive(Serialize)]
struct NodeStatus {
    is_online: bool,
    peer_count: u32,
    current_block_height: u64,
    version: String,
}

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
pub async fn app_router() -> Router { // Renamed to app_router and made async for RT init
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
        .route("/dag/get", post(dag_get_handler))   // These will use RT context's DAG store
        .route("/governance/submit", post(gov_submit_handler)) // Uses RT context's Gov mod
        .route("/governance/vote", post(gov_vote_handler))     // Uses RT context's Gov mod
        .route("/governance/proposals", get(gov_list_proposals_handler)) // Uses RT context's Gov mod
        .route("/governance/proposal/:proposal_id", get(gov_get_proposal_handler)) // Uses RT context's Gov mod
        .route("/mesh/submit", post(mesh_submit_job_handler)) // New endpoint
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
    
    let node_name = cli.node_name.clone().unwrap_or_else(|| "ICN Node".to_string());
    info!("Starting {} with DID: {}", node_name, node_did);

    // --- Create RuntimeContext with Networking ---
    let rt_ctx = if cli.enable_p2p {
        #[cfg(feature = "enable-libp2p")]
        {
            info!("Enabling libp2p networking with P2P listen address: {}", cli.p2p_listen_addr);
            
            // Parse bootstrap peers if provided
            let bootstrap_peers = if let Some(peer_strings) = &cli.bootstrap_peers {
                let mut parsed_peers = Vec::new();
                for peer_str in peer_strings {
                    match peer_str.parse::<Multiaddr>() {
                        Ok(multiaddr) => {
                            // Extract PeerID from multiaddr if present
                            if let Some(libp2p::core::multiaddr::Protocol::P2p(peer_id)) = multiaddr.iter().last() {
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
                            error!("Failed to parse bootstrap peer multiaddr '{}': {}", peer_str, e);
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

            match RuntimeContext::new_with_real_libp2p(&node_did_string, bootstrap_peers).await {
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
        .route("/governance/proposal/:proposal_id", get(gov_get_proposal_handler))
        .route("/mesh/submit", post(mesh_submit_job_handler))
        .with_state(app_state.clone()); 

    let addr: SocketAddr = cli.http_listen_addr.parse().expect("Invalid HTTP listen address");
    info!("🌐 {} HTTP server listening on {}", node_name, addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, router.into_make_service()).await.unwrap();
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
    match state.runtime_context.dag_store.put(&block.data).await { // Assuming block.data is Vec<u8>
        Ok(cid) => (StatusCode::CREATED, Json(cid)).into_response(),
        Err(e) => map_rust_error_to_json_response(format!("DAG put error: {}", e), StatusCode::INTERNAL_SERVER_ERROR).into_response(),
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
        Ok(None) => map_rust_error_to_json_response("Block not found", StatusCode::NOT_FOUND).into_response(),
        Err(e) => map_rust_error_to_json_response(format!("DAG get error: {}", e), StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

#[derive(Deserialize)]
struct CidRequest { cid: String }


// POST /governance/submit – Submit a proposal. (Body: SubmitProposalRequest JSON)
async fn gov_submit_handler(
    State(state): State<AppState>,
    Json(request): Json<ApiSubmitProposalRequest>,
) -> impl IntoResponse {
    debug!("Received /governance/submit request: {:?}", request);
    // TODO: Governance operations need to be implemented in RuntimeContext
    map_rust_error_to_json_response("Governance operations not yet implemented", StatusCode::NOT_IMPLEMENTED).into_response()
    
    /* TODO: Uncomment when governance methods are implemented
    let mut gov_mod = state.runtime_context.governance_module.lock().await;

    let proposer_did: Did = match request.proposer_did.parse() {
        Ok(did) => did,
        Err(e) => return map_rust_error_to_json_response(e, StatusCode::BAD_REQUEST).into_response(),
    };

    if proposer_did != state.runtime_context.current_identity {
        warn!("Gov submit by {} but context identity is {}. Allowing for now.", proposer_did, state.runtime_context.current_identity);
    }

    let proposal_type: icn_governance::ProposalType = match request.proposal {
        icn_api::governance_trait::ProposalInputType::SystemParameterChange { param, value } => {
            icn_governance::ProposalType::SystemParameterChange(param, value)
        }
        icn_api::governance_trait::ProposalInputType::MemberAdmission { did } => {
            match Did::from_str(&did) { 
                Ok(parsed_did) => icn_governance::ProposalType::NewMemberInvitation(parsed_did),
                Err(e) => return map_rust_error_to_json_response(format!("Failed to parse MemberAdmission DID: {}", e), StatusCode::BAD_REQUEST).into_response(),
            }
        }
        icn_api::governance_trait::ProposalInputType::SoftwareUpgrade { version } => {
            icn_governance::ProposalType::SoftwareUpgrade(version)
        }
        icn_api::governance_trait::ProposalInputType::GenericText { text } => {
            icn_governance::ProposalType::GenericText(text)
        }
    };

    match (*gov_mod).submit_proposal(
        proposer_did,
        proposal_type,
        request.description,
        request.duration_secs,
    ) {
        Ok(proposal_id) => (StatusCode::CREATED, Json(proposal_id)).into_response(),
        Err(e) => map_rust_error_to_json_response(format!("Governance submit error: {}", e), StatusCode::BAD_REQUEST).into_response(),
    }
    */
}

// POST /governance/vote – Cast a vote. (Body: CastVoteRequest JSON)
async fn gov_vote_handler(
    State(state): State<AppState>,
    Json(request): Json<ApiCastVoteRequest>,
) -> impl IntoResponse {
    debug!("Received /governance/vote request: {:?}", request);
    // TODO: Governance operations need to be implemented in RuntimeContext
    map_rust_error_to_json_response("Governance operations not yet implemented", StatusCode::NOT_IMPLEMENTED).into_response()

    /* TODO: Uncomment when governance methods are implemented
    let mut gov_mod = state.runtime_context.governance_module.lock().await;

    let voter_did: Did = match request.voter_did.parse() {
        Ok(did) => did,
        Err(e) => return map_rust_error_to_json_response(e, StatusCode::BAD_REQUEST).into_response(),
    };
    
    if voter_did != state.runtime_context.current_identity {
        warn!("Gov vote by {} but context identity is {}. Allowing for now.", voter_did, state.runtime_context.current_identity);
    }

    let proposal_id = ProposalId(request.proposal_id.clone());
    let vote_option = match request.vote_option.to_lowercase().as_str() {
        "yes" => VoteOption::Yes,
        "no" => VoteOption::No,
        "abstain" => VoteOption::Abstain,
        _ => return map_rust_error_to_json_response("Invalid vote option", StatusCode::BAD_REQUEST).into_response(),
    };

    match (*gov_mod).cast_vote(voter_did, proposal_id, vote_option) {
        Ok(_) => (StatusCode::OK, Json("Vote cast successfully")).into_response(),
        Err(e) => map_rust_error_to_json_response(format!("Governance vote error: {}",e), StatusCode::BAD_REQUEST).into_response(),
    }
    */
}

// GET /governance/proposals
async fn gov_list_proposals_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    debug!("Received /governance/proposals request");
    // TODO: Governance operations need to be implemented in RuntimeContext
    map_rust_error_to_json_response("Governance operations not yet implemented", StatusCode::NOT_IMPLEMENTED).into_response()
    
    /* TODO: Uncomment when governance methods are implemented
    let gov_mod = state.runtime_context.governance_module.lock().await;
    let proposals = (*gov_mod).get_all_proposals();
    (StatusCode::OK, Json(proposals))
    */
}

// GET /governance/proposal/:proposal_id
async fn gov_get_proposal_handler(
    State(state): State<AppState>,
    AxumPath(proposal_id_str): AxumPath<String>,
) -> impl IntoResponse {
    debug!("Received /governance/proposal/{} request", proposal_id_str);
    // TODO: Governance operations need to be implemented in RuntimeContext
    map_rust_error_to_json_response("Governance operations not yet implemented", StatusCode::NOT_IMPLEMENTED).into_response()
    
    /* TODO: Uncomment when governance methods are implemented
    let gov_mod = state.runtime_context.governance_module.lock().await;
    let proposal_id = ProposalId(proposal_id_str);
    match (*gov_mod).get_proposal(&proposal_id) {
        Some(proposal) => (StatusCode::OK, Json(proposal)).into_response(),
        None => map_rust_error_to_json_response("Proposal not found", StatusCode::NOT_FOUND).into_response(),
    }
    */
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
        Err(e) => return map_rust_error_to_json_response(format!("Failed to parse job spec: {}", e), StatusCode::BAD_REQUEST).into_response(),
    };

    // This temporary job is just to satisfy the current host_submit_mesh_job ABI,
    // which expects a fully formed ActualMeshJob. The real job ID and potentially
    // other fields will be determined by the runtime.
    let temp_job_for_serialization = ActualMeshJob { // icn_mesh::ActualMeshJob
        id: Cid::new_v1_dummy(0, 0, b"placeholder_default_cid"), // Will be replaced by host_submit_mesh_job
        manifest_cid,
        spec: job_spec,
        creator_did: state.runtime_context.current_identity.clone(), // Use node's DID as creator
        cost_mana: request.cost_mana,
        signature: SignatureBytes(vec![]), // Will be ignored and re-signed if host_submit_mesh_job handles it, or added before.
    };

    let job_json = match serde_json::to_string(&temp_job_for_serialization) {
        Ok(json) => json,
        Err(e) => return map_rust_error_to_json_response(format!("Failed to serialize job for submission: {}", e), StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    };

    match host_submit_mesh_job(&state.runtime_context, &job_json).await {
        Ok(job_id_cid) => {
            info!("[NODE] Job submitted via runtime, Job ID: {}", job_id_cid);
            (StatusCode::ACCEPTED, Json(serde_json::json!({ "job_id": job_id_cid.to_string() }))).into_response()
        }
        Err(e) => {
            error!("[NODE] Error submitting job via runtime: {:?}", e);
            map_rust_error_to_json_response(format!("Mesh job submission failed: {}", e), StatusCode::INTERNAL_SERVER_ERROR).into_response()
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
    use icn_mesh::{JobSpec}; // Assuming JobSpec can be created

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
            spec_json: serde_json::json!({ "type": "echo", "payload": "hello" }),
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
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(body_json.get("job_id").is_some());
        info!("Mesh submit response: {:?}", body_json);
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