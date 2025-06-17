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
use icn_api::{query_data, submit_transaction};
use icn_common::DagBlock as CoreDagBlock;
use icn_common::{
    parse_cid_from_string, Cid, Did, NodeInfo, NodeStatus, Transaction, ICN_CORE_VERSION,
};
#[cfg(feature = "persist-sqlite")]
use icn_dag::sqlite_store::SqliteDagStore;
use icn_dag::{self, FileDagStore, InMemoryDagStore};
use icn_identity::{
    did_key_from_verifying_key, generate_ed25519_keypair,
    ExecutionReceipt as IdentityExecutionReceipt, SignatureBytes,
};
use icn_mesh::ActualMeshJob;
use icn_network::NetworkService;
use icn_runtime::context::{
    RuntimeContext, StubDagStore as RuntimeStubDagStore, StubMeshNetworkService,
    StubSigner as RuntimeStubSigner,
};
use icn_runtime::{host_anchor_receipt, host_submit_mesh_job, ReputationUpdater};

use axum::{
    extract::{Path as AxumPath, State},
    http::StatusCode,
    middleware::{self, Next},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use bs58;
use clap::{ArgMatches, CommandFactory, FromArgMatches, Parser};
#[cfg(feature = "enable-libp2p")]
use log::warn;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex as AsyncMutex, Mutex as TokioMutex};

use crate::config::{NodeConfig, StorageBackendType};

#[cfg(feature = "enable-libp2p")]
use libp2p::{Multiaddr, PeerId as Libp2pPeerId};

// --- CLI Arguments ---

#[derive(Parser, Debug)]
#[clap(author, version = ICN_CORE_VERSION, about = "ICN Node HTTP Server", long_about = None)]
pub struct Cli {
    /// Optional path to a configuration file (TOML or YAML)
    #[clap(long, value_name = "PATH")]
    pub config: Option<PathBuf>,

    #[clap(long, value_enum)]
    pub storage_backend: Option<StorageBackendType>,

    #[clap(long)]
    pub storage_path: Option<PathBuf>,

    #[clap(long)]
    pub mana_ledger_path: Option<PathBuf>,

    #[clap(long)]
    pub governance_db_path: Option<PathBuf>,

    #[clap(long)]
    pub http_listen_addr: Option<String>,

    #[clap(
        long,
        help = "Optional fixed DID for the node (e.g., did:key:zExample...)"
    )]
    pub node_did: Option<String>,

    #[clap(
        long,
        help = "Optional fixed Ed25519 private key (bs58 encoded string) for the node DID. If not provided and node_did is, it implies did:key or resolvable DID. If neither, a new key is generated."
    )]
    pub node_private_key_bs58: Option<String>,

    /// Path to store or load the node DID string.
    #[clap(long)]
    pub node_did_path: Option<PathBuf>,

    /// Path to store or load the node private key (bs58 encoded).
    #[clap(long)]
    pub node_private_key_path: Option<PathBuf>,

    #[clap(
        long,
        help = "Human-readable name for this node (for logging and identification)"
    )]
    pub node_name: Option<String>,

    #[clap(long = "listen-address", alias = "p2p-listen-addr")]
    pub listen_address: Option<String>,

    #[clap(
        long,
        help = "Bootstrap peer multiaddrs for P2P discovery (format: /ip4/1.2.3.4/tcp/port/p2p/PeerID)",
        value_delimiter = ','
    )]
    pub bootstrap_peers: Option<Vec<String>>,

    #[clap(long, action)]
    pub enable_p2p: bool,

    #[clap(long)]
    pub api_key: Option<String>,

    #[clap(long)]
    pub open_rate_limit: Option<u64>,
}

/// Load or generate the node identity based on the provided configuration.
pub fn load_or_generate_identity(
    config: &mut NodeConfig,
) -> (icn_identity::SigningKey, icn_identity::VerifyingKey, String) {
    if let (Some(did_str), Some(sk_bs58)) = (
        config.node_did.clone(),
        config.node_private_key_bs58.clone(),
    ) {
        let sk_bytes = bs58::decode(sk_bs58)
            .into_vec()
            .expect("Invalid base58 private key");
        let sk_array: [u8; 32] = sk_bytes.try_into().expect("Invalid private key length");
        let sk = icn_identity::SigningKey::from_bytes(&sk_array);
        let pk = sk.verifying_key();
        (sk, pk, did_str)
    } else if config.node_did_path.exists() && config.node_private_key_path.exists() {
        let did_str = fs::read_to_string(&config.node_did_path)
            .expect("Failed to read DID file")
            .trim()
            .to_string();
        let sk_bs58 = fs::read_to_string(&config.node_private_key_path)
            .expect("Failed to read key file")
            .trim()
            .to_string();
        let sk_bytes = bs58::decode(sk_bs58.clone())
            .into_vec()
            .expect("Invalid base58 private key");
        let sk_array: [u8; 32] = sk_bytes.try_into().expect("Invalid private key length");
        let sk = icn_identity::SigningKey::from_bytes(&sk_array);
        let pk = sk.verifying_key();
        config.node_did = Some(did_str.clone());
        config.node_private_key_bs58 = Some(sk_bs58);
        (sk, pk, did_str)
    } else {
        let (sk, pk) = generate_ed25519_keypair();
        let did_str = did_key_from_verifying_key(&pk);
        let sk_bs58 = bs58::encode(sk.to_bytes()).into_string();
        if let Err(e) = fs::write(&config.node_did_path, &did_str) {
            error!("Failed to write DID file: {}", e);
        }
        if let Err(e) = fs::write(&config.node_private_key_path, &sk_bs58) {
            error!("Failed to write key file: {}", e);
        }
        config.node_did = Some(did_str.clone());
        config.node_private_key_bs58 = Some(sk_bs58);
        (sk, pk, did_str)
    }
}

// --- Supporting Types ---

#[derive(Deserialize)]
struct DagBlockPayload {
    data: Vec<u8>,
}

// --- Application State ---
#[derive(Clone)]
struct AppState {
    runtime_context: Arc<RuntimeContext>,
    node_name: String,
    node_version: String,
    api_key: Option<String>,
    rate_limiter: Option<Arc<AsyncMutex<RateLimitData>>>,
}

struct RateLimitData {
    last: Instant,
    count: u64,
    limit: u64,
}

async fn require_api_key(
    State(state): State<AppState>,
    req: axum::http::Request<axum::body::Body>,
    next: Next,
) -> impl IntoResponse {
    if let Some(ref expected) = state.api_key {
        match req.headers().get("x-api-key").and_then(|v| v.to_str().ok()) {
            Some(provided) if provided == expected => next.run(req).await,
            _ => (
                StatusCode::UNAUTHORIZED,
                Json(JsonErrorResponse {
                    error: "unauthorized".to_string(),
                }),
            )
                .into_response(),
        }
    } else {
        next.run(req).await
    }
}

async fn rate_limit_middleware(
    State(state): State<AppState>,
    req: axum::http::Request<axum::body::Body>,
    next: Next,
) -> impl IntoResponse {
    if let Some(ref limiter) = state.rate_limiter {
        let mut data = limiter.lock().await;
        let now = Instant::now();
        if now.duration_since(data.last) > Duration::from_secs(60) {
            data.last = now;
            data.count = 0;
        }
        if data.count >= data.limit {
            return (
                StatusCode::TOO_MANY_REQUESTS,
                Json(JsonErrorResponse {
                    error: "rate limit exceeded".to_string(),
                }),
            )
                .into_response();
        }
        data.count += 1;
    }
    next.run(req).await
}

// --- Public App Constructor (for tests or embedding) ---
pub async fn app_router() -> Router {
    app_router_with_options(None, None, None, None).await.0
}

/// Construct a router for tests or embedding with optional API key and rate limit.
pub async fn app_router_with_options(
    api_key: Option<String>,
    rate_limit: Option<u64>,
    mana_ledger_path: Option<PathBuf>,
    governance_db_path: Option<PathBuf>,
) -> (Router, Arc<RuntimeContext>) {
    // Generate a new identity for this test/embedded instance
    let (sk, pk) = generate_ed25519_keypair();
    let node_did_string = did_key_from_verifying_key(&pk);
    let node_did = Did::from_str(&node_did_string).expect("Failed to create test node DID");
    info!("Test/Embedded Node DID: {}", node_did);

    let signer = Arc::new(RuntimeStubSigner::new_with_keys(sk, pk));
    let dag_store_for_rt = Arc::new(TokioMutex::new(RuntimeStubDagStore::new()));
    let mesh_network_service = Arc::new(StubMeshNetworkService::new());
    // GovernanceModule is initialized inside RuntimeContext::new

    let rt_ctx = RuntimeContext::new_with_ledger_path(
        node_did.clone(),
        mesh_network_service,
        signer,
        Arc::new(icn_identity::KeyDidResolver),
        dag_store_for_rt,
        mana_ledger_path.unwrap_or_else(|| PathBuf::from("./mana_ledger.sled")),
    );

    #[cfg(feature = "persist-sled")]
    {
        let gov_path = governance_db_path.unwrap_or_else(|| PathBuf::from("./governance_db"));
        let gov_mod = icn_governance::GovernanceModule::new_sled(gov_path)
            .unwrap_or_else(|_| icn_governance::GovernanceModule::new());
        rt_ctx.governance_module = Arc::new(TokioMutex::new(gov_mod));
    }

    // Initialize the test node with some mana for testing
    rt_ctx
        .credit_mana(&node_did, 1000)
        .await
        .expect("Failed to initialize test node with mana");
    info!("✅ Test node initialized with 1000 mana");

    rt_ctx.clone().spawn_mesh_job_manager().await; // Start the job manager

    let rate_limiter = rate_limit.filter(|l| *l > 0).map(|limit| {
        Arc::new(AsyncMutex::new(RateLimitData {
            last: Instant::now(),
            count: 0,
            limit,
        }))
    });

    let app_state = AppState {
        runtime_context: rt_ctx.clone(),
        node_name: "ICN Test/Embedded Node".to_string(),
        node_version: ICN_CORE_VERSION.to_string(),
        api_key,
        rate_limiter: rate_limiter.clone(),
    };

    (
        Router::new()
            .route("/info", get(info_handler))
            .route("/status", get(status_handler))
            .route("/dag/put", post(dag_put_handler)) // These will use RT context's DAG store
            .route("/dag/get", post(dag_get_handler)) // These will use RT context's DAG store
            .route("/transaction/submit", post(tx_submit_handler))
            .route("/data/query", post(data_query_handler))
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
            .with_state(app_state.clone())
            .layer(middleware::from_fn_with_state(
                app_state.clone(),
                require_api_key,
            ))
            .layer(middleware::from_fn_with_state(
                app_state.clone(),
                rate_limit_middleware,
            )),
        rt_ctx,
    )
}

// --- Main Application Logic ---
#[tokio::main]
async fn main() {
    env_logger::init(); // Initialize logger
    let cmd = Cli::command();
    let matches = cmd.get_matches();
    let cli = Cli::from_arg_matches(&matches).expect("CLI parsing failed");

    let config_path = cli.config.clone();
    let mut config = if let Some(path) = config_path {
        match NodeConfig::from_file(&path) {
            Ok(cfg) => cfg,
            Err(e) => {
                error!("Failed to load config file {}: {}", path.display(), e);
                NodeConfig::default()
            }
        }
    } else {
        NodeConfig::default()
    };
    config.apply_cli_overrides(&cli, &matches);

    // --- Initialize Node Identity ---
    let (node_sk, node_pk, node_did_string) = load_or_generate_identity(&mut config);

    let node_did = Did::from_str(&node_did_string).expect("Failed to create node DID");

    let node_name = config.node_name.clone();
    info!("Starting {} with DID: {}", node_name, node_did);

    // --- Create RuntimeContext with Networking ---
    let rt_ctx = if config.enable_p2p {
        #[cfg(feature = "enable-libp2p")]
        {
            info!(
                "Enabling libp2p networking with P2P listen address: {}",
                config.listen_address
            );

            // Parse bootstrap peers if provided
            let bootstrap_peers = if let Some(peer_strings) = &config.bootstrap_peers {
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

            let listen_addr = config
                .listen_address
                .parse::<Multiaddr>()
                .expect("Invalid p2p listen multiaddr");
            let listen_addrs = vec![listen_addr];

            match RuntimeContext::new_with_real_libp2p(
                &node_did_string,
                listen_addrs,
                bootstrap_peers,
                config.mana_ledger_path.clone(),
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
        let dag_store_for_rt: Arc<TokioMutex<dyn icn_dag::StorageService<CoreDagBlock> + Send>> =
            match config.storage_backend {
                StorageBackendType::Memory => Arc::new(TokioMutex::new(RuntimeStubDagStore::new())),
                StorageBackendType::File => {
                    let store = FileDagStore::new(config.storage_path.clone())
                        .expect("Failed to init file store");
                    Arc::new(TokioMutex::new(store))
                }
                StorageBackendType::Sqlite => {
                    #[cfg(feature = "persist-sqlite")]
                    {
                        let store = SqliteDagStore::new(config.storage_path.clone())
                            .expect("Failed to init sqlite store");
                        Arc::new(TokioMutex::new(store))
                    }
                    #[cfg(not(feature = "persist-sqlite"))]
                    {
                        error!(
                            "SQLite backend selected but the 'persist-sqlite' feature is not enabled"
                        );
                        std::process::exit(1);
                    }
                }
            };
        let mesh_network_service = Arc::new(StubMeshNetworkService::new());

        RuntimeContext::new_with_ledger_path(
            node_did.clone(),
            mesh_network_service,
            signer,
            Arc::new(icn_identity::KeyDidResolver),
            dag_store_for_rt,
            config.mana_ledger_path.clone(),
        )
    };

    #[cfg(feature = "persist-sled")]
    {
        let gov_mod = icn_governance::GovernanceModule::new_sled(config.governance_db_path.clone())
            .unwrap_or_else(|_| icn_governance::GovernanceModule::new());
        rt_ctx.governance_module = Arc::new(TokioMutex::new(gov_mod));
    }

    // Start the job manager
    rt_ctx.clone().spawn_mesh_job_manager().await;
    info!("ICN RuntimeContext initialized and JobManager spawned.");

    // --- Create AppState for Axum ---
    let rate_limiter = if config.api_key.is_none() && config.open_rate_limit > 0 {
        Some(Arc::new(AsyncMutex::new(RateLimitData {
            last: Instant::now(),
            count: 0,
            limit: config.open_rate_limit,
        })))
    } else {
        None
    };

    let app_state = AppState {
        runtime_context: rt_ctx.clone(),
        node_name: node_name.clone(),
        node_version: ICN_CORE_VERSION.to_string(),
        api_key: config.api_key.clone(),
        rate_limiter: rate_limiter.clone(),
    };

    // --- Define HTTP Routes ---
    let router = Router::new()
        .route("/info", get(info_handler))
        .route("/status", get(status_handler))
        .route("/dag/put", post(dag_put_handler))
        .route("/dag/get", post(dag_get_handler))
        .route("/transaction/submit", post(tx_submit_handler))
        .route("/data/query", post(data_query_handler))
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
        .with_state(app_state.clone())
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            require_api_key,
        ))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            rate_limit_middleware,
        ));

    let addr: SocketAddr = config
        .http_listen_addr
        .parse()
        .expect("Invalid HTTP listen address");
    info!("🌐 {} HTTP server listening on {}", node_name, addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, router.into_make_service())
        .await
        .unwrap();

    if config.enable_p2p {
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

// GET /info – Node version, name, status message pulled from RuntimeContext.
async fn info_handler(State(state): State<AppState>) -> impl IntoResponse {
    let mana_balance = state
        .runtime_context
        .mana_ledger
        .get_balance(&state.runtime_context.current_identity);
    let status_message = format!(
        "Node DID: {}, Mana: {}",
        state.runtime_context.current_identity, mana_balance
    );

    let info = NodeInfo {
        name: state.node_name.clone(),
        version: state.node_version.clone(),
        status_message,
    };
    (StatusCode::OK, Json(info))
}

// GET /status – Node status derived from RuntimeContext.
async fn status_handler(State(state): State<AppState>) -> impl IntoResponse {
    #[cfg(feature = "enable-libp2p")]
    let peer_count = match state.runtime_context.get_libp2p_service() {
        Ok(service) => match service.get_network_stats().await {
            Ok(stats) => stats.peer_count as u32,
            Err(_) => 0,
        },
        Err(_) => 0,
    };
    #[cfg(not(feature = "enable-libp2p"))]
    let peer_count = 0u32;

    let current_block_height = 0u64; // Placeholder until DAG exposes height

    let status = NodeStatus {
        is_online: true,
        peer_count,
        current_block_height,
        version: state.node_version.clone(),
    };
    (StatusCode::OK, Json(status))
}

// POST /dag/put – Store a DAG block. (Body: block JSON)
async fn dag_put_handler(
    State(state): State<AppState>,
    Json(block): Json<DagBlockPayload>,
) -> impl IntoResponse {
    // Use RuntimeContext's dag_store now
    let dag_block = CoreDagBlock {
        cid: Cid::new_v1_dummy(0x71, 0x12, &block.data),
        data: block.data,
        links: vec![],
    };
    let mut store = state.runtime_context.dag_store.lock().await;
    match store.put(&dag_block) {
        Ok(()) => (StatusCode::CREATED, Json(dag_block.cid)).into_response(),
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
    let cid_to_get = match parse_cid_from_string(&cid_request.cid) {
        Ok(cid) => cid,
        Err(e) => {
            return map_rust_error_to_json_response(
                format!("Invalid CID: {}", e),
                StatusCode::BAD_REQUEST,
            )
            .into_response();
        }
    };
    let store = state.runtime_context.dag_store.lock().await;
    match store.get(&cid_to_get) {
        Ok(Some(block)) => (StatusCode::OK, Json(block.data)).into_response(),
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

// POST /transaction/submit – Submit a transaction
async fn tx_submit_handler(
    State(_state): State<AppState>,
    Json(tx): Json<Transaction>,
) -> impl IntoResponse {
    let tx_json = match serde_json::to_string(&tx) {
        Ok(j) => j,
        Err(e) => {
            return map_rust_error_to_json_response(
                format!("Failed to serialize transaction: {}", e),
                StatusCode::BAD_REQUEST,
            )
            .into_response();
        }
    };

    match submit_transaction(tx_json) {
        Ok(id) => (
            StatusCode::ACCEPTED,
            Json(serde_json::json!({ "tx_id": id })),
        )
            .into_response(),
        Err(e) => map_rust_error_to_json_response(
            format!("Transaction submit error: {}", e),
            StatusCode::BAD_REQUEST,
        )
        .into_response(),
    }
}

// POST /data/query – Retrieve a DAG block via API query_data
async fn data_query_handler(
    State(state): State<AppState>,
    Json(req): Json<CidRequest>,
) -> impl IntoResponse {
    let cid_json = match serde_json::to_string(&req.cid) {
        Ok(j) => j,
        Err(e) => {
            return map_rust_error_to_json_response(
                format!("Failed to serialize CID: {}", e),
                StatusCode::BAD_REQUEST,
            )
            .into_response();
        }
    };

    match query_data(state.runtime_context.dag_store.clone(), cid_json).await {
        Ok(Some(block)) => (StatusCode::OK, Json(block)).into_response(),
        Ok(None) => map_rust_error_to_json_response("Block not found", StatusCode::NOT_FOUND)
            .into_response(),
        Err(e) => {
            map_rust_error_to_json_response(format!("Query error: {}", e), StatusCode::BAD_REQUEST)
                .into_response()
        }
    }
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

    let manifest_cid = match parse_cid_from_string(&request.manifest_cid) {
        Ok(cid) => cid,
        Err(e) => {
            return map_rust_error_to_json_response(
                format!("Invalid manifest CID: {}", e),
                StatusCode::BAD_REQUEST,
            )
            .into_response()
        }
    };

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

    // Build minimal JSON; runtime fills in missing fields
    let minimal_job_json = serde_json::json!({
        "manifest_cid": manifest_cid,
        "spec": job_spec,
        "cost_mana": request.cost_mana,
    });

    let job_json = match serde_json::to_string(&minimal_job_json) {
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
    pub success: bool,
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
        success: request.success,
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

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let info: NodeInfo = serde_json::from_slice(&body).unwrap();

        assert_eq!(info.name, "ICN Test/Embedded Node");
        assert_eq!(info.version, ICN_CORE_VERSION);
        assert!(info.status_message.contains("Mana: 1000"));
        assert!(info.status_message.contains("Node DID:"));
    }

    #[tokio::test]
    async fn mesh_submit_job_endpoint_basic() {
        let app = test_app().await;

        let job_req = SubmitJobRequest {
            manifest_cid: Cid::new_v1_dummy(0x55, 0x14, b"test_manifest").to_string(),
            spec_json: serde_json::to_value(&icn_mesh::JobSpec {
                kind: icn_mesh::JobKind::Echo { payload: "hello".into() },
                ..Default::default()
            })
            .unwrap(),
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
            spec_json: serde_json::to_value(&icn_mesh::JobSpec {
                kind: icn_mesh::JobKind::Echo { payload: "HTTP pipeline test".into() },
                ..Default::default()
            })
            .unwrap(),
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
            spec_json: serde_json::to_value(&icn_mesh::JobSpec {
                kind: icn_mesh::JobKind::Echo { payload: "simple test".into() },
                ..Default::default()
            })
            .unwrap(),
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

    #[tokio::test]
    async fn wasm_contract_execution_via_http() {
        use icn_ccl::compile_ccl_source_to_wasm;
        use icn_identity::{did_key_from_verifying_key, generate_ed25519_keypair};
        use icn_runtime::executor::WasmExecutor;

        let (app, ctx) = app_router_with_options(None, None, None).await;

        // Compile a tiny CCL contract
        let (wasm, _) =
            compile_ccl_source_to_wasm("fn run() -> Integer { return 7; }").expect("compile ccl");

        // Store WASM in DAG via HTTP
        let put_req = Request::builder()
            .method("POST")
            .uri("/dag/put")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_vec(&serde_json::json!({ "data": wasm })).unwrap(),
            ))
            .unwrap();
        let put_resp = app.clone().oneshot(put_req).await.unwrap();
        assert_eq!(put_resp.status(), StatusCode::CREATED);
        let cid_bytes = axum::body::to_bytes(put_resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let wasm_cid: Cid = serde_json::from_slice(&cid_bytes).unwrap();

        // Submit job referencing the WASM CID
        let job_req = SubmitJobRequest {
            manifest_cid: wasm_cid.to_string(),
            spec_json: serde_json::to_value(&icn_mesh::JobSpec::default()).unwrap(),
            cost_mana: 0,
        };
        let job_req_json = serde_json::to_string(&job_req).unwrap();
        let submit_resp = app
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
        assert_eq!(submit_resp.status(), StatusCode::ACCEPTED);
        let body = axum::body::to_bytes(submit_resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let submit_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let job_id: Cid = serde_json::from_str(submit_json["job_id"].as_str().unwrap()).unwrap();

        // Execute the job using WasmExecutor and anchor the receipt
        let (sk, vk) = generate_ed25519_keypair();
        let exec_did = did_key_from_verifying_key(&vk);
        let exec_did = Did::from_str(&exec_did).unwrap();
        let executor = WasmExecutor::new(ctx.clone(), exec_did.clone(), sk);
        let job = ActualMeshJob {
            id: job_id.clone(),
            manifest_cid: wasm_cid.clone(),
            spec: JobSpec::default(),
            creator_did: ctx.current_identity.clone(),
            cost_mana: 0,
            max_execution_wait_ms: None,
            signature: SignatureBytes(vec![]),
        };
        let receipt_cid = executor
            .execute_and_anchor_job(&job)
            .await
            .expect("exec and anchor");

        // Fetch the anchored receipt via HTTP
        let get_req = Request::builder()
            .method("POST")
            .uri("/dag/get")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_vec(&serde_json::json!({
                    "cid": receipt_cid.to_string()
                }))
                .unwrap(),
            ))
            .unwrap();
        let get_resp = app.clone().oneshot(get_req).await.unwrap();
        assert_eq!(get_resp.status(), StatusCode::OK);
        let receipt_bytes = axum::body::to_bytes(get_resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let receipt: icn_identity::ExecutionReceipt =
            serde_json::from_slice(&receipt_bytes).unwrap();
        assert_eq!(receipt.job_id, job_id);
        assert_eq!(receipt.executor_did, exec_did);
    }
}
