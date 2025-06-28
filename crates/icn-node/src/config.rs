use clap::ArgMatches;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Storage backends supported by the node.
#[derive(clap::ValueEnum, Clone, Debug, Serialize, Deserialize)]
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
    pub node_name: String,
    pub listen_address: String,
    pub bootstrap_peers: Option<Vec<String>>,
    pub enable_p2p: bool,
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
}

fn default_ledger_backend() -> icn_runtime::context::LedgerBackend {
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
            mana_ledger_path: "./mana_ledger.sled".into(),
            reputation_db_path: "./reputation.sled".into(),
            governance_db_path: "./governance_db".into(),
            http_listen_addr: "127.0.0.1:7845".to_string(),
            node_did: None,
            node_private_key_bs58: None,
            node_did_path: "./icn_data/node_did.txt".into(),
            node_private_key_path: "./icn_data/node_sk.bs58".into(),
            node_name: "ICN Node".to_string(),
            listen_address: "/ip4/0.0.0.0/tcp/0".to_string(),
            bootstrap_peers: None,
            enable_p2p: false,
            api_key: None,
            auth_token: None,
            auth_token_path: None,
            open_rate_limit: 60,
            tls_cert_path: None,
            tls_key_path: None,
        }
    }
}

impl NodeConfig {
    /// Load configuration from a TOML or YAML file. The format is inferred from the file extension.
    pub fn from_file(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let data = fs::read_to_string(path)?;
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let config = match ext {
            "toml" => toml::from_str(&data)?,
            "yaml" | "yml" => serde_yaml::from_str(&data)?,
            _ => return Err(format!("unsupported config extension: {ext}").into()),
        };
        Ok(config)
    }

    /// Apply CLI overrides onto this configuration.
    pub fn apply_cli_overrides(&mut self, cli: &super::node::Cli, matches: &ArgMatches) {
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
        if let Some(v) = &cli.api_key {
            self.api_key = Some(v.clone());
        }
        if let Some(v) = cli.open_rate_limit {
            self.open_rate_limit = v;
        }
        if let Some(v) = &cli.tls_cert_path {
            self.tls_cert_path = Some(v.clone());
        }
        if let Some(v) = &cli.tls_key_path {
            self.tls_key_path = Some(v.clone());
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
}
