#![doc = include_str!("../README.md")]

//! # ICN Node Crate
//! This crate provides the main binary for running a long-lived InterCooperative Network (ICN) daemon.
//! It integrates various core components to operate a functional ICN node, handling initialization,
//! lifecycle, configuration, service hosting, and persistence.

use icn_common::{Cid, DagBlock, NodeInfo, NodeStatus, ICN_CORE_VERSION, Did};
use icn_dag::{StorageService, InMemoryDagStore, FileDagStore};
use icn_governance::{GovernanceModule, ProposalId, VoteOption};
use icn_api::governance_trait::{SubmitProposalRequest as ApiSubmitProposalRequest, CastVoteRequest as ApiCastVoteRequest};

use axum::{
    extract::{State, Path as AxumPath},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use clap::Parser;
use serde::Serialize;
use std::{net::SocketAddr, path::PathBuf, sync::Arc, sync::Mutex, str::FromStr};

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
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum StorageBackendType {
    Memory,
    File,
    // Sled, // Add when Sled backend is implemented
}

// --- Application State ---
#[derive(Clone)]
struct AppState {
    dag_storage: Arc<Mutex<dyn StorageService<DagBlock> + Send + Sync>>,
    governance_module: Arc<Mutex<GovernanceModule>>,
    node_name: String,
    node_version: String,
}

// --- Public App Constructor ---
pub fn app() -> Router {
    // For tests, we'll use in-memory storage and default governance.
    // A more sophisticated setup might allow configuration for tests.
    let dag_storage: Arc<Mutex<dyn StorageService<DagBlock> + Send + Sync>> =
        Arc::new(Mutex::new(InMemoryDagStore::new()));
    
    let governance_module = Arc::new(Mutex::new(GovernanceModule::new()));

    let app_state = AppState {
        dag_storage,
        governance_module,
        node_name: "ICN Reference Node".to_string(), // Consistent with test assertion
        node_version: ICN_CORE_VERSION.to_string(), // Consistent with test assertion
    };

    Router::new()
        .route("/info", get(info_handler))
        .route("/status", get(status_handler))
        .route("/dag/put", post(dag_put_handler))
        .route("/dag/get", post(dag_get_handler))
        .route("/governance/submit", post(gov_submit_handler))
        .route("/governance/vote", post(gov_vote_handler))
        .route("/governance/proposals", get(gov_list_proposals_handler))
        .route("/governance/proposal/:proposal_id", get(gov_get_proposal_handler))
        .with_state(app_state)
}

// --- Main Application Logic ---
#[allow(dead_code)] // Allowed because this main is not used when icn-node is a library (e.g. for tests)
#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let dag_storage: Arc<Mutex<dyn StorageService<DagBlock> + Send + Sync>> =
        match cli.storage_backend {
            StorageBackendType::Memory => Arc::new(Mutex::new(InMemoryDagStore::new())),
        StorageBackendType::File => {
                let store_path = cli.storage_path.clone();
                println!("Using FileDagStore at: {:?}", store_path);
                let file_store = FileDagStore::new(store_path)
                    .map_err(|e| {
                        eprintln!("Failed to initialize FileDagStore: {}", e);
                        std::process::exit(1);
                    })
                    .unwrap();
                Arc::new(Mutex::new(file_store)) as Arc<Mutex<dyn StorageService<DagBlock> + Send + Sync>>
            }
        };

    let governance_module = Arc::new(Mutex::new(GovernanceModule::new())); // Consider sled for main app

    let app_state = AppState {
        dag_storage,
        governance_module,
        node_name: "ICN Reference Node".to_string(), // This could be configurable
        node_version: ICN_CORE_VERSION.to_string(),
    };

    let router = Router::new()
        .route("/info", get(info_handler))
        .route("/status", get(status_handler))
        .route("/dag/put", post(dag_put_handler))
        .route("/dag/get", post(dag_get_handler))
        .route("/governance/submit", post(gov_submit_handler))
        .route("/governance/vote", post(gov_vote_handler))
        .route("/governance/proposals", get(gov_list_proposals_handler))
        .route("/governance/proposal/:proposal_id", get(gov_get_proposal_handler))
        .with_state(app_state); // Use the state built from CLI args

    let addr_str = cli.listen_addr;
    let addr: SocketAddr = addr_str.parse().expect("Invalid listen address");
    println!("ICN Node HTTP server listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, router.into_make_service()).await.unwrap(); // Use the router from main
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
        status_message: "ICN Node is operational".to_string(),
    };
    (StatusCode::OK, Json(info))
}

// GET /status – Node status.
async fn status_handler(State(state): State<AppState>) -> impl IntoResponse {
    let status = NodeStatus {
        is_online: true,
        peer_count: 0,
        current_block_height: 0,
        version: state.node_version.clone(),
    };
    (StatusCode::OK, Json(status))
}

// POST /dag/put – Store a DAG block. (Body: block JSON)
async fn dag_put_handler(
    State(state): State<AppState>,
    Json(block): Json<DagBlock>,
) -> impl IntoResponse {
    match state.dag_storage.lock().unwrap().put(&block) {
        Ok(_) => (StatusCode::CREATED, Json(block.cid)).into_response(),
        Err(e) => map_rust_error_to_json_response(e, StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

// POST /dag/get – Retrieve a DAG block. (Body: CID JSON)
async fn dag_get_handler(
    State(state): State<AppState>,
    Json(cid): Json<Cid>,
) -> impl IntoResponse {
    match state.dag_storage.lock().unwrap().get(&cid) {
        Ok(Some(block)) => (StatusCode::OK, Json(block)).into_response(),
        Ok(None) => map_rust_error_to_json_response("Block not found", StatusCode::NOT_FOUND).into_response(),
        Err(e) => map_rust_error_to_json_response(e, StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

// POST /governance/submit – Submit a proposal. (Body: SubmitProposalRequest JSON)
async fn gov_submit_handler(
    State(state): State<AppState>,
    Json(request): Json<ApiSubmitProposalRequest>,
) -> impl IntoResponse {
    let proposer_did: Did = match request.proposer_did.parse() {
        Ok(did) => did,
        Err(e) => return map_rust_error_to_json_response(e, StatusCode::BAD_REQUEST).into_response(),
    };

    let proposal_type: icn_governance::ProposalType = match request.proposal {
        icn_api::governance_trait::ProposalInputType::SystemParameterChange { param, value } => {
            icn_governance::ProposalType::SystemParameterChange(param, value)
        }
        icn_api::governance_trait::ProposalInputType::MemberAdmission { did } => {
            match icn_common::Did::from_str(&did) { // Assuming Did is from icn_common and FromStr
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

    match state.governance_module.lock().unwrap().submit_proposal(
        proposer_did,
        proposal_type,
        request.description,
        request.duration_secs,
    ) {
        Ok(proposal_id) => (StatusCode::CREATED, Json(proposal_id)).into_response(),
        Err(e) => map_rust_error_to_json_response(e, StatusCode::BAD_REQUEST).into_response(),
    }
}

// POST /governance/vote – Cast a vote. (Body: CastVoteRequest JSON)
async fn gov_vote_handler(
    State(state): State<AppState>,
    Json(request): Json<ApiCastVoteRequest>,
) -> impl IntoResponse {
    let voter_did: Did = match request.voter_did.parse() {
        Ok(did) => did,
        Err(e) => return map_rust_error_to_json_response(e, StatusCode::BAD_REQUEST).into_response(),
    };

    let proposal_id = ProposalId(request.proposal_id.clone());

    let vote_option = match request.vote_option.to_lowercase().as_str() {
        "yes" => VoteOption::Yes,
        "no" => VoteOption::No,
        "abstain" => VoteOption::Abstain,
        other => return map_rust_error_to_json_response(format!("Invalid vote option: {}", other), StatusCode::BAD_REQUEST).into_response(),
    };

    match state.governance_module.lock().unwrap().cast_vote(
        voter_did,
        &proposal_id,
        vote_option,
    ) {
        Ok(()) => (StatusCode::OK, Json(serde_json::json!({ "message": "Vote cast successfully"}))) .into_response(),
        Err(e) => map_rust_error_to_json_response(e, StatusCode::BAD_REQUEST).into_response(),
    }
}

// GET /governance/proposals – List proposals.
async fn gov_list_proposals_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.governance_module.lock().unwrap().list_proposals() {
        Ok(proposals) => (StatusCode::OK, Json(proposals)).into_response(),
        Err(e) => map_rust_error_to_json_response(e, StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

// GET /governance/proposal/:proposal_id – Get a specific proposal.
async fn gov_get_proposal_handler(
    State(state): State<AppState>,
    AxumPath(proposal_id_str): AxumPath<String>,
) -> impl IntoResponse {
    let proposal_id: ProposalId = match ProposalId::from_str(&proposal_id_str) {
        Ok(id) => id,
        Err(e) => return map_rust_error_to_json_response(
            format!("Invalid proposal ID format: {}. Error: {}", proposal_id_str, e),
            StatusCode::BAD_REQUEST
        ).into_response(),
    };

    match state.governance_module.lock().unwrap().get_proposal(&proposal_id) {
        Ok(Some(proposal)) => (StatusCode::OK, Json(proposal)).into_response(), // Proposal is already owned
        Ok(None) => map_rust_error_to_json_response(
            format!("Proposal not found for ID: {}", proposal_id_str),
            StatusCode::NOT_FOUND,
        ).into_response(),
        Err(e) => map_rust_error_to_json_response(e, StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

// --- Test module (can be expanded later) ---
#[cfg(test)]
mod tests {
    // Basic compile test
    #[test]
    fn it_compiles() {
        assert_eq!(1 + 1, 2);
    }

    // TODO: Add HTTP endpoint tests using a test client like `reqwest` or `hyper`.
    // These tests would involve:
    // 1. Setting up a test AppState (e.g., with InMemoryDagStore).
    // 2. Creating a test server or directly calling handlers with mocked state/requests.
    // 3. Asserting status codes and JSON response bodies.
} 