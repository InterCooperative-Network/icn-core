use icn_common::{NodeInfo, NodeStatus};
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub timestamp: u64,
    pub uptime_seconds: u64,
    pub checks: HealthChecks,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HealthChecks {
    pub runtime: String,
    pub dag_store: String,
    pub network: String,
    pub mana_ledger: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReadinessStatus {
    pub ready: bool,
    pub timestamp: u64,
    pub checks: ReadinessChecks,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReadinessChecks {
    pub can_serve_requests: bool,
    pub mana_ledger_available: bool,
    pub dag_store_available: bool,
    pub network_initialized: bool,
}

/// High-level client for interacting with an ICN HTTP node.
#[derive(Clone)]
pub struct IcnClient {
    base_url: Url,
    http: Client,
}

impl IcnClient {
    /// Create a new client pointing at the given base URL.
    pub fn new(base_url: &str) -> Result<Self, url::ParseError> {
        Ok(Self {
            base_url: Url::parse(base_url)?,
            http: Client::new(),
        })
    }

    /// Generic GET helper returning a deserialized type.
    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, reqwest::Error> {
        let url = self.base_url.join(path).expect("valid url");
        self.http.get(url).send().await?.json().await
    }

    /// Generic POST helper sending JSON and returning a deserialized type.
    async fn post<Req: Serialize, Resp: DeserializeOwned>(
        &self,
        path: &str,
        body: &Req,
    ) -> Result<Resp, reqwest::Error> {
        let url = self.base_url.join(path).expect("valid url");
        self.http.post(url).json(body).send().await?.json().await
    }

    /// Get node information.
    pub async fn info(&self) -> Result<NodeInfo, reqwest::Error> {
        self.get("/info").await
    }

    /// Get current node status.
    pub async fn status(&self) -> Result<NodeStatus, reqwest::Error> {
        self.get("/status").await
    }

    /// Health check.
    pub async fn health(&self) -> Result<HealthStatus, reqwest::Error> {
        self.get("/health").await
    }

    /// Readiness probe.
    pub async fn ready(&self) -> Result<ReadinessStatus, reqwest::Error> {
        self.get("/ready").await
    }

    /// Submit a mesh job. Returns JSON with the created job id.
    pub async fn submit_mesh_job<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/mesh/submit", body).await
    }

    /// List mesh jobs.
    pub async fn list_mesh_jobs(&self) -> Result<serde_json::Value, reqwest::Error> {
        self.get("/mesh/jobs").await
    }

    /// Get mesh job status.
    pub async fn mesh_job(&self, job_id: &str) -> Result<serde_json::Value, reqwest::Error> {
        let path = format!("/mesh/jobs/{job_id}");
        self.get(&path).await
    }

    /// Submit execution receipt.
    pub async fn submit_mesh_receipt<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/mesh/receipt", body).await
    }

    /// Inject a mesh bid (stub only).
    pub async fn mesh_stub_bid<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/mesh/stub/bid", body).await
    }

    /// Inject a mesh receipt (stub only).
    pub async fn mesh_stub_receipt<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/mesh/stub/receipt", body).await
    }

    /// List governance proposals.
    pub async fn list_proposals(&self) -> Result<serde_json::Value, reqwest::Error> {
        self.get("/governance/proposals").await
    }

    /// Fetch a proposal by id.
    pub async fn proposal(&self, id: &str) -> Result<serde_json::Value, reqwest::Error> {
        let path = format!("/governance/proposal/{id}");
        self.get(&path).await
    }

    /// Submit a governance proposal.
    pub async fn submit_proposal<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/governance/submit", body).await
    }

    /// Cast a vote.
    pub async fn cast_vote<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/governance/vote", body).await
    }

    /// Delegate voting power.
    pub async fn delegate<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/governance/delegate", body).await
    }

    /// Revoke a delegation.
    pub async fn revoke<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/governance/revoke", body).await
    }

    /// Close voting.
    pub async fn close_vote<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/governance/close", body).await
    }

    /// Execute a proposal.
    pub async fn execute<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/governance/execute", body).await
    }

    /// Store data in the DAG.
    pub async fn dag_put<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/dag/put", body).await
    }

    /// Retrieve data from the DAG.
    pub async fn dag_get<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/dag/get", body).await
    }

    /// Retrieve DAG metadata.
    pub async fn dag_meta<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/dag/meta", body).await
    }

    /// Pin a DAG block.
    pub async fn dag_pin<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/dag/pin", body).await
    }

    /// Unpin a DAG block.
    pub async fn dag_unpin<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/dag/unpin", body).await
    }

    /// Prune the DAG store.
    pub async fn dag_prune<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/dag/prune", body).await
    }

    /// Retrieve the current DAG root CID.
    pub async fn dag_root(&self) -> Result<String, reqwest::Error> {
        self.get("/dag/root").await
    }

    /// Return the node's peer id.
    pub async fn local_peer_id(&self) -> Result<serde_json::Value, reqwest::Error> {
        self.get("/network/local-peer-id").await
    }

    /// Connect to a peer.
    pub async fn connect_peer<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/network/connect", body).await
    }

    /// List connected peers.
    pub async fn peers(&self) -> Result<serde_json::Value, reqwest::Error> {
        self.get("/network/peers").await
    }

    /// Get mana balance for a DID.
    pub async fn account_mana(&self, did: &str) -> Result<serde_json::Value, reqwest::Error> {
        let path = format!("/account/{did}/mana");
        self.get(&path).await
    }

    /// Retrieve the node DID and public key.
    pub async fn keys(&self) -> Result<serde_json::Value, reqwest::Error> {
        self.get("/keys").await
    }

    /// Fetch reputation score for a DID.
    pub async fn reputation(&self, did: &str) -> Result<serde_json::Value, reqwest::Error> {
        let path = format!("/reputation/{did}");
        self.get(&path).await
    }

    /// Submit a transaction.
    pub async fn submit_transaction<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/transaction/submit", body).await
    }

    /// Query data.
    pub async fn data_query<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/data/query", body).await
    }

    /// Upload a WASM contract.
    pub async fn upload_contract<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/contracts", body).await
    }

    /// List federation peers.
    pub async fn federation_peers(&self) -> Result<serde_json::Value, reqwest::Error> {
        self.get("/federation/peers").await
    }

    /// Add a federation peer.
    pub async fn add_federation_peer<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/federation/peers", body).await
    }

    /// Join a federation.
    pub async fn federation_join<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/federation/join", body).await
    }

    /// Leave the federation.
    pub async fn federation_leave<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/federation/leave", body).await
    }

    /// Get current federation status.
    pub async fn federation_status(&self) -> Result<serde_json::Value, reqwest::Error> {
        self.get("/federation/status").await
    }

    /// Fetch Prometheus metrics.
    pub async fn metrics(&self) -> Result<String, reqwest::Error> {
        let url = self.base_url.join("/metrics").expect("valid url");
        self.http.get(url).send().await?.text().await
    }
}
