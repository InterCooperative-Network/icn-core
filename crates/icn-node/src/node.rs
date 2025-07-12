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

use crate::parameter_store::ParameterStore;
use icn_api::governance_trait::{
    CastVoteRequest as ApiCastVoteRequest, DelegateRequest as ApiDelegateRequest,
    RevokeDelegationRequest as ApiRevokeDelegationRequest,
    SubmitProposalRequest as ApiSubmitProposalRequest,
};
use icn_api::{get_dag_metadata, query_data, submit_transaction};
use icn_common::DagBlock as CoreDagBlock;
use icn_common::{
    parse_cid_from_string, Cid, CommonError, Did, NodeInfo, NodeStatus, Transaction,
    ICN_CORE_VERSION,
};
use icn_dag;
use icn_identity::{
    did_key_from_verifying_key, generate_ed25519_keypair,
    ExecutionReceipt as IdentityExecutionReceipt, SignatureBytes,
};
use icn_mesh::{ActualMeshJob, JobId, JobSpec};
#[allow(unused_imports)]
use icn_network::{NetworkService, PeerId};
use icn_protocol::{
    FederationJoinRequestMessage, GossipMessage, NodeCapabilities, ResourceRequirements,
};
use icn_protocol::{MessagePayload, ProtocolMessage};
use icn_runtime::context::{
    DefaultMeshNetworkService, Ed25519Signer, LocalMeshSubmitReceiptMessage,
    MeshNetworkServiceType, RuntimeContext, Signer, StubMeshNetworkService,
};
use icn_runtime::{host_anchor_receipt, host_submit_mesh_job, ReputationUpdater};
use prometheus_client::{encoding::text::encode, registry::Registry};

use axum::{
    extract::{Path as AxumPath, State},
    http::StatusCode,
    middleware::{self, Next},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use axum_server::tls_rustls::RustlsConfig;
use bs58;
use clap::{ArgMatches, CommandFactory, FromArgMatches, Parser};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::process;
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use subtle::ConstantTimeEq;
use tokio::sync::{Mutex as AsyncMutex, Mutex as TokioMutex};
use uuid::Uuid;

use crate::config::{NodeConfig, StorageBackendType};

#[cfg(feature = "enable-libp2p")]
use icn_network::libp2p_service::{Libp2pNetworkService, NetworkConfig};
#[cfg(feature = "enable-libp2p")]
use libp2p::Multiaddr;

static NODE_START_TIME: AtomicU64 = AtomicU64::new(0);

// Initialize node start time (call this when the node starts)
fn init_node_start_time() {
    let start_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    NODE_START_TIME.store(start_time, Ordering::Relaxed);
}

#[derive(serde::Serialize)]
struct HealthStatus {
    status: String,
    timestamp: u64,
    uptime_seconds: u64,
    checks: HealthChecks,
}

#[derive(serde::Serialize)]
struct HealthChecks {
    runtime: String,
    dag_store: String,
    network: String,
    mana_ledger: String,
}

#[derive(serde::Serialize)]
struct ReadinessStatus {
    ready: bool,
    timestamp: u64,
    checks: ReadinessChecks,
}

#[derive(serde::Serialize)]
struct ReadinessChecks {
    can_serve_requests: bool,
    mana_ledger_available: bool,
    dag_store_available: bool,
    network_initialized: bool,
}

#[derive(serde::Serialize)]
struct SystemMetrics {
    uptime_seconds: u64,
    memory_usage_bytes: u64,
    process_id: u32,
    connected_peers: usize,
    pending_jobs: usize,
    mana_accounts: usize,
}

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

    #[clap(long, value_enum)]
    pub mana_ledger_backend: Option<icn_runtime::context::LedgerBackend>,

    #[clap(long)]
    pub mana_ledger_path: Option<PathBuf>,

    #[clap(long)]
    pub reputation_db_path: Option<PathBuf>,

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

    /// Encrypted private key file for the node identity.
    #[clap(long)]
    pub key_path: Option<PathBuf>,

    /// Environment variable name containing the passphrase for `key_path`.
    #[clap(long)]
    pub key_passphrase_env: Option<String>,

    /// Path to an HSM library for key retrieval.
    #[clap(long)]
    pub hsm_library: Option<PathBuf>,

    /// Identifier of the key within the HSM.
    #[clap(long)]
    pub hsm_key_id: Option<String>,

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

    #[clap(long, action)]
    pub enable_mdns: bool,

    #[clap(long, action)]
    pub test_mode: bool,

    #[clap(long)]
    pub api_key: Option<String>,

    #[clap(long)]
    pub open_rate_limit: Option<u64>,

    /// Bearer token required via the `Authorization` header
    #[clap(long)]
    pub auth_token: Option<String>,

    /// Path to a file containing the bearer token
    #[clap(long)]
    pub auth_token_path: Option<PathBuf>,

    #[clap(long)]
    pub tls_cert_path: Option<PathBuf>,

    #[clap(long)]
    pub tls_key_path: Option<PathBuf>,

    #[clap(long)]
    pub tls_min_version: Option<String>,

    #[clap(long)]
    pub key_rotation_days: Option<u64>,
}

/// Load or generate the node identity based on the provided configuration.
pub fn load_or_generate_identity(
    config: &mut NodeConfig,
) -> Result<(icn_runtime::context::Ed25519Signer, String), CommonError> {
    if let (Some(lib), Some(key_id)) = (&config.hsm_library, &config.hsm_key_id) {
        let hsm = icn_runtime::context::signers::ExampleHsm::with_key(lib, key_id.clone());
        let signer = icn_runtime::context::Ed25519Signer::from_hsm(&hsm)?;
        let did_str = signer.did().to_string();
        config.node_did = Some(did_str.clone());
        return Ok((signer, did_str));
    }
    if let Some(path) = &config.key_path {
        let env_name = config
            .key_passphrase_env
            .as_deref()
            .unwrap_or("ICN_KEY_PASSPHRASE");
        let passphrase = std::env::var(env_name).map_err(|_| {
            CommonError::ConfigError(format!("missing key passphrase env {env_name}"))
        })?;
        let signer =
            icn_runtime::context::Ed25519Signer::from_encrypted_file(path, passphrase.as_bytes())?;
        let did_str = signer.did().to_string();
        config.node_did = Some(did_str.clone());
        return Ok((signer, did_str));
    }
    if let (Some(did_str), Some(sk_bs58)) = (
        config.node_did.clone(),
        config.node_private_key_bs58.clone(),
    ) {
        let sk_bytes = bs58::decode(sk_bs58)
            .into_vec()
            .map_err(|_| CommonError::IdentityError("Invalid base58 private key".into()))?;
        let sk_array: [u8; 32] = sk_bytes
            .try_into()
            .map_err(|_| CommonError::IdentityError("Invalid private key length".into()))?;
        let sk = icn_identity::SigningKey::from_bytes(&sk_array);
        let pk = sk.verifying_key();
        Ok((
            icn_runtime::context::Ed25519Signer::new_with_keys(sk, pk),
            did_str,
        ))
    } else if config.node_did_path.exists() && config.node_private_key_path.exists() {
        let did_str = fs::read_to_string(&config.node_did_path)
            .map_err(|e| CommonError::IoError(format!("Failed to read DID file: {e}")))?
            .trim()
            .to_string();
        let sk_bs58 = fs::read_to_string(&config.node_private_key_path)
            .map_err(|e| CommonError::IoError(format!("Failed to read key file: {e}")))?
            .trim()
            .to_string();
        let sk_bytes = bs58::decode(sk_bs58.clone())
            .into_vec()
            .map_err(|_| CommonError::IdentityError("Invalid base58 private key".into()))?;
        let sk_array: [u8; 32] = sk_bytes
            .try_into()
            .map_err(|_| CommonError::IdentityError("Invalid private key length".into()))?;
        let sk = icn_identity::SigningKey::from_bytes(&sk_array);
        let pk = sk.verifying_key();
        config.node_did = Some(did_str.clone());
        config.node_private_key_bs58 = Some(sk_bs58);
        Ok((
            icn_runtime::context::Ed25519Signer::new_with_keys(sk, pk),
            did_str,
        ))
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
        Ok((
            icn_runtime::context::Ed25519Signer::new_with_keys(sk, pk),
            did_str,
        ))
    }
}

// --- Supporting Types ---

#[derive(Deserialize)]
struct DagBlockPayload {
    data: Vec<u8>,
}

#[derive(Deserialize)]
struct ContractSourcePayload {
    source: String,
}

#[derive(Deserialize)]
struct AddPeerPayload {
    peer: String,
}

#[derive(Deserialize)]
struct PeerPayload {
    peer: String,
}

#[derive(Serialize)]
struct FederationStatus {
    peer_count: usize,
    peers: Vec<String>,
}

#[derive(Deserialize)]
struct ProposalIdPayload {
    proposal_id: String,
}

// --- Application State ---
#[derive(Clone)]
struct AppState {
    runtime_context: Arc<RuntimeContext>,
    node_name: String,
    node_version: String,
    api_key: Option<String>,
    auth_token: Option<String>,
    rate_limiter: Option<Arc<AsyncMutex<RateLimitData>>>,
    peers: Arc<TokioMutex<Vec<String>>>,
    config: Arc<TokioMutex<NodeConfig>>,
    parameter_store: Option<Arc<TokioMutex<ParameterStore>>>,
}

struct RateLimitData {
    last: Instant,
    count: u64,
    limit: u64,
    failed_attempts: u64,
}

async fn require_api_key(
    State(state): State<AppState>,
    req: axum::http::Request<axum::body::Body>,
    next: Next,
) -> impl IntoResponse {
    if let Some(ref expected) = state.api_key {
        let provided = req.headers().get("x-api-key").and_then(|v| v.to_str().ok());
        let valid = provided
            .map(|p| ConstantTimeEq::ct_eq(p.as_bytes(), expected.as_bytes()).into())
            .unwrap_or(false);
        if !valid {
            warn!("Invalid API key attempt");
            info!(target: "audit", "auth_failed reason=invalid_api_key");
            if let Some(ref limiter) = state.rate_limiter {
                let mut data = limiter.lock().await;
                let now = Instant::now();
                if now.duration_since(data.last) > Duration::from_secs(60) {
                    data.last = now;
                    data.count = 0;
                    data.failed_attempts = 0;
                }
                data.count += 1;
                data.failed_attempts += 1;
            }
            return (
                StatusCode::UNAUTHORIZED,
                Json(JsonErrorResponse {
                    error: "missing or invalid api key".to_string(),
                }),
            )
                .into_response();
        }
        info!(target: "audit", "auth_success method=api_key");
    }

    if let Some(ref token) = state.auth_token {
        let provided = req
            .headers()
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "));
        let valid = provided
            .map(|p| ConstantTimeEq::ct_eq(p.as_bytes(), token.as_bytes()).into())
            .unwrap_or(false);
        if !valid {
            warn!("Invalid bearer token attempt");
            info!(target: "audit", "auth_failed reason=invalid_bearer_token");
            if let Some(ref limiter) = state.rate_limiter {
                let mut data = limiter.lock().await;
                let now = Instant::now();
                if now.duration_since(data.last) > Duration::from_secs(60) {
                    data.last = now;
                    data.count = 0;
                    data.failed_attempts = 0;
                }
                data.count += 1;
                data.failed_attempts += 1;
            }
            return (
                StatusCode::UNAUTHORIZED,
                Json(JsonErrorResponse {
                    error: "missing or invalid bearer token".to_string(),
                }),
            )
                .into_response();
        }
        info!(target: "audit", "auth_success method=bearer");
    }

    next.run(req).await
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
            data.failed_attempts = 0;
        }
        if data.count >= data.limit {
            info!(target: "audit", "rate_limit_exceeded" );
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

async fn correlation_id_middleware(
    mut req: axum::http::Request<axum::body::Body>,
    next: Next,
) -> impl IntoResponse {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let cid = req
        .headers()
        .get("x-correlation-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    req.extensions_mut().insert(cid.clone());
    info!(target: "request", "start cid={} {} {}", cid, method, path);
    let start = Instant::now();
    let mut res = next.run(req).await.into_response();
    res.headers_mut()
        .insert("x-correlation-id", cid.parse().unwrap());
    let latency = start.elapsed().as_millis();
    let status = res.status().as_u16();
    info!(target: "request", "end cid={} status={} latency_ms={}", cid, status, latency);
    res
}

/// Select the appropriate network service based on `NodeConfig`.
pub async fn build_network_service(
    config: &NodeConfig,
) -> Result<Arc<dyn NetworkService>, CommonError> {
    if config.test_mode {
        return Ok(Arc::new(icn_network::StubNetworkService::default()));
    }

    if config.enable_p2p {
        #[cfg(feature = "enable-libp2p")]
        {
            let net_cfg = config.libp2p_config()?;
            let service = icn_network::libp2p_service::Libp2pNetworkService::new(net_cfg)
                .await
                .map_err(|e| {
                    CommonError::NetworkError(format!("Failed to create libp2p service: {}", e))
                })?;
            return Ok(Arc::new(service));
        }
        #[cfg(not(feature = "enable-libp2p"))]
        {
            return Err(CommonError::ConfigError(
                "libp2p support not compiled".into(),
            ));
        }
    }

    Ok(Arc::new(icn_network::StubNetworkService::default()))
}

// --- Public App Constructor (for tests or embedding) ---
pub async fn app_router() -> Router {
    app_router_with_options(None, None, None, None, None, None, None, None, None, None)
        .await
        .0
}

/// Construct a router for tests or embedding with optional API key and rate limit.
#[allow(clippy::too_many_arguments)]
pub async fn app_router_with_options(
    api_key: Option<String>,
    auth_token: Option<String>,
    rate_limit: Option<u64>,
    mana_ledger_backend: Option<icn_runtime::context::LedgerBackend>,
    mana_ledger_path: Option<PathBuf>,
    storage_backend: Option<StorageBackendType>,
    storage_path: Option<PathBuf>,
    _governance_db_path: Option<PathBuf>,
    reputation_db_path: Option<PathBuf>,
    parameter_store_path: Option<PathBuf>,
) -> (Router, Arc<RuntimeContext>) {
    // Generate a new identity for this test/embedded instance
    let (sk, pk) = generate_ed25519_keypair();
    let node_did_string = did_key_from_verifying_key(&pk);
    let node_did = Did::from_str(&node_did_string).expect("Failed to create test node DID");
    info!("Test/Embedded Node DID: {}", node_did);

    let _signer = Arc::new(Ed25519Signer::new_with_keys(sk, pk)); // Not used with stubs
    let cfg = NodeConfig {
        storage_backend: storage_backend.unwrap_or(StorageBackendType::Memory),
        storage_path: storage_path
            .clone()
            .unwrap_or_else(|| PathBuf::from("./dag_store")),
        ..NodeConfig::default()
    };
    let parameter_store = parameter_store_path.map(|p| {
        Arc::new(TokioMutex::new(
            ParameterStore::load(p).expect("failed to load parameter store"),
        ))
    });
    let dag_store_for_rt = cfg
        .init_dag_store()
        .expect("Failed to init DAG store for test context");

    #[cfg(feature = "enable-libp2p")]
    let (mesh_network_service, service_dyn) = {
        let cfg = NetworkConfig::default();
        let service = Libp2pNetworkService::new(cfg)
            .await
            .expect("Failed to create libp2p service");
        let service_dyn: Arc<dyn NetworkService> = Arc::new(service);
        let default_service = DefaultMeshNetworkService::new(service_dyn.clone(), _signer.clone());
        (
            Arc::new(MeshNetworkServiceType::Default(default_service)),
            Some(service_dyn),
        )
    };
    #[cfg(not(feature = "enable-libp2p"))]
    let (mesh_network_service, service_dyn) = {
        let stub_service = StubMeshNetworkService::new();
        (
            Arc::new(MeshNetworkServiceType::Stub(stub_service)),
            None::<Arc<dyn icn_network::NetworkService>>,
        )
    };
    // GovernanceModule is initialized inside RuntimeContext::new

    // Create production-ready RuntimeContext with proper services
    let node_did_string = node_did.to_string();

    // Create mana ledger with initial balance
    let mana_ledger = icn_runtime::context::SimpleManaLedger::new(
        mana_ledger_path.unwrap_or_else(|| PathBuf::from("./tests/fixtures/mana_ledger.json")),
    );
    mana_ledger
        .set_balance(&node_did, 1000)
        .expect("Failed to set initial mana balance");

    // Create production RuntimeContext using the mesh network service we created above
    let mut rt_ctx = {
        #[cfg(feature = "enable-libp2p")]
        {
            // For production/development with real networking
            RuntimeContext::new_development(
                node_did.clone(),
                _signer.clone(),
                mana_ledger,
                service_dyn,
                Some(dag_store_for_rt.clone()),
            )
            .expect("Failed to create production RuntimeContext")
        }
        #[cfg(not(feature = "enable-libp2p"))]
        {
            // For testing without networking - use stub services
            RuntimeContext::new_testing(node_did.clone(), Some(1000))
                .expect("Failed to create testing RuntimeContext")
        }
    };

    #[cfg(feature = "persist-sled")]
    {
        let gov_path = _governance_db_path.unwrap_or_else(|| PathBuf::from("./governance_db"));
        let gov_mod = icn_governance::GovernanceModule::new_sled(gov_path)
            .unwrap_or_else(|_| icn_governance::GovernanceModule::new());
        if let Some(ctx) = Arc::get_mut(&mut rt_ctx) {
            ctx.governance_module = Arc::new(TokioMutex::new(gov_mod));
            if let Some(path) = &reputation_db_path {
                if let Ok(store) = icn_reputation::SledReputationStore::new(path.clone()) {
                    ctx.reputation_store = Arc::new(store);
                }
            }
        }
    }

    #[cfg(feature = "enable-libp2p")]
    info!("✅ Node initialized with 1000 mana (production services with libp2p)");
    #[cfg(not(feature = "enable-libp2p"))]
    info!("✅ Node initialized with 1000 mana (testing services without libp2p)");

    rt_ctx.clone().spawn_mesh_job_manager().await; // Start the job manager

    // Start the executor manager so this node can act as an executor
    rt_ctx.clone().spawn_mesh_executor_manager().await;

    info!("ICN RuntimeContext initialized and JobManager + ExecutorManager spawned.");

    let config = Arc::new(TokioMutex::new(cfg.clone()));

    let mut effective_limit = rate_limit;
    if effective_limit.is_none() {
        if let Some(ps) = &parameter_store {
            let ps_guard = ps.lock().await;
            effective_limit = Some(ps_guard.open_rate_limit());
        }
    }

    let rate_limiter = effective_limit.filter(|l| *l > 0).map(|limit| {
        Arc::new(AsyncMutex::new(RateLimitData {
            last: Instant::now(),
            count: 0,
            limit,
            failed_attempts: 0,
        }))
    });

    let app_state = AppState {
        runtime_context: rt_ctx.clone(),
        node_name: "ICN Test/Embedded Node".to_string(),
        node_version: ICN_CORE_VERSION.to_string(),
        api_key,
        auth_token,
        rate_limiter: rate_limiter.clone(),
        peers: Arc::new(TokioMutex::new(Vec::new())),
        config: config.clone(),
        parameter_store: parameter_store.clone(),
    };

    // Register governance callback for parameter changes
    {
        let gov_mod = rt_ctx.governance_module.clone();
        let rate_opt = rate_limiter.clone();
        let param_store_opt = parameter_store.clone();
        let handle = tokio::runtime::Handle::current();
        let mut gov = gov_mod.lock().await;
        gov.set_callback(move |proposal| {
            if let icn_governance::ProposalType::SystemParameterChange(param, value) =
                &proposal.proposal_type
            {
                if param == "open_rate_limit" {
                    if let Some(ref limiter) = rate_opt {
                        let new_lim: u64 = value
                            .parse::<u64>()
                            .map_err(|e| CommonError::InvalidInputError(e.to_string()))?;
                        handle.block_on(async {
                            let mut data = limiter.lock().await;
                            data.limit = new_lim;
                        });
                    }
                    if let Some(ref store) = param_store_opt {
                        let val = value.clone();
                        handle.block_on(async {
                            let mut ps = store.lock().await;
                            let _ = ps.set_parameter(param, &val);
                        });
                    }
                }
            }
            Ok(())
        });
    }

    (
        Router::new()
            .route("/info", get(info_handler))
            .route("/status", get(status_handler))
            .route("/health", get(health_handler))
            .route("/ready", get(readiness_handler))
            .route("/metrics", get(metrics_handler))
            .route("/network/local-peer-id", get(network_local_peer_id_handler))
            .route("/network/peers", get(network_peers_handler))
            .route("/network/connect", post(network_connect_handler))
            .route("/account/{did}/mana", get(account_mana_handler))
            .route("/keys", get(keys_handler))
            .route("/reputation/{did}", get(reputation_handler))
            .route("/dag/put", post(dag_put_handler)) // These will use RT context's DAG store
            .route("/dag/get", post(dag_get_handler)) // These will use RT context's DAG store
            .route("/dag/meta", post(dag_meta_handler))
            .route("/dag/root", get(dag_root_handler))
            .route("/dag/pin", post(dag_pin_handler))
            .route("/dag/unpin", post(dag_unpin_handler))
            .route("/dag/prune", post(dag_prune_handler))
            .route("/transaction/submit", post(tx_submit_handler))
            .route("/data/query", post(data_query_handler))
            .route("/governance/submit", post(gov_submit_handler)) // Uses RT context's Gov mod
            .route("/governance/vote", post(gov_vote_handler)) // Uses RT context's Gov mod
            .route("/governance/delegate", post(gov_delegate_handler))
            .route("/governance/revoke", post(gov_revoke_handler))
            .route("/governance/close", post(gov_close_handler))
            .route("/governance/execute", post(gov_execute_handler))
            .route("/governance/proposals", get(gov_list_proposals_handler)) // Uses RT context's Gov mod
            .route(
                "/governance/proposal/{proposal_id}",
                get(gov_get_proposal_handler),
            ) // Uses RT context's Gov mod
            .route("/mesh/submit", post(mesh_submit_job_handler)) // Job submission
            .route("/mesh/jobs", get(mesh_list_jobs_handler)) // List all jobs
            .route("/mesh/jobs/{job_id}", get(mesh_get_job_status_handler)) // Get specific job status
            .route("/mesh/receipt", post(mesh_submit_receipt_handler)) // Submit execution receipt
            .route("/mesh/stub/bid", post(mesh_stub_bid_handler)) // Stub: inject bid for testing
            .route("/mesh/stub/receipt", post(mesh_stub_receipt_handler)) // Stub: inject receipt for testing
            .route("/contracts", post(contracts_post_handler))
            .route("/federation/peers", get(federation_list_peers_handler))
            .route("/federation/peers", post(federation_add_peer_handler))
            .route("/federation/join", post(federation_join_handler))
            .route("/federation/leave", post(federation_leave_handler))
            .route("/federation/status", get(federation_status_handler))
            .with_state(app_state.clone())
            .layer(middleware::from_fn(correlation_id_middleware))
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

/// Build an Axum [`Router`] using an already initialized [`RuntimeContext`].
///
/// This is primarily intended for integration tests that construct custom
/// runtime contexts (e.g. with real libp2p networking) and need to expose the
/// standard HTTP API surface.
pub async fn app_router_from_context(
    ctx: Arc<RuntimeContext>,
    api_key: Option<String>,
    auth_token: Option<String>,
    rate_limit: Option<u64>,
) -> Router {
    let rate_limiter = rate_limit.filter(|l| *l > 0).map(|limit| {
        Arc::new(AsyncMutex::new(RateLimitData {
            last: Instant::now(),
            count: 0,
            limit,
            failed_attempts: 0,
        }))
    });

    let config = Arc::new(TokioMutex::new(NodeConfig::default()));

    let app_state = AppState {
        runtime_context: ctx.clone(),
        node_name: "ICN Test/Embedded Node".to_string(),
        node_version: ICN_CORE_VERSION.to_string(),
        api_key,
        auth_token,
        rate_limiter: rate_limiter.clone(),
        peers: Arc::new(TokioMutex::new(Vec::new())),
        config: config.clone(),
        parameter_store: None,
    };

    {
        let gov_mod = ctx.governance_module.clone();
        let rate_opt = rate_limiter.clone();
        let param_store_opt: Option<Arc<TokioMutex<ParameterStore>>> = None;
        let handle = tokio::runtime::Handle::current();
        let mut gov = gov_mod.lock().await;
        gov.set_callback(move |proposal| {
            if let icn_governance::ProposalType::SystemParameterChange(param, value) =
                &proposal.proposal_type
            {
                if param == "open_rate_limit" {
                    if let Some(ref limiter) = rate_opt {
                        let new_lim: u64 = value
                            .parse::<u64>()
                            .map_err(|e| CommonError::InvalidInputError(e.to_string()))?;
                        handle.block_on(async {
                            let mut data = limiter.lock().await;
                            data.limit = new_lim;
                        });
                    }
                    if let Some(ref store) = param_store_opt {
                        let val = value.clone();
                        handle.block_on(async {
                            let mut ps = store.lock().await;
                            let _ = ps.set_parameter(param, &val);
                        });
                    }
                }
            }
            Ok(())
        });
    }

    Router::new()
        .route("/info", get(info_handler))
        .route("/status", get(status_handler))
        .route("/health", get(health_handler))
        .route("/ready", get(readiness_handler))
        .route("/metrics", get(metrics_handler))
        .route("/network/local-peer-id", get(network_local_peer_id_handler))
        .route("/network/peers", get(network_peers_handler))
        .route("/network/connect", post(network_connect_handler))
        .route("/account/{did}/mana", get(account_mana_handler))
        .route("/keys", get(keys_handler))
        .route("/reputation/{did}", get(reputation_handler))
        .route("/dag/put", post(dag_put_handler))
        .route("/dag/get", post(dag_get_handler))
        .route("/dag/meta", post(dag_meta_handler))
        .route("/dag/root", get(dag_root_handler))
        .route("/dag/pin", post(dag_pin_handler))
        .route("/dag/unpin", post(dag_unpin_handler))
        .route("/dag/prune", post(dag_prune_handler))
        .route("/transaction/submit", post(tx_submit_handler))
        .route("/data/query", post(data_query_handler))
        .route("/governance/submit", post(gov_submit_handler))
        .route("/governance/vote", post(gov_vote_handler))
        .route("/governance/delegate", post(gov_delegate_handler))
        .route("/governance/revoke", post(gov_revoke_handler))
        .route("/governance/close", post(gov_close_handler))
        .route("/governance/execute", post(gov_execute_handler))
        .route("/governance/proposals", get(gov_list_proposals_handler))
        .route(
            "/governance/proposal/{proposal_id}",
            get(gov_get_proposal_handler),
        )
        .route("/mesh/submit", post(mesh_submit_job_handler))
        .route("/mesh/jobs", get(mesh_list_jobs_handler))
        .route("/mesh/jobs/{job_id}", get(mesh_get_job_status_handler))
        .route("/mesh/receipt", post(mesh_submit_receipt_handler))
        .route("/mesh/stub/bid", post(mesh_stub_bid_handler))
        .route("/mesh/stub/receipt", post(mesh_stub_receipt_handler))
        .route("/contracts", post(contracts_post_handler))
        .route("/federation/peers", get(federation_list_peers_handler))
        .route("/federation/peers", post(federation_add_peer_handler))
        .route("/federation/join", post(federation_join_handler))
        .route("/federation/leave", post(federation_leave_handler))
        .route("/federation/status", get(federation_status_handler))
        .route("/federation/join", post(federation_join_handler))
        .route("/federation/leave", post(federation_leave_handler))
        .route("/federation/status", get(federation_status_handler))
        .with_state(app_state.clone())
        .layer(middleware::from_fn(correlation_id_middleware))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            require_api_key,
        ))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            rate_limit_middleware,
        ))
}

// --- Main Application Logic ---
pub async fn run_node() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init(); // Initialize logger
    init_node_start_time(); // Initialize uptime tracking

    let cmd = Cli::command();
    let matches = cmd.get_matches();
    let cli = Cli::from_arg_matches(&matches).expect("CLI parsing failed");

    let config_path = cli.config.clone();
    let mut config = if let Some(path) = config_path.clone() {
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
    info!(target: "audit", "config_loaded path={:?}", config_path);
    let param_store_path = config_path.unwrap_or_else(|| PathBuf::from("node_config.toml"));
    let mut parameter_store = ParameterStore::load(param_store_path.clone()).unwrap_or_else(|e| {
        warn!(
            "Failed to load parameter store from {}: {}, using defaults",
            param_store_path.display(),
            e
        );
        // The load method should handle this case, but if not, we can't recover
        panic!("Failed to create parameter store")
    });
    // Start with persisted parameter values
    config.http.open_rate_limit = parameter_store.open_rate_limit();
    config.apply_env_overrides();
    config.apply_cli_overrides(&cli, &matches);
    if let Err(e) =
        parameter_store.set_parameter("open_rate_limit", &config.http.open_rate_limit.to_string())
    {
        warn!("Failed to update parameter store: {}", e);
    }
    let _ = parameter_store.save();
    if let Err(e) = config.prepare_paths() {
        error!("Failed to prepare config directories: {}", e);
    }

    let shared_config = Arc::new(TokioMutex::new(config.clone()));
    let parameter_store = Arc::new(TokioMutex::new(parameter_store));

    if config.http.auth_token.is_none() {
        if let Some(path) = &config.http.auth_token_path {
            match fs::read_to_string(path) {
                Ok(tok) => {
                    config.http.auth_token = Some(tok.trim().to_string());
                }
                Err(e) => {
                    error!("Failed to read auth token file {}: {}", path.display(), e);
                }
            }
        }
    }

    // --- Initialize Node Identity ---
    let (signer, node_did_string) = match load_or_generate_identity(&mut config) {
        Ok(ids) => ids,
        Err(e) => {
            error!("Failed to initialize node identity: {}", e);
            std::process::exit(1);
        }
    };
    info!(target: "audit", "node_identity did={}", node_did_string);

    let node_did = Did::from_str(&node_did_string).expect("Failed to create node DID");

    let node_name = config.node_name.clone();
    info!("Starting {} with DID: {}", node_name, node_did);

    // --- Create RuntimeContext with Networking ---
    let dag_store_for_rt = match config.init_dag_store() {
        Ok(store) => store,
        Err(e) => {
            error!("Failed to initialize DAG store: {}", e);
            std::process::exit(1);
        }
    };

    let mana_ledger = icn_runtime::context::SimpleManaLedger::new(config.mana_ledger_path.clone());
    let signer = Arc::new(signer);

    let network_service = match build_network_service(&config).await {
        Ok(svc) => svc,
        Err(e) => {
            error!("Network service initialization failed: {}", e);
            std::process::exit(1);
        }
    };

    let rt_ctx = if config.test_mode {
        RuntimeContext::new_testing(node_did.clone(), Some(1000))
            .expect("Failed to create RuntimeContext for testing")
    } else {
        RuntimeContext::new_production(
            node_did.clone(),
            network_service,
            signer,
            Arc::new(icn_identity::KeyDidResolver),
            dag_store_for_rt,
            mana_ledger,
            Arc::new(icn_reputation::InMemoryReputationStore::new()),
            None,
        )
        .expect("Failed to create RuntimeContext for production")
    };

    #[cfg(feature = "persist-sled")]
    {
        let gov_mod = icn_governance::GovernanceModule::new_sled(config.governance_db_path.clone())
            .unwrap_or_else(|_| icn_governance::GovernanceModule::new());
        if let Some(ctx) = Arc::get_mut(&mut rt_ctx) {
            ctx.governance_module = Arc::new(TokioMutex::new(gov_mod));
            if let Ok(store) =
                icn_reputation::SledReputationStore::new(config.reputation_db_path.clone())
            {
                ctx.reputation_store = Arc::new(store);
            }
        }
    }

    // Start the job manager
    rt_ctx.clone().spawn_mesh_job_manager().await;

    // Start the executor manager so this node can act as an executor
    rt_ctx.clone().spawn_mesh_executor_manager().await;

    info!("ICN RuntimeContext initialized and JobManager + ExecutorManager spawned.");

    // Initialize the node with some mana for job submission
    match rt_ctx.credit_mana(&node_did, 1000).await {
        Ok(()) => {
            info!("✅ Node initialized with 1000 mana");
        }
        Err(e) => {
            error!("❌ Failed to initialize node with mana: {:?}", e);
            error!("Node DID: {:?}", node_did);
            error!("Mana ledger type: {:?}", rt_ctx.mana_ledger);
            // Try to get the current balance to see if account exists
            match rt_ctx.get_mana(&node_did).await {
                Ok(balance) => {
                    error!("Current mana balance for node: {}", balance);
                }
                Err(balance_err) => {
                    error!("Failed to get mana balance: {:?}", balance_err);
                }
            }
            return Err(Box::new(e));
        }
    }

    // --- Create AppState for Axum ---
    let rate_limiter = if config.http.api_key.is_none()
        && config.http.auth_token.is_none()
        && config.http.open_rate_limit > 0
    {
        Some(Arc::new(AsyncMutex::new(RateLimitData {
            last: Instant::now(),
            count: 0,
            limit: config.http.open_rate_limit,
            failed_attempts: 0,
        })))
    } else {
        None
    };

    let app_state = AppState {
        runtime_context: rt_ctx.clone(),
        node_name: node_name.clone(),
        node_version: ICN_CORE_VERSION.to_string(),
        api_key: config.http.api_key.clone(),
        auth_token: config.http.auth_token.clone(),
        rate_limiter: rate_limiter.clone(),
        peers: Arc::new(TokioMutex::new(Vec::new())),
        config: shared_config.clone(),
        parameter_store: Some(parameter_store.clone()),
    };

    {
        let gov_mod = rt_ctx.governance_module.clone();
        let rate_opt = rate_limiter.clone();
        let handle = tokio::runtime::Handle::current();
        let mut gov = gov_mod.lock().await;
        gov.set_callback(move |proposal| {
            if let icn_governance::ProposalType::SystemParameterChange(param, value) =
                &proposal.proposal_type
            {
                if param == "open_rate_limit" {
                    if let Some(ref limiter) = rate_opt {
                        let new_lim: u64 = value
                            .parse::<u64>()
                            .map_err(|e| CommonError::InvalidInputError(e.to_string()))?;
                        handle.block_on(async {
                            let mut data = limiter.lock().await;
                            data.limit = new_lim;
                        });
                    }
                }
            }
            Ok(())
        });
    }

    // --- Define HTTP Routes ---
    let router = Router::new()
        .route("/info", get(info_handler))
        .route("/status", get(status_handler))
        .route("/health", get(health_handler))
        .route("/ready", get(readiness_handler))
        .route("/metrics", get(metrics_handler))
        .route("/network/local-peer-id", get(network_local_peer_id_handler))
        .route("/network/peers", get(network_peers_handler))
        .route("/network/connect", post(network_connect_handler))
        .route("/dag/put", post(dag_put_handler))
        .route("/dag/get", post(dag_get_handler))
        .route("/dag/meta", post(dag_meta_handler))
        .route("/dag/root", get(dag_root_handler))
        .route("/dag/pin", post(dag_pin_handler))
        .route("/dag/unpin", post(dag_unpin_handler))
        .route("/dag/prune", post(dag_prune_handler))
        .route("/transaction/submit", post(tx_submit_handler))
        .route("/data/query", post(data_query_handler))
        .route("/governance/submit", post(gov_submit_handler))
        .route("/governance/vote", post(gov_vote_handler))
        .route("/governance/delegate", post(gov_delegate_handler))
        .route("/governance/revoke", post(gov_revoke_handler))
        .route("/governance/close", post(gov_close_handler))
        .route("/governance/execute", post(gov_execute_handler))
        .route("/governance/proposals", get(gov_list_proposals_handler))
        .route(
            "/governance/proposal/{proposal_id}",
            get(gov_get_proposal_handler),
        )
        .route("/mesh/submit", post(mesh_submit_job_handler))
        .route("/mesh/jobs", get(mesh_list_jobs_handler))
        .route("/mesh/jobs/{job_id}", get(mesh_get_job_status_handler))
        .route("/mesh/receipt", post(mesh_submit_receipt_handler))
        .route("/contracts", post(contracts_post_handler))
        .route("/federation/peers", get(federation_list_peers_handler))
        .route("/federation/peers", post(federation_add_peer_handler))
        .route("/federation/join", post(federation_join_handler))
        .route("/federation/leave", post(federation_leave_handler))
        .route("/federation/status", get(federation_status_handler))
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

    if let (Some(cert), Some(key)) = (&config.tls_cert_path, &config.tls_key_path) {
        info!(target: "audit", "tls_enabled cert={:?} min_version={:?}", cert, config.tls_min_version);
        let tls_config = axum_server::tls_rustls::RustlsConfig::from_pem_file(cert, key)
            .await
            .expect("failed to load TLS certificate");
        axum_server::bind_rustls(addr, tls_config)
            .serve(router.into_make_service())
            .await
            .unwrap();
    } else {
        info!(target: "audit", "tls_disabled");
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, router.into_make_service())
            .await
            .unwrap();
    }

    if config.enable_p2p {
        #[cfg(feature = "enable-libp2p")]
        {
            if let Err(e) = rt_ctx.shutdown_network().await {
                error!("Network shutdown failed: {}", e);
            }
        }
    }

    Ok(())
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

// GET /health – Node health status
async fn health_handler(State(state): State<AppState>) -> impl IntoResponse {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let start_time = NODE_START_TIME.load(Ordering::Relaxed);
    let uptime = if start_time > 0 { now - start_time } else { 0 };

    // Perform health checks
    let mut checks = HealthChecks {
        runtime: "OK".to_string(),
        dag_store: "OK".to_string(),
        network: "OK".to_string(),
        mana_ledger: "OK".to_string(),
    };

    let mut overall_status = "OK";

    // Check DAG store health
    match state.runtime_context.dag_store.try_lock() {
        Ok(_) => checks.dag_store = "OK".to_string(),
        Err(_) => {
            checks.dag_store = "BUSY".to_string();
            overall_status = "DEGRADED";
        }
    }

    // Check mana ledger health
    let test_did = icn_common::Did::new("key", "health_check_test");
    let _balance_check = state.runtime_context.mana_ledger.get_balance(&test_did);
    // get_balance returns u64, so it's always >= 0
    checks.mana_ledger = "OK".to_string();

    // Check network health
    #[cfg(feature = "enable-libp2p")]
    {
        match state.runtime_context.get_libp2p_service() {
            Ok(_) => checks.network = "OK".to_string(),
            Err(_) => {
                checks.network = "NOT_AVAILABLE".to_string();
                overall_status = "DEGRADED";
            }
        }
    }
    #[cfg(not(feature = "enable-libp2p"))]
    {
        checks.network = "DISABLED".to_string();
    }

    let health_status = HealthStatus {
        status: overall_status.to_string(),
        timestamp: now,
        uptime_seconds: uptime,
        checks,
    };

    let status_code = match overall_status {
        "OK" => StatusCode::OK,
        "DEGRADED" => StatusCode::OK, // Still serving requests
        _ => StatusCode::SERVICE_UNAVAILABLE,
    };

    (status_code, Json(health_status))
}

// GET /ready – Node readiness status
async fn readiness_handler(State(state): State<AppState>) -> impl IntoResponse {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mut checks = ReadinessChecks {
        can_serve_requests: true,
        mana_ledger_available: false,
        dag_store_available: false,
        network_initialized: false,
    };

    // Check if DAG store is accessible
    checks.dag_store_available = state.runtime_context.dag_store.try_lock().is_ok();

    // Check if mana ledger is working
    let test_did = icn_common::Did::new("key", "readiness_test");
    let _balance_check = state.runtime_context.mana_ledger.get_balance(&test_did);
    // get_balance returns u64, so it's always available
    checks.mana_ledger_available = true;

    // Check network initialization
    #[cfg(feature = "enable-libp2p")]
    {
        checks.network_initialized = state.runtime_context.get_libp2p_service().is_ok();
    }
    #[cfg(not(feature = "enable-libp2p"))]
    {
        checks.network_initialized = true; // Consider initialized if networking is disabled
    }

    let ready = checks.can_serve_requests
        && checks.mana_ledger_available
        && checks.dag_store_available
        && checks.network_initialized;

    let readiness_status = ReadinessStatus {
        ready,
        timestamp: now,
        checks,
    };

    let status_code = if ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (status_code, Json(readiness_status))
}

// GET /metrics – Prometheus metrics text
async fn metrics_handler() -> impl IntoResponse {
    use icn_api::metrics::register_core_metrics;
    use prometheus_client::metrics::{gauge::Gauge, histogram::Histogram};

    let mut registry = Registry::default();

    register_core_metrics(&mut registry);

    // Add system metrics
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let start_time = NODE_START_TIME.load(Ordering::Relaxed);
    let uptime = if start_time > 0 { now - start_time } else { 0 };

    let uptime_gauge: Gauge<f64, std::sync::atomic::AtomicU64> = Gauge::default();
    uptime_gauge.set(uptime as f64);
    registry.register(
        "node_uptime_seconds",
        "Node uptime in seconds",
        uptime_gauge,
    );

    let process_id_gauge: Gauge<f64, std::sync::atomic::AtomicU64> = Gauge::default();
    process_id_gauge.set(process::id() as f64);
    registry.register("node_process_id", "Node process ID", process_id_gauge);

    // Memory usage (basic approximation - could be enhanced with proper system monitoring)
    if let Ok(status) = fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                if let Some(kb_str) = line.split_whitespace().nth(1) {
                    if let Ok(kb) = kb_str.parse::<u64>() {
                        let memory_gauge: Gauge<f64, std::sync::atomic::AtomicU64> =
                            Gauge::default();
                        memory_gauge.set((kb * 1024) as f64); // Convert KB to bytes
                        registry.register(
                            "node_memory_usage_bytes",
                            "Node memory usage in bytes",
                            memory_gauge,
                        );
                    }
                }
                break;
            }
        }
    }

    let mut buffer = String::new();
    encode(&mut buffer, &registry).unwrap();
    (StatusCode::OK, buffer)
}

// POST /dag/put – Store a DAG block. (Body: block JSON)
async fn dag_put_handler(
    State(state): State<AppState>,
    Json(block): Json<DagBlockPayload>,
) -> impl IntoResponse {
    // Use RuntimeContext's dag_store now
    let ts = 0u64;
    let author = Did::new("key", "tester");
    let sig_opt = None;
    let cid = icn_common::compute_merkle_cid(0x71, &block.data, &[], ts, &author, &sig_opt, &None);
    let dag_block = CoreDagBlock {
        cid,
        data: block.data,
        links: vec![],
        timestamp: ts,
        author_did: author,
        signature: sig_opt,
        scope: None,
    };
    let mut store = state.runtime_context.dag_store.lock().await;
    match store.put(&dag_block).await {
        Ok(()) => (StatusCode::CREATED, Json(dag_block.cid.to_string())).into_response(),
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
    match store.get(&cid_to_get).await {
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

// POST /dag/meta – Retrieve metadata for a DAG block. (Body: CID JSON)
async fn dag_meta_handler(
    State(state): State<AppState>,
    Json(cid_request): Json<CidRequest>,
) -> impl IntoResponse {
    let cid = match parse_cid_from_string(&cid_request.cid) {
        Ok(c) => c,
        Err(e) => {
            return map_rust_error_to_json_response(
                format!("Invalid CID: {e}"),
                StatusCode::BAD_REQUEST,
            )
            .into_response();
        }
    };
    let _cid_json = match serde_json::to_string(&cid) {
        Ok(j) => j,
        Err(e) => {
            return map_rust_error_to_json_response(
                format!("CID serialization error: {e}"),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into_response();
        }
    };

    // Get metadata synchronously
    let metadata_result: Result<Option<icn_dag::DagBlockMetadata>, CommonError> = {
        let store = state.runtime_context.dag_store.lock().await;
        match store.get(&cid).await {
            Ok(block_opt) => Ok(block_opt.map(|b| icn_dag::metadata_from_block(&b))),
            Err(e) => Err(match e {
                CommonError::StorageError(msg) => {
                    CommonError::StorageError(format!("API: Failed to retrieve DagBlock: {}", msg))
                }
                CommonError::DeserializationError(msg) => CommonError::DeserializationError(
                    format!("API: Deserialization error during get: {}", msg),
                ),
                CommonError::PolicyDenied(msg) => {
                    CommonError::PolicyDenied(format!("API: {}", msg))
                }
                _ => CommonError::ApiError(format!(
                    "API: Unexpected error during store.get: {:?}",
                    e
                )),
            }),
        }
    };

    match metadata_result {
        Ok(Some(meta)) => (StatusCode::OK, Json(meta)).into_response(),
        Ok(None) => map_rust_error_to_json_response("Block not found", StatusCode::NOT_FOUND)
            .into_response(),
        Err(e) => map_rust_error_to_json_response(
            format!("DAG meta error: {e}"),
            StatusCode::INTERNAL_SERVER_ERROR,
        )
        .into_response(),
    }
}

/// GET /dag/root – Retrieve the current DAG root CID.
/// Clients can compare this value across peers for synchronization.
async fn dag_root_handler(State(state): State<AppState>) -> impl IntoResponse {
    let store = state.runtime_context.dag_store.lock().await;
    match icn_dag::current_root(&*store).await {
        Ok(Some(cid)) => (StatusCode::OK, Json(cid.to_string())).into_response(),
        Ok(None) => (StatusCode::OK, Json(String::new())).into_response(),
        Err(e) => map_rust_error_to_json_response(
            format!("DAG root error: {e}"),
            StatusCode::INTERNAL_SERVER_ERROR,
        )
        .into_response(),
    }
}

// POST /dag/pin – Pin a block with optional TTL
async fn dag_pin_handler(
    State(state): State<AppState>,
    Json(req): Json<PinRequest>,
) -> impl IntoResponse {
    let cid = match parse_cid_from_string(&req.cid) {
        Ok(cid) => cid,
        Err(e) => {
            return map_rust_error_to_json_response(
                format!("Invalid CID: {e}"),
                StatusCode::BAD_REQUEST,
            )
            .into_response();
        }
    };
    let mut store = state.runtime_context.dag_store.lock().await;
    if let Err(e) = store.pin_block(&cid).await {
        return map_rust_error_to_json_response(
            format!("Pin error: {e}"),
            StatusCode::INTERNAL_SERVER_ERROR,
        )
        .into_response();
    }
    if let Err(e) = store.set_ttl(&cid, req.ttl).await {
        return map_rust_error_to_json_response(
            format!("Set TTL error: {e}"),
            StatusCode::INTERNAL_SERVER_ERROR,
        )
        .into_response();
    }
    (StatusCode::OK, Json(cid)).into_response()
}

// POST /dag/unpin – Remove pin from a block
async fn dag_unpin_handler(
    State(state): State<AppState>,
    Json(req): Json<CidRequest>,
) -> impl IntoResponse {
    let cid = match parse_cid_from_string(&req.cid) {
        Ok(cid) => cid,
        Err(e) => {
            return map_rust_error_to_json_response(
                format!("Invalid CID: {e}"),
                StatusCode::BAD_REQUEST,
            )
            .into_response();
        }
    };
    let mut store = state.runtime_context.dag_store.lock().await;
    match store.unpin_block(&cid).await {
        Ok(()) => (StatusCode::OK, Json(cid)).into_response(),
        Err(e) => map_rust_error_to_json_response(
            format!("Unpin error: {e}"),
            StatusCode::INTERNAL_SERVER_ERROR,
        )
        .into_response(),
    }
}

// POST /dag/prune – Remove expired blocks
async fn dag_prune_handler(State(state): State<AppState>) -> impl IntoResponse {
    let now = state.runtime_context.time_provider.unix_seconds();
    let mut store = state.runtime_context.dag_store.lock().await;
    match store.prune_expired(now).await {
        Ok(removed) => (
            StatusCode::OK,
            Json(serde_json::json!({"removed": removed.iter().map(|c| c.to_string()).collect::<Vec<_>>() })),
        )
            .into_response(),
        Err(e) => map_rust_error_to_json_response(
            format!("Prune error: {e}"),
            StatusCode::INTERNAL_SERVER_ERROR,
        )
        .into_response(),
    }
}

// POST /contracts - Compile CCL source and store resulting WASM in DAG
async fn contracts_post_handler(
    State(state): State<AppState>,
    Json(payload): Json<ContractSourcePayload>,
) -> impl IntoResponse {
    use icn_ccl::compile_ccl_source_to_wasm;

    let (wasm, _meta) = match compile_ccl_source_to_wasm(&payload.source) {
        Ok(res) => res,
        Err(e) => {
            return map_rust_error_to_json_response(
                format!("Compilation error: {e}"),
                StatusCode::BAD_REQUEST,
            )
            .into_response();
        }
    };

    let ts = 0u64;
    let author = Did::new("key", "tester");
    let sig_opt = None;
    let cid = icn_common::compute_merkle_cid(0x71, &wasm, &[], ts, &author, &sig_opt, &None);
    let block = CoreDagBlock {
        cid: cid.clone(),
        data: wasm,
        links: vec![],
        timestamp: ts,
        author_did: author,
        signature: sig_opt,
        scope: None,
    };

    let block_json = match serde_json::to_string(&block) {
        Ok(j) => j,
        Err(e) => {
            return map_rust_error_to_json_response(
                format!("Failed to serialize DagBlock: {e}"),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into_response();
        }
    };

    match icn_api::submit_dag_block(
        state.runtime_context.dag_store.clone(),
        block_json,
        state.runtime_context.policy_enforcer.clone(),
        state.runtime_context.current_identity.clone(),
    )
    .await
    {
        Ok(_) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "manifest_cid": cid.to_string() })),
        )
            .into_response(),
        Err(e) => map_rust_error_to_json_response(
            format!("Failed to store contract: {e}"),
            StatusCode::INTERNAL_SERVER_ERROR,
        )
        .into_response(),
    }
}

#[derive(Deserialize)]
struct CidRequest {
    cid: String,
}

#[derive(Deserialize)]
struct PinRequest {
    cid: String,
    ttl: Option<u64>,
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
        icn_api::governance_trait::ProposalInputType::RemoveMember { did } => {
            ("RemoveMember".to_string(), did.into_bytes())
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
        quorum: request.quorum,
        threshold: request.threshold,
        body: request.body,
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

// POST /governance/delegate – Delegate vote from one DID to another
async fn gov_delegate_handler(
    State(state): State<AppState>,
    Json(req): Json<ApiDelegateRequest>,
) -> impl IntoResponse {
    match icn_runtime::host_delegate_vote(&state.runtime_context, &req.from_did, &req.to_did).await
    {
        Ok(_) => (StatusCode::OK, Json("delegated".to_string())).into_response(),
        Err(e) => map_rust_error_to_json_response(
            format!("Delegate error: {}", e),
            StatusCode::BAD_REQUEST,
        )
        .into_response(),
    }
}

// POST /governance/revoke – Revoke delegation
async fn gov_revoke_handler(
    State(state): State<AppState>,
    Json(req): Json<ApiRevokeDelegationRequest>,
) -> impl IntoResponse {
    match icn_runtime::host_revoke_delegation(&state.runtime_context, &req.from_did).await {
        Ok(_) => (StatusCode::OK, Json("revoked".to_string())).into_response(),
        Err(e) => {
            map_rust_error_to_json_response(format!("Revoke error: {}", e), StatusCode::BAD_REQUEST)
                .into_response()
        }
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

// POST /governance/close – tally votes and, if accepted, execute proposal
async fn gov_close_handler(
    State(state): State<AppState>,
    Json(req): Json<ProposalIdPayload>,
) -> impl IntoResponse {
    let result = icn_runtime::host_close_governance_proposal_voting(
        &state.runtime_context,
        &req.proposal_id,
    )
    .await;
    let status_json = match result {
        Ok(s) => s,
        Err(e) => {
            return map_rust_error_to_json_response(
                format!("Close error: {}", e),
                StatusCode::BAD_REQUEST,
            )
            .into_response()
        }
    };
    let close: icn_api::governance_trait::CloseProposalResponse =
        match serde_json::from_str(&status_json) {
            Ok(c) => c,
            Err(e) => {
                return map_rust_error_to_json_response(
                    format!("Serialization error: {}", e),
                    StatusCode::INTERNAL_SERVER_ERROR,
                )
                .into_response()
            }
        };
    if close.status == format!("{:?}", icn_governance::ProposalStatus::Accepted) {
        if let Err(e) =
            icn_runtime::host_execute_governance_proposal(&state.runtime_context, &req.proposal_id)
                .await
        {
            return map_rust_error_to_json_response(
                format!("Execute error: {}", e),
                StatusCode::BAD_REQUEST,
            )
            .into_response();
        }
    }
    (StatusCode::OK, Json(close)).into_response()
}

// POST /governance/execute – force execute an accepted proposal
async fn gov_execute_handler(
    State(state): State<AppState>,
    Json(req): Json<ProposalIdPayload>,
) -> impl IntoResponse {
    match icn_runtime::host_execute_governance_proposal(&state.runtime_context, &req.proposal_id)
        .await
    {
        Ok(_) => (StatusCode::OK, Json("executed".to_string())).into_response(),
        Err(e) => map_rust_error_to_json_response(
            format!("Execute error: {}", e),
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

    // Build complete ActualMeshJob structure with placeholder values
    // The runtime will override id, creator_did, and signature
    let complete_job = icn_mesh::ActualMeshJob {
        id: icn_mesh::JobId::from(manifest_cid.clone()), // Placeholder, will be overridden
        manifest_cid: manifest_cid,
        spec: job_spec,
        creator_did: icn_common::Did::from_str("did:placeholder:creator").unwrap(), // Placeholder, will be overridden
        cost_mana: request.cost_mana,
        max_execution_wait_ms: None,
        signature: icn_identity::SignatureBytes(vec![]), // Placeholder, will be overridden
    };

    let job_json = match serde_json::to_string(&complete_job) {
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
            info!(target: "audit", "job_submitted id={}" , actual_job_id_cid);
            (
                StatusCode::ACCEPTED,
                Json(serde_json::json!({ "job_id": actual_job_id_cid.to_string() })),
            )
                .into_response()
        }
        Err(e) => {
            error!("[NODE] Error submitting job via runtime: {:?}", e);
            info!(target: "audit", "job_submission_failed error={}" , e);
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

    let job_states = &state.runtime_context.job_states;
    let jobs: Vec<serde_json::Value> = job_states
        .iter()
        .map(|entry| {
            let job_id = entry.key();
            let job_state = entry.value();
            serde_json::json!({
                "job_id": job_id.to_string(),
                "status": match &*job_state {
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

    let job_states = &state.runtime_context.job_states;
    info!(
        "[Node] Looking for job_id {:?} in {} stored jobs",
        job_id,
        job_states.len()
    );

    // Debug: List all stored job IDs
    for entry in job_states.iter() {
        info!("[Node] Stored job ID: {:?}", entry.key());
    }

    match job_states.get(&icn_mesh::JobId::from(job_id.clone())) {
        Some(job_state) => {
            let response = serde_json::json!({
                "job_id": job_id.to_string(),
                "status": match &*job_state {
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

// Stub endpoints for testing mesh job lifecycle

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StubBidRequest {
    pub job_id: String,
    pub executor_id: String,
    pub estimated_cost: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StubReceiptRequest {
    pub job_id: String,
    pub executor_id: String,
    pub result: serde_json::Value,
}

// POST /mesh/stub/bid - inject a bid for testing
async fn mesh_stub_bid_handler(
    State(state): State<AppState>,
    Json(request): Json<StubBidRequest>,
) -> impl IntoResponse {
    use icn_runtime::context::MeshNetworkServiceType;

    // Parse job ID - create a CID from the string
    let cid = icn_common::Cid::new_v1_sha256(0x55, request.job_id.as_bytes());
    let job_id = icn_mesh::JobId(cid);

    // Parse executor DID
    let executor_did = match icn_common::Did::from_str(&request.executor_id) {
        Ok(did) => did,
        Err(_) => {
            return map_rust_error_to_json_response(
                "Invalid executor DID format",
                StatusCode::BAD_REQUEST,
            )
            .into_response()
        }
    };

    // Create a bid
    let bid = icn_mesh::MeshJobBid {
        job_id: job_id.clone(),
        executor_did: executor_did.clone(),
        price_mana: request.estimated_cost,
        resources: icn_mesh::Resources {
            cpu_cores: 1,
            memory_mb: 128,
        },
        signature: icn_identity::SignatureBytes(vec![0; 64]), // Dummy signature for testing
    };

    // Check if using stub network service
    match &*state.runtime_context.mesh_network_service {
        MeshNetworkServiceType::Stub(stub_service) => {
            stub_service.stage_bid(job_id, bid).await;
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "success": true,
                    "message": "Bid staged successfully",
                    "job_id": request.job_id,
                    "executor_id": request.executor_id,
                    "estimated_cost": request.estimated_cost
                })),
            )
                .into_response()
        }
        _ => map_rust_error_to_json_response(
            "Stub bid injection only works with stub network service",
            StatusCode::BAD_REQUEST,
        )
        .into_response(),
    }
}

// POST /mesh/stub/receipt - inject a receipt for testing
async fn mesh_stub_receipt_handler(
    State(state): State<AppState>,
    Json(request): Json<StubReceiptRequest>,
) -> impl IntoResponse {
    use icn_runtime::context::{LocalMeshSubmitReceiptMessage, MeshNetworkServiceType};

    // Parse job ID - create a CID from the string
    let cid = icn_common::Cid::new_v1_sha256(0x55, request.job_id.as_bytes());
    let job_id = icn_mesh::JobId(cid);

    // Parse executor DID
    let executor_did = match icn_common::Did::from_str(&request.executor_id) {
        Ok(did) => did,
        Err(_) => {
            return map_rust_error_to_json_response(
                "Invalid executor DID format",
                StatusCode::BAD_REQUEST,
            )
            .into_response()
        }
    };

    // Create a dummy result CID
    let result_cid = icn_common::Cid::new_v1_sha256(0x55, b"test_result");

    // Create execution receipt
    let receipt = icn_identity::ExecutionReceipt {
        job_id: job_id.clone().into(),
        executor_did: executor_did.clone(),
        result_cid: result_cid.clone(),
        cpu_ms: 100,
        success: true,
        sig: icn_identity::SignatureBytes(vec![0; 64]), // Dummy signature for testing
    };

    // Create receipt message
    let receipt_message = LocalMeshSubmitReceiptMessage {
        receipt: receipt.clone(),
    };

    // Check if using stub network service
    match &*state.runtime_context.mesh_network_service {
        MeshNetworkServiceType::Stub(stub_service) => {
            stub_service.stage_receipt(job_id, receipt_message).await;
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "success": true,
                    "message": "Receipt staged successfully",
                    "job_id": request.job_id,
                    "executor_id": request.executor_id,
                    "result_cid": result_cid.to_string()
                })),
            )
                .into_response()
        }
        _ => map_rust_error_to_json_response(
            "Stub receipt injection only works with stub network service",
            StatusCode::BAD_REQUEST,
        )
        .into_response(),
    }
}

// GET /federation/peers - list known peers
async fn federation_list_peers_handler(State(state): State<AppState>) -> impl IntoResponse {
    let peers = state.peers.lock().await.clone();
    (StatusCode::OK, Json(peers))
}

// POST /federation/peers - add a peer identifier
async fn federation_add_peer_handler(
    State(state): State<AppState>,
    Json(payload): Json<AddPeerPayload>,
) -> impl IntoResponse {
    let mut peers = state.peers.lock().await;
    if !peers.contains(&payload.peer) {
        peers.push(payload.peer.clone());
        info!(target: "audit", "peer_added peer={}" , payload.peer);
    }
    (
        StatusCode::CREATED,
        Json(serde_json::json!({ "peer": payload.peer })),
    )
}

// POST /federation/join - join a federation via peer identifier
async fn federation_join_handler(
    State(state): State<AppState>,
    Json(payload): Json<PeerPayload>,
) -> impl IntoResponse {
    {
        let mut peers = state.peers.lock().await;
        if !peers.contains(&payload.peer) {
            peers.push(payload.peer.clone());
        }
    }
    {
        let mut cfg = state.config.lock().await;
        cfg.federation_peers.push(payload.peer.clone());
    }
    #[cfg(feature = "enable-libp2p")]
    if let Ok(service) = state.runtime_context.get_libp2p_service() {
        let join_msg = ProtocolMessage::new(
            MessagePayload::FederationJoinRequest(FederationJoinRequestMessage {
                requesting_node: state.runtime_context.current_identity.clone(),
                federation_id: "default".to_string(),
                node_capabilities: NodeCapabilities {
                    compute_resources: ResourceRequirements::default(),
                    supported_job_kinds: vec![],
                    network_bandwidth_mbps: 0,
                    storage_capacity_gb: 0,
                    uptime_percentage: 0.0,
                },
                referral_from: None,
            }),
            Did::default(),
            None,
        );
        if let Err(e) = service.broadcast_message(join_msg).await {
            error!("Failed to broadcast join: {:?}", e);
        }
    }
    (
        StatusCode::OK,
        Json(serde_json::json!({ "joined": payload.peer })),
    )
}

// POST /federation/leave - leave a federation / remove peer
async fn federation_leave_handler(
    State(state): State<AppState>,
    Json(payload): Json<PeerPayload>,
) -> impl IntoResponse {
    {
        let mut peers = state.peers.lock().await;
        peers.retain(|p| p != &payload.peer);
    }
    {
        let mut cfg = state.config.lock().await;
        cfg.federation_peers.retain(|p| p != &payload.peer);
    }
    #[cfg(feature = "enable-libp2p")]
    if let Ok(service) = state.runtime_context.get_libp2p_service() {
        let leave_msg = ProtocolMessage::new(
            MessagePayload::GossipMessage(GossipMessage {
                topic: "federation_leave".to_string(),
                payload: payload.peer.clone().into_bytes(),
                ttl: 1,
            }),
            Did::default(),
            None,
        );
        if let Err(e) = service.broadcast_message(leave_msg).await {
            error!("Failed to broadcast leave: {:?}", e);
        }
    }
    (
        StatusCode::OK,
        Json(serde_json::json!({ "left": payload.peer })),
    )
}

// GET /federation/status - current federation status
async fn federation_status_handler(State(state): State<AppState>) -> impl IntoResponse {
    let peers = state.peers.lock().await.clone();
    let status = FederationStatus {
        peer_count: peers.len(),
        peers,
    };
    (StatusCode::OK, Json(status))
}

// GET /network/local-peer-id - return this node's peer ID
async fn network_local_peer_id_handler(State(state): State<AppState>) -> impl IntoResponse {
    #[cfg(feature = "enable-libp2p")]
    {
        match state.runtime_context.get_libp2p_service() {
            Ok(service) => {
                let id = service.local_peer_id().to_string();
                (StatusCode::OK, Json(serde_json::json!({ "peer_id": id }))).into_response()
            }
            Err(e) => map_rust_error_to_json_response(e, StatusCode::BAD_REQUEST).into_response(),
        }
    }
    #[cfg(not(feature = "enable-libp2p"))]
    {
        (
            StatusCode::OK,
            Json(serde_json::json!({ "peer_id": "p2p_disabled" })),
        )
            .into_response()
    }
}

// GET /network/peers - list peers discovered via the network service
async fn network_peers_handler(State(state): State<AppState>) -> impl IntoResponse {
    #[cfg(feature = "enable-libp2p")]
    {
        match state.runtime_context.get_libp2p_service() {
            Ok(service) => match service.discover_peers(None).await {
                Ok(peers) => {
                    let list: Vec<String> = peers.into_iter().map(|p| p.0).collect();
                    (StatusCode::OK, Json(list)).into_response()
                }
                Err(e) => {
                    map_rust_error_to_json_response(e, StatusCode::BAD_REQUEST).into_response()
                }
            },
            Err(e) => map_rust_error_to_json_response(e, StatusCode::BAD_REQUEST).into_response(),
        }
    }
    #[cfg(not(feature = "enable-libp2p"))]
    {
        (StatusCode::OK, Json(Vec::<String>::new())).into_response()
    }
}

// POST /network/connect - connect to a peer via multiaddress
async fn network_connect_handler(
    State(state): State<AppState>,
    Json(payload): Json<PeerPayload>,
) -> impl IntoResponse {
    #[cfg(feature = "enable-libp2p")]
    {
        match state.runtime_context.get_libp2p_service() {
            Ok(service) => match payload.peer.parse::<Multiaddr>() {
                Ok(addr) => match service.connect_peer(addr).await {
                    Ok(()) => (
                        StatusCode::OK,
                        Json(serde_json::json!({ "connected": payload.peer })),
                    )
                        .into_response(),
                    Err(e) => {
                        map_rust_error_to_json_response(e, StatusCode::BAD_REQUEST).into_response()
                    }
                },
                Err(e) => (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": format!("invalid multiaddr: {}", e) })),
                )
                    .into_response(),
            },
            Err(e) => map_rust_error_to_json_response(e, StatusCode::BAD_REQUEST).into_response(),
        }
    }
    #[cfg(not(feature = "enable-libp2p"))]
    {
        (
            StatusCode::OK,
            Json(serde_json::json!({ "connected": payload.peer })),
        )
            .into_response()
    }
}

// GET /account/:did/mana - return mana balance for an account
async fn account_mana_handler(
    AxumPath(did_str): AxumPath<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match Did::from_str(&did_str) {
        Ok(did) => match state.runtime_context.get_mana(&did).await {
            Ok(balance) => (
                StatusCode::OK,
                Json(serde_json::json!({ "balance": balance })),
            )
                .into_response(),
            Err(e) => map_rust_error_to_json_response(
                format!("Query error: {e}"),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into_response(),
        },
        Err(e) => {
            map_rust_error_to_json_response(format!("Invalid DID: {e}"), StatusCode::BAD_REQUEST)
                .into_response()
        }
    }
}

// GET /keys - return node DID and public key
async fn keys_handler(State(state): State<AppState>) -> impl IntoResponse {
    let did = state.runtime_context.current_identity.to_string();
    let pk_bs58 = bs58::encode(state.runtime_context.signer.public_key_bytes()).into_string();
    (
        StatusCode::OK,
        Json(serde_json::json!({ "did": did, "public_key_bs58": pk_bs58 })),
    )
}

// GET /reputation/:did - fetch reputation score
async fn reputation_handler(
    AxumPath(did_str): AxumPath<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match Did::from_str(&did_str) {
        Ok(did) => {
            let score = state.runtime_context.reputation_store.get_reputation(&did);
            (StatusCode::OK, Json(serde_json::json!({ "score": score }))).into_response()
        }
        Err(e) => {
            map_rust_error_to_json_response(format!("Invalid DID: {e}"), StatusCode::BAD_REQUEST)
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
            manifest_cid: Cid::new_v1_sha256(0x55, b"test_manifest").to_string(),
            spec_json: serde_json::to_value(&icn_mesh::JobSpec {
                kind: icn_mesh::JobKind::Echo {
                    payload: "hello".into(),
                },
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
            manifest_cid: Cid::new_v1_sha256(0x55, b"pipeline_test_manifest").to_string(),
            spec_json: serde_json::to_value(&icn_mesh::JobSpec {
                kind: icn_mesh::JobKind::Echo {
                    payload: "HTTP pipeline test".into(),
                },
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
            manifest_cid: Cid::new_v1_sha256(0x55, b"simple_test").to_string(),
            spec_json: serde_json::to_value(&icn_mesh::JobSpec {
                kind: icn_mesh::JobKind::Echo {
                    payload: "simple test".into(),
                },
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

        let (app, ctx) =
            app_router_with_options(None, None, None, None, None, None, None, None, None, None)
                .await;

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
        let signer =
            std::sync::Arc::new(icn_runtime::context::Ed25519Signer::new_with_keys(sk, vk));
        let executor = WasmExecutor::new(
            ctx.clone(),
            signer,
            icn_runtime::executor::WasmExecutorConfig::default(),
        );
        let job = ActualMeshJob {
            id: JobId(job_id.clone()),
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

    #[tokio::test]
    async fn network_connect_endpoint_basic() {
        let app = test_app().await;
        let payload = serde_json::json!({ "peer": "/ip4/127.0.0.1/tcp/1234" });
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/network/connect")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
