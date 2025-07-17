#![doc = include_str!("../README.md")]
#![allow(clippy::uninlined_format_args)]

//! # ICN CLI Crate
//! This crate provides a command-line interface (CLI) for interacting with an ICN HTTP node.

use base64::{self, Engine};
extern crate bincode;
use bs58;
use clap::{Parser, Subcommand};
use icn_common::CommonError;
use prometheus_parse;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue; // For generic JSON data if needed
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::exit; // Added for reading from stdin
use std::str::FromStr;

// Types from our ICN crates that CLI will interact with (serialize/deserialize)
// These types are expected to be sent to/received from the icn-node HTTP API.
use icn_common::{Cid, DagBlock, Did, NodeInfo, NodeStatus, ZkCredentialProof, ZkProofType};
// Using aliased request structs from icn-api for clarity, these are what the node expects
use icn_api::governance_trait::{
    CastVoteRequest as ApiCastVoteRequest, SubmitProposalRequest as ApiSubmitProposalRequest,
};
use icn_api::identity_trait::{BatchVerificationResponse, VerifyProofsRequest};
use icn_ccl::{check_ccl_file, compile_ccl_file, compile_ccl_file_to_wasm, explain_ccl_policy};
use icn_governance::{Proposal, ProposalId};
use icn_identity::generate_ed25519_keypair;
use icn_runtime::context::{Ed25519Signer, Signer};
use icn_templates;
use chrono;

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

    #[clap(
        long,
        global = true,
        env = "ICN_API_KEY",
        help = "API key for authenticated requests"
    )]
    api_key: Option<String>,

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
    /// Monitoring commands using metrics
    Monitor {
        #[clap(subcommand)]
        command: MonitorCommands,
    },
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
    /// Mutual aid resource registry
    Aid {
        #[clap(subcommand)]
        command: AidCommands,
    },
    /// Emergency response coordination
    Emergency {
        #[clap(subcommand)]
        command: EmergencyCommands,
    },
    /// Interactive cooperative formation wizard
    Wizard {
        #[clap(subcommand)]
        command: WizardCommands,
    },
    /// Cooperative discovery and management
    Cooperative {
        #[clap(subcommand)]
        command: CooperativeCommands,
    },
    /// Trust management and graph operations
    Trust {
        #[clap(subcommand)]
        command: TrustCommands,
    },
}

#[derive(Subcommand, Debug)]
enum CooperativeCommands {
    /// Register a new cooperative in the federation
    Register {
        #[clap(help = "Cooperative profile JSON or '-' for stdin")]
        profile_json_or_stdin: String,
    },
    /// Search for cooperatives in the federation
    Search {
        #[clap(long, help = "Cooperative type filter")]
        coop_type: Option<String>,
        #[clap(long, help = "Required capabilities (comma-separated)")]
        capabilities: Option<String>,
        #[clap(long, help = "Country filter")]
        country: Option<String>,
        #[clap(long, help = "Region filter")]
        region: Option<String>,
        #[clap(long, help = "Locality filter")]
        locality: Option<String>,
        #[clap(long, help = "Federation filter")]
        federation: Option<String>,
        #[clap(long, help = "Search query")]
        query: Option<String>,
        #[clap(long, help = "Maximum results", default_value = "10")]
        limit: usize,
    },
    /// Get a cooperative profile by DID
    Profile {
        #[clap(help = "Cooperative DID")]
        did: String,
    },
    /// Add a trust relationship with another cooperative
    Trust {
        #[clap(help = "Trust relationship JSON or '-' for stdin")]
        trust_json_or_stdin: String,
    },
    /// List trust relationships for a cooperative
    TrustList {
        #[clap(help = "Cooperative DID")]
        did: String,
    },
    /// Find providers of a specific capability
    Providers {
        #[clap(help = "Capability type")]
        capability_type: String,
    },
    /// Show registry statistics
    Stats,
    /// Interactive cooperative onboarding wizard
    Onboard {
        #[clap(long, help = "Skip registration with node")]
        dry_run: bool,
    },
}

/// Trust management commands
#[derive(Subcommand, Debug)]
enum TrustCommands {
    /// Get trust relationship between two entities
    Get {
        #[clap(help = "Source DID (who extends trust)")]
        from: String,
        #[clap(help = "Target DID (who receives trust)")]
        to: String,
        #[clap(long, help = "Trust context")]
        context: String,
    },
    /// List trust relationships for an entity
    List {
        #[clap(help = "Entity DID")]
        entity: String,
        #[clap(long, help = "Filter by trust context")]
        context: Option<String>,
        #[clap(long, help = "Filter by minimum trust level")]
        min_level: Option<String>,
        #[clap(long, help = "Filter by federation")]
        federation: Option<String>,
        #[clap(long, help = "Include inherited trust", default_value = "true")]
        include_inherited: bool,
        #[clap(long, help = "Include cross-federation trust", default_value = "true")]
        include_cross_federation: bool,
    },
    /// Find trust paths between two entities
    Paths {
        #[clap(help = "Source DID")]
        from: String,
        #[clap(help = "Target DID")]
        to: String,
        #[clap(long, help = "Trust context")]
        context: String,
        #[clap(long, help = "Maximum path length", default_value = "5")]
        max_length: usize,
        #[clap(long, help = "Maximum number of paths to return", default_value = "3")]
        max_paths: usize,
        #[clap(long, help = "Minimum trust level")]
        min_level: Option<String>,
    },
    /// Get trust score for an entity
    Score {
        #[clap(help = "Entity DID")]
        entity: String,
    },
    /// Get trust scores for multiple entities
    Scores {
        #[clap(help = "Entity DIDs (comma-separated)")]
        entities: String,
    },
    /// Update trust relationship
    Update {
        #[clap(help = "Trust update request JSON or '-' for stdin")]
        update_json_or_stdin: String,
    },
    /// Remove trust relationship
    Remove {
        #[clap(help = "Source DID")]
        from: String,
        #[clap(help = "Target DID")]
        to: String,
        #[clap(long, help = "Trust context")]
        context: String,
    },
    /// Get trust graph statistics
    Stats,
    /// Get federation trust statistics
    FederationStats {
        #[clap(help = "Federation ID")]
        federation: String,
    },
    /// Search entities by trust criteria
    Search {
        #[clap(long, help = "Trust context filter")]
        context: Option<String>,
        #[clap(long, help = "Minimum trust level")]
        min_level: Option<String>,
        #[clap(long, help = "Federation filter")]
        federation: Option<String>,
        #[clap(long, help = "Maximum results", default_value = "10")]
        limit: usize,
        #[clap(long, help = "Results offset", default_value = "0")]
        offset: usize,
    },
    /// Validate trust for an operation
    Validate {
        #[clap(help = "Actor DID")]
        actor: String,
        #[clap(help = "Target DID")]
        target: String,
        #[clap(long, help = "Trust context")]
        context: String,
        #[clap(long, help = "Operation name")]
        operation: String,
    },
    /// Get trust network neighbors
    Neighbors {
        #[clap(help = "Entity DID")]
        entity: String,
        #[clap(long, help = "Maximum distance", default_value = "2")]
        max_distance: usize,
        #[clap(long, help = "Minimum trust level")]
        min_level: Option<String>,
    },
    /// Recalculate trust scores
    Recalculate {
        #[clap(long, help = "Specific entities to recalculate (comma-separated)")]
        entities: Option<String>,
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
    /// Initialize a new federation
    #[clap(name = "init")]
    Init,
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
    /// Synchronize federation state
    #[clap(name = "sync")]
    Sync,
    /// Federation trust management commands
    Trust {
        #[clap(subcommand)]
        command: FederationTrustCommands,
    },
    /// Federation metadata management commands
    Metadata {
        #[clap(subcommand)]
        command: FederationMetadataCommands,
    },
    /// DID document operations
    Did {
        #[clap(subcommand)]
        command: FederationDidCommands,
    },
}

#[derive(Subcommand, Debug)]
enum FederationTrustCommands {
    /// Configure trust policy for a federation
    Configure {
        #[clap(help = "Federation ID")]
        federation_id: String,
        #[clap(help = "Trust policy JSON or '-' for stdin")]
        policy_json_or_stdin: String,
    },
    /// Add a trust relationship to a federation
    Add {
        #[clap(help = "Trust relationship JSON or '-' for stdin")]
        trust_json_or_stdin: String,
    },
    /// Remove a trust relationship from a federation
    Remove {
        #[clap(help = "Source DID")]
        from: String,
        #[clap(help = "Target DID")]
        to: String,
        #[clap(long, help = "Trust context")]
        context: String,
        #[clap(long, help = "Federation ID")]
        federation: String,
    },
    /// List trust relationships in a federation
    List {
        #[clap(help = "Federation ID")]
        federation_id: String,
        #[clap(long, help = "Filter by trust context")]
        context: Option<String>,
        #[clap(long, help = "Filter by minimum trust level")]
        min_level: Option<String>,
    },
    /// Validate trust for a federation operation
    Validate {
        #[clap(help = "Actor DID")]
        actor: String,
        #[clap(help = "Target DID")]
        target: String,
        #[clap(long, help = "Trust context")]
        context: String,
        #[clap(long, help = "Operation name")]
        operation: String,
        #[clap(long, help = "Federation ID")]
        federation: String,
    },
    /// Add cross-federation trust bridge
    Bridge {
        #[clap(help = "Bridge configuration JSON or '-' for stdin")]
        bridge_json_or_stdin: String,
    },
    /// Bootstrap trust with another federation
    Bootstrap {
        #[clap(help = "Target federation peer")]
        peer: String,
        #[clap(long, help = "Trust contexts to establish (comma-separated)")]
        contexts: String,
        #[clap(long, help = "Initial trust level")]
        trust_level: String,
    },
}

#[derive(Subcommand, Debug)]
enum FederationMetadataCommands {
    /// Get federation metadata
    Get {
        #[clap(help = "Federation ID")]
        federation_id: String,
    },
    /// Set federation metadata
    Set {
        #[clap(help = "Federation ID")]
        federation_id: String,
        #[clap(help = "Metadata JSON or '-' for stdin")]
        metadata_json_or_stdin: String,
    },
    /// Update federation scope configuration
    Scope {
        #[clap(help = "Federation ID")]
        federation_id: String,
        #[clap(help = "Scope configuration JSON or '-' for stdin")]
        scope_json_or_stdin: String,
    },
    /// Configure federation quorum policies
    Quorum {
        #[clap(help = "Federation ID")]
        federation_id: String,
        #[clap(help = "Quorum policy JSON or '-' for stdin")]
        policy_json_or_stdin: String,
    },
    /// List member cooperatives in federation
    Members {
        #[clap(help = "Federation ID")]
        federation_id: String,
    },
    /// Add member cooperative to federation
    AddMember {
        #[clap(help = "Federation ID")]
        federation_id: String,
        #[clap(help = "Member DID")]
        member_did: String,
        #[clap(help = "Member profile JSON or '-' for stdin")]
        profile_json_or_stdin: String,
    },
    /// Remove member cooperative from federation
    RemoveMember {
        #[clap(help = "Federation ID")]
        federation_id: String,
        #[clap(help = "Member DID")]
        member_did: String,
    },
}

#[derive(Subcommand, Debug)]
enum FederationDidCommands {
    /// Generate DID document for federation
    Generate {
        #[clap(help = "Federation ID")]
        federation_id: String,
        #[clap(long, help = "Output file (default: stdout)")]
        output: Option<String>,
    },
    /// Verify DID document
    Verify {
        #[clap(help = "DID document JSON or '-' for stdin")]
        document_json_or_stdin: String,
    },
    /// Publish DID document to federation
    Publish {
        #[clap(help = "Federation ID")]
        federation_id: String,
        #[clap(help = "DID document JSON or '-' for stdin")]
        document_json_or_stdin: String,
    },
    /// Resolve DID document from federation
    Resolve {
        #[clap(help = "DID to resolve")]
        did: String,
        #[clap(long, help = "Federation ID")]
        federation: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum WizardCommands {
    /// Generate a cooperative template interactively
    Cooperative {
        #[clap(long, help = "Output directory", required = false)]
        output: Option<String>,
    },
    /// Interactive node setup wizard
    Setup {
        #[clap(long, help = "Output config file", default_value = "node_config.toml")]
        config: String,
    },
    /// Developer onboarding wizard - create DID, submit test job, vote on test proposal
    #[clap(name = "init-dev")]
    InitDev {
        #[clap(long, help = "API URL of the target node", default_value = "http://127.0.0.1:7845")]
        api_url: String,
    },
    /// Federation onboarding wizard - prompts for coop info, generates config
    #[clap(name = "onboard-federation")]
    OnboardFederation {
        #[clap(long, help = "Output config file", default_value = "federation_config.toml")]
        config: String,
    },
}

#[derive(Subcommand, Debug)]
enum EmergencyCommands {
    /// List open aid requests
    List,
    /// Submit a new aid request
    Request {
        #[clap(help = "Aid request JSON or '-' for stdin")]
        request_json_or_stdin: String,
    },
}

#[derive(Subcommand, Debug)]
enum AidCommands {
    /// List available aid resources
    List,
    /// Register a new aid resource
    Register {
        #[clap(help = "Aid resource JSON or '-' for stdin")]
        resource_json_or_stdin: String,
    },
}

#[derive(Subcommand, Debug)]
enum MonitorCommands {
    /// Display node uptime using the metrics endpoint
    Uptime,
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
    /// Verify a revocation proof (JSON string or '-' for stdin)
    VerifyRevocation {
        #[clap(help = "ZkRevocationProof JSON or '-' for stdin")]
        proof_json_or_stdin: String,
    },
    /// Verify multiple proofs from a JSON array
    VerifyProofs {
        #[clap(help = "JSON array or '-' for stdin")]
        proofs_json_or_stdin: String,
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
    /// Request proof generation from the node
    GenerateProofRemote {
        #[clap(long)]
        issuer: String,
        #[clap(long)]
        holder: String,
        #[clap(long)]
        claim_type: String,
        #[clap(long)]
        schema: String,
        #[clap(long)]
        backend: String,
        #[clap(long)]
        public_inputs: Option<String>,
    },
    /// Verify a proof via the node
    VerifyProofRemote {
        #[clap(help = "ZkCredentialProof JSON or '-' for stdin")]
        proof_json_or_stdin: String,
    },
}

#[derive(Subcommand, Debug)]
enum ZkCommands {
    /// Generate a Groth16 proving key and sign the verifying key
    #[clap(name = "generate-key")]
    GenerateKey,
    /// Count constraints for a circuit
    #[clap(name = "analyze")]
    Analyze {
        #[clap(help = "Circuit name to analyze")]
        circuit: String,
    },
    /// Benchmark a circuit with Criterion
    #[clap(name = "profile")]
    Profile {
        #[clap(help = "Circuit name to benchmark")]
        circuit: String,
    },
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
            IdentityCommands::VerifyRevocation {
                proof_json_or_stdin,
            } => handle_identity_verify_revocation(cli, client, proof_json_or_stdin).await?,
            IdentityCommands::VerifyProofs {
                proofs_json_or_stdin,
            } => handle_identity_verify_batch(cli, client, proofs_json_or_stdin).await?,
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
            IdentityCommands::GenerateProofRemote {
                issuer,
                holder,
                claim_type,
                schema,
                backend,
                public_inputs,
            } => {
                handle_identity_generate_remote(
                    cli,
                    client,
                    issuer,
                    holder,
                    claim_type,
                    schema,
                    backend,
                    public_inputs,
                )
                .await?;
            }
            IdentityCommands::VerifyProofRemote { proof_json_or_stdin } => {
                handle_identity_verify_remote(cli, client, proof_json_or_stdin).await?;
            }
        },
        Commands::Ccl { command } => match command {
            CclCommands::Compile { file } => handle_ccl_compile(file)?,
            CclCommands::Lint { file } => handle_ccl_lint(file)?,
            CclCommands::Explain { file, target } => handle_ccl_explain(file, target).await?,
        },
        Commands::Zk { command } => match command {
            ZkCommands::GenerateKey => handle_zk_generate_key().await?,
            ZkCommands::Analyze { circuit } => handle_zk_analyze(circuit).await?,
            ZkCommands::Profile { circuit } => handle_zk_profile(circuit).await?,
        },
        Commands::CompileCcl { file } => handle_compile_ccl_upload(cli, client, file).await?,
        Commands::SubmitJob {
            job_request_json_or_stdin,
        } => handle_mesh_submit(cli, client, job_request_json_or_stdin).await?,
        Commands::JobStatus { job_id } => handle_mesh_status(cli, client, job_id).await?,
        Commands::Federation { command } => match command {
            FederationCommands::Init => handle_fed_init(cli, client).await?,
            FederationCommands::Join { peer } => handle_fed_join(cli, client, peer).await?,
            FederationCommands::Leave { peer } => handle_fed_leave(cli, client, peer).await?,
            FederationCommands::ListPeers => handle_fed_list_peers(cli, client).await?,
            FederationCommands::Status => handle_fed_status(cli, client).await?,
            FederationCommands::Sync => handle_fed_sync(cli, client).await?,
            FederationCommands::Trust { command } => match command {
                FederationTrustCommands::Configure { federation_id, policy_json_or_stdin } => {
                    handle_fed_trust_configure(cli, client, federation_id, policy_json_or_stdin).await?
                }
                FederationTrustCommands::Add { trust_json_or_stdin } => {
                    handle_fed_trust_add(cli, client, trust_json_or_stdin).await?
                }
                FederationTrustCommands::Remove { from, to, context, federation } => {
                    handle_fed_trust_remove(cli, client, from, to, context, federation).await?
                }
                FederationTrustCommands::List { federation_id, context, min_level } => {
                    handle_fed_trust_list(cli, client, federation_id, context, min_level).await?
                }
                FederationTrustCommands::Validate { actor, target, context, operation, federation } => {
                    handle_fed_trust_validate(cli, client, actor, target, context, operation, federation).await?
                }
                FederationTrustCommands::Bridge { bridge_json_or_stdin } => {
                    handle_fed_trust_bridge(cli, client, bridge_json_or_stdin).await?
                }
                FederationTrustCommands::Bootstrap { peer, contexts, trust_level } => {
                    handle_fed_trust_bootstrap(cli, client, peer, contexts, trust_level).await?
                }
            },
            FederationCommands::Metadata { command } => match command {
                FederationMetadataCommands::Get { federation_id } => {
                    handle_fed_metadata_get(cli, client, federation_id).await?
                }
                FederationMetadataCommands::Set { federation_id, metadata_json_or_stdin } => {
                    handle_fed_metadata_set(cli, client, federation_id, metadata_json_or_stdin).await?
                }
                FederationMetadataCommands::Scope { federation_id, scope_json_or_stdin } => {
                    handle_fed_metadata_scope(cli, client, federation_id, scope_json_or_stdin).await?
                }
                FederationMetadataCommands::Quorum { federation_id, policy_json_or_stdin } => {
                    handle_fed_metadata_quorum(cli, client, federation_id, policy_json_or_stdin).await?
                }
                FederationMetadataCommands::Members { federation_id } => {
                    handle_fed_metadata_members(cli, client, federation_id).await?
                }
                FederationMetadataCommands::AddMember { federation_id, member_did, profile_json_or_stdin } => {
                    handle_fed_metadata_add_member(cli, client, federation_id, member_did, profile_json_or_stdin).await?
                }
                FederationMetadataCommands::RemoveMember { federation_id, member_did } => {
                    handle_fed_metadata_remove_member(cli, client, federation_id, member_did).await?
                }
            },
            FederationCommands::Did { command } => match command {
                FederationDidCommands::Generate { federation_id, output } => {
                    handle_fed_did_generate(cli, client, federation_id, output).await?
                }
                FederationDidCommands::Verify { document_json_or_stdin } => {
                    handle_fed_did_verify(cli, client, document_json_or_stdin).await?
                }
                FederationDidCommands::Publish { federation_id, document_json_or_stdin } => {
                    handle_fed_did_publish(cli, client, federation_id, document_json_or_stdin).await?
                }
                FederationDidCommands::Resolve { did, federation } => {
                    handle_fed_did_resolve(cli, client, did, federation).await?
                }
            },
        },
        Commands::Aid { command } => match command {
            AidCommands::List => handle_aid_list(cli, client).await?,
            AidCommands::Register { resource_json_or_stdin } => {
                handle_aid_register(cli, client, resource_json_or_stdin).await?
            }
        },
        Commands::Emergency { command } => match command {
            EmergencyCommands::List => handle_emergency_list(cli, client).await?,
            EmergencyCommands::Request {
                request_json_or_stdin,
            } => handle_emergency_request(cli, client, request_json_or_stdin).await?,
        },
        Commands::Wizard { command } => match command {
            WizardCommands::Cooperative { output } => handle_wizard_cooperative(output.clone())?,
            WizardCommands::Setup { config } => handle_wizard_setup(config)?,
            WizardCommands::InitDev { api_url } => handle_wizard_init_dev(api_url, client).await?,
            WizardCommands::OnboardFederation { config } => handle_wizard_onboard_federation(config)?,
        },
        Commands::Cooperative { command } => match command {
            CooperativeCommands::Register { profile_json_or_stdin } => {
                handle_coop_register(cli, client, profile_json_or_stdin).await?
            }
            CooperativeCommands::Search {
                coop_type,
                capabilities,
                country,
                region,
                locality,
                federation,
                query,
                limit,
            } => handle_coop_search(cli, client, coop_type, capabilities, country, region, locality, federation, query, limit).await?,
            CooperativeCommands::Profile { did } => handle_coop_profile(cli, client, did).await?,
            CooperativeCommands::Trust { trust_json_or_stdin } => {
                handle_coop_trust(cli, client, trust_json_or_stdin).await?
            }
            CooperativeCommands::TrustList { did } => handle_coop_trust_list(cli, client, did).await?,
            CooperativeCommands::Providers { capability_type } => handle_coop_providers(cli, client, capability_type).await?,
            CooperativeCommands::Stats => handle_coop_stats(cli, client).await?,
            CooperativeCommands::Onboard { dry_run } => handle_coop_onboard(cli, client, dry_run).await?,
        },
        Commands::Trust { command } => match command {
            TrustCommands::Get { from, to, context } => {
                handle_trust_get(cli, client, from, to, context).await?
            }
            TrustCommands::List { 
                entity, 
                context, 
                min_level, 
                federation, 
                include_inherited, 
                include_cross_federation 
            } => {
                handle_trust_list(cli, client, entity, context, min_level, federation, *include_inherited, *include_cross_federation).await?
            }
            TrustCommands::Paths { 
                from, 
                to, 
                context, 
                max_length, 
                max_paths, 
                min_level 
            } => {
                handle_trust_paths(cli, client, from, to, context, *max_length, *max_paths, min_level).await?
            }
            TrustCommands::Score { entity } => {
                handle_trust_score(cli, client, entity).await?
            }
            TrustCommands::Scores { entities } => {
                handle_trust_scores(cli, client, entities).await?
            }
            TrustCommands::Update { update_json_or_stdin } => {
                handle_trust_update(cli, client, update_json_or_stdin).await?
            }
            TrustCommands::Remove { from, to, context } => {
                handle_trust_remove(cli, client, from, to, context).await?
            }
            TrustCommands::Stats => {
                handle_trust_stats(cli, client).await?
            }
            TrustCommands::FederationStats { federation } => {
                handle_trust_federation_stats(cli, client, federation).await?
            }
            TrustCommands::Search { 
                context, 
                min_level, 
                federation, 
                limit, 
                offset 
            } => {
                handle_trust_search(cli, client, context, min_level, federation, *limit, *offset).await?
            }
            TrustCommands::Validate { actor, target, context, operation } => {
                handle_trust_validate(cli, client, actor, target, context, operation).await?
            }
            TrustCommands::Neighbors { entity, max_distance, min_level } => {
                handle_trust_neighbors(cli, client, entity, *max_distance, min_level).await?
            }
            TrustCommands::Recalculate { entities } => {
                handle_trust_recalculate(cli, client, entities).await?
            }
        },
        Commands::Monitor { command } => match command {
            MonitorCommands::Uptime => handle_monitor_uptime(cli, client).await?,
        },
    }
    Ok(())
}

// --- HTTP Helper Functions ---

async fn get_request<T: for<'de> Deserialize<'de>>(
    api_url: &str,
    client: &Client,
    path: &str,
    api_key: Option<&str>,
) -> Result<T, anyhow::Error> {
    let url = format!("{}{}", api_url, path);
    let res = icn_common::retry_with_backoff(
        || async {
            let req = client.get(&url);
            let req = if let Some(k) = api_key { req.header("x-api-key", k) } else { req };
            req
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
    api_key: Option<&str>,
) -> Result<T, anyhow::Error> {
    let url = format!("{}{}", api_url, path);
    let res = icn_common::retry_with_backoff(
        || async {
            let req = client.post(&url).json(body);
            let req = if let Some(k) = api_key { req.header("x-api-key", k) } else { req };
            req
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
    let response: NodeInfo = get_request(&cli.api_url, client, "/info", cli.api_key.as_deref()).await?;
    println!("--- Node Information ---");
    println!("Name:    {}", response.name);
    println!("Version: {}", response.version);
    println!("Status:  {}", response.status_message);
    println!("------------------------");
    Ok(())
}

async fn handle_status(cli: &Cli, client: &Client) -> Result<(), anyhow::Error> {
    let response: NodeStatus = get_request(&cli.api_url, client, "/status", cli.api_key.as_deref()).await?;
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
    let api_key = cli.api_key.as_deref();
    let res = icn_common::retry_with_backoff(
        || async {
            let req = client.get(&url);
            let req = if let Some(k) = api_key { req.header("x-api-key", k) } else { req };
            req
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
    let response_cid: Cid = post_request(&cli.api_url, client, "/dag/put", &block, cli.api_key.as_deref()).await?;
    println!(
        "Successfully submitted block. CID: {}",
        serde_json::to_string_pretty(&response_cid)?
    );
    Ok(())
}

async fn handle_dag_get(cli: &Cli, client: &Client, cid_json: &str) -> Result<(), anyhow::Error> {
    let cid: Cid = serde_json::from_str(cid_json)
        .map_err(|e| anyhow::anyhow!("Invalid CID JSON provided: {}. Error: {}", cid_json, e))?;
    let response_block: DagBlock = post_request(&cli.api_url, client, "/dag/get", &cid, cli.api_key.as_deref()).await?;
    println!("--- Retrieved DAG Block ---");
    println!("{}", serde_json::to_string_pretty(&response_block)?);
    println!("-------------------------");
    Ok(())
}

async fn handle_dag_meta(cli: &Cli, client: &Client, cid_json: &str) -> Result<(), anyhow::Error> {
    let cid: Cid = serde_json::from_str(cid_json)
        .map_err(|e| anyhow::anyhow!("Invalid CID JSON provided: {}. Error: {}", cid_json, e))?;
    let meta: icn_dag::DagBlockMetadata =
        post_request(&cli.api_url, client, "/dag/meta", &cid, cli.api_key.as_deref()).await?;
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
    let _: JsonValue = post_request(&cli.api_url, client, "/dag/pin", &body, cli.api_key.as_deref()).await?;
    println!("Pinned block {cid_json}");
    Ok(())
}

async fn handle_dag_unpin(cli: &Cli, client: &Client, cid_json: &str) -> Result<(), anyhow::Error> {
    let cid: Cid = serde_json::from_str(cid_json)
        .map_err(|e| anyhow::anyhow!("Invalid CID JSON: {cid_json}. Error: {e}"))?;
    let _: JsonValue = post_request(&cli.api_url, client, "/dag/unpin", &cid, cli.api_key.as_deref()).await?;
    println!("Unpinned block {cid_json}");
    Ok(())
}

async fn handle_dag_prune(cli: &Cli, client: &Client) -> Result<(), anyhow::Error> {
    let _: JsonValue = post_request(&cli.api_url, client, "/dag/prune", &(), cli.api_key.as_deref()).await?;
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
        post_request(&cli.api_url, client, "/governance/submit", &request, cli.api_key.as_deref()).await?;
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
        post_request(&cli.api_url, client, "/governance/vote", &request, cli.api_key.as_deref()).await?;
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
        post_request(&cli.api_url, client, "/governance/close", &req, cli.api_key.as_deref()).await?;
    println!(
        "Tally result: yes={} no={} abstain={} status={}",
        result.yes, result.no, result.abstain, result.status
    );
    Ok(())
}

async fn handle_gov_list_proposals(cli: &Cli, client: &Client) -> Result<(), anyhow::Error> {
    let proposals: Vec<Proposal> =
        get_request(&cli.api_url, client, "/governance/proposals", cli.api_key.as_deref()).await?;
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
    let proposal: Proposal = get_request(&cli.api_url, client, &path, cli.api_key.as_deref()).await?;
    println!("--- Proposal Details (ID: {}) ---", proposal_id);
    println!("{}", serde_json::to_string_pretty(&proposal)?);
    println!("-----------------------------------");
    Ok(())
}

async fn handle_mesh_jobs(cli: &Cli, client: &Client) -> Result<(), anyhow::Error> {
    let response: serde_json::Value = get_request(&cli.api_url, client, "/mesh/jobs", cli.api_key.as_deref()).await?;
    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}

async fn handle_mesh_status(cli: &Cli, client: &Client, job_id: &str) -> Result<(), anyhow::Error> {
    let path = format!("/mesh/jobs/{}", job_id);
    let response: serde_json::Value = get_request(&cli.api_url, client, &path, cli.api_key.as_deref()).await?;
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
        let cid: Cid = post_request(&cli.api_url, client, "/dag/put", &payload, cli.api_key.as_deref()).await?;

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
        post_request(&cli.api_url, client, "/mesh/submit", &request_value, cli.api_key.as_deref()).await?;
    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}

async fn handle_network_stats(cli: &Cli, client: &Client) -> Result<(), anyhow::Error> {
    let status: NodeStatus = get_request(&cli.api_url, client, "/status", cli.api_key.as_deref()).await?;
    println!("{}", serde_json::to_string_pretty(&status)?);
    Ok(())
}

/// Send a ping to the specified peer using the stubbed network service
async fn handle_network_ping(
    cli: &Cli,
    client: &Client,
    peer_id: &str,
) -> Result<(), anyhow::Error> {
    let info: NodeInfo = get_request(&cli.api_url, client, "/info", cli.api_key.as_deref()).await?;
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
    let cid: Cid = post_request(&cli.api_url, client, "/dag/put", &payload, cli.api_key.as_deref()).await?;
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
    let peers: Vec<String> = get_request(&cli.api_url, client, "/federation/peers", cli.api_key.as_deref()).await?;
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
        post_request(&cli.api_url, client, "/federation/join", &req, cli.api_key.as_deref()).await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

async fn handle_fed_leave(cli: &Cli, client: &Client, peer: &str) -> Result<(), anyhow::Error> {
    let req = PeerReq { peer };
    let resp: serde_json::Value =
        post_request(&cli.api_url, client, "/federation/leave", &req, cli.api_key.as_deref()).await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

async fn handle_fed_status(cli: &Cli, client: &Client) -> Result<(), anyhow::Error> {
    let status: serde_json::Value = get_request(&cli.api_url, client, "/federation/status", cli.api_key.as_deref()).await?;
    println!("{}", serde_json::to_string_pretty(&status)?);
    Ok(())
}

async fn handle_fed_init(cli: &Cli, client: &Client) -> Result<(), anyhow::Error> {
    let resp: serde_json::Value =
        post_request(&cli.api_url, client, "/federation/init", &(), cli.api_key.as_deref()).await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

async fn handle_fed_sync(cli: &Cli, client: &Client) -> Result<(), anyhow::Error> {
    let resp: serde_json::Value =
        post_request(&cli.api_url, client, "/federation/sync", &(), cli.api_key.as_deref()).await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

async fn handle_account_balance(
    cli: &Cli,
    client: &Client,
    did: &str,
) -> Result<(), anyhow::Error> {
    let path = format!("/account/{}/mana", did);
    let v: serde_json::Value = get_request(&cli.api_url, client, &path, cli.api_key.as_deref()).await?;
    println!("{}", serde_json::to_string_pretty(&v)?);
    Ok(())
}

async fn handle_keys_show(cli: &Cli, client: &Client) -> Result<(), anyhow::Error> {
    let v: serde_json::Value = get_request(&cli.api_url, client, "/keys", cli.api_key.as_deref()).await?;
    println!("{}", serde_json::to_string_pretty(&v)?);
    Ok(())
}

async fn handle_reputation_get(cli: &Cli, client: &Client, did: &str) -> Result<(), anyhow::Error> {
    let path = format!("/reputation/{}", did);
    let v: serde_json::Value = get_request(&cli.api_url, client, &path, cli.api_key.as_deref()).await?;
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
        post_request(&cli.api_url, client, "/identity/verify", &proof, cli.api_key.as_deref()).await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

async fn handle_identity_verify_revocation(
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

    let proof: icn_common::ZkRevocationProof = serde_json::from_str(&proof_json_content)
        .map_err(|e| anyhow::anyhow!("Invalid ZkRevocationProof JSON: {}", e))?;

    let resp: serde_json::Value = post_request(&cli.api_url, client, "/identity/verify/revocation", &proof, cli.api_key.as_deref()).await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

async fn handle_identity_verify_batch(
    cli: &Cli,
    client: &Client,
    proofs_json_or_stdin: &str,
) -> Result<(), anyhow::Error> {
    let proofs_json_content = if proofs_json_or_stdin == "-" {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else {
        proofs_json_or_stdin.to_string()
    };

    let proofs: Vec<ZkCredentialProof> = serde_json::from_str(&proofs_json_content)
        .map_err(|e| anyhow::anyhow!("Invalid proofs JSON: {}", e))?;
    let req = VerifyProofsRequest { proofs };
    let resp: BatchVerificationResponse =
        post_request(&cli.api_url, client, "/identity/verify/batch", &req, cli.api_key.as_deref()).await?;
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

async fn handle_zk_analyze(circuit: &str) -> Result<(), anyhow::Error> {
    use icn_zk::devtools::count_constraints;
    use icn_zk::{
        AgeOver18Circuit, AgeRepMembershipCircuit, BalanceRangeCircuit, MembershipCircuit,
        MembershipProofCircuit, ReputationCircuit, TimestampValidityCircuit,
    };

    let count = match circuit {
        "age_over_18" => count_constraints(AgeOver18Circuit {
            birth_year: 2000,
            current_year: 2024,
        })?,
        "membership" => count_constraints(MembershipCircuit { is_member: true })?,
        "membership_proof" => count_constraints(MembershipProofCircuit {
            membership_flag: true,
            expected: true,
        })?,
        "reputation" => count_constraints(ReputationCircuit {
            reputation: 10,
            threshold: 5,
        })?,
        "timestamp_validity" => count_constraints(TimestampValidityCircuit {
            timestamp: 0,
            not_before: 0,
            not_after: 1,
        })?,
        "balance_range" => count_constraints(BalanceRangeCircuit {
            balance: 10,
            min: 0,
            max: 100,
        })?,
        "age_rep_membership" => count_constraints(AgeRepMembershipCircuit {
            birth_year: 2000,
            current_year: 2024,
            reputation: 10,
            threshold: 5,
            is_member: true,
        })?,
        other => anyhow::bail!("Unknown circuit '{}'.", other),
    };

    println!("constraints: {}", count);
    Ok(())
}

async fn handle_zk_profile(circuit: &str) -> Result<(), anyhow::Error> {
    use tokio::process::Command;

    let status = Command::new("cargo")
        .arg("bench")
        .arg("-p")
        .arg("icn-zk")
        .arg("--")
        .arg(circuit)
        .status()
        .await?;

    if !status.success() {
        anyhow::bail!("cargo bench failed");
    }
    Ok(())
}

fn handle_wizard_cooperative(output: Option<String>) -> Result<(), anyhow::Error> {
    use std::io::{self, Write};
    let mut name = String::new();
    print!("Cooperative name: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut name)?;
    let name = name.trim();

    println!("Select governance template:");
    println!("1) Rotating stewards");
    println!("2) Rotating council");
    println!("3) Rotating assembly");
    print!("Choice: ");
    io::stdout().flush()?;
    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;

    let template = match choice.trim() {
        "1" => icn_templates::ROTATING_STEWARDS,
        "2" => icn_templates::ROTATING_COUNCIL,
        _ => icn_templates::ROTATING_ASSEMBLY,
    };

    let file_name = format!("{}_governance.ccl", name.replace(' ', "_"));
    let dir = output.unwrap_or_else(|| ".".to_string());
    let path = std::path::Path::new(&dir).join(&file_name);
    std::fs::write(&path, template)?;
    println!("Template written to {}", path.display());
    Ok(())
}

fn handle_wizard_setup(config: &str) -> Result<(), anyhow::Error> {
    use std::io::{self, Write};
    let mut name = String::new();
    print!("Node name: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut name)?;
    let name = name.trim();

    let mut api_key = String::new();
    print!("API key: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut api_key)?;
    let api_key = api_key.trim();

    let mut peers_input = String::new();
    print!("Federation peers (comma separated): ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut peers_input)?;
    let peers: Vec<String> = peers_input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let (sk, pk) = generate_ed25519_keypair();
    let did = icn_identity::did_key_from_verifying_key(&pk);
    let sk_bs58 = bs58::encode(sk.to_bytes()).into_string();

    let did_value = did.clone();
    let cfg = toml::toml! {
        node_name = name
        http_listen_addr = "0.0.0.0:7845"
        storage_backend = "sqlite"
        storage_path = "./icn_data/node.sqlite"
        api_key = api_key
        open_rate_limit = 0
        federation_peers = peers
        node_did = did_value
        node_private_key_bs58 = sk_bs58
    };

    std::fs::write(config, toml::to_string_pretty(&cfg)?)?;
    println!("Configuration written to {}", config);
    println!("Node DID: {}", did);
    Ok(())
}

async fn handle_wizard_init_dev(api_url: &str, client: &Client) -> Result<(), anyhow::Error> {
    use std::io::{self, Write};
    
    println!("🚀 ICN Developer Onboarding Wizard");
    println!("==================================");
    
    // Step 1: Generate a DID
    println!("\n1. Generating DID and keypair...");
    let (sk, pk) = generate_ed25519_keypair();
    let did = icn_identity::did_key_from_verifying_key(&pk);
    let sk_bs58 = bs58::encode(sk.to_bytes()).into_string();
    
    println!("✅ DID created: {}", did);
    println!("   Private key: {}", sk_bs58);
    
    // Step 2: Check node connection
    println!("\n2. Checking node connection...");
    let url = format!("{}/status", api_url);
    match client.get(&url).send().await {
        Ok(response) if response.status().is_success() => {
            println!("✅ Connected to node at {}", api_url);
        }
        Ok(response) => {
            println!("⚠️  Node responded with status: {}", response.status());
            println!("   Continuing anyway...");
        }
        Err(e) => {
            println!("❌ Failed to connect to node: {}", e);
            println!("   Make sure a node is running at {}", api_url);
            println!("   You can start one with: just run-devnet");
            return Ok(());
        }
    }
    
    // Step 3: Submit a test job
    println!("\n3. Submitting test mesh job...");
    let test_job = serde_json::json!({
        "manifest_cid": "bafy2bzaceczkxubnrr4dzs7b2t3vv5jm3z7xzx7xzx7xzx7xzx7xzx7x",
        "spec_bytes": base64::engine::general_purpose::STANDARD.encode(b"Hello, ICN mesh!"),
        "cost_mana": 10
    });
    
    let url = format!("{}/mesh/submit", api_url);
    match client.post(&url).json(&test_job).send().await {
        Ok(response) if response.status().is_success() => {
            let result: serde_json::Value = response.json().await?;
            if let Some(job_id) = result.get("job_id") {
                println!("✅ Test job submitted with ID: {}", job_id);
            } else {
                println!("✅ Test job submitted successfully");
            }
        }
        Ok(response) => {
            println!("⚠️  Job submission failed with status: {}", response.status());
            let text = response.text().await.unwrap_or_default();
            println!("   Response: {}", text);
        }
        Err(e) => {
            println!("⚠️  Job submission error: {}", e);
        }
    }
    
    // Step 4: Submit a test proposal
    println!("\n4. Submitting test governance proposal...");
    let test_proposal = serde_json::json!({
        "proposer_did": did,
        "proposal_type_json": {"GenericText": "Test proposal for developer onboarding"},
        "description": "This is a test proposal created during developer onboarding",
        "duration_secs": 86400,
        "quorum": null,
        "threshold": null
    });
    
    let url = format!("{}/governance/submit", api_url);
    let mut proposal_id = None;
    match client.post(&url).json(&test_proposal).send().await {
        Ok(response) if response.status().is_success() => {
            let result: serde_json::Value = response.json().await?;
            if let Some(id) = result.get("proposal_id") {
                proposal_id = Some(id.to_string().trim_matches('"').to_string());
                println!("✅ Test proposal submitted with ID: {}", id);
            } else {
                println!("✅ Test proposal submitted successfully");
            }
        }
        Ok(response) => {
            println!("⚠️  Proposal submission failed with status: {}", response.status());
            let text = response.text().await.unwrap_or_default();
            println!("   Response: {}", text);
        }
        Err(e) => {
            println!("⚠️  Proposal submission error: {}", e);
        }
    }
    
    // Step 5: Vote on the proposal if we got an ID
    if let Some(prop_id) = proposal_id {
        println!("\n5. Voting on test proposal...");
        let vote_request = serde_json::json!({
            "voter_did": did,
            "proposal_id": prop_id,
            "vote_option": "yes"
        });
        
        let url = format!("{}/governance/vote", api_url);
        match client.post(&url).json(&vote_request).send().await {
            Ok(response) if response.status().is_success() => {
                println!("✅ Vote cast successfully");
            }
            Ok(response) => {
                println!("⚠️  Vote failed with status: {}", response.status());
                let text = response.text().await.unwrap_or_default();
                println!("   Response: {}", text);
            }
            Err(e) => {
                println!("⚠️  Vote error: {}", e);
            }
        }
    } else {
        println!("\n5. Skipping vote (no proposal ID available)");
    }
    
    println!("\n🎉 Developer onboarding complete!");
    println!("Next steps:");
    println!("  - Read the docs at docs/DEVELOPER_GUIDE.md");
    println!("  - Explore the codebase in crates/");
    println!("  - Check out open issues on GitHub");
    println!("  - Join the community discussions");
    
    Ok(())
}

fn handle_wizard_onboard_federation(config: &str) -> Result<(), anyhow::Error> {
    use std::io::{self, Write};
    
    println!("🏛️  ICN Federation Onboarding Wizard");
    println!("====================================");
    
    // Cooperative Information
    println!("\n1. Cooperative Information");
    print!("Cooperative name: ");
    io::stdout().flush()?;
    let mut coop_name = String::new();
    io::stdin().read_line(&mut coop_name)?;
    let coop_name = coop_name.trim();
    
    print!("Cooperative type (consumer, worker, platform, multi-stakeholder): ");
    io::stdout().flush()?;
    let mut coop_type = String::new();
    io::stdin().read_line(&mut coop_type)?;
    let coop_type = coop_type.trim();
    
    print!("Brief description: ");
    io::stdout().flush()?;
    let mut description = String::new();
    io::stdin().read_line(&mut description)?;
    let description = description.trim();
    
    // Network Configuration
    println!("\n2. Network Configuration");
    print!("Node name (default: {}-node): ", coop_name.replace(' ', "-"));
    io::stdout().flush()?;
    let mut node_name = String::new();
    io::stdin().read_line(&mut node_name)?;
    let node_name = if node_name.trim().is_empty() {
        format!("{}-node", coop_name.replace(' ', "-"))
    } else {
        node_name.trim().to_string()
    };
    
    print!("HTTP listen address (default: 0.0.0.0:7845): ");
    io::stdout().flush()?;
    let mut listen_addr = String::new();
    io::stdin().read_line(&mut listen_addr)?;
    let listen_addr = if listen_addr.trim().is_empty() {
        "0.0.0.0:7845".to_string()
    } else {
        listen_addr.trim().to_string()
    };
    
    print!("API key for authentication: ");
    io::stdout().flush()?;
    let mut api_key = String::new();
    io::stdin().read_line(&mut api_key)?;
    let api_key = api_key.trim();
    
    // Federation Peers
    println!("\n3. Federation Setup");
    print!("Bootstrap peers (comma-separated, press Enter if starting new federation): ");
    io::stdout().flush()?;
    let mut peers_input = String::new();
    io::stdin().read_line(&mut peers_input)?;
    let peers: Vec<String> = peers_input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    
    // Generate identity
    println!("\n4. Generating federation identity...");
    let (sk, pk) = generate_ed25519_keypair();
    let did = icn_identity::did_key_from_verifying_key(&pk);
    let sk_bs58 = bs58::encode(sk.to_bytes()).into_string();
    println!("✅ Node DID: {}", did);
    
    // Create configuration
    let cfg = toml::toml! {
        [cooperative]
        name = coop_name
        type = coop_type
        description = description
        
        [node]
        node_name = node_name
        http_listen_addr = listen_addr
        storage_backend = "sqlite"
        storage_path = "./icn_data/node.sqlite"
        mana_ledger_backend = "sled"
        mana_ledger_path = "./icn_data/mana.sled"
        api_key = api_key
        open_rate_limit = 60
        enable_p2p = true
        p2p_listen_addr = "/ip4/0.0.0.0/tcp/4001"
        
        [identity]
        node_did = did
        node_private_key_bs58 = sk_bs58
        
        [federation]
        bootstrap_peers = peers
    };
    
    std::fs::write(config, toml::to_string_pretty(&cfg)?)?;
    
    println!("\n🎉 Federation configuration complete!");
    println!("Configuration saved to: {}", config);
    println!("\nNext steps:");
    println!("1. Review the configuration file");
    println!("2. Start your node: cargo run -p icn-node -- --config {}", config);
    println!("3. Share your node's P2P address with other federations");
    println!("4. Read the deployment guide: docs/deployment-guide.md");
    
    Ok(())
}

// --- Cooperative Discovery Handlers ---

async fn handle_coop_register(
    cli: &Cli,
    client: &Client,
    profile_json_or_stdin: &str,
) -> Result<(), anyhow::Error> {
    let profile_content = if profile_json_or_stdin == "-" {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else {
        profile_json_or_stdin.to_string()
    };

    let profile: JsonValue = serde_json::from_str(&profile_content)?;
    let response: JsonValue = post_request(
        &cli.api_url,
        client,
        "/cooperative/register",
        &profile,
        cli.api_key.as_deref(),
    )
    .await?;
    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn handle_coop_search(
    cli: &Cli,
    client: &Client,
    coop_type: &Option<String>,
    capabilities: &Option<String>,
    country: &Option<String>,
    region: &Option<String>,
    locality: &Option<String>,
    federation: &Option<String>,
    query: &Option<String>,
    limit: &usize,
) -> Result<(), anyhow::Error> {
    let capabilities_vec = if let Some(caps) = capabilities {
        caps.split(',').map(|s| s.trim().to_string()).collect()
    } else {
        Vec::new()
    };

    let search_filter = serde_json::json!({
        "cooperative_type": coop_type,
        "required_capabilities": capabilities_vec,
        "country": country,
        "region": region,
        "locality": locality,
        "federation": federation,
        "search_query": query,
        "limit": limit
    });

    let results: JsonValue = post_request(
        &cli.api_url,
        client,
        "/cooperative/search",
        &search_filter,
        cli.api_key.as_deref(),
    )
    .await?;
    println!("{}", serde_json::to_string_pretty(&results)?);
    Ok(())
}

async fn handle_coop_profile(
    cli: &Cli,
    client: &Client,
    did: &str,
) -> Result<(), anyhow::Error> {
    let profile: JsonValue = get_request(
        &cli.api_url,
        client,
        &format!("/cooperative/profile/{}", did),
        cli.api_key.as_deref(),
    )
    .await?;
    println!("{}", serde_json::to_string_pretty(&profile)?);
    Ok(())
}

async fn handle_coop_trust(
    cli: &Cli,
    client: &Client,
    trust_json_or_stdin: &str,
) -> Result<(), anyhow::Error> {
    let trust_content = if trust_json_or_stdin == "-" {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else {
        trust_json_or_stdin.to_string()
    };

    let trust: JsonValue = serde_json::from_str(&trust_content)?;
    let response: JsonValue = post_request(
        &cli.api_url,
        client,
        "/cooperative/trust",
        &trust,
        cli.api_key.as_deref(),
    )
    .await?;
    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}

async fn handle_coop_trust_list(
    cli: &Cli,
    client: &Client,
    did: &str,
) -> Result<(), anyhow::Error> {
    let relationships: JsonValue = get_request(
        &cli.api_url,
        client,
        &format!("/cooperative/trust/{}", did),
        cli.api_key.as_deref(),
    )
    .await?;
    println!("{}", serde_json::to_string_pretty(&relationships)?);
    Ok(())
}

async fn handle_coop_providers(
    cli: &Cli,
    client: &Client,
    capability_type: &str,
) -> Result<(), anyhow::Error> {
    let providers: JsonValue = get_request(
        &cli.api_url,
        client,
        &format!("/cooperative/capabilities/{}", capability_type),
        cli.api_key.as_deref(),
    )
    .await?;
    println!("{}", serde_json::to_string_pretty(&providers)?);
    Ok(())
}

async fn handle_coop_stats(cli: &Cli, client: &Client) -> Result<(), anyhow::Error> {
    let stats: JsonValue = get_request(
        &cli.api_url,
        client,
        "/cooperative/registry/stats",
        cli.api_key.as_deref(),
    )
    .await?;
    println!("{}", serde_json::to_string_pretty(&stats)?);
    Ok(())
}

async fn handle_coop_onboard(
    cli: &Cli,
    client: &Client,
    dry_run: &bool,
) -> Result<(), anyhow::Error> {
    use std::io::{self, Write};

    println!("🌟 Welcome to ICN Cooperative Onboarding!");
    println!("Let's get your cooperative registered in the federation.");
    println!();

    // Basic cooperative information
    print!("Cooperative name: ");
    io::stdout().flush()?;
    let mut name = String::new();
    io::stdin().read_line(&mut name)?;
    let name = name.trim().to_string();

    print!("Brief description: ");
    io::stdout().flush()?;
    let mut description = String::new();
    io::stdin().read_line(&mut description)?;
    let description = description.trim().to_string();

    // Cooperative type
    println!("\nSelect cooperative type:");
    println!("1) Worker cooperative");
    println!("2) Consumer cooperative");
    println!("3) Multi-stakeholder cooperative");
    println!("4) Housing cooperative");
    println!("5) Financial/Credit union");
    println!("6) Platform cooperative");
    println!("7) Agricultural cooperative");
    println!("8) Education/Research cooperative");
    println!("9) Energy cooperative");
    println!("10) Healthcare cooperative");
    println!("11) Commons/Community land trust");
    println!("12) General purpose");
    print!("Choice (1-12): ");
    io::stdout().flush()?;
    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;

    let cooperative_type = match choice.trim() {
        "1" => "Worker",
        "2" => "Consumer",
        "3" => "MultiStakeholder",
        "4" => "Housing",
        "5" => "Financial",
        "6" => "Platform",
        "7" => "Agricultural",
        "8" => "Education",
        "9" => "Energy",
        "10" => "Healthcare",
        "11" => "Commons",
        _ => "General",
    };

    // Geographic scope
    print!("\nCountry (optional): ");
    io::stdout().flush()?;
    let mut country = String::new();
    io::stdin().read_line(&mut country)?;
    let country = country.trim();

    print!("Region/State (optional): ");
    io::stdout().flush()?;
    let mut region = String::new();
    io::stdin().read_line(&mut region)?;
    let region = region.trim();

    print!("City/Locality (optional): ");
    io::stdout().flush()?;
    let mut locality = String::new();
    io::stdin().read_line(&mut locality)?;
    let locality = locality.trim();

    print!("Operates globally? (y/n): ");
    io::stdout().flush()?;
    let mut global_input = String::new();
    io::stdin().read_line(&mut global_input)?;
    let global = global_input.trim().to_lowercase() == "y" || global_input.trim().to_lowercase() == "yes";

    // Capabilities
    println!("\nWhat capabilities does your cooperative offer?");
    println!("Enter capability types (one per line, empty line to finish):");
    println!("Examples: web_development, housing, food_production, education, healthcare, etc.");
    
    let mut capabilities = Vec::new();
    loop {
        print!("> ");
        io::stdout().flush()?;
        let mut capability = String::new();
        io::stdin().read_line(&mut capability)?;
        let capability = capability.trim();
        if capability.is_empty() {
            break;
        }

        print!("  Description for '{}': ", capability);
        io::stdout().flush()?;
        let mut desc = String::new();
        io::stdin().read_line(&mut desc)?;
        let desc = desc.trim();

        print!("  Exchange model (time_bank/mutual_credit/mana/market): ");
        io::stdout().flush()?;
        let mut exchange = String::new();
        io::stdin().read_line(&mut exchange)?;
        let exchange = exchange.trim();

        capabilities.push(serde_json::json!({
            "capability_type": capability,
            "description": desc,
            "specifications": {},
            "available": true,
            "exchange_model": exchange
        }));
    }

    // Member count
    print!("\nApproximate member count (optional): ");
    io::stdout().flush()?;
    let mut member_count_str = String::new();
    io::stdin().read_line(&mut member_count_str)?;
    let member_count = member_count_str.trim().parse::<u32>().ok();

    // Contact information
    print!("Website URL (optional): ");
    io::stdout().flush()?;
    let mut website = String::new();
    io::stdin().read_line(&mut website)?;
    let website = website.trim();

    print!("Contact email (optional): ");
    io::stdout().flush()?;
    let mut email = String::new();
    io::stdin().read_line(&mut email)?;
    let email = email.trim();

    // Get node DID for the cooperative
    let node_info: JsonValue = get_request(&cli.api_url, client, "/keys", cli.api_key.as_deref()).await?;
    let did = node_info["did"].as_str().unwrap_or("did:key:unknown").to_string();

    // Build the cooperative profile
    let profile = serde_json::json!({
        "did": did,
        "name": name,
        "cooperative_type": cooperative_type,
        "description": description,
                        "website": if website.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(website.to_string()) },
                "contact_email": if email.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(email.to_string()) },
                "geographic_scope": {
                    "country": if country.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(country.to_string()) },
                    "region": if region.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(region.to_string()) },
                    "locality": if locality.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(locality.to_string()) },
            "global": global
        },
        "capabilities": capabilities,
        "member_count": member_count,
        "founded_year": null,
        "legal_structure": null,
        "federation_memberships": [],
        "trusted_cooperatives": [],
        "public_keys": {},
        "created_at": chrono::Utc::now().timestamp(),
        "updated_at": chrono::Utc::now().timestamp(),
        "signature": null
    });

    println!("\n🎉 Cooperative profile created!");
    println!("{}", serde_json::to_string_pretty(&profile)?);

    if *dry_run {
        println!("\n⚠️  Dry run mode - profile not registered with node");
        return Ok(());
    }

    println!("\nRegister this profile with the federation? (y/n): ");
    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm)?;
    
    if confirm.trim().to_lowercase() == "y" || confirm.trim().to_lowercase() == "yes" {
        let response: JsonValue = post_request(
            &cli.api_url,
            client,
            "/cooperative/register",
            &profile,
            cli.api_key.as_deref(),
        )
        .await?;
        
        println!("\n✅ Cooperative successfully registered!");
        println!("{}", serde_json::to_string_pretty(&response)?);
        
        println!("\n🔍 Next steps:");
        println!("1. Search for other cooperatives: icn-cli cooperative search");
        println!("2. Build trust relationships: icn-cli cooperative trust");
        println!("3. Explore capabilities: icn-cli cooperative providers <capability>");
        println!("4. Check federation stats: icn-cli cooperative stats");
    } else {
        println!("Registration cancelled.");
    }

    Ok(())
}

async fn handle_monitor_uptime(cli: &Cli, client: &Client) -> Result<(), anyhow::Error> {
    let metrics = get_request::<String>(&cli.api_url, client, "/metrics", cli.api_key.as_deref()).await?;
    let scrape = prometheus_parse::Scrape::parse(metrics.lines().map(|l| Ok(l.to_string())))?;
    let uptime = scrape
        .samples
        .iter()
        .find(|s| s.metric == "node_uptime_seconds")
        .and_then(|s| match s.value {
            prometheus_parse::Value::Counter(v)
            | prometheus_parse::Value::Gauge(v)
            | prometheus_parse::Value::Untyped(v) => Some(v),
            _ => None,
        })
        .unwrap_or(0.0);
    println!("Uptime: {} seconds", uptime);
    Ok(())
}

async fn handle_emergency_list(cli: &Cli, client: &Client) -> Result<(), anyhow::Error> {
    let v: serde_json::Value = get_request(&cli.api_url, client, "/emergency/requests", cli.api_key.as_deref()).await?;
    println!("{}", serde_json::to_string_pretty(&v)?);
    Ok(())
}

async fn handle_emergency_request(
    cli: &Cli,
    client: &Client,
    request_json_or_stdin: &str,
) -> Result<(), anyhow::Error> {
    let content = if request_json_or_stdin == "-" {
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        buf
    } else {
        request_json_or_stdin.to_string()
    };
    let body: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("Invalid aid request JSON: {}", e))?;
    let _: serde_json::Value =
        post_request(&cli.api_url, client, "/emergency/request", &body, cli.api_key.as_deref()).await?;
    println!("Aid request submitted");
    Ok(())
}

async fn handle_aid_list(cli: &Cli, client: &Client) -> Result<(), anyhow::Error> {
    let v: serde_json::Value = get_request(&cli.api_url, client, "/aid/resources", cli.api_key.as_deref()).await?;
    println!("{}", serde_json::to_string_pretty(&v)?);
    Ok(())
}

async fn handle_aid_register(
    cli: &Cli,
    client: &Client,
    resource_json_or_stdin: &str,
) -> Result<(), anyhow::Error> {
    let content = if resource_json_or_stdin == "-" {
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        buf
    } else {
        resource_json_or_stdin.to_string()
    };
    let body: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("Invalid aid resource JSON: {}", e))?;
    let _: serde_json::Value = post_request(&cli.api_url, client, "/aid/resource", &body, cli.api_key.as_deref()).await?;
    println!("Aid resource registered");
    Ok(())
}

async fn handle_identity_generate_remote(
    cli: &Cli,
    client: &Client,
    issuer: &str,
    holder: &str,
    claim_type: &str,
    schema: &str,
    backend: &str,
    public_inputs: &Option<String>,
) -> Result<(), anyhow::Error> {
    let req = icn_api::identity_trait::GenerateProofRequest {
        issuer: Did::from_str(issuer)?,
        holder: Did::from_str(holder)?,
        claim_type: claim_type.to_string(),
        schema: icn_common::parse_cid_from_string(schema)?,
        backend: backend.to_string(),
        public_inputs: if let Some(s) = public_inputs {
            Some(serde_json::from_str(s)?)
        } else {
            None
        },
    };
    let proof: icn_common::ZkCredentialProof = post_request(
        &cli.api_url,
        client,
        "/identity/generate-proof",
        &req,
        cli.api_key.as_deref(),
    )
    .await?;
    println!("{}", serde_json::to_string_pretty(&proof)?);
    Ok(())
}

async fn handle_identity_verify_remote(
    cli: &Cli,
    client: &Client,
    proof_json_or_stdin: &str,
) -> Result<(), anyhow::Error> {
    let content = if proof_json_or_stdin == "-" {
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        buf
    } else {
        proof_json_or_stdin.to_string()
    };
    let proof: icn_common::ZkCredentialProof = serde_json::from_str(&content)?;
    let resp: serde_json::Value = post_request(
        &cli.api_url,
        client,
        "/identity/verify-proof",
        &proof,
        cli.api_key.as_deref(),
    )
    .await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

// === Trust Command Handlers ===

async fn handle_trust_get(
    cli: &Cli,
    client: &Client,
    from: &str,
    to: &str,
    context: &str,
) -> Result<(), anyhow::Error> {
    let path = format!("/trust/relationship?from={}&to={}&context={}", from, to, context);
    let resp: serde_json::Value = get_request(&cli.api_url, client, &path, cli.api_key.as_deref()).await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

async fn handle_trust_list(
    cli: &Cli,
    client: &Client,
    entity: &str,
    context: &Option<String>,
    min_level: &Option<String>,
    federation: &Option<String>,
    include_inherited: bool,
    include_cross_federation: bool,
) -> Result<(), anyhow::Error> {
    let mut path = format!("/trust/relationships?entity={}", entity);
    if let Some(ctx) = context {
        path.push_str(&format!("&context={}", ctx));
    }
    if let Some(level) = min_level {
        path.push_str(&format!("&min_level={}", level));
    }
    if let Some(fed) = federation {
        path.push_str(&format!("&federation={}", fed));
    }
    path.push_str(&format!("&include_inherited={}&include_cross_federation={}", 
        include_inherited, include_cross_federation));
    
    let resp: serde_json::Value = get_request(&cli.api_url, client, &path, cli.api_key.as_deref()).await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

async fn handle_trust_paths(
    cli: &Cli,
    client: &Client,
    from: &str,
    to: &str,
    context: &str,
    max_length: usize,
    max_paths: usize,
    min_level: &Option<String>,
) -> Result<(), anyhow::Error> {
    let mut path = format!("/trust/paths?from={}&to={}&context={}&max_length={}&max_paths={}", 
        from, to, context, max_length, max_paths);
    if let Some(level) = min_level {
        path.push_str(&format!("&min_level={}", level));
    }
    
    let resp: serde_json::Value = get_request(&cli.api_url, client, &path, cli.api_key.as_deref()).await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

async fn handle_trust_score(
    cli: &Cli,
    client: &Client,
    entity: &str,
) -> Result<(), anyhow::Error> {
    let path = format!("/trust/score?entity={}", entity);
    let resp: serde_json::Value = get_request(&cli.api_url, client, &path, cli.api_key.as_deref()).await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

async fn handle_trust_scores(
    cli: &Cli,
    client: &Client,
    entities: &str,
) -> Result<(), anyhow::Error> {
    let path = format!("/trust/scores?entities={}", entities);
    let resp: serde_json::Value = get_request(&cli.api_url, client, &path, cli.api_key.as_deref()).await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

async fn handle_trust_update(
    cli: &Cli,
    client: &Client,
    update_json_or_stdin: &str,
) -> Result<(), anyhow::Error> {
    let content = if update_json_or_stdin == "-" {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else {
        update_json_or_stdin.to_string()
    };
    let update_req: serde_json::Value = serde_json::from_str(&content)?;
    let resp: serde_json::Value = post_request(
        &cli.api_url,
        client,
        "/trust/update",
        &update_req,
        cli.api_key.as_deref(),
    ).await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

async fn handle_trust_remove(
    cli: &Cli,
    client: &Client,
    from: &str,
    to: &str,
    context: &str,
) -> Result<(), anyhow::Error> {
    let path = format!("/trust/remove?from={}&to={}&context={}", from, to, context);
    let resp: serde_json::Value = post_request(
        &cli.api_url,
        client,
        &path,
        &serde_json::json!({}),
        cli.api_key.as_deref(),
    ).await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

async fn handle_trust_stats(
    cli: &Cli,
    client: &Client,
) -> Result<(), anyhow::Error> {
    let resp: serde_json::Value = get_request(&cli.api_url, client, "/trust/stats", cli.api_key.as_deref()).await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

async fn handle_trust_federation_stats(
    cli: &Cli,
    client: &Client,
    federation: &str,
) -> Result<(), anyhow::Error> {
    let path = format!("/trust/federation-stats?federation={}", federation);
    let resp: serde_json::Value = get_request(&cli.api_url, client, &path, cli.api_key.as_deref()).await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

async fn handle_trust_search(
    cli: &Cli,
    client: &Client,
    context: &Option<String>,
    min_level: &Option<String>,
    federation: &Option<String>,
    limit: usize,
    offset: usize,
) -> Result<(), anyhow::Error> {
    let mut path = format!("/trust/search?limit={}&offset={}", limit, offset);
    if let Some(ctx) = context {
        path.push_str(&format!("&context={}", ctx));
    }
    if let Some(level) = min_level {
        path.push_str(&format!("&min_level={}", level));
    }
    if let Some(fed) = federation {
        path.push_str(&format!("&federation={}", fed));
    }
    
    let resp: serde_json::Value = get_request(&cli.api_url, client, &path, cli.api_key.as_deref()).await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

async fn handle_trust_validate(
    cli: &Cli,
    client: &Client,
    actor: &str,
    target: &str,
    context: &str,
    operation: &str,
) -> Result<(), anyhow::Error> {
    let path = format!("/trust/validate?actor={}&target={}&context={}&operation={}", 
        actor, target, context, operation);
    let resp: serde_json::Value = get_request(&cli.api_url, client, &path, cli.api_key.as_deref()).await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

async fn handle_trust_neighbors(
    cli: &Cli,
    client: &Client,
    entity: &str,
    max_distance: usize,
    min_level: &Option<String>,
) -> Result<(), anyhow::Error> {
    let mut path = format!("/trust/neighbors?entity={}&max_distance={}", entity, max_distance);
    if let Some(level) = min_level {
        path.push_str(&format!("&min_level={}", level));
    }
    
    let resp: serde_json::Value = get_request(&cli.api_url, client, &path, cli.api_key.as_deref()).await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

async fn handle_trust_recalculate(
    cli: &Cli,
    client: &Client,
    entities: &Option<String>,
) -> Result<(), anyhow::Error> {
    let mut path = "/trust/recalculate".to_string();
    if let Some(ents) = entities {
        path.push_str(&format!("?entities={}", ents));
    }
    
    let resp: serde_json::Value = post_request(
        &cli.api_url,
        client,
        &path,
        &serde_json::json!({}),
        cli.api_key.as_deref(),
    ).await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

// === Federation Trust Command Handlers ===

async fn handle_fed_trust_configure(
    cli: &Cli,
    client: &Client,
    federation_id: &str,
    policy_json_or_stdin: &str,
) -> Result<(), anyhow::Error> {
    let policy_content = if policy_json_or_stdin == "-" {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else {
        policy_json_or_stdin.to_string()
    };

    let policy: JsonValue = serde_json::from_str(&policy_content)?;
    let request = serde_json::json!({
        "federation_id": federation_id,
        "policy": policy
    });

    let response: JsonValue = post_request(
        &cli.api_url,
        client,
        "/federation/trust/configure",
        &request,
        cli.api_key.as_deref(),
    )
    .await?;
    
    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}

async fn handle_fed_trust_add(
    cli: &Cli,
    client: &Client,
    trust_json_or_stdin: &str,
) -> Result<(), anyhow::Error> {
    let trust_content = if trust_json_or_stdin == "-" {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else {
        trust_json_or_stdin.to_string()
    };

    let trust: JsonValue = serde_json::from_str(&trust_content)?;
    let response: JsonValue = post_request(
        &cli.api_url,
        client,
        "/federation/trust/add",
        &trust,
        cli.api_key.as_deref(),
    )
    .await?;
    
    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}

async fn handle_fed_trust_remove(
    cli: &Cli,
    client: &Client,
    from: &str,
    to: &str,
    context: &str,
    federation: &str,
) -> Result<(), anyhow::Error> {
    let path = format!(
        "/federation/trust/remove?from={}&to={}&context={}&federation={}",
        from, to, context, federation
    );
    
    let response: JsonValue = post_request(
        &cli.api_url,
        client,
        &path,
        &serde_json::json!({}),
        cli.api_key.as_deref(),
    )
    .await?;
    
    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}

async fn handle_fed_trust_list(
    cli: &Cli,
    client: &Client,
    federation_id: &str,
    context: &Option<String>,
    min_level: &Option<String>,
) -> Result<(), anyhow::Error> {
    let mut path = format!("/federation/trust/list?federation_id={}", federation_id);
    
    if let Some(ctx) = context {
        path.push_str(&format!("&context={}", ctx));
    }
    if let Some(level) = min_level {
        path.push_str(&format!("&min_level={}", level));
    }
    
    let response: JsonValue = get_request(&cli.api_url, client, &path, cli.api_key.as_deref()).await?;
    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}

async fn handle_fed_trust_validate(
    cli: &Cli,
    client: &Client,
    actor: &str,
    target: &str,
    context: &str,
    operation: &str,
    federation: &str,
) -> Result<(), anyhow::Error> {
    let path = format!(
        "/federation/trust/validate?actor={}&target={}&context={}&operation={}&federation={}",
        actor, target, context, operation, federation
    );
    
    let response: JsonValue = get_request(&cli.api_url, client, &path, cli.api_key.as_deref()).await?;
    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}

async fn handle_fed_trust_bridge(
    cli: &Cli,
    client: &Client,
    bridge_json_or_stdin: &str,
) -> Result<(), anyhow::Error> {
    let bridge_content = if bridge_json_or_stdin == "-" {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else {
        bridge_json_or_stdin.to_string()
    };

    let bridge: JsonValue = serde_json::from_str(&bridge_content)?;
    let response: JsonValue = post_request(
        &cli.api_url,
        client,
        "/federation/trust/bridge",
        &bridge,
        cli.api_key.as_deref(),
    )
    .await?;
    
    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}

async fn handle_fed_trust_bootstrap(
    cli: &Cli,
    client: &Client,
    peer: &str,
    contexts: &str,
    trust_level: &str,
) -> Result<(), anyhow::Error> {
    let context_list: Vec<String> = contexts
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();
    
    let request = serde_json::json!({
        "peer": peer,
        "contexts": context_list,
        "trust_level": trust_level
    });

    let response: JsonValue = post_request(
        &cli.api_url,
        client,
        "/federation/trust/bootstrap",
        &request,
        cli.api_key.as_deref(),
    )
    .await?;
    
    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}

// === Federation Metadata Command Handlers ===

async fn handle_fed_metadata_get(
    cli: &Cli,
    client: &Client,
    federation_id: &str,
) -> Result<(), anyhow::Error> {
    let path = format!("/federation/metadata/{}", federation_id);
    let response: JsonValue = get_request(&cli.api_url, client, &path, cli.api_key.as_deref()).await?;
    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}

async fn handle_fed_metadata_set(
    cli: &Cli,
    client: &Client,
    federation_id: &str,
    metadata_json_or_stdin: &str,
) -> Result<(), anyhow::Error> {
    let metadata_content = if metadata_json_or_stdin == "-" {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else {
        metadata_json_or_stdin.to_string()
    };

    let metadata: JsonValue = serde_json::from_str(&metadata_content)?;
    let request = serde_json::json!({
        "federation_id": federation_id,
        "metadata": metadata
    });

    let response: JsonValue = post_request(
        &cli.api_url,
        client,
        "/federation/metadata/set",
        &request,
        cli.api_key.as_deref(),
    )
    .await?;
    
    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}

async fn handle_fed_metadata_scope(
    cli: &Cli,
    client: &Client,
    federation_id: &str,
    scope_json_or_stdin: &str,
) -> Result<(), anyhow::Error> {
    let scope_content = if scope_json_or_stdin == "-" {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else {
        scope_json_or_stdin.to_string()
    };

    let scope: JsonValue = serde_json::from_str(&scope_content)?;
    let request = serde_json::json!({
        "federation_id": federation_id,
        "scope": scope
    });

    let response: JsonValue = post_request(
        &cli.api_url,
        client,
        "/federation/metadata/scope",
        &request,
        cli.api_key.as_deref(),
    )
    .await?;
    
    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}

async fn handle_fed_metadata_quorum(
    cli: &Cli,
    client: &Client,
    federation_id: &str,
    policy_json_or_stdin: &str,
) -> Result<(), anyhow::Error> {
    let policy_content = if policy_json_or_stdin == "-" {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else {
        policy_json_or_stdin.to_string()
    };

    let policy: JsonValue = serde_json::from_str(&policy_content)?;
    let request = serde_json::json!({
        "federation_id": federation_id,
        "quorum_policy": policy
    });

    let response: JsonValue = post_request(
        &cli.api_url,
        client,
        "/federation/metadata/quorum",
        &request,
        cli.api_key.as_deref(),
    )
    .await?;
    
    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}

async fn handle_fed_metadata_members(
    cli: &Cli,
    client: &Client,
    federation_id: &str,
) -> Result<(), anyhow::Error> {
    let path = format!("/federation/metadata/{}/members", federation_id);
    let response: JsonValue = get_request(&cli.api_url, client, &path, cli.api_key.as_deref()).await?;
    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}

async fn handle_fed_metadata_add_member(
    cli: &Cli,
    client: &Client,
    federation_id: &str,
    member_did: &str,
    profile_json_or_stdin: &str,
) -> Result<(), anyhow::Error> {
    let profile_content = if profile_json_or_stdin == "-" {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else {
        profile_json_or_stdin.to_string()
    };

    let profile: JsonValue = serde_json::from_str(&profile_content)?;
    let request = serde_json::json!({
        "federation_id": federation_id,
        "member_did": member_did,
        "profile": profile
    });

    let response: JsonValue = post_request(
        &cli.api_url,
        client,
        "/federation/metadata/add-member",
        &request,
        cli.api_key.as_deref(),
    )
    .await?;
    
    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}

async fn handle_fed_metadata_remove_member(
    cli: &Cli,
    client: &Client,
    federation_id: &str,
    member_did: &str,
) -> Result<(), anyhow::Error> {
    let request = serde_json::json!({
        "federation_id": federation_id,
        "member_did": member_did
    });

    let response: JsonValue = post_request(
        &cli.api_url,
        client,
        "/federation/metadata/remove-member",
        &request,
        cli.api_key.as_deref(),
    )
    .await?;
    
    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}

// === Federation DID Command Handlers ===

async fn handle_fed_did_generate(
    cli: &Cli,
    client: &Client,
    federation_id: &str,
    output: &Option<String>,
) -> Result<(), anyhow::Error> {
    let request = serde_json::json!({
        "federation_id": federation_id
    });

    let response: JsonValue = post_request(
        &cli.api_url,
        client,
        "/federation/did/generate",
        &request,
        cli.api_key.as_deref(),
    )
    .await?;

    let output_content = serde_json::to_string_pretty(&response)?;
    
    if let Some(file_path) = output {
        std::fs::write(file_path, &output_content)?;
        println!("DID document saved to: {}", file_path);
    } else {
        println!("{}", output_content);
    }
    
    Ok(())
}

async fn handle_fed_did_verify(
    cli: &Cli,
    client: &Client,
    document_json_or_stdin: &str,
) -> Result<(), anyhow::Error> {
    let document_content = if document_json_or_stdin == "-" {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else {
        document_json_or_stdin.to_string()
    };

    let document: JsonValue = serde_json::from_str(&document_content)?;
    let response: JsonValue = post_request(
        &cli.api_url,
        client,
        "/federation/did/verify",
        &document,
        cli.api_key.as_deref(),
    )
    .await?;
    
    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}

async fn handle_fed_did_publish(
    cli: &Cli,
    client: &Client,
    federation_id: &str,
    document_json_or_stdin: &str,
) -> Result<(), anyhow::Error> {
    let document_content = if document_json_or_stdin == "-" {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else {
        document_json_or_stdin.to_string()
    };

    let document: JsonValue = serde_json::from_str(&document_content)?;
    let request = serde_json::json!({
        "federation_id": federation_id,
        "document": document
    });

    let response: JsonValue = post_request(
        &cli.api_url,
        client,
        "/federation/did/publish",
        &request,
        cli.api_key.as_deref(),
    )
    .await?;
    
    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}

async fn handle_fed_did_resolve(
    cli: &Cli,
    client: &Client,
    did: &str,
    federation: &Option<String>,
) -> Result<(), anyhow::Error> {
    let mut path = format!("/federation/did/resolve?did={}", did);
    
    if let Some(fed) = federation {
        path.push_str(&format!("&federation={}", fed));
    }
    
    let response: JsonValue = get_request(&cli.api_url, client, &path, cli.api_key.as_deref()).await?;
    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}
