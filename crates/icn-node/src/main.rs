#![doc = include_str!("../README.md")]

//! # ICN Node Crate
//! This crate provides the main binary for running a long-lived InterCooperative Network (ICN) daemon.
//! It integrates various core components to operate a functional ICN node, handling initialization,
//! lifecycle, configuration, service hosting, and persistence.

use icn_common::{Cid, DagBlock, NodeInfo, NodeStatus, ICN_CORE_VERSION, Did};
use icn_dag::{StorageService, InMemoryDagStore, FileDagStore};
use icn_governance::{GovernanceModule, ProposalId, VoteOption};
use icn_api::governance_trait::{SubmitProposalRequest as ApiSubmitProposalRequest, CastVoteRequest as ApiCastVoteRequest};

// ICN Runtime imports
use icn_runtime::context::{
    RuntimeContext,
    StubSigner, 
    StubDagStore as RuntimeStubDagStore, // Alias to avoid conflict with icn_dag::StubDagStore if any
    StubMeshNetworkService,
};
use icn_runtime::{
    host_submit_mesh_job,
    // We might need more ABI functions later, e.g. for submitting receipts if done via HTTP endpoint
};
use icn_identity::{generate_ed25519_keypair, did_key_from_verifying_key, SignatureBytes};
use icn_mesh::ActualMeshJob; // For the submit endpoint

use axum::{
    extract::{State, Path as AxumPath},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use clap::Parser;
use log::{info, error, warn, debug}; // Added log macros
use serde::{Serialize, Deserialize}; // Added Deserialize
use std::{net::SocketAddr, path::PathBuf, sync::Arc, str::FromStr};

// --- CLI Arguments --- 

#[derive(Parser, Debug)]
#[clap(author, version = ICN_CORE_VERSION, about = "ICN Node HTTP Server", long_about = None)]
struct Cli {
    #[clap(long, value_enum, default_value = "memory", help = "Storage backend type")]
    storage_backend: StorageBackendType,

    #[clap(long, default_value = "./icn_data/node_store", help = "Path for file-based storage (if 'file' backend is chosen)")]
    storage_path: PathBuf,

    #[clap(long, default_value = "127.0.0.1:7845", help = "Listen address for the HTTP server")]
    listen_addr: String,

    #[clap(long, help = "Optional fixed DID for the node (e.g., did:key:zExample...)")]
    node_did: Option<String>,

    #[clap(long, help = "Optional fixed Ed25519 private key (bs58 encoded string) for the node DID. If not provided and node_did is, it implies did:key or resolvable DID. If neither, a new key is generated.")]
    node_private_key_bs58: Option<String>,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum StorageBackendType {
    Memory,
    File,
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
    let node_did = Did::from_string(&node_did_string).expect("Failed to create test node DID");
    info!("Test/Embedded Node DID: {}", node_did);

    let signer = Arc::new(StubSigner::new_with_keys(sk, pk)); // Use the generated keys
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
    rt_ctx.spawn_mesh_job_manager().await; // Start the job manager

    let app_state = AppState {
        runtime_context: Arc::new(rt_ctx),
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
    // TODO: Proper key loading/generation, potentially from a wallet file or KMS
    let (node_sk, node_pk) = generate_ed25519_keypair(); // Generate fresh for now
    let node_did_string = did_key_from_verifying_key(&node_pk);
    let node_did = Did::from_string(&node_did_string).expect("Failed to create node DID");
    info!("Node Operator DID: {}", node_did);
    // In a real scenario, you might load sk from cli.node_private_key_bs58

    // --- Initialize RuntimeContext Components ---
    let signer = Arc::new(StubSigner::new_with_keys(node_sk, node_pk));
    
    let dag_store_for_rt: Arc<dyn icn_runtime::context::StorageService + Send + Sync> = match cli.storage_backend {
        StorageBackendType::Memory => Arc::new(RuntimeStubDagStore::new()), // Use RT's stub for now
        StorageBackendType::File => {
            // If FileDagStore is to be used with RT, it needs to impl runtime's StorageService trait
            // For now, let's stick to RuntimeStubDagStore for simplicity or implement the trait for FileDagStore.
            // This example uses a distinct DagStore for the main app vs the RuntimeContext for now.
            warn!("FileDagStore for RuntimeContext not fully implemented yet, using RuntimeStubDagStore.");
            Arc::new(RuntimeStubDagStore::new())
        }
    };

    // TODO: Initialize real NetworkService based on CLI args (e.g., libp2p with bootstrap peers)
    let mesh_network_service = Arc::new(StubMeshNetworkService::new()); 
    info!("Using StubMeshNetworkService. P2P features for mesh are not enabled.");

    // Initialize GovernanceModule (it's internal to RuntimeContext now)
    // let governance_module = Arc::new(Mutex::new(GovernanceModule::new()));

    // --- Create RuntimeContext ---
    let rt_ctx = RuntimeContext::new(
        node_did.clone(), 
        mesh_network_service, 
        signer, 
        dag_store_for_rt,
        // GovernanceModule::new() // RuntimeContext::new handles its own GovernanceModule
    );
    rt_ctx.spawn_mesh_job_manager().await; // Start the job manager task
    info!("ICN RuntimeContext initialized and JobManager spawned.");

    // --- Create AppState for Axum ---
    let app_state = AppState {
        runtime_context: Arc::new(rt_ctx),
        node_name: "ICN Node".to_string(), // This could be configurable
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
        .route("/mesh/submit", post(mesh_submit_job_handler)) // New endpoint
        .with_state(app_state.clone()); 

    let addr: SocketAddr = cli.listen_addr.parse().expect("Invalid listen address");
    info!("ICN Node HTTP server listening on {}", addr);
    
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
    let cid_to_get = match Cid::from_str(&cid_request.cid) {
        Ok(c) => c,
        Err(e) => return map_rust_error_to_json_response(format!("Invalid CID format: {}", e), StatusCode::BAD_REQUEST).into_response(),
    };
    match state.runtime_context.dag_store.get(&cid_to_get).await {
        Ok(Some(data_bytes)) => {
            // Attempt to deserialize as DagBlock. This assumes stored data is always a DagBlock.
            // A more robust system might store raw bytes and metadata separately.
            let block = DagBlock { cid: cid_to_get, data: data_bytes, links: vec![] }; // Reconstruct DagBlock, links missing
            (StatusCode::OK, Json(block)).into_response()
        },
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
    // Governance operations are now via RuntimeContext methods if they exist,
    // or directly via its governance_module.
    // RuntimeContext has `create_governance_proposal` which is synchronous.
    // For Axum, it's okay if the handler awaits on async RT calls, but sync calls inside RT are fine.
    
    // Temporary direct access, ideally RuntimeContext provides async wrappers or methods.
    let mut gov_mod = state.runtime_context.governance_module.lock().await; // Assuming Tokio Mutex in RT

    let proposer_did: Did = match request.proposer_did.parse() {
        Ok(did) => did,
        Err(e) => return map_rust_error_to_json_response(e, StatusCode::BAD_REQUEST).into_response(),
    };

    // TODO: Ensure proposer_did matches state.runtime_context.current_identity or has permission
    if proposer_did != state.runtime_context.current_identity {
        warn!("Gov submit by {} but context identity is {}. Allowing for now.", proposer_did, state.runtime_context.current_identity);
        // return map_rust_error_to_json_response("Proposer DID does not match node identity", StatusCode::FORBIDDEN).into_response();
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

    match gov_mod.submit_proposal(
        proposer_did, // Proposer is from the request
        proposal_type,
        request.description,
        request.duration_secs,
    ) {
        Ok(proposal_id) => (StatusCode::CREATED, Json(proposal_id)).into_response(),
        Err(e) => map_rust_error_to_json_response(format!("Governance submit error: {}", e), StatusCode::BAD_REQUEST).into_response(),
    }
}

// POST /governance/vote – Cast a vote. (Body: CastVoteRequest JSON)
async fn gov_vote_handler(
    State(state): State<AppState>,
    Json(request): Json<ApiCastVoteRequest>,
) -> impl IntoResponse {
    debug!("Received /governance/vote request: {:?}", request);
    let mut gov_mod = state.runtime_context.governance_module.lock().await;

    let voter_did: Did = match request.voter_did.parse() {
        Ok(did) => did,
        Err(e) => return map_rust_error_to_json_response(e, StatusCode::BAD_REQUEST).into_response(),
    };
     // TODO: Ensure voter_did matches state.runtime_context.current_identity or has permission
    if voter_did != state.runtime_context.current_identity {
        warn!("Gov vote by {} but context identity is {}. Allowing for now.", voter_did, state.runtime_context.current_identity);
        // return map_rust_error_to_json_response("Voter DID does not match node identity", StatusCode::FORBIDDEN).into_response();
    }

    let proposal_id = ProposalId(request.proposal_id.clone());
    let vote_option = match request.vote_option.to_lowercase().as_str() {
        "yes" => VoteOption::Yes,
        "no" => VoteOption::No,
        "abstain" => VoteOption::Abstain,
        _ => return map_rust_error_to_json_response("Invalid vote option", StatusCode::BAD_REQUEST).into_response(),
    };

    match gov_mod.cast_vote(voter_did, proposal_id, vote_option) {
        Ok(_) => (StatusCode::OK, Json("Vote cast successfully")).into_response(),
        Err(e) => map_rust_error_to_json_response(format!("Governance vote error: {}",e), StatusCode::BAD_REQUEST).into_response(),
    }
}

// GET /governance/proposals
async fn gov_list_proposals_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    debug!("Received /governance/proposals request");
    let gov_mod = state.runtime_context.governance_module.lock().await;
    let proposals = gov_mod.get_all_proposals(); // Assuming this returns Vec<ProposalOutput> or similar
    (StatusCode::OK, Json(proposals))
}

// GET /governance/proposal/:proposal_id
async fn gov_get_proposal_handler(
    State(state): State<AppState>,
    AxumPath(proposal_id_str): AxumPath<String>,
) -> impl IntoResponse {
    debug!("Received /governance/proposal/{} request", proposal_id_str);
    let gov_mod = state.runtime_context.governance_module.lock().await;
    let proposal_id = ProposalId(proposal_id_str);
    match gov_mod.get_proposal(&proposal_id) {
        Some(proposal) => (StatusCode::OK, Json(proposal)).into_response(),
        None => map_rust_error_to_json_response("Proposal not found", StatusCode::NOT_FOUND).into_response(),
    }
}

// --- Mesh Job Endpoints ---

/// Request body for submitting a mesh job.
/// Note: `id` and `creator_did` will be overridden by the runtime.
/// Signature also will be handled by the job submission logic after this initial DTO.
#[derive(Deserialize, Debug)] // Debug for logging
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
    info!("[NODE] /mesh/submit received: {:?}", request);

    let manifest_cid = match Cid::from_str(&request.manifest_cid) {
        Ok(cid) => cid,
        Err(e) => return map_rust_error_to_json_response(format!("Invalid manifest_cid: {}", e), StatusCode::BAD_REQUEST).into_response(),
    };

    // Reconstruct ActualMeshJob from request. For now, spec is tricky.
    // If JobSpec becomes more complex, need a proper way to deserialize it.
    // Assuming JobSpec is simple for now or can be default.
    // The host_submit_mesh_job in runtime expects a full ActualMeshJob JSON string.
    // We are constructing parts of it here.

    // For ActualMeshJob, the `id` and `creator_did` and `signature` are set by the runtime/signing process.
    // We need to create a temporary ActualMeshJob that host_submit_mesh_job can work with.
    // The signature will be added *after* this initial DTO, by the node itself before calling host_submit_mesh_job,
    // or host_submit_mesh_job needs to take raw components and construct+sign.
    // Current host_submit_mesh_job takes a JSON string and *re-parses* it, then sets id/creator.
    // This is a bit awkward. Let's create a job_template that host_submit_mesh_job can fill.

    // Create a temporary ActualMeshJob to be serialized for host_submit_mesh_job
    // The `id`, `creator_did`, and `signature` fields will be ignored by `host_submit_mesh_job` 
    // as it re-parses and then sets them itself based on the context.
    // This is not ideal. `host_submit_mesh_job` should ideally take components.
    let temp_job_for_serialization = icn_mesh::ActualMeshJob {
        id: Cid::default(), // Will be replaced by host_submit_mesh_job
        manifest_cid,
        spec: Default::default(), // TODO: Deserialize request.spec_json into JobSpec
        creator_did: Default::default(), // Will be replaced by host_submit_mesh_job's context identity
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