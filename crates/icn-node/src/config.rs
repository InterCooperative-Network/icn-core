use icn_common::{CommonError, DagBlock};
#[cfg(feature = "persist-rocksdb")]
use icn_dag::rocksdb_store::RocksDagStore;
#[cfg(feature = "persist-sled")]
use icn_dag::sled_store::SledDagStore;
#[cfg(feature = "persist-sqlite")]
use icn_dag::sqlite_store::SqliteDagStore;
use icn_dag::{CompatAsyncStore, TokioFileDagStore};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tokio::sync::Mutex as TokioMutex;

/// Storage backends supported by the node.
#[derive(clap::ValueEnum, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum StorageBackendType {
    /// In-memory store, volatile.
    Memory,
    /// File-based persistence.
    File,
    /// SQLite database backend (requires `persist-sqlite` feature).
    Sqlite,
    /// Sled database backend (requires `persist-sled` feature).
    Sled,
    /// RocksDB database backend (requires `persist-rocksdb` feature).
    Rocksdb,
    /// Postgres database backend (requires `persist-postgres` feature).
    #[cfg(feature = "persist-postgres")]
    Postgres,
}

impl std::str::FromStr for StorageBackendType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "memory" => Ok(Self::Memory),
            "file" => Ok(Self::File),
            "sqlite" => Ok(Self::Sqlite),
            "sled" => Ok(Self::Sled),
            "rocksdb" => Ok(Self::Rocksdb),
            #[cfg(feature = "persist-postgres")]
            "postgres" => Ok(Self::Postgres),
            _ => Err(format!("invalid storage backend: {s}")),
        }
    }
}

/// Configuration values for running an ICN node.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NodeConfig {
    pub storage_backend: StorageBackendType,
    pub storage_path: std::path::PathBuf,
    pub mana_ledger_backend: icn_runtime::context::LedgerBackend,
    pub mana_ledger_path: std::path::PathBuf,
    /// Path where executor reputation is persisted via sled.
    pub reputation_db_path: std::path::PathBuf,
    /// Path where governance proposals and votes are persisted via sled.
    pub governance_db_path: std::path::PathBuf,
    pub http_listen_addr: String,
    pub node_did: Option<String>,
    pub node_private_key_bs58: Option<String>,
    /// Path where the node DID string will be stored/loaded.
    pub node_did_path: std::path::PathBuf,
    /// Path where the node's private key will be stored/loaded (bs58 encoded).
    pub node_private_key_path: std::path::PathBuf,
    /// Encrypted private key file for the node identity.
    pub key_path: Option<std::path::PathBuf>,
    /// Environment variable name containing the passphrase for `key_path`.
    pub key_passphrase_env: Option<String>,
    /// Optional path to an HSM library or connection string.
    pub hsm_library: Option<std::path::PathBuf>,
    /// Identifier of the key within the HSM.
    pub hsm_key_id: Option<String>,
    pub node_name: String,
    pub listen_address: String,
    pub bootstrap_peers: Option<Vec<String>>,
    pub enable_p2p: bool,
    pub enable_mdns: bool,
    /// Force stub services for development and testing
    pub test_mode: bool,
    pub api_key: Option<String>,
    /// Optional bearer token for Authorization header auth.
    pub auth_token: Option<String>,
    /// Path to read the bearer token from if not provided inline.
    pub auth_token_path: Option<std::path::PathBuf>,
    pub open_rate_limit: u64,
    /// TLS certificate path for HTTPS. Requires `tls_key_path` as well.
    pub tls_cert_path: Option<std::path::PathBuf>,
    /// TLS private key path for HTTPS. Requires `tls_cert_path` as well.
    pub tls_key_path: Option<std::path::PathBuf>,
    /// Minimum TLS version to accept (e.g. "1.3").
    pub tls_min_version: Option<String>,
    /// How many days between automatic key rotations.
    pub key_rotation_days: u64,
    /// Peers this node has joined in a federation.
    pub federation_peers: Vec<String>,
}

pub(crate) fn default_ledger_backend() -> icn_runtime::context::LedgerBackend {
    #[cfg(feature = "persist-sled")]
    {
        icn_runtime::context::LedgerBackend::Sled
    }
    #[cfg(all(not(feature = "persist-sled"), feature = "persist-sqlite"))]
    {
        icn_runtime::context::LedgerBackend::Sqlite
    }
    #[cfg(all(
        not(feature = "persist-sled"),
        not(feature = "persist-sqlite"),
        feature = "persist-rocksdb"
    ))]
    {
        icn_runtime::context::LedgerBackend::Rocksdb
    }
    #[cfg(all(
        not(feature = "persist-sled"),
        not(feature = "persist-sqlite"),
        not(feature = "persist-rocksdb")
    ))]
    {
        icn_runtime::context::LedgerBackend::File
    }
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            storage_backend: StorageBackendType::Memory,
            storage_path: "./icn_data/node_store".into(),
            mana_ledger_backend: default_ledger_backend(),
            mana_ledger_path: "./tests/fixtures/mana_ledger.json".into(),
            reputation_db_path: "./reputation.sled".into(),
            governance_db_path: "./governance_db".into(),
            http_listen_addr: "127.0.0.1:7845".to_string(),
            node_did: None,
            node_private_key_bs58: None,
            node_did_path: "./icn_data/node_did.txt".into(),
            node_private_key_path: "./icn_data/node_sk.bs58".into(),
            key_path: None,
            key_passphrase_env: None,
            hsm_library: None,
            hsm_key_id: None,
            node_name: "ICN Node".to_string(),
            listen_address: "/ip4/0.0.0.0/tcp/0".to_string(),
            bootstrap_peers: None,
            enable_p2p: cfg!(feature = "enable-libp2p"),
            enable_mdns: true, // Enable mDNS by default for local networks
            test_mode: false,
            api_key: None,
            auth_token: None,
            auth_token_path: None,
            open_rate_limit: 60,
            tls_cert_path: None,
            tls_key_path: None,
            tls_min_version: Some("1.3".into()),
            key_rotation_days: 90,
            federation_peers: Vec::new(),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct StorageSection {
    backend: Option<StorageBackendType>,
    path: Option<PathBuf>,
    mana_ledger_path: Option<PathBuf>,
    reputation_db_path: Option<PathBuf>,
    governance_db_path: Option<PathBuf>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct IdentitySection {
    node_did: Option<String>,
    node_private_key_bs58: Option<String>,
    node_did_path: Option<PathBuf>,
    node_private_key_path: Option<PathBuf>,
    key_path: Option<PathBuf>,
    key_passphrase_env: Option<String>,
    hsm_library: Option<PathBuf>,
    hsm_key_id: Option<String>,
    key_rotation_days: Option<u64>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct HttpSection {
    listen_addr: Option<String>,
    api_key: Option<String>,
    auth_token: Option<String>,
    auth_token_path: Option<PathBuf>,
    open_rate_limit: Option<u64>,
    tls_cert_path: Option<PathBuf>,
    tls_key_path: Option<PathBuf>,
    tls_min_version: Option<String>,
    key_rotation_days: Option<u64>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct P2pSection {
    listen_address: Option<String>,
    bootstrap_peers: Option<Vec<String>>,
    enable_p2p: Option<bool>,
    enable_mdns: Option<bool>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct FileNodeConfig {
    node_name: Option<String>,
    storage_backend: Option<StorageBackendType>,
    storage_path: Option<PathBuf>,
    mana_ledger_path: Option<PathBuf>,
    reputation_db_path: Option<PathBuf>,
    governance_db_path: Option<PathBuf>,
    http_listen_addr: Option<String>,
    node_did: Option<String>,
    node_private_key_bs58: Option<String>,
    node_did_path: Option<PathBuf>,
    node_private_key_path: Option<PathBuf>,
    key_path: Option<PathBuf>,
    key_passphrase_env: Option<String>,
    hsm_library: Option<PathBuf>,
    hsm_key_id: Option<String>,
    listen_address: Option<String>,
    bootstrap_peers: Option<Vec<String>>,
    enable_p2p: Option<bool>,
    enable_mdns: Option<bool>,
    test_mode: Option<bool>,
    api_key: Option<String>,
    auth_token: Option<String>,
    auth_token_path: Option<PathBuf>,
    open_rate_limit: Option<u64>,
    tls_cert_path: Option<PathBuf>,
    tls_key_path: Option<PathBuf>,
    #[serde(default)]
    storage: StorageSection,
    #[serde(default)]
    identity: IdentitySection,
    #[serde(default)]
    http: HttpSection,
    #[serde(default)]
    p2p: P2pSection,
}

impl NodeConfig {
    /// Load configuration from a TOML or YAML file. The format is inferred from the file extension.
    pub fn from_file(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let data = fs::read_to_string(path)?;
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let file_cfg: FileNodeConfig = match ext {
            "toml" => toml::from_str(&data)?,
            "yaml" | "yml" => serde_yaml::from_str(&data)?,
            _ => return Err(format!("unsupported config extension: {ext}").into()),
        };

        let mut cfg = NodeConfig::default();
        cfg.apply_file_config(file_cfg);
        Ok(cfg)
    }

    fn apply_file_config(&mut self, file: FileNodeConfig) {
        if let Some(v) = file.node_name {
            self.node_name = v;
        }
        if let Some(v) = file.storage_backend {
            self.storage_backend = v;
        }
        if let Some(v) = file.storage_path {
            self.storage_path = v;
        }
        if let Some(v) = file.mana_ledger_path {
            self.mana_ledger_path = v;
        }
        if let Some(v) = file.reputation_db_path {
            self.reputation_db_path = v;
        }
        if let Some(v) = file.governance_db_path {
            self.governance_db_path = v;
        }
        if let Some(v) = file.http_listen_addr {
            self.http_listen_addr = v;
        }
        if let Some(v) = file.node_did {
            self.node_did = Some(v);
        }
        if let Some(v) = file.node_private_key_bs58 {
            self.node_private_key_bs58 = Some(v);
        }
        if let Some(v) = file.node_did_path {
            self.node_did_path = v;
        }
        if let Some(v) = file.node_private_key_path {
            self.node_private_key_path = v;
        }
        if let Some(v) = file.listen_address {
            self.listen_address = v;
        }
        if let Some(v) = file.bootstrap_peers {
            self.bootstrap_peers = Some(v);
        }
        if let Some(v) = file.enable_p2p {
            self.enable_p2p = v;
        }
        if let Some(v) = file.enable_mdns {
            self.enable_mdns = v;
        }
        if let Some(v) = file.test_mode {
            self.test_mode = v;
        }
        if let Some(v) = file.api_key {
            self.api_key = Some(v);
        }
        if let Some(v) = file.auth_token {
            self.auth_token = Some(v);
        }
        if let Some(v) = file.auth_token_path {
            self.auth_token_path = Some(v);
        }
        if let Some(v) = file.open_rate_limit {
            self.open_rate_limit = v;
        }
        if let Some(v) = file.tls_cert_path {
            self.tls_cert_path = Some(v);
        }
        if let Some(v) = file.tls_key_path {
            self.tls_key_path = Some(v);
        }

        // nested sections
        let s = file.storage;
        if let Some(v) = s.backend {
            self.storage_backend = v;
        }
        if let Some(v) = s.path {
            self.storage_path = v;
        }
        if let Some(v) = s.mana_ledger_path {
            self.mana_ledger_path = v;
        }
        if let Some(v) = s.reputation_db_path {
            self.reputation_db_path = v;
        }
        if let Some(v) = s.governance_db_path {
            self.governance_db_path = v;
        }

        let id = file.identity;
        if let Some(v) = id.node_did {
            self.node_did = Some(v);
        }
        if let Some(v) = id.node_private_key_bs58 {
            self.node_private_key_bs58 = Some(v);
        }
        if let Some(v) = id.node_did_path {
            self.node_did_path = v;
        }
        if let Some(v) = id.node_private_key_path {
            self.node_private_key_path = v;
        }
        if let Some(v) = id.key_path {
            self.key_path = Some(v);
        }
        if let Some(v) = id.key_passphrase_env {
            self.key_passphrase_env = Some(v);
        }
        if let Some(v) = id.hsm_library {
            self.hsm_library = Some(v);
        }
        if let Some(v) = id.hsm_key_id {
            self.hsm_key_id = Some(v);
        }
        if let Some(v) = id.key_rotation_days {
            self.key_rotation_days = v;
        }

        let http = file.http;
        if let Some(v) = http.listen_addr {
            self.http_listen_addr = v;
        }
        if let Some(v) = http.api_key {
            self.api_key = Some(v);
        }
        if let Some(v) = http.auth_token {
            self.auth_token = Some(v);
        }
        if let Some(v) = http.auth_token_path {
            self.auth_token_path = Some(v);
        }
        if let Some(v) = http.open_rate_limit {
            self.open_rate_limit = v;
        }
        if let Some(v) = http.tls_cert_path {
            self.tls_cert_path = Some(v);
        }
        if let Some(v) = http.tls_key_path {
            self.tls_key_path = Some(v);
        }
        if let Some(v) = http.tls_min_version {
            self.tls_min_version = Some(v);
        }
        if let Some(v) = http.key_rotation_days {
            self.key_rotation_days = v;
        }

        let p2p = file.p2p;
        if let Some(v) = p2p.listen_address {
            self.listen_address = v;
        }
        if let Some(v) = p2p.bootstrap_peers {
            self.bootstrap_peers = Some(v);
        }
        if let Some(v) = p2p.enable_p2p {
            self.enable_p2p = v;
        }
        if let Some(v) = p2p.enable_mdns {
            self.enable_mdns = v;
        }
    }

    /// Override configuration values with `ICN_*` environment variables.
    pub fn apply_env_overrides(&mut self) {
        macro_rules! set_from_env {
            ($field:ident, $var:expr, $parse:expr) => {
                if let Ok(val) = std::env::var($var) {
                    if let Ok(parsed) = $parse(&val) {
                        self.$field = parsed;
                    }
                }
            };
        }
        macro_rules! set_opt_from_env {
            ($field:ident, $var:expr) => {
                if let Ok(val) = std::env::var($var) {
                    self.$field = Some(val.into());
                }
            };
        }

        set_from_env!(storage_backend, "ICN_STORAGE_BACKEND", |v: &str| {
            v.parse::<StorageBackendType>()
        });
        set_from_env!(enable_p2p, "ICN_ENABLE_P2P", |v: &str| v.parse::<bool>());
        set_from_env!(enable_mdns, "ICN_ENABLE_MDNS", |v: &str| v.parse::<bool>());
        set_from_env!(test_mode, "ICN_TEST_MODE", |v: &str| v.parse::<bool>());
        set_from_env!(open_rate_limit, "ICN_OPEN_RATE_LIMIT", |v: &str| v
            .parse::<u64>());

        if let Ok(val) = std::env::var("ICN_STORAGE_PATH") {
            self.storage_path = val.into();
        }
        if let Ok(val) = std::env::var("ICN_MANA_LEDGER_PATH") {
            self.mana_ledger_path = val.into();
        }
        if let Ok(val) = std::env::var("ICN_REPUTATION_DB_PATH") {
            self.reputation_db_path = val.into();
        }
        if let Ok(val) = std::env::var("ICN_GOVERNANCE_DB_PATH") {
            self.governance_db_path = val.into();
        }
        if let Ok(val) = std::env::var("ICN_HTTP_LISTEN_ADDR") {
            self.http_listen_addr = val;
        }
        if let Ok(val) = std::env::var("ICN_NODE_DID") {
            self.node_did = Some(val);
        }
        if let Ok(val) = std::env::var("ICN_NODE_PRIVATE_KEY_BS58") {
            self.node_private_key_bs58 = Some(val);
        }
        if let Ok(val) = std::env::var("ICN_NODE_DID_PATH") {
            self.node_did_path = val.into();
        }
        if let Ok(val) = std::env::var("ICN_NODE_PRIVATE_KEY_PATH") {
            self.node_private_key_path = val.into();
        }
        if let Ok(val) = std::env::var("ICN_KEY_PATH") {
            self.key_path = Some(val.into());
        }
        if let Ok(val) = std::env::var("ICN_KEY_PASSPHRASE_ENV") {
            self.key_passphrase_env = Some(val);
        }
        if let Ok(val) = std::env::var("ICN_HSM_LIBRARY") {
            self.hsm_library = Some(val.into());
        }
        if let Ok(val) = std::env::var("ICN_HSM_KEY_ID") {
            self.hsm_key_id = Some(val);
        }
        if let Ok(val) = std::env::var("ICN_NODE_NAME") {
            self.node_name = val;
        }
        if let Ok(val) = std::env::var("ICN_LISTEN_ADDRESS") {
            self.listen_address = val;
        }
        if let Ok(val) = std::env::var("ICN_BOOTSTRAP_PEERS") {
            self.bootstrap_peers = Some(val.split(',').map(|s| s.to_string()).collect());
        }
        set_opt_from_env!(api_key, "ICN_HTTP_API_KEY");
        set_opt_from_env!(auth_token, "ICN_AUTH_TOKEN");
        if let Ok(val) = std::env::var("ICN_AUTH_TOKEN_PATH") {
            self.auth_token_path = Some(val.into());
        }
        if let Ok(val) = std::env::var("ICN_TLS_CERT_PATH") {
            self.tls_cert_path = Some(val.into());
        }
        if let Ok(val) = std::env::var("ICN_TLS_KEY_PATH") {
            self.tls_key_path = Some(val.into());
        }
        if let Ok(val) = std::env::var("ICN_TLS_MIN_VERSION") {
            self.tls_min_version = Some(val);
        }
        if let Ok(val) = std::env::var("ICN_KEY_ROTATION_DAYS") {
            if let Ok(days) = val.parse::<u64>() {
                self.key_rotation_days = days;
            }
        }
    }

    /// Apply CLI overrides onto this configuration.
    pub fn apply_cli_overrides(&mut self, cli: &super::node::Cli, matches: &clap::ArgMatches) {
        if let Some(v) = &cli.storage_backend {
            self.storage_backend = v.clone();
        }
        if let Some(v) = &cli.storage_path {
            self.storage_path = v.clone();
        }
        if let Some(v) = &cli.mana_ledger_backend {
            self.mana_ledger_backend = *v;
        }
        if let Some(v) = &cli.mana_ledger_path {
            self.mana_ledger_path = v.clone();
        }
        if let Some(v) = &cli.reputation_db_path {
            self.reputation_db_path = v.clone();
        }
        if let Some(v) = &cli.governance_db_path {
            self.governance_db_path = v.clone();
        }
        if let Some(v) = &cli.http_listen_addr {
            self.http_listen_addr = v.clone();
        }
        if let Some(v) = &cli.node_did {
            self.node_did = Some(v.clone());
        }
        if let Some(v) = &cli.node_private_key_bs58 {
            self.node_private_key_bs58 = Some(v.clone());
        }
        if let Some(v) = &cli.node_did_path {
            self.node_did_path = v.clone();
        }
        if let Some(v) = &cli.node_private_key_path {
            self.node_private_key_path = v.clone();
        }
        if let Some(v) = &cli.key_path {
            self.key_path = Some(v.clone());
        }
        if let Some(v) = &cli.key_passphrase_env {
            self.key_passphrase_env = Some(v.clone());
        }
        if let Some(v) = &cli.hsm_library {
            self.hsm_library = Some(v.clone());
        }
        if let Some(v) = &cli.hsm_key_id {
            self.hsm_key_id = Some(v.clone());
        }
        if let Some(v) = &cli.node_name {
            self.node_name = v.clone();
        }
        if let Some(v) = &cli.listen_address {
            self.listen_address = v.clone();
        }
        if let Some(v) = &cli.bootstrap_peers {
            self.bootstrap_peers = Some(v.clone());
        }
        if matches.contains_id("enable_p2p") {
            self.enable_p2p = true;
        }
        if matches.contains_id("enable_mdns") {
            self.enable_mdns = true;
        }
        if cli.test_mode || matches.contains_id("test_mode") {
            self.test_mode = true;
        }
        if let Some(v) = &cli.api_key {
            self.api_key = Some(v.clone());
        }
        if let Some(v) = cli.open_rate_limit {
            self.open_rate_limit = v;
        }
        if let Some(v) = &cli.auth_token {
            self.auth_token = Some(v.clone());
        }
        if let Some(v) = &cli.auth_token_path {
            self.auth_token_path = Some(v.clone());
        }
        if let Some(v) = &cli.tls_cert_path {
            self.tls_cert_path = Some(v.clone());
        }
        if let Some(v) = &cli.tls_key_path {
            self.tls_key_path = Some(v.clone());
        }
        if let Some(v) = &cli.tls_min_version {
            self.tls_min_version = Some(v.clone());
        }
        if let Some(v) = cli.key_rotation_days {
            self.key_rotation_days = v;
        }
    }

    /// Ensure directories for all configured paths exist.
    pub fn prepare_paths(&self) -> std::io::Result<()> {
        if let Some(parent) = self.storage_path.parent() {
            fs::create_dir_all(parent)?;
        }
        if let Some(parent) = self.mana_ledger_path.parent() {
            fs::create_dir_all(parent)?;
        }
        if let Some(parent) = self.reputation_db_path.parent() {
            fs::create_dir_all(parent)?;
        }
        if let Some(parent) = self.governance_db_path.parent() {
            fs::create_dir_all(parent)?;
        }
        if let Some(parent) = self.node_did_path.parent() {
            fs::create_dir_all(parent)?;
        }
        if let Some(parent) = self.node_private_key_path.parent() {
            fs::create_dir_all(parent)?;
        }
        if let Some(path) = &self.auth_token_path {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
        }
        Ok(())
    }

    /// Initialize a DAG store based on this configuration.
    pub fn init_dag_store(
        &self,
    ) -> Result<
        std::sync::Arc<TokioMutex<dyn icn_dag::AsyncStorageService<DagBlock> + Send>>,
        CommonError,
    > {
        let store: std::sync::Arc<TokioMutex<dyn icn_dag::AsyncStorageService<DagBlock> + Send>> =
            match self.storage_backend {
                StorageBackendType::Memory => std::sync::Arc::new(TokioMutex::new(
                    CompatAsyncStore::new(icn_dag::InMemoryDagStore::new()),
                )) as std::sync::Arc<TokioMutex<_>>,
                StorageBackendType::File => std::sync::Arc::new(TokioMutex::new(
                    TokioFileDagStore::new(self.storage_path.clone())?,
                )) as std::sync::Arc<TokioMutex<_>>,
                StorageBackendType::Sqlite => {
                    #[cfg(feature = "persist-sqlite")]
                    {
                        std::sync::Arc::new(TokioMutex::new(CompatAsyncStore::new(
                            SqliteDagStore::new(self.storage_path.clone())?,
                        ))) as std::sync::Arc<TokioMutex<_>>
                    }
                    #[cfg(not(feature = "persist-sqlite"))]
                    {
                        return Err(CommonError::ConfigError(
                            "sqlite backend requires 'persist-sqlite' feature".into(),
                        ));
                    }
                }
                StorageBackendType::Sled => {
                    #[cfg(feature = "persist-sled")]
                    {
                        std::sync::Arc::new(TokioMutex::new(CompatAsyncStore::new(
                            SledDagStore::new(self.storage_path.clone())?,
                        ))) as std::sync::Arc<TokioMutex<_>>
                    }
                    #[cfg(not(feature = "persist-sled"))]
                    {
                        return Err(CommonError::ConfigError(
                            "sled backend requires 'persist-sled' feature".into(),
                        ));
                    }
                }
                StorageBackendType::Rocksdb => {
                    #[cfg(feature = "persist-rocksdb")]
                    {
                        std::sync::Arc::new(TokioMutex::new(CompatAsyncStore::new(
                            RocksDagStore::new(self.storage_path.clone())?,
                        ))) as std::sync::Arc<TokioMutex<_>>
                    }
                    #[cfg(not(feature = "persist-rocksdb"))]
                    {
                        return Err(CommonError::ConfigError(
                            "rocksdb backend requires 'persist-rocksdb' feature".into(),
                        ));
                    }
                }
                #[cfg(feature = "persist-postgres")]
                StorageBackendType::Postgres => std::sync::Arc::new(TokioMutex::new(
                    CompatAsyncStore::new(icn_dag::postgres_store::PostgresDagStore::new(
                        &self.storage_path.to_string_lossy(),
                    )?),
                )) as std::sync::Arc<TokioMutex<_>>,
            };
        Ok(store)
    }

    /// Persist this configuration to the given path in TOML format.
    pub fn save_to_file(&self, path: &std::path::Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let toml_str = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        std::fs::write(path, toml_str)
    }
}

#[cfg(feature = "enable-libp2p")]
use icn_network::libp2p_service::NetworkConfig;
#[cfg(feature = "enable-libp2p")]
use libp2p::{Multiaddr, PeerId as Libp2pPeerId};

impl NodeConfig {
    #[cfg(feature = "enable-libp2p")]
    pub fn libp2p_config(&self) -> Result<NetworkConfig, CommonError> {
        let listen_addr = self
            .listen_address
            .parse::<Multiaddr>()
            .map_err(|e| CommonError::ConfigError(format!("invalid p2p listen address: {e}")))?;

        let bootstrap_peers = if let Some(peers) = &self.bootstrap_peers {
            let mut parsed = Vec::new();
            for peer_str in peers {
                let addr = peer_str.parse::<Multiaddr>().map_err(|e| {
                    CommonError::ConfigError(format!("invalid bootstrap peer '{peer_str}': {e}"))
                })?;
                if let Some(libp2p::core::multiaddr::Protocol::P2p(pid)) = addr.iter().last() {
                    let id: Libp2pPeerId = pid
                        .try_into()
                        .map_err(|_| CommonError::ConfigError("invalid peer id".into()))?;
                    parsed.push((id, addr));
                } else {
                    return Err(CommonError::ConfigError(format!(
                        "bootstrap peer missing peer id: {peer_str}"
                    )));
                }
            }
            parsed
        } else {
            Vec::new()
        };

        Ok(NetworkConfig {
            listen_addresses: vec![listen_addr],
            bootstrap_peers,
            enable_mdns: self.enable_mdns,
            ..Default::default()
        })
    }
}
