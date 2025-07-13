#![doc = include_str!("../README.md")]
#![allow(clippy::uninlined_format_args)]

//! # ICN CLI Crate
//! This crate provides a command-line interface (CLI) for interacting with an ICN HTTP node.

use base64::{self, Engine};
extern crate bincode;
use clap::{Parser, Subcommand};
use icn_common::CommonError;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue; // For generic JSON data if needed
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::exit; // Added for reading from stdin

// Types from our ICN crates that CLI will interact with (serialize/deserialize)
// These types are expected to be sent to/received from the icn-node HTTP API.
use icn_common::{Cid, DagBlock, Did, NodeInfo, NodeStatus, ZkCredentialProof, ZkProofType};
// Using aliased request structs from icn-api for clarity, these are what the node expects
use icn_api::governance_trait::{
    CastVoteRequest as ApiCastVoteRequest, SubmitProposalRequest as ApiSubmitProposalRequest,
};
use icn_ccl::{check_ccl_file, compile_ccl_file, compile_ccl_file_to_wasm, explain_ccl_policy};
use icn_governance::{Proposal, ProposalId};
use icn_identity::generate_ed25519_keypair;
use icn_runtime::context::{Ed25519Signer, Signer};

fn anyhow_to_common(e: anyhow::Error) -> CommonError {
    if let Some(c) = e.downcast_ref::<CommonError>() {
        c.clone()
    } else {
        CommonError::UnknownError(e.to_string())
    }
}

fn collect_block_files(dir: &std::path::Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            collect_block_files(&path, out)?;
        } else if entry.file_type()?.is_file() {
            out.push(path);
        }
    }
    Ok(())
}

// --- CLI Argument Parsing ---

#[derive(Parser, Debug)]
#[clap(author, version, about = "ICN Command Line Interface")]
struct Cli {
    #[clap(
        long,
        global = true,
        default_value = "http://127.0.0.1:7845",
        help = "Base URL of the ICN node API"
    )]
    api_url: String,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Get node information (version, name, status message)
    Info,
    /// Get node status (online, peers, block height)
    Status,
    /// Fetch Prometheus metrics text
    Metrics,
    /// DAG block operations
    Dag {
        #[clap(subcommand)]
        command: DagCommands,
    },
    /// Governance operations
    Governance {
        #[clap(subcommand)]
        command: GovernanceCommands,
    },
    /// Mesh job operations
    Mesh {
        #[clap(subcommand)]
        command: MeshCommands,
    },
    /// Network operations
    Network {
        #[clap(subcommand)]
        command: NetworkCommands,
    },
    /// Account queries
    Accounts {
        #[clap(subcommand)]
        command: AccountCommands,
    },
    /// Key management
    Keys {
        #[clap(subcommand)]
        command: KeyCommands,
    },
    /// Reputation queries
    Reputation {
        #[clap(subcommand)]
        command: ReputationCommands,
    },
    /// Identity operations
    Identity {
        #[clap(subcommand)]
        command: IdentityCommands,
    },
    /// Cooperative Contract Language operations
    Ccl {
        #[clap(subcommand)]
        command: CclCommands,
    },
    /// Zero-knowledge tooling
    Zk {
        #[clap(subcommand)]
        command: ZkCommands,
    },
    /// Compile a CCL file to WASM and upload to the node
    #[clap(name = "compile-ccl")]
    CompileCcl {
        #[clap(help = "Path to the CCL source file")]
        file: String,
    },
    /// Submit a mesh job (JSON string or '-' for stdin)
    #[clap(name = "submit-job")]
    SubmitJob {
        #[clap(help = "Mesh job submission JSON or '-' for stdin")]
        job_request_json_or_stdin: String,
    },
    /// Query mesh job status by ID
    #[clap(name = "job-status")]
    JobStatus {
        #[clap(help = "Job ID (CID string)")]
        job_id: String,
    },
    /// Federation management commands
    Federation {
        #[clap(subcommand)]
        command: FederationCommands,
    },
}

#[derive(Subcommand, Debug)]
enum DagCommands {
    /// Store a DAG block (provide block as JSON string, or '-' to read from stdin)
    Put {
        #[clap(help = "DAG block content as a JSON string, or '-' to read from stdin")]
        block_json_or_stdin: String,
    },
    /// Retrieve a DAG block by its CID (provide CID as JSON string)
    Get {
        #[clap(help = "CID of the block to retrieve, as a JSON string")]
        cid_json: String,
    },
    /// Retrieve metadata for a DAG block by CID
    Meta {
        #[clap(help = "CID of the block to inspect, as a JSON string")]
        cid_json: String,
    },
    /// Backup the DAG store to the specified directory
    Backup {
        #[clap(help = "Path to store the backup data")]
        path: String,
    },
    /// Restore the DAG store from the specified directory
    Restore {
        #[clap(help = "Path of the backup to restore from")]
        path: String,
    },
    /// Verify blocks in the DAG store
    Verify {
        #[clap(long, help = "Verify all blocks, not just a sample")]
        full: bool,
    },
    /// Pin a DAG block with optional TTL
    Pin {
        #[clap(help = "CID of the block to pin as JSON string")]
        cid_json: String,
        #[clap(long, help = "Optional TTL in seconds")]
        ttl: Option<u64>,
    },
    /// Unpin a DAG block
    Unpin {
        #[clap(help = "CID of the block to unpin as JSON string")]
        cid_json: String,
    },
    /// Prune expired blocks
    Prune,
}

#[derive(Subcommand, Debug)]
enum GovernanceCommands {
    /// Submit a new proposal (provide proposal request as JSON string, or '-' to read from stdin)
    Submit {
        #[clap(
            help = "Proposal submission request as a JSON string (ApiSubmitProposalRequest format), or '-' to read from stdin"
        )]
        proposal_request_json_or_stdin: String,
    },
    /// Cast a vote on a proposal (provide vote request as JSON string)
    Vote {
        #[clap(help = "Vote casting request as a JSON string (ApiCastVoteRequest format)")]
        vote_request_json: String,
    },
    /// Tally votes and close a proposal
    Tally {
        #[clap(help = "ID of the proposal to tally and close")]
        id: String,
    },
    /// List all proposals
    Proposals,
    /// Get a specific proposal by its ID
    Proposal {
        #[clap(help = "ID of the proposal to retrieve")]
        id: String,
    },
}

#[derive(Subcommand, Debug)]
enum MeshCommands {
    /// List all mesh jobs
    Jobs,
    /// Get status for a specific job
    Status {
        #[clap(help = "Job ID (CID string)")]
        job_id: String,
    },
    /// Submit a new mesh job
    Submit {
        #[clap(help = "Mesh job submission request as a JSON string, or '-' to read from stdin")]
        job_request_json_or_stdin: String,
    },
}

#[derive(Subcommand, Debug)]
enum NetworkCommands {
    /// Display network statistics
    Stats,
    /// Send a ping to a peer
    Ping {
        #[clap(help = "Target peer ID")]
        peer_id: String,
    },
    /// Fetch local peer ID and list of discovered peers
    Peers,
}

#[derive(Subcommand, Debug)]
enum CclCommands {
    /// Compile a CCL source file
    Compile {
        #[clap(help = "Path to the CCL source file")]
        file: String,
    },
    /// Lint a CCL source file for errors
    Lint {
        #[clap(help = "Path to the CCL source file")]
        file: String,
    },
    /// Explain constructs within a CCL policy
    Explain {
        #[clap(help = "Path to the CCL source file")]
        file: String,
        #[clap(long, help = "Specific function or rule to explain")]
        target: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum FederationCommands {
    /// Join a federation by specifying a peer
    #[clap(name = "join")]
    Join {
        #[clap(help = "Peer identifier string")]
        peer: String,
    },
    /// Leave a federation
    #[clap(name = "leave")]
    Leave {
        #[clap(help = "Peer identifier string")]
        peer: String,
    },
    /// List peers known to the node
    #[clap(name = "list-peers")]
    ListPeers,
    /// Show federation status
    #[clap(name = "status")]
    Status,
}

#[derive(Subcommand, Debug)]
enum AccountCommands {
    /// Get mana balance for an account
    Balance {
        #[clap(help = "Target account DID")]
        did: String,
    },
}

#[derive(Subcommand, Debug)]
enum KeyCommands {
    /// Show node DID and public key
    Show,
}

#[derive(Subcommand, Debug)]
enum ReputationCommands {
    /// Get reputation score for an identity
    Get {
        #[clap(help = "Target DID")]
        did: String,
    },
}

#[derive(Subcommand, Debug)]
enum IdentityCommands {
    /// Verify a zero-knowledge credential proof (JSON string or '-' for stdin)
    VerifyProof {
        #[clap(help = "ZkCredentialProof JSON or '-' for stdin")]
        proof_json_or_stdin: String,
    },
    /// Generate a dummy zero-knowledge credential proof
    GenerateProof {
        #[clap(long, help = "Issuer DID string")]
        issuer: String,
        #[clap(long, help = "Holder DID string")]
        holder: String,
        #[clap(long, help = "Claim type tag")]
        claim_type: String,
        #[clap(long, help = "Credential schema CID string")]
        schema: String,
        #[clap(long, help = "Proof backend (groth16|bulletproofs|other:<name>)")]
        backend: String,
        #[clap(long, help = "Hex-encoded verification key bytes", required = false)]
        verification_key: Option<String>,
        #[clap(long, help = "Public inputs as JSON string", required = false)]
        public_inputs: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum ZkCommands {
    /// Generate a Groth16 proving key and sign the verifying key
    #[clap(name = "generate-key")]
    GenerateKey,
}

// --- Main CLI Logic ---

#[tokio::main]
async fn main() {
    env_logger::init();
    let cli = Cli::parse();
    let client = Client::new();

    if let Err(e) = run_command(&cli, &client).await {
        let err = anyhow_to_common(e);
        eprintln!("Error: {}", err);
        exit(1);
    }
}

async fn run_command(cli: &Cli, client: &Client) -> Result<(), anyhow::Error> {
    match &cli.command {
        Commands::Info => handle_info(cli, client).await?,
        Commands::Status => handle_status(cli, client).await?,
        Commands::Metrics => handle_metrics(cli, client).await?,
        Commands::Dag { command } => match command {
            DagCommands::Put {
                block_json_or_stdin,
            } => handle_dag_put(cli, client, block_json_or_stdin).await?,
            DagCommands::Get { cid_json } => handle_dag_get(cli, client, cid_json).await?,
            DagCommands::Meta { cid_json } => handle_dag_meta(cli, client, cid_json).await?,
            DagCommands::Backup { path } => handle_dag_backup(path)?,
            DagCommands::Restore { path } => handle_dag_restore(path)?,
            DagCommands::Verify { full } => handle_dag_verify(*full)?,
            DagCommands::Pin { cid_json, ttl } => {
                handle_dag_pin(cli, client, cid_json, *ttl).await?
            }
            DagCommands::Unpin { cid_json } => handle_dag_unpin(cli, client, cid_json).await?,
            DagCommands::Prune => handle_dag_prune(cli, client).await?,
        },
        Commands::Governance { command } => match command {
            GovernanceCommands::Submit {
                proposal_request_json_or_stdin,
            } => handle_gov_submit(cli, client, proposal_request_json_or_stdin).await?,
            GovernanceCommands::Vote { vote_request_json } => {
                handle_gov_vote(cli, client, vote_request_json).await?
            }
            GovernanceCommands::Tally { id } => handle_gov_tally(cli, client, id).await?,
            GovernanceCommands::Proposals => handle_gov_list_proposals(cli, client).await?,
            GovernanceCommands::Proposal { id } => handle_gov_get_proposal(cli, client, id).await?,
        },
        Commands::Mesh { command } => match command {
            MeshCommands::Jobs => handle_mesh_jobs(cli, client).await?,
            MeshCommands::Status { job_id } => handle_mesh_status(cli, client, job_id).await?,
            MeshCommands::Submit {
                job_request_json_or_stdin,
            } => handle_mesh_submit(cli, client, job_request_json_or_stdin).await?,
        },
        Commands::Network { command } => match command {
            NetworkCommands::Stats => handle_network_stats(cli, client).await?,
            NetworkCommands::Ping { peer_id } => handle_network_ping(cli, client, peer_id).await?,
            NetworkCommands::Peers => handle_network_peers(cli, client).await?,
        },
        Commands::Accounts { command } => match command {
            AccountCommands::Balance { did } => handle_account_balance(cli, client, did).await?,
        },
        Commands::Keys { command } => match command {
            KeyCommands::Show => handle_keys_show(cli, client).await?,
        },
        Commands::Reputation { command } => match command {
            ReputationCommands::Get { did } => handle_reputation_get(cli, client, did).await?,
        },
        Commands::Identity { command } => match command {
            IdentityCommands::VerifyProof {
                proof_json_or_stdin,
            } => handle_identity_verify(cli, client, proof_json_or_stdin).await?,
            IdentityCommands::GenerateProof {
                issuer,
                holder,
                claim_type,
                schema,
                backend,
                verification_key,
                public_inputs,
            } => {
                handle_identity_generate_inner(
                    issuer,
                    holder,
                    claim_type,
                    schema,
                    backend,
                    verification_key,
                    public_inputs,
                )?;
            }
        },
        Commands::Ccl { command } => match command {
            CclCommands::Compile { file } => handle_ccl_compile(file)?,
            CclCommands::Lint { file } => handle_ccl_lint(file)?,
            CclCommands::Explain { file, target } => handle_ccl_explain(file, target).await?,
        },
        Commands::Zk { command } => match command {
            ZkCommands::GenerateKey => handle_zk_generate_key().await?,
        },
        Commands::CompileCcl { file } => handle_compile_ccl_upload(cli, client, file).await?,
        Commands::SubmitJob {
            job_request_json_or_stdin,
        } => handle_mesh_submit(cli, client, job_request_json_or_stdin).await?,
        Commands::JobStatus { job_id } => handle_mesh_status(cli, client, job_id).await?,
        Commands::Federation { command } => match command {
            FederationCommands::Join { peer } => handle_fed_join(cli, client, peer).await?,
            FederationCommands::Leave { peer } => handle_fed_leave(cli, client, peer).await?,
            FederationCommands::ListPeers => handle_fed_list_peers(cli, client).await?,
            FederationCommands::Status => handle_fed_status(cli, client).await?,
        },
    }
    Ok(())
}

// --- HTTP Helper Functions ---

async fn get_request<T: for<'de> Deserialize<'de>>(
    api_url: &str,
    client: &Client,
    path: &str,
) -> Result<T, anyhow::Error> {
    let url = format!("{}{}", api_url, path);
    let res = icn_common::retry_with_backoff(
        || async {
            client
                .get(&url)
                .send()
                .await
                .map_err(|e| anyhow::anyhow!(e))
        },
        3,
        std::time::Duration::from_millis(100),
        std::time::Duration::from_secs(2),
    )
    .await?;

    if res.status().is_success() {
        let body = res.json::<T>().await?;
        Ok(body)
    } else {
        let status = res.status();
        let error_text = res
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read error body".to_string());
        Err(anyhow::anyhow!(
            "Request failed with status {}: {}\nURL: {}",
            status,
            error_text,
            url
        ))
    }
}

async fn post_request<S: Serialize, T: for<'de> Deserialize<'de>>(
    api_url: &str,
    client: &Client,
    path: &str,
    body: &S,
) -> Result<T, anyhow::Error> {
    let url = format!("{}{}", api_url, path);
    let res = icn_common::retry_with_backoff(
        || async {
            client
                .post(&url)
                .json(body)
                .send()
                .await
                .map_err(|e| anyhow::anyhow!(e))
        },
        3,
        std::time::Duration::from_millis(100),
        std::time::Duration::from_secs(2),
    )
    .await?;

    if res.status().is_success() {
        let response_body = res.json::<T>().await?;
        Ok(response_body)
    } else {
        let status = res.status();
        let error_text = res
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read error body".to_string());
        Err(anyhow::anyhow!(
            "Request failed with status {}: {}\nURL: {}",
            status,
            error_text,
            url
        ))
    }
}

// --- Command Handlers ---

async fn handle_info(cli: &Cli, client: &Client) -> Result<(), anyhow::Error> {
    let response: NodeInfo = get_request(&cli.api_url, client, "/info").await?;
    println!("--- Node Information ---");
    println!("Name:    {}", response.name);
    println!("Version: {}", response.version);
    println!("Status:  {}", response.status_message);
    println!("------------------------");
    Ok(())
}

async fn handle_status(cli: &Cli, client: &Client) -> Result<(), anyhow::Error> {
    let response: NodeStatus = get_request(&cli.api_url, client, "/status").await?;
    println!("--- Node Status ---");
    println!("Online:         {}", response.is_online);
    println!("Peer Count:     {}", response.peer_count);
    println!("Block Height:   {}", response.current_block_height);
    println!("Version:        {}", response.version);
    println!("-------------------");
    Ok(())
}

async fn handle_metrics(cli: &Cli, client: &Client) -> Result<(), anyhow::Error> {
    let url = format!("{}{}", &cli.api_url, "/metrics");
    let res = icn_common::retry_with_backoff(
        || async {
            client
                .get(&url)
                .send()
                .await
                .map_err(|e| anyhow::anyhow!(e))
        },
        3,
        std::time::Duration::from_millis(100),
        std::time::Duration::from_secs(2),
    )
    .await?;
    if res.status().is_success() {
        let body = res.text().await?;
        println!("{}", body);
        Ok(())
    } else {
        let status = res.status();
        let text = res
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read error body".to_string());
        Err(anyhow::anyhow!(
            "Request failed with status {}: {}\nURL: {}",
            status,
            text,
            url
        ))
    }
}

async fn handle_dag_put(
    cli: &Cli,
    client: &Client,
    block_json_or_stdin: &str,
) -> Result<(), anyhow::Error> {
    let block_json_content = if block_json_or_stdin == "-" {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else {
        block_json_or_stdin.to_string()
    };

    let block: DagBlock = serde_json::from_str(&block_json_content)
        .map_err(|e| anyhow::anyhow!("Invalid DagBlock JSON provided. Error: {}", e))?;
    let response_cid: Cid = post_request(&cli.api_url, client, "/dag/put", &block).await?;
    println!(
        "Successfully submitted block. CID: {}",
        serde_json::to_string_pretty(&response_cid)?
    );
    Ok(())
}

async fn handle_dag_get(cli: &Cli, client: &Client, cid_json: &str) -> Result<(), anyhow::Error> {
    let cid: Cid = serde_json::from_str(cid_json)
        .map_err(|e| anyhow::anyhow!("Invalid CID JSON provided: {}. Error: {}", cid_json, e))?;
    let response_block: DagBlock = post_request(&cli.api_url, client, "/dag/get", &cid).await?;
    println!("--- Retrieved DAG Block ---");
    println!("{}", serde_json::to_string_pretty(&response_block)?);
    println!("-------------------------");
    Ok(())
}

async fn handle_dag_meta(cli: &Cli, client: &Client, cid_json: &str) -> Result<(), anyhow::Error> {
    let cid: Cid = serde_json::from_str(cid_json)
        .map_err(|e| anyhow::anyhow!("Invalid CID JSON provided: {}. Error: {}", cid_json, e))?;
    let meta: icn_dag::DagBlockMetadata =
        post_request(&cli.api_url, client, "/dag/meta", &cid).await?;
    println!("--- DAG Block Metadata ---");
    println!("{}", serde_json::to_string_pretty(&meta)?);
    println!("--------------------------");
    Ok(())
}

fn handle_dag_backup(path: &str) -> Result<(), anyhow::Error> {
    let src = PathBuf::from("./icn_data/node_store");
    let dest = PathBuf::from(path);
    std::fs::create_dir_all(&dest)?;
    let mut files = Vec::new();
    collect_block_files(&src, &mut files)?;
    for file in files {
        let target = dest.join(file.file_name().unwrap());
        std::fs::copy(&file, target)?;
    }
    println!("Backup created at {}", dest.display());
    Ok(())
}

fn handle_dag_restore(path: &str) -> Result<(), anyhow::Error> {
    let src = PathBuf::from(path);
    let dest = PathBuf::from("./icn_data/node_store");
    std::fs::create_dir_all(&dest)?;
    let mut files = Vec::new();
    collect_block_files(&src, &mut files)?;
    for file in files {
        let target = dest.join(file.file_name().unwrap());
        std::fs::copy(&file, target)?;
    }
    println!("Restored DAG store from {}", src.display());
    Ok(())
}

fn handle_dag_verify(full: bool) -> Result<(), anyhow::Error> {
    let store_path = PathBuf::from("./icn_data/node_store");
    let mut verified = 0usize;
    let mut files = Vec::new();
    collect_block_files(&store_path, &mut files)?;
    for file in files {
        let content = std::fs::read_to_string(&file)?;
        let block: DagBlock = serde_json::from_str(&content)?;
        icn_common::verify_block_integrity(&block)?;
        verified += 1;
        if !full {
            break;
        }
    }
    println!("Verified {verified} block(s)");
    Ok(())
}

async fn handle_dag_pin(
    cli: &Cli,
    client: &Client,
    cid_json: &str,
    ttl: Option<u64>,
) -> Result<(), anyhow::Error> {
    let cid: Cid = serde_json::from_str(cid_json)
        .map_err(|e| anyhow::anyhow!("Invalid CID JSON: {cid_json}. Error: {e}"))?;
    let body = serde_json::json!({ "cid": cid, "ttl": ttl });
    let _: JsonValue = post_request(&cli.api_url, client, "/dag/pin", &body).await?;
    println!("Pinned block {cid_json}");
    Ok(())
}

async fn handle_dag_unpin(cli: &Cli, client: &Client, cid_json: &str) -> Result<(), anyhow::Error> {
    let cid: Cid = serde_json::from_str(cid_json)
        .map_err(|e| anyhow::anyhow!("Invalid CID JSON: {cid_json}. Error: {e}"))?;
    let _: JsonValue = post_request(&cli.api_url, client, "/dag/unpin", &cid).await?;
    println!("Unpinned block {cid_json}");
    Ok(())
}

async fn handle_dag_prune(cli: &Cli, client: &Client) -> Result<(), anyhow::Error> {
    let _: JsonValue = post_request(&cli.api_url, client, "/dag/prune", &()).await?;
    println!("Prune triggered");
    Ok(())
}

async fn handle_gov_submit(
    cli: &Cli,
    client: &Client,
    proposal_request_json_or_stdin: &str,
) -> Result<(), anyhow::Error> {
    let proposal_request_content = if proposal_request_json_or_stdin == "-" {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else {
        proposal_request_json_or_stdin.to_string()
    };

    let request: ApiSubmitProposalRequest = serde_json::from_str(&proposal_request_content)
        .map_err(|e| anyhow::anyhow!("Invalid ApiSubmitProposalRequest JSON. Error: {}", e))?;
    let response_proposal_id: ProposalId =
        post_request(&cli.api_url, client, "/governance/submit", &request).await?;
    println!(
        "Successfully submitted proposal. Proposal ID: {}",
        serde_json::to_string_pretty(&response_proposal_id)?
    );
    Ok(())
}

async fn handle_gov_vote(
    cli: &Cli,
    client: &Client,
    vote_request_json: &str,
) -> Result<(), anyhow::Error> {
    let request: ApiCastVoteRequest = serde_json::from_str(vote_request_json).map_err(|e| {
        anyhow::anyhow!(
            "Invalid ApiCastVoteRequest JSON: {}. Error: {}",
            vote_request_json,
            e
        )
    })?;
    // Assuming the response is a simple success message or confirmation JSON
    let response: JsonValue =
        post_request(&cli.api_url, client, "/governance/vote", &request).await?;
    println!(
        "Vote response: {}",
        serde_json::to_string_pretty(&response)?
    );
    Ok(())
}

async fn handle_gov_tally(
    cli: &Cli,
    client: &Client,
    proposal_id: &str,
) -> Result<(), anyhow::Error> {
    let req = serde_json::json!({ "proposal_id": proposal_id });
    let result: icn_api::governance_trait::CloseProposalResponse =
        post_request(&cli.api_url, client, "/governance/close", &req).await?;
    println!(
        "Tally result: yes={} no={} abstain={} status={}",
        result.yes, result.no, result.abstain, result.status
    );
    Ok(())
}

async fn handle_gov_list_proposals(cli: &Cli, client: &Client) -> Result<(), anyhow::Error> {
    let proposals: Vec<Proposal> =
        get_request(&cli.api_url, client, "/governance/proposals").await?;
    println!("--- All Proposals ---");
    if proposals.is_empty() {
        println!("No proposals found.");
    } else {
        println!("{}", serde_json::to_string_pretty(&proposals)?);
    }
    println!("---------------------");
    Ok(())
}

async fn handle_gov_get_proposal(
    cli: &Cli,
    client: &Client,
    proposal_id: &str,
) -> Result<(), anyhow::Error> {
    let path = format!("/governance/proposal/{}", proposal_id);
    let proposal: Proposal = get_request(&cli.api_url, client, &path).await?;
    println!("--- Proposal Details (ID: {}) ---", proposal_id);
    println!("{}", serde_json::to_string_pretty(&proposal)?);
    println!("-----------------------------------");
    Ok(())
}

async fn handle_mesh_jobs(cli: &Cli, client: &Client) -> Result<(), anyhow::Error> {
    let response: serde_json::Value = get_request(&cli.api_url, client, "/mesh/jobs").await?;
    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}

async fn handle_mesh_status(cli: &Cli, client: &Client, job_id: &str) -> Result<(), anyhow::Error> {
    let path = format!("/mesh/jobs/{}", job_id);
    let response: serde_json::Value = get_request(&cli.api_url, client, &path).await?;
    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}

/// Submit a new mesh job to the node
async fn handle_mesh_submit(
    cli: &Cli,
    client: &Client,
    job_request_json_or_stdin: &str,
) -> Result<(), anyhow::Error> {
    let job_json_content = if job_request_json_or_stdin == "-" {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else {
        job_request_json_or_stdin.to_string()
    };

    let mut request_value: serde_json::Value = serde_json::from_str(&job_json_content)
        .map_err(|e| anyhow::anyhow!("Invalid mesh job JSON: {}", e))?;

    if let Some(manifest_path) = request_value
        .get("manifest_cid")
        .and_then(|v| v.as_str())
        .filter(|s| s.ends_with(".ccl"))
    {
        let path = PathBuf::from(manifest_path);
        let (wasm, _meta) = compile_ccl_file_to_wasm(&path).map_err(anyhow::Error::msg)?;
        let payload = DagBlockPayload { data: wasm };
        let cid: Cid = post_request(&cli.api_url, client, "/dag/put", &payload).await?;

        if let Some(obj) = request_value.as_object_mut() {
            obj.insert(
                "manifest_cid".to_string(),
                serde_json::json!(cid.to_string()),
            );
            if !obj.contains_key("spec_bytes") {
                let spec = icn_mesh::JobSpec {
                    kind: icn_mesh::JobKind::CclWasm,
                    ..Default::default()
                };
                let bytes = bincode::serialize(&spec).unwrap();
                obj.insert(
                    "spec_bytes".to_string(),
                    serde_json::json!(base64::engine::general_purpose::STANDARD.encode(bytes)),
                );
            }
        }
    }

    let response: serde_json::Value =
        post_request(&cli.api_url, client, "/mesh/submit", &request_value).await?;
    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}

async fn handle_network_stats(cli: &Cli, client: &Client) -> Result<(), anyhow::Error> {
    let status: NodeStatus = get_request(&cli.api_url, client, "/status").await?;
    println!("{}", serde_json::to_string_pretty(&status)?);
    Ok(())
}

/// Send a ping to the specified peer using the stubbed network service
async fn handle_network_ping(
    cli: &Cli,
    client: &Client,
    peer_id: &str,
) -> Result<(), anyhow::Error> {
    let info: NodeInfo = get_request(&cli.api_url, client, "/info").await?;
    let result = icn_network::send_network_ping(&info, peer_id).await?;
    println!("{}", result);
    Ok(())
}

async fn handle_network_peers(cli: &Cli, _client: &Client) -> Result<(), anyhow::Error> {
    let peer_id = icn_api::http_get_local_peer_id(&cli.api_url).await?;
    let peers = icn_api::http_get_peer_list(&cli.api_url).await?;
    println!("Local Peer ID: {}", peer_id);
    println!("Discovered Peers:");
    for p in peers {
        println!("- {}", p);
    }
    Ok(())
}

fn handle_ccl_compile(file: &str) -> Result<(), anyhow::Error> {
    let source_path = PathBuf::from(file);
    let wasm_path = source_path.with_extension("wasm");
    let meta_path = source_path.with_extension("json");
    let meta =
        compile_ccl_file(&source_path, &wasm_path, &meta_path).map_err(|e| anyhow::anyhow!(e))?;
    println!("{}", serde_json::to_string_pretty(&meta)?);
    Ok(())
}

fn handle_ccl_lint(file: &str) -> Result<(), anyhow::Error> {
    let source_path = PathBuf::from(file);
    check_ccl_file(&source_path).map_err(|e| anyhow::anyhow!(e))?;
    println!("{} passed linting", file);
    Ok(())
}

async fn handle_ccl_explain(file: &str, target: &Option<String>) -> Result<(), anyhow::Error> {
    let source_path = PathBuf::from(file);
    let explanation =
        explain_ccl_policy(&source_path, target.clone()).map_err(|e| anyhow::anyhow!(e))?;
    println!("{}", explanation);
    Ok(())
}

#[derive(Serialize)]
struct DagBlockPayload {
    data: Vec<u8>,
}

async fn handle_compile_ccl_upload(
    cli: &Cli,
    client: &Client,
    file: &str,
) -> Result<(), anyhow::Error> {
    let path = PathBuf::from(file);
    let (wasm, meta) = compile_ccl_file_to_wasm(&path)?;
    let payload = DagBlockPayload { data: wasm };
    let cid: Cid = post_request(&cli.api_url, client, "/dag/put", &payload).await?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "cid": cid,
            "metadata": meta
        }))?
    );
    Ok(())
}

async fn handle_fed_list_peers(cli: &Cli, client: &Client) -> Result<(), anyhow::Error> {
    let peers: Vec<String> = get_request(&cli.api_url, client, "/federation/peers").await?;
    println!("{}", serde_json::to_string_pretty(&peers)?);
    Ok(())
}

#[derive(Serialize)]
struct PeerReq<'a> {
    peer: &'a str,
}

async fn handle_fed_join(cli: &Cli, client: &Client, peer: &str) -> Result<(), anyhow::Error> {
    let req = PeerReq { peer };
    let resp: serde_json::Value =
        post_request(&cli.api_url, client, "/federation/join", &req).await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

async fn handle_fed_leave(cli: &Cli, client: &Client, peer: &str) -> Result<(), anyhow::Error> {
    let req = PeerReq { peer };
    let resp: serde_json::Value =
        post_request(&cli.api_url, client, "/federation/leave", &req).await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

async fn handle_fed_status(cli: &Cli, client: &Client) -> Result<(), anyhow::Error> {
    let status: serde_json::Value = get_request(&cli.api_url, client, "/federation/status").await?;
    println!("{}", serde_json::to_string_pretty(&status)?);
    Ok(())
}

async fn handle_account_balance(
    cli: &Cli,
    client: &Client,
    did: &str,
) -> Result<(), anyhow::Error> {
    let path = format!("/account/{}/mana", did);
    let v: serde_json::Value = get_request(&cli.api_url, client, &path).await?;
    println!("{}", serde_json::to_string_pretty(&v)?);
    Ok(())
}

async fn handle_keys_show(cli: &Cli, client: &Client) -> Result<(), anyhow::Error> {
    let v: serde_json::Value = get_request(&cli.api_url, client, "/keys").await?;
    println!("{}", serde_json::to_string_pretty(&v)?);
    Ok(())
}

async fn handle_reputation_get(cli: &Cli, client: &Client, did: &str) -> Result<(), anyhow::Error> {
    let path = format!("/reputation/{}", did);
    let v: serde_json::Value = get_request(&cli.api_url, client, &path).await?;
    println!("{}", serde_json::to_string_pretty(&v)?);
    Ok(())
}

async fn handle_identity_verify(
    cli: &Cli,
    client: &Client,
    proof_json_or_stdin: &str,
) -> Result<(), anyhow::Error> {
    let proof_json_content = if proof_json_or_stdin == "-" {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else {
        proof_json_or_stdin.to_string()
    };

    let proof: ZkCredentialProof = serde_json::from_str(&proof_json_content)
        .map_err(|e| anyhow::anyhow!("Invalid ZkCredentialProof JSON: {}", e))?;

    let resp: serde_json::Value =
        post_request(&cli.api_url, client, "/identity/verify", &proof).await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

fn parse_backend(s: &str) -> ZkProofType {
    match s.to_ascii_lowercase().as_str() {
        "groth16" => ZkProofType::Groth16,
        "bulletproofs" => ZkProofType::Bulletproofs,
        other => ZkProofType::Other(other.to_string()),
    }
}

fn handle_identity_generate_inner(
    issuer: &str,
    holder: &str,
    claim_type: &str,
    schema: &str,
    backend: &str,
    verification_key: &Option<String>,
    public_inputs: &Option<String>,
) -> Result<(), anyhow::Error> {
    use std::str::FromStr;

    let issuer_did = Did::from_str(issuer)?;
    let holder_did = Did::from_str(holder)?;
    let schema_cid = icn_common::parse_cid_from_string(schema)?;
    let backend_parsed = parse_backend(backend);
    let mut proof_bytes = vec![0u8; 32];
    fastrand::fill(&mut proof_bytes);

    let verification_key_bytes = if let Some(vk_hex) = verification_key {
        Some(hex::decode(vk_hex.trim_start_matches("0x"))?)
    } else {
        None
    };

    let public_inputs_value = if let Some(json) = public_inputs {
        Some(serde_json::from_str(json)?)
    } else {
        None
    };

    let proof = ZkCredentialProof {
        issuer: issuer_did,
        holder: holder_did,
        claim_type: claim_type.to_string(),
        proof: proof_bytes,
        schema: schema_cid,
        vk_cid: None,
        disclosed_fields: Vec::new(),
        challenge: None,
        backend: backend_parsed,
        verification_key: verification_key_bytes,
        public_inputs: public_inputs_value,
    };

    println!("{}", serde_json::to_string_pretty(&proof)?);
    Ok(())
}

struct Groth16KeyManager;

impl Groth16KeyManager {
    fn generate_proving_key(
        path: &std::path::Path,
        signer: &Ed25519Signer,
    ) -> Result<Vec<u8>, anyhow::Error> {
        use ark_serialize::CanonicalSerialize;
        use ark_std::rand::{rngs::StdRng, SeedableRng};
        use icn_zk::{setup, AgeOver18Circuit};

        let circuit = AgeOver18Circuit {
            birth_year: 2000,
            current_year: 2020,
        };
        let mut rng = StdRng::seed_from_u64(42);
        let pk = setup(circuit, &mut rng)?;

        let mut file = std::fs::File::create(path)?;
        pk.serialize_compressed(&mut file)?;

        let mut vk_bytes = Vec::new();
        pk.vk.serialize_compressed(&mut vk_bytes)?;
        let sig = signer.sign(&vk_bytes)?;
        Ok(sig)
    }
}

async fn handle_zk_generate_key() -> Result<(), anyhow::Error> {
    let (sk, pk) = generate_ed25519_keypair();
    let signer = Ed25519Signer::new_with_keys(sk, pk);
    let path = std::path::PathBuf::from("groth16_proving_key.bin");
    let sig = Groth16KeyManager::generate_proving_key(&path, &signer)?;
    let output = serde_json::json!({
        "proving_key_path": path.to_string_lossy(),
        "verifying_key_signature_hex": hex::encode(sig),
    });
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

// CLI command behavior is covered by tests in `crates/icn-cli/tests`.
