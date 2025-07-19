#![doc = include_str!("../README.md")]

//! # ICN SDK
//!
//! This crate provides a high-level Rust SDK for interacting with InterCooperative Network (ICN) nodes
//! via their HTTP API. It offers a convenient, type-safe interface for building applications that
//! integrate with ICN.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use icn_sdk::IcnClient;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = IcnClient::new("http://localhost:8080")?;
//!
//! // Get node information
//! let info = client.info().await?;
//! println!("Node: {} v{}", info.name, info.version);
//!
//! // Check health
//! let health = client.health().await?;
//! println!("Health: {}", health.status);
//! # Ok(())
//! # }
//! ```

use icn_common::{NodeInfo, NodeStatus};
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use url::Url;

/// Health status information from an ICN node.
///
/// This structure represents the health check response from an ICN node,
/// including overall status and individual component checks.
#[derive(Debug, Clone, Deserialize)]
pub struct HealthStatus {
    /// Overall health status (typically "healthy" or "unhealthy").
    pub status: String,
    /// Unix timestamp when the health check was performed.
    pub timestamp: u64,
    /// Node uptime in seconds.
    pub uptime_seconds: u64,
    /// Individual component health checks.
    pub checks: HealthChecks,
}

/// Individual component health checks.
#[derive(Debug, Clone, Deserialize)]
pub struct HealthChecks {
    /// Runtime component health status.
    pub runtime: String,
    /// DAG store component health status.
    pub dag_store: String,
    /// Network component health status.
    pub network: String,
    /// Mana ledger component health status.
    pub mana_ledger: String,
}

/// Readiness status information from an ICN node.
///
/// This structure represents the readiness check response from an ICN node,
/// indicating whether the node is ready to serve requests.
#[derive(Debug, Clone, Deserialize)]
pub struct ReadinessStatus {
    /// Whether the node is ready to serve requests.
    pub ready: bool,
    /// Unix timestamp when the readiness check was performed.
    pub timestamp: u64,
    /// Individual readiness checks.
    pub checks: ReadinessChecks,
}

/// Individual readiness checks.
#[derive(Debug, Clone, Deserialize)]
pub struct ReadinessChecks {
    /// Whether the node can serve requests.
    pub can_serve_requests: bool,
    /// Whether the mana ledger is available.
    pub mana_ledger_available: bool,
    /// Whether the DAG store is available.
    pub dag_store_available: bool,
    /// Whether the network is initialized.
    pub network_initialized: bool,
}

/// HTTP client for interacting with ICN nodes.
///
/// This is the main interface for interacting with ICN nodes via their HTTP API.
/// It provides methods for all major ICN operations including mesh computing,
/// governance, DAG operations, and network management.
///
/// # Examples
///
/// ```rust,no_run
/// use icn_sdk::IcnClient;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create a new client
/// let client = IcnClient::new("http://localhost:8080")?;
///
/// // Get node information
/// let info = client.info().await?;
/// println!("Connected to {} v{}", info.name, info.version);
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct IcnClient {
    base_url: Url,
    http: Client,
}

impl IcnClient {
    /// Create a new client pointing at the given base URL.
    ///
    /// # Arguments
    ///
    /// * `base_url` - The base URL of the ICN node (e.g., "http://localhost:8080")
    ///
    /// # Returns
    ///
    /// * `Ok(IcnClient)` - A new client instance
    /// * `Err(url::ParseError)` - If the base URL is invalid
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use icn_sdk::IcnClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = IcnClient::new("http://localhost:8080")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(base_url: &str) -> Result<Self, url::ParseError> {
        let base_url = if base_url.ends_with('/') {
            base_url.trim_end_matches('/').to_string()
        } else {
            base_url.to_string()
        };

        let url = Url::parse(&base_url)?;
        let http = Client::new();

        Ok(Self {
            base_url: url,
            http,
        })
    }

    /// Create a new client with a custom HTTP client.
    ///
    /// # Arguments
    ///
    /// * `base_url` - The base URL of the ICN node
    /// * `client` - A custom reqwest::Client instance
    ///
    /// # Returns
    ///
    /// * `Ok(IcnClient)` - A new client instance
    /// * `Err(url::ParseError)` - If the base URL is invalid
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use icn_sdk::IcnClient;
    /// use reqwest::Client;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let custom_client = Client::builder()
    ///     .timeout(std::time::Duration::from_secs(30))
    ///     .build()?;
    /// let client = IcnClient::with_client("http://localhost:8080", custom_client)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_client(base_url: &str, client: Client) -> Result<Self, url::ParseError> {
        let url = Url::parse(base_url)?;
        Ok(Self {
            base_url: url,
            http: client,
        })
    }

    /// Get the base URL of this client.
    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    /// Helper method to make GET requests.
    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, reqwest::Error> {
        let url = self.base_url.join(path).unwrap();
        self.http.get(url).send().await?.json().await
    }

    /// Helper method to make POST requests.
    async fn post<Req: Serialize, Resp: DeserializeOwned>(
        &self,
        path: &str,
        body: &Req,
    ) -> Result<Resp, reqwest::Error> {
        let url = self.base_url.join(path).unwrap();
        self.http.post(url).json(body).send().await?.json().await
    }

    // === Node Information ===

    /// Get basic information about the ICN node.
    ///
    /// Returns basic information about the node including name, version, and capabilities.
    ///
    /// # Returns
    ///
    /// * `Ok(NodeInfo)` - Node information
    /// * `Err(reqwest::Error)` - If the request fails
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use icn_sdk::IcnClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = IcnClient::new("http://localhost:8080")?;
    /// let info = client.info().await?;
    /// println!("Node: {} v{}", info.name, info.version);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn info(&self) -> Result<NodeInfo, reqwest::Error> {
        self.get("/info").await
    }

    /// Get current node status.
    ///
    /// Returns the current operational status of the ICN node, including online status,
    /// peer count, and other operational metrics.
    ///
    /// # Returns
    ///
    /// * `Ok(NodeStatus)` - Node status information
    /// * `Err(reqwest::Error)` - If the request fails
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use icn_sdk::IcnClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = IcnClient::new("http://localhost:8080")?;
    /// let status = client.status().await?;
    /// println!("Node is online: {}", status.is_online);
    /// println!("Peer count: {}", status.peer_count);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn status(&self) -> Result<NodeStatus, reqwest::Error> {
        self.get("/status").await
    }

    /// Get health status of the ICN node.
    ///
    /// Returns detailed health information about the node and its components.
    ///
    /// # Returns
    ///
    /// * `Ok(HealthStatus)` - Health status information
    /// * `Err(reqwest::Error)` - If the request fails
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use icn_sdk::IcnClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = IcnClient::new("http://localhost:8080")?;
    /// let health = client.health().await?;
    /// println!("Health: {}", health.status);
    /// println!("Runtime: {}", health.checks.runtime);
    /// println!("DAG Store: {}", health.checks.dag_store);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn health(&self) -> Result<HealthStatus, reqwest::Error> {
        self.get("/health").await
    }

    /// Get readiness status of the ICN node.
    ///
    /// Returns information about whether the node is ready to serve requests.
    /// This is useful for load balancer health checks.
    ///
    /// # Returns
    ///
    /// * `Ok(ReadinessStatus)` - Readiness status information
    /// * `Err(reqwest::Error)` - If the request fails
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use icn_sdk::IcnClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = IcnClient::new("http://localhost:8080")?;
    /// let ready = client.ready().await?;
    /// if ready.ready {
    ///     println!("Node is ready to serve requests");
    /// } else {
    ///     println!("Node is not ready");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn ready(&self) -> Result<ReadinessStatus, reqwest::Error> {
        self.get("/ready").await
    }

    // === Mesh Computing ===

    /// Submit a mesh job for execution.
    ///
    /// Submits a computational job to the ICN mesh network for execution by available nodes.
    ///
    /// # Arguments
    ///
    /// * `body` - The job specification to submit
    ///
    /// # Returns
    ///
    /// * `Ok(serde_json::Value)` - Job submission response
    /// * `Err(reqwest::Error)` - If the request fails
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use icn_sdk::IcnClient;
    /// use serde_json::json;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = IcnClient::new("http://localhost:8080")?;
    /// let job_spec = json!({
    ///     "command": "echo 'Hello, ICN!'",
    ///     "max_cost": 1000,
    ///     "timeout": 300
    /// });
    /// let response = client.submit_mesh_job(&job_spec).await?;
    /// println!("Job submitted: {:?}", response);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn submit_mesh_job<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/mesh/job", body).await
    }

    /// List mesh jobs.
    ///
    /// Retrieves a list of mesh jobs, optionally filtered by status, submitter, or other criteria.
    ///
    /// # Returns
    ///
    /// * `Ok(serde_json::Value)` - List of mesh jobs
    /// * `Err(reqwest::Error)` - If the request fails
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use icn_sdk::IcnClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = IcnClient::new("http://localhost:8080")?;
    /// let jobs = client.list_mesh_jobs().await?;
    /// println!("Found {} jobs", jobs.as_array().unwrap().len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_mesh_jobs(&self) -> Result<serde_json::Value, reqwest::Error> {
        self.get("/mesh/jobs").await
    }

    /// Get details of a specific mesh job.
    ///
    /// Retrieves detailed information about a mesh job including its status, results, and execution history.
    ///
    /// # Arguments
    ///
    /// * `job_id` - The unique identifier of the job
    ///
    /// # Returns
    ///
    /// * `Ok(serde_json::Value)` - Job details
    /// * `Err(reqwest::Error)` - If the request fails
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use icn_sdk::IcnClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = IcnClient::new("http://localhost:8080")?;
    /// let job = client.mesh_job("job_12345").await?;
    /// println!("Job status: {}", job["status"]);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn mesh_job(&self, job_id: &str) -> Result<serde_json::Value, reqwest::Error> {
        self.get(&format!("/mesh/job/{job_id}")).await
    }

    /// Submit a mesh execution receipt.
    ///
    /// Submits a receipt confirming the execution of a mesh job. This is typically called
    /// by executor nodes to report job completion.
    ///
    /// # Arguments
    ///
    /// * `body` - The execution receipt to submit
    ///
    /// # Returns
    ///
    /// * `Ok(serde_json::Value)` - Receipt submission response
    /// * `Err(reqwest::Error)` - If the request fails
    pub async fn submit_mesh_receipt<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/mesh/receipt", body).await
    }

    /// Submit a mesh stub bid.
    ///
    /// Submits a bid for executing a mesh job. This is typically called by executor nodes
    /// to indicate their willingness and capability to execute a job.
    ///
    /// # Arguments
    ///
    /// * `body` - The bid to submit
    ///
    /// # Returns
    ///
    /// * `Ok(serde_json::Value)` - Bid submission response
    /// * `Err(reqwest::Error)` - If the request fails
    pub async fn mesh_stub_bid<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/mesh/stub/bid", body).await
    }

    /// Submit a mesh stub receipt.
    ///
    /// Submits a stub receipt for testing or development purposes.
    ///
    /// # Arguments
    ///
    /// * `body` - The stub receipt to submit
    ///
    /// # Returns
    ///
    /// * `Ok(serde_json::Value)` - Stub receipt submission response
    /// * `Err(reqwest::Error)` - If the request fails
    pub async fn mesh_stub_receipt<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/mesh/stub/receipt", body).await
    }

    // === Governance ===

    /// List governance proposals.
    ///
    /// Retrieves a list of governance proposals, including their status, votes, and details.
    ///
    /// # Returns
    ///
    /// * `Ok(serde_json::Value)` - List of governance proposals
    /// * `Err(reqwest::Error)` - If the request fails
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use icn_sdk::IcnClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = IcnClient::new("http://localhost:8080")?;
    /// let proposals = client.list_proposals().await?;
    /// println!("Found {} proposals", proposals.as_array().unwrap().len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_proposals(&self) -> Result<serde_json::Value, reqwest::Error> {
        self.get("/governance/proposals").await
    }

    /// Get details of a specific governance proposal.
    ///
    /// Retrieves detailed information about a governance proposal including its description,
    /// voting results, and current status.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the proposal
    ///
    /// # Returns
    ///
    /// * `Ok(serde_json::Value)` - Proposal details
    /// * `Err(reqwest::Error)` - If the request fails
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use icn_sdk::IcnClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = IcnClient::new("http://localhost:8080")?;
    /// let proposal = client.proposal("prop_12345").await?;
    /// println!("Proposal title: {}", proposal["title"]);
    /// println!("Status: {}", proposal["status"]);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn proposal(&self, id: &str) -> Result<serde_json::Value, reqwest::Error> {
        self.get(&format!("/governance/proposal/{id}")).await
    }

    /// Submit a governance proposal.
    ///
    /// Submits a new governance proposal for community voting. The proposal can be for
    /// parameter changes, protocol updates, or other governance actions.
    ///
    /// # Arguments
    ///
    /// * `body` - The proposal to submit
    ///
    /// # Returns
    ///
    /// * `Ok(serde_json::Value)` - Proposal submission response
    /// * `Err(reqwest::Error)` - If the request fails
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use icn_sdk::IcnClient;
    /// use serde_json::json;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = IcnClient::new("http://localhost:8080")?;
    /// let proposal = json!({
    ///     "title": "Increase block size limit",
    ///     "description": "Proposal to increase the maximum block size...",
    ///     "type": "parameter_change"
    /// });
    /// let response = client.submit_proposal(&proposal).await?;
    /// println!("Proposal submitted: {:?}", response);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn submit_proposal<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/governance/proposal", body).await
    }

    /// Cast a vote on a governance proposal.
    ///
    /// Submits a vote for or against a governance proposal. The vote is weighted based
    /// on the voter's stake and reputation in the network.
    ///
    /// # Arguments
    ///
    /// * `body` - The vote to cast
    ///
    /// # Returns
    ///
    /// * `Ok(serde_json::Value)` - Vote submission response
    /// * `Err(reqwest::Error)` - If the request fails
    pub async fn cast_vote<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/governance/vote", body).await
    }

    /// Delegate voting power to another participant.
    ///
    /// Allows a participant to delegate their voting power to another trusted participant
    /// for governance decisions.
    ///
    /// # Arguments
    ///
    /// * `body` - The delegation configuration
    ///
    /// # Returns
    ///
    /// * `Ok(serde_json::Value)` - Delegation response
    /// * `Err(reqwest::Error)` - If the request fails
    pub async fn delegate<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/governance/delegate", body).await
    }

    /// Revoke a voting delegation.
    ///
    /// Revokes a previously granted voting delegation, returning voting power to the delegator.
    ///
    /// # Arguments
    ///
    /// * `body` - The revocation request
    ///
    /// # Returns
    ///
    /// * `Ok(serde_json::Value)` - Revocation response
    /// * `Err(reqwest::Error)` - If the request fails
    pub async fn revoke<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/governance/revoke", body).await
    }

    /// Close voting on a proposal.
    ///
    /// Closes the voting period for a governance proposal and tallies the final results.
    /// This is typically called when the voting period expires.
    ///
    /// # Arguments
    ///
    /// * `body` - The close vote request
    ///
    /// # Returns
    ///
    /// * `Ok(serde_json::Value)` - Close vote response
    /// * `Err(reqwest::Error)` - If the request fails
    pub async fn close_vote<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/governance/close", body).await
    }

    /// Execute a passed governance proposal.
    ///
    /// Executes the changes specified in a governance proposal that has passed voting.
    /// This applies the proposal's changes to the network.
    ///
    /// # Arguments
    ///
    /// * `body` - The execution request
    ///
    /// # Returns
    ///
    /// * `Ok(serde_json::Value)` - Execution response
    /// * `Err(reqwest::Error)` - If the request fails
    pub async fn execute<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/governance/execute", body).await
    }

    /// Revoke a credential by CID.
    pub async fn revoke_credential<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/identity/credentials/revoke", body).await
    }

    /// Verify a revocation proof.
    pub async fn verify_revocation<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/identity/verify/revocation", body).await
    }

    // === DAG Operations ===

    /// Store data in the DAG.
    ///
    /// Stores data in the content-addressed DAG storage system, returning a content identifier (CID).
    ///
    /// # Arguments
    ///
    /// * `body` - The data to store
    ///
    /// # Returns
    ///
    /// * `Ok(serde_json::Value)` - Storage response with CID
    /// * `Err(reqwest::Error)` - If the request fails
    pub async fn dag_put<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/dag/put", body).await
    }

    /// Retrieve data from the DAG.
    ///
    /// Retrieves data from the DAG storage system using its content identifier (CID).
    ///
    /// # Arguments
    ///
    /// * `body` - The retrieval request containing the CID
    ///
    /// # Returns
    ///
    /// * `Ok(serde_json::Value)` - The retrieved data
    /// * `Err(reqwest::Error)` - If the request fails
    pub async fn dag_get<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/dag/get", body).await
    }

    /// Get metadata for DAG content.
    ///
    /// Retrieves metadata information about content stored in the DAG without retrieving the content itself.
    ///
    /// # Arguments
    ///
    /// * `body` - The metadata request
    ///
    /// # Returns
    ///
    /// * `Ok(serde_json::Value)` - Content metadata
    /// * `Err(reqwest::Error)` - If the request fails
    pub async fn dag_meta<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/dag/meta", body).await
    }

    /// Pin content in the DAG.
    ///
    /// Pins content in the DAG to prevent it from being garbage collected.
    ///
    /// # Arguments
    ///
    /// * `body` - The pin request
    ///
    /// # Returns
    ///
    /// * `Ok(serde_json::Value)` - Pin response
    /// * `Err(reqwest::Error)` - If the request fails
    pub async fn dag_pin<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/dag/pin", body).await
    }

    /// Unpin content in the DAG.
    ///
    /// Removes a pin from DAG content, allowing it to be garbage collected if not referenced elsewhere.
    ///
    /// # Arguments
    ///
    /// * `body` - The unpin request
    ///
    /// # Returns
    ///
    /// * `Ok(serde_json::Value)` - Unpin response
    /// * `Err(reqwest::Error)` - If the request fails
    pub async fn dag_unpin<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/dag/unpin", body).await
    }

    /// Prune the DAG.
    ///
    /// Removes unpinned and unreferenced content from the DAG to free up storage space.
    ///
    /// # Arguments
    ///
    /// * `body` - The prune request
    ///
    /// # Returns
    ///
    /// * `Ok(serde_json::Value)` - Prune response
    /// * `Err(reqwest::Error)` - If the request fails
    pub async fn dag_prune<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/dag/prune", body).await
    }

    /// Get the root CID of the DAG.
    ///
    /// Returns the root content identifier of the DAG, which represents the current state.
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The root CID
    /// * `Err(reqwest::Error)` - If the request fails
    pub async fn dag_root(&self) -> Result<String, reqwest::Error> {
        self.get("/dag/root").await
    }

    // === Network Operations ===

    /// Get the local peer ID.
    ///
    /// Returns the peer ID of the local node in the network.
    ///
    /// # Returns
    ///
    /// * `Ok(serde_json::Value)` - Local peer ID information
    /// * `Err(reqwest::Error)` - If the request fails
    pub async fn local_peer_id(&self) -> Result<serde_json::Value, reqwest::Error> {
        self.get("/network/local_peer_id").await
    }

    /// Connect to a peer.
    ///
    /// Initiates a connection to a specific peer in the network.
    ///
    /// # Arguments
    ///
    /// * `body` - The connection request
    ///
    /// # Returns
    ///
    /// * `Ok(serde_json::Value)` - Connection response
    /// * `Err(reqwest::Error)` - If the request fails
    pub async fn connect_peer<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/network/connect", body).await
    }

    /// List connected peers.
    ///
    /// Returns a list of currently connected peers in the network.
    ///
    /// # Returns
    ///
    /// * `Ok(serde_json::Value)` - List of connected peers
    /// * `Err(reqwest::Error)` - If the request fails
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use icn_sdk::IcnClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = IcnClient::new("http://localhost:8080")?;
    /// let peers = client.peers().await?;
    /// println!("Connected to {} peers", peers.as_array().unwrap().len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn peers(&self) -> Result<serde_json::Value, reqwest::Error> {
        self.get("/network/peers").await
    }

    // === Economics ===

    /// Get mana account information.
    ///
    /// Retrieves mana balance and account information for a specific DID.
    ///
    /// # Arguments
    ///
    /// * `did` - The decentralized identifier of the account
    ///
    /// # Returns
    ///
    /// * `Ok(serde_json::Value)` - Mana account information
    /// * `Err(reqwest::Error)` - If the request fails
    pub async fn account_mana(&self, did: &str) -> Result<serde_json::Value, reqwest::Error> {
        self.get(&format!("/account/mana/{did}")).await
    }

    /// Get cryptographic keys.
    ///
    /// Retrieves the cryptographic keys associated with this node.
    ///
    /// # Returns
    ///
    /// * `Ok(serde_json::Value)` - Key information
    /// * `Err(reqwest::Error)` - If the request fails
    pub async fn keys(&self) -> Result<serde_json::Value, reqwest::Error> {
        self.get("/keys").await
    }

    /// Get reputation score.
    ///
    /// Retrieves the reputation score for a specific DID.
    ///
    /// # Arguments
    ///
    /// * `did` - The decentralized identifier
    ///
    /// # Returns
    ///
    /// * `Ok(serde_json::Value)` - Reputation information
    /// * `Err(reqwest::Error)` - If the request fails
    pub async fn reputation(&self, did: &str) -> Result<serde_json::Value, reqwest::Error> {
        self.get(&format!("/reputation/{did}")).await
    }

    /// Submit a transaction.
    ///
    /// Submits a transaction to the network for processing.
    ///
    /// # Arguments
    ///
    /// * `body` - The transaction to submit
    ///
    /// # Returns
    ///
    /// * `Ok(serde_json::Value)` - Transaction submission response
    /// * `Err(reqwest::Error)` - If the request fails
    pub async fn submit_transaction<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/transaction", body).await
    }

    /// List token classes.
    ///
    /// Retrieves a list of token classes available in the network.
    ///
    /// # Returns
    ///
    /// * `Ok(serde_json::Value)` - List of token classes
    /// * `Err(reqwest::Error)` - If the request fails
    pub async fn token_classes(&self) -> Result<serde_json::Value, reqwest::Error> {
        self.get("/token_classes").await
    }

    /// Create a new token class.
    ///
    /// Creates a new class of tokens with specific properties and rules.
    ///
    /// # Arguments
    ///
    /// * `body` - The token class definition
    ///
    /// # Returns
    ///
    /// * `Ok(serde_json::Value)` - Token class creation response
    /// * `Err(reqwest::Error)` - If the request fails
    pub async fn create_token_class<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/token_class", body).await
    }

    /// Mint tokens.
    ///
    /// Mints new tokens of a specific class to a designated recipient.
    ///
    /// # Arguments
    ///
    /// * `body` - The mint request
    ///
    /// # Returns
    ///
    /// * `Ok(serde_json::Value)` - Mint response
    /// * `Err(reqwest::Error)` - If the request fails
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use icn_sdk::IcnClient;
    /// use serde_json::json;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = IcnClient::new("http://localhost:8080")?;
    /// let mint_request = json!({
    ///     "token_class": "my_token_class",
    ///     "amount": 1000,
    ///     "recipient": "did:example:alice"
    /// });
    /// let response = client.mint_tokens(&mint_request).await?;
    /// println!("Minted tokens: {:?}", response);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn mint_tokens<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/mint", body).await
    }

    /// Transfer tokens.
    ///
    /// Transfers tokens from one account to another.
    ///
    /// # Arguments
    ///
    /// * `body` - The transfer request
    ///
    /// # Returns
    ///
    /// * `Ok(serde_json::Value)` - Transfer response
    /// * `Err(reqwest::Error)` - If the request fails
    pub async fn transfer_tokens<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/transfer", body).await
    }

    /// Burn tokens.
    ///
    /// Permanently removes tokens from circulation.
    ///
    /// # Arguments
    ///
    /// * `body` - The burn request
    ///
    /// # Returns
    ///
    /// * `Ok(serde_json::Value)` - Burn response
    /// * `Err(reqwest::Error)` - If the request fails
    pub async fn burn_tokens<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/burn", body).await
    }

    /// Query data.
    ///
    /// Performs a data query against the network's data stores.
    ///
    /// # Arguments
    ///
    /// * `body` - The query request
    ///
    /// # Returns
    ///
    /// * `Ok(serde_json::Value)` - Query results
    /// * `Err(reqwest::Error)` - If the request fails
    pub async fn data_query<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/data/query", body).await
    }

    /// Upload a contract.
    ///
    /// Uploads a smart contract or governance policy to the network.
    ///
    /// # Arguments
    ///
    /// * `body` - The contract upload request
    ///
    /// # Returns
    ///
    /// * `Ok(serde_json::Value)` - Upload response
    /// * `Err(reqwest::Error)` - If the request fails
    pub async fn upload_contract<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/contract/upload", body).await
    }

    // === Federation ===

    /// List federation peers.
    ///
    /// Returns a list of peers in the current federation.
    ///
    /// # Returns
    ///
    /// * `Ok(serde_json::Value)` - List of federation peers
    /// * `Err(reqwest::Error)` - If the request fails
    pub async fn federation_peers(&self) -> Result<serde_json::Value, reqwest::Error> {
        self.get("/federation/peers").await
    }

    /// Add a peer to the federation.
    ///
    /// Adds a new peer to the federation with the specified role and permissions.
    ///
    /// # Arguments
    ///
    /// * `body` - The add peer request
    ///
    /// # Returns
    ///
    /// * `Ok(serde_json::Value)` - Add peer response
    /// * `Err(reqwest::Error)` - If the request fails
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use icn_sdk::IcnClient;
    /// use serde_json::json;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = IcnClient::new("http://localhost:8080")?;
    /// let add_peer_request = json!({
    ///     "peer_id": "12D3KooWExample",
    ///     "address": "/ip4/192.168.1.100/tcp/4001",
    ///     "role": "validator"
    /// });
    /// let response = client.add_federation_peer(&add_peer_request).await?;
    /// println!("Added peer: {:?}", response);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn add_federation_peer<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/federation/peer", body).await
    }

    /// Join a federation.
    ///
    /// Requests to join an existing federation.
    ///
    /// # Arguments
    ///
    /// * `body` - The join request
    ///
    /// # Returns
    ///
    /// * `Ok(serde_json::Value)` - Join response
    /// * `Err(reqwest::Error)` - If the request fails
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use icn_sdk::IcnClient;
    /// use serde_json::json;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = IcnClient::new("http://localhost:8080")?;
    /// let join_request = json!({
    ///     "federation_id": "fed_12345",
    ///     "credentials": "..."
    /// });
    /// let response = client.federation_join(&join_request).await?;
    /// println!("Join response: {:?}", response);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn federation_join<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/federation/join", body).await
    }

    /// Leave a federation.
    ///
    /// Requests to leave the current federation.
    ///
    /// # Arguments
    ///
    /// * `body` - The leave request
    ///
    /// # Returns
    ///
    /// * `Ok(serde_json::Value)` - Leave response
    /// * `Err(reqwest::Error)` - If the request fails
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use icn_sdk::IcnClient;
    /// use serde_json::json;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = IcnClient::new("http://localhost:8080")?;
    /// let leave_request = json!({
    ///     "reason": "Planned maintenance"
    /// });
    /// let response = client.federation_leave(&leave_request).await?;
    /// println!("Leave response: {:?}", response);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn federation_leave<B: Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.post("/federation/leave", body).await
    }

    /// Get federation status.
    ///
    /// Returns the current status of the federation including membership and health.
    ///
    /// # Returns
    ///
    /// * `Ok(serde_json::Value)` - Federation status
    /// * `Err(reqwest::Error)` - If the request fails
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use icn_sdk::IcnClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = IcnClient::new("http://localhost:8080")?;
    /// let status = client.federation_status().await?;
    /// println!("Federation status: {:?}", status);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn federation_status(&self) -> Result<serde_json::Value, reqwest::Error> {
        self.get("/federation/status").await
    }

    // === Metrics ===

    /// Get Prometheus metrics.
    ///
    /// Returns metrics in Prometheus format for monitoring and observability.
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - Prometheus metrics data
    /// * `Err(reqwest::Error)` - If the request fails
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use icn_sdk::IcnClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = IcnClient::new("http://localhost:8080")?;
    /// let metrics = client.metrics().await?;
    /// println!("Metrics data length: {} bytes", metrics.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn metrics(&self) -> Result<String, reqwest::Error> {
        let url = self.base_url.join("/metrics").unwrap();
        self.http.get(url).send().await?.text().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::time::Duration;

    #[test]
    fn test_client_creation() {
        let client = IcnClient::new("http://localhost:8080").unwrap();
        assert_eq!(client.base_url().as_str(), "http://localhost:8080/");
    }

    #[test]
    fn test_client_creation_with_trailing_slash() {
        let client = IcnClient::new("http://localhost:8080/").unwrap();
        assert_eq!(client.base_url().as_str(), "http://localhost:8080/");
    }

    #[test]
    fn test_client_creation_invalid_url() {
        let result = IcnClient::new("not-a-url");
        assert!(result.is_err());
    }

    #[test]
    fn test_client_creation_with_custom_client() {
        let custom_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();

        let client = IcnClient::with_client("http://localhost:8080", custom_client).unwrap();
        assert_eq!(client.base_url().as_str(), "http://localhost:8080/");
    }

    #[test]
    fn test_client_clone() {
        let client = IcnClient::new("http://localhost:8080").unwrap();
        let cloned = client.clone();
        assert_eq!(client.base_url(), cloned.base_url());
    }

    #[tokio::test]
    async fn test_client_url_construction() {
        let client = IcnClient::new("http://localhost:8080").unwrap();

        // Test that URLs are constructed correctly
        let url = client.base_url.join("/info").unwrap();
        assert_eq!(url.as_str(), "http://localhost:8080/info");

        let url = client.base_url.join("/mesh/jobs").unwrap();
        assert_eq!(url.as_str(), "http://localhost:8080/mesh/jobs");
    }

    // Tests for error handling and edge cases
    #[tokio::test]
    async fn test_client_connection_error() {
        // Use a port that should not be in use
        let client = IcnClient::new("http://localhost:9999").unwrap();

        let result = client.info().await;
        assert!(result.is_err());

        // Check that it's a connection error
        let error = result.unwrap_err();
        assert!(error.is_connect() || error.is_timeout());
    }

    #[tokio::test]
    async fn test_client_serialization() {
        // Test that the client can handle various data types
        let client = IcnClient::new("http://localhost:8080").unwrap();

        // Test with a simple JSON object
        let test_data = json!({
            "test": "value",
            "number": 42
        });

        // This will fail with a connection error, but it tests serialization
        let result = client.submit_mesh_job(&test_data).await;
        assert!(result.is_err());

        // Verify it's a connection error, not a serialization error
        let error = result.unwrap_err();
        assert!(error.is_connect() || error.is_timeout());
    }
}
