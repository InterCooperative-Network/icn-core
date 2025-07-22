#![doc = include_str!("../README.md")]
#![allow(clippy::uninlined_format_args)]

//! # ICN CLI Crate
//! This crate provides a command-line interface (CLI) for interacting with an ICN HTTP node.

mod credential_lifecycle;

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
use chrono;
use icn_action::{Action, ActionEncoder, QrGenerator, VoteChoice};
use icn_api::governance_trait::{
    CastVoteRequest as ApiCastVoteRequest, SubmitProposalRequest as ApiSubmitProposalRequest,
};
use icn_api::identity_trait::{BatchVerificationResponse, VerifyProofsRequest};
use icn_ccl::{check_ccl_file, compile_ccl_file, compile_ccl_file_to_wasm, explain_ccl_policy};
use icn_governance::{Proposal, ProposalId};
use icn_identity::generate_ed25519_keypair;
use icn_runtime::context::{Ed25519Signer, Signer};
use icn_templates;

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
    /// Token operations (mint, transfer, balance)
    Token {
        #[clap(subcommand)]
        command: TokenCommands,
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
    /// Credential lifecycle management
    Credential {
        #[clap(subcommand)]
        command: credential_lifecycle::CredentialLifecycleCommands,
    },
    /// QR code and NFC action generation
    Qr {
        #[clap(subcommand)]
        command: QrCommands,
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
    /// Get progress information for a job
    Progress {
        #[clap(help = "Job ID (CID string)")]
        job_id: String,
    },
    /// Stream output from a job (tail-like behavior)
    Tail {
        #[clap(help = "Job ID (CID string)")]
        job_id: String,
        #[clap(
            short,
            long,
            help = "Follow output (like tail -f)",
            default_value = "false"
        )]
        follow: bool,
        #[clap(short, long, help = "Start from sequence number")]
        from: Option<u64>,
    },
    /// Cancel a running job
    Cancel {
        #[clap(help = "Job ID (CID string)")]
        job_id: String,
    },
    /// Resume a failed job from checkpoint
    Resume {
        #[clap(help = "Job ID (CID string)")]
        job_id: String,
    },
    /// Get mesh execution metrics
    Metrics,
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
    /// Discover federations on the network
    #[clap(name = "discover")]
    Discover,
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
        #[clap(
            long,
            help = "API URL of the target node",
            default_value = "http://127.0.0.1:7845"
        )]
        api_url: String,
    },
    /// Federation onboarding wizard - prompts for coop info, generates config
    #[clap(name = "onboard-federation")]
    OnboardFederation {
        #[clap(
            long,
            help = "Output config file",
            default_value = "federation_config.toml"
        )]
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
enum TokenCommands {
    /// Mint new scoped tokens
    Mint {
        #[clap(help = "Token class ID")]
        class_id: String,
        #[clap(help = "Target recipient DID")]
        to_did: String,
        #[clap(help = "Amount to mint")]
        amount: u64,
        #[clap(long, help = "Issuer DID (defaults to node DID)")]
        issuer: Option<String>,
        #[clap(long, help = "Optional scope for token")]
        scope: Option<String>,
    },
    /// Transfer tokens between accounts
    Transfer {
        #[clap(help = "Token class ID")]
        class_id: String,
        #[clap(help = "Source DID")]
        from_did: String,
        #[clap(help = "Target DID")]
        to_did: String,
        #[clap(help = "Amount to transfer")]
        amount: u64,
        #[clap(long, help = "Issuer DID (defaults to node DID)")]
        issuer: Option<String>,
        #[clap(long, help = "Optional scope for transfer")]
        scope: Option<String>,
    },
    /// Check token balance for an account
    Balance {
        #[clap(help = "Token class ID")]
        class_id: String,
        #[clap(help = "Account DID")]
        did: String,
    },
    /// Burn tokens from an account
    Burn {
        #[clap(help = "Token class ID")]
        class_id: String,
        #[clap(help = "Owner DID")]
        from_did: String,
        #[clap(help = "Amount to burn")]
        amount: u64,
        #[clap(long, help = "Issuer DID (defaults to node DID)")]
        issuer: Option<String>,
        #[clap(long, help = "Optional scope for burn")]
        scope: Option<String>,
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

#[derive(Subcommand, Debug)]
enum QrCommands {
    /// Generate QR code for sharing identity (DID)
    #[clap(name = "share-identity")]
    ShareIdentity {
        #[clap(help = "DID to share")]
        did: String,
        #[clap(long, help = "QR code size in pixels", default_value = "256")]
        size: u32,
        #[clap(long, help = "Output file path (optional, defaults to terminal display)")]
        output: Option<String>,
    },
    /// Generate QR code for token transfer
    #[clap(name = "transfer")]
    Transfer {
        #[clap(help = "Token type/name")]
        token: String,
        #[clap(help = "Amount to transfer")]
        amount: u64,
        #[clap(help = "Recipient DID")]
        to: String,
        #[clap(long, help = "Optional memo")]
        memo: Option<String>,
        #[clap(long, help = "QR code size in pixels", default_value = "256")]
        size: u32,
        #[clap(long, help = "Output file path (optional, defaults to terminal display)")]
        output: Option<String>,
    },
    /// Generate QR code for voting on a proposal
    #[clap(name = "vote")]
    Vote {
        #[clap(help = "Proposal CID")]
        proposal: String,
        #[clap(help = "Vote choice: approve, reject, or abstain")]
        vote: String,
        #[clap(long, help = "Voter DID (optional)")]
        voter: Option<String>,
        #[clap(long, help = "QR code size in pixels", default_value = "256")]
        size: u32,
        #[clap(long, help = "Output file path (optional, defaults to terminal display)")]
        output: Option<String>,
    },
    /// Generate QR code for joining a federation
    #[clap(name = "join")]
    Join {
        #[clap(help = "Federation ID or name")]
        federation: String,
        #[clap(long, help = "Invitation code (optional)")]
        code: Option<String>,
        #[clap(long, help = "QR code size in pixels", default_value = "256")]
        size: u32,
        #[clap(long, help = "Output file path (optional, defaults to terminal display)")]
        output: Option<String>,
    },
    /// Generate QR code for verifying a credential
    #[clap(name = "verify-credential")]
    VerifyCredential {
        #[clap(help = "Credential CID")]
        credential: String,
        #[clap(long, help = "Challenge string (optional)")]
        challenge: Option<String>,
        #[clap(long, help = "QR code size in pixels", default_value = "256")]
        size: u32,
        #[clap(long, help = "Output file path (optional, defaults to terminal display)")]
        output: Option<String>,
    },
    /// Decode an ICN action URL and display its contents
    #[clap(name = "decode")]
    Decode {
        #[clap(help = "ICN action URL to decode")]
        url: String,
    },
    /// Generate QR code from any ICN action URL
    #[clap(name = "encode")]
    Encode {
        #[clap(help = "ICN action URL to encode")]
        url: String,
        #[clap(long, help = "QR code size in pixels", default_value = "256")]
        size: u32,
        #[clap(long, help = "Output file path (optional, defaults to terminal display)")]
        output: Option<String>,
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
            MeshCommands::Progress { job_id } => handle_mesh_progress(cli, client, job_id).await?,
            MeshCommands::Tail {
                job_id,
                follow,
                from,
            } => handle_mesh_tail(cli, client, job_id, *follow, *from).await?,
            MeshCommands::Cancel { job_id } => handle_mesh_cancel(cli, client, job_id).await?,
            MeshCommands::Resume { job_id } => handle_mesh_resume(cli, client, job_id).await?,
            MeshCommands::Metrics => handle_mesh_metrics(cli, client).await?,
        },
        Commands::Network { command } => match command {
            NetworkCommands::Stats => handle_network_stats(cli, client).await?,
            NetworkCommands::Ping { peer_id } => handle_network_ping(cli, client, peer_id).await?,
            NetworkCommands::Peers => handle_network_peers(cli, client).await?,
        },
        Commands::Accounts { command } => match command {
            AccountCommands::Balance { did } => handle_account_balance(cli, client, did).await?,
        },
        Commands::Token { command } => match command {
            TokenCommands::Mint {
                class_id,
                to_did,
                amount,
                issuer,
                scope,
            } => handle_token_mint(cli, client, class_id, to_did, *amount, issuer, scope).await?,
            TokenCommands::Transfer {
                class_id,
                from_did,
                to_did,
                amount,
                issuer,
                scope,
            } => {
                handle_token_transfer(
                    cli, client, class_id, from_did, to_did, *amount, issuer, scope,
                )
                .await?
            }
            TokenCommands::Balance { class_id, did } => {
                handle_token_balance(cli, client, class_id, did).await?
            }
            TokenCommands::Burn {
                class_id,
                from_did,
                amount,
                issuer,
                scope,
            } => handle_token_burn(cli, client, class_id, from_did, *amount, issuer, scope).await?,
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
            IdentityCommands::VerifyProofRemote {
                proof_json_or_stdin,
            } => {
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
            FederationCommands::Discover => handle_fed_discover(cli, client).await?,
            FederationCommands::Trust { command } => match command {
                FederationTrustCommands::Configure {
                    federation_id,
                    policy_json_or_stdin,
                } => {
                    handle_fed_trust_configure(cli, client, federation_id, policy_json_or_stdin)
                        .await?
                }
                FederationTrustCommands::Add {
                    trust_json_or_stdin,
                } => handle_fed_trust_add(cli, client, trust_json_or_stdin).await?,
                FederationTrustCommands::Remove {
                    from,
                    to,
                    context,
                    federation,
                } => handle_fed_trust_remove(cli, client, from, to, context, federation).await?,
                FederationTrustCommands::List {
                    federation_id,
                    context,
                    min_level,
                } => handle_fed_trust_list(cli, client, federation_id, context, min_level).await?,
                FederationTrustCommands::Validate {
                    actor,
                    target,
                    context,
                    operation,
                    federation,
                } => {
                    handle_fed_trust_validate(
                        cli, client, actor, target, context, operation, federation,
                    )
                    .await?
                }
                FederationTrustCommands::Bridge {
                    bridge_json_or_stdin,
                } => handle_fed_trust_bridge(cli, client, bridge_json_or_stdin).await?,
                FederationTrustCommands::Bootstrap {
                    peer,
                    contexts,
                    trust_level,
                } => handle_fed_trust_bootstrap(cli, client, peer, contexts, trust_level).await?,
            },
            FederationCommands::Metadata { command } => match command {
                FederationMetadataCommands::Get { federation_id } => {
                    handle_fed_metadata_get(cli, client, federation_id).await?
                }
                FederationMetadataCommands::Set {
                    federation_id,
                    metadata_json_or_stdin,
                } => {
                    handle_fed_metadata_set(cli, client, federation_id, metadata_json_or_stdin)
                        .await?
                }
                FederationMetadataCommands::Scope {
                    federation_id,
                    scope_json_or_stdin,
                } => {
                    handle_fed_metadata_scope(cli, client, federation_id, scope_json_or_stdin)
                        .await?
                }
                FederationMetadataCommands::Quorum {
                    federation_id,
                    policy_json_or_stdin,
                } => {
                    handle_fed_metadata_quorum(cli, client, federation_id, policy_json_or_stdin)
                        .await?
                }
                FederationMetadataCommands::Members { federation_id } => {
                    handle_fed_metadata_members(cli, client, federation_id).await?
                }
                FederationMetadataCommands::AddMember {
                    federation_id,
                    member_did,
                    profile_json_or_stdin,
                } => {
                    handle_fed_metadata_add_member(
                        cli,
                        client,
                        federation_id,
                        member_did,
                        profile_json_or_stdin,
                    )
                    .await?
                }
                FederationMetadataCommands::RemoveMember {
                    federation_id,
                    member_did,
                } => {
                    handle_fed_metadata_remove_member(cli, client, federation_id, member_did)
                        .await?
                }
            },
            FederationCommands::Did { command } => match command {
                FederationDidCommands::Generate {
                    federation_id,
                    output,
                } => handle_fed_did_generate(cli, client, federation_id, output).await?,
                FederationDidCommands::Verify {
                    document_json_or_stdin,
                } => handle_fed_did_verify(cli, client, document_json_or_stdin).await?,
                FederationDidCommands::Publish {
                    federation_id,
                    document_json_or_stdin,
                } => {
                    handle_fed_did_publish(cli, client, federation_id, document_json_or_stdin)
                        .await?
                }
                FederationDidCommands::Resolve { did, federation } => {
                    handle_fed_did_resolve(cli, client, did, federation).await?
                }
            },
        },
        Commands::Aid { command } => match command {
            AidCommands::List => handle_aid_list(cli, client).await?,
            AidCommands::Register {
                resource_json_or_stdin,
            } => handle_aid_register(cli, client, resource_json_or_stdin).await?,
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
            WizardCommands::OnboardFederation { config } => {
                handle_wizard_onboard_federation(config)?
            }
        },
        Commands::Cooperative { command } => match command {
            CooperativeCommands::Register {
                profile_json_or_stdin,
            } => handle_coop_register(cli, client, profile_json_or_stdin).await?,
            CooperativeCommands::Search {
                coop_type,
                capabilities,
                country,
                region,
                locality,
                federation,
                query,
                limit,
            } => {
                handle_coop_search(
                    cli,
                    client,
                    coop_type,
                    capabilities,
                    country,
                    region,
                    locality,
                    federation,
                    query,
                    limit,
                )
                .await?
            }
            CooperativeCommands::Profile { did } => handle_coop_profile(cli, client, did).await?,
            CooperativeCommands::Trust {
                trust_json_or_stdin,
            } => handle_coop_trust(cli, client, trust_json_or_stdin).await?,
            CooperativeCommands::TrustList { did } => {
                handle_coop_trust_list(cli, client, did).await?
            }
            CooperativeCommands::Providers { capability_type } => {
                handle_coop_providers(cli, client, capability_type).await?
            }
            CooperativeCommands::Stats => handle_coop_stats(cli, client).await?,
            CooperativeCommands::Onboard { dry_run } => {
                handle_coop_onboard(cli, client, dry_run).await?
            }
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
                include_cross_federation,
            } => {
                handle_trust_list(
                    cli,
                    client,
                    entity,
                    context,
                    min_level,
                    federation,
                    *include_inherited,
                    *include_cross_federation,
                )
                .await?
            }
            TrustCommands::Paths {
                from,
                to,
                context,
                max_length,
                max_paths,
                min_level,
            } => {
                handle_trust_paths(
                    cli,
                    client,
                    from,
                    to,
                    context,
                    *max_length,
                    *max_paths,
                    min_level,
                )
                .await?
            }
            TrustCommands::Score { entity } => handle_trust_score(cli, client, entity).await?,
            TrustCommands::Scores { entities } => {
                handle_trust_scores(cli, client, entities).await?
            }
            TrustCommands::Update {
                update_json_or_stdin,
            } => handle_trust_update(cli, client, update_json_or_stdin).await?,
            TrustCommands::Remove { from, to, context } => {
                handle_trust_remove(cli, client, from, to, context).await?
            }
            TrustCommands::Stats => handle_trust_stats(cli, client).await?,
            TrustCommands::FederationStats { federation } => {
                handle_trust_federation_stats(cli, client, federation).await?
            }
            TrustCommands::Search {
                context,
                min_level,
                federation,
                limit,
                offset,
            } => {
                handle_trust_search(cli, client, context, min_level, federation, *limit, *offset)
                    .await?
            }
            TrustCommands::Validate {
                actor,
                target,
                context,
                operation,
            } => handle_trust_validate(cli, client, actor, target, context, operation).await?,
            TrustCommands::Neighbors {
                entity,
                max_distance,
                min_level,
            } => handle_trust_neighbors(cli, client, entity, *max_distance, min_level).await?,
            TrustCommands::Recalculate { entities } => {
                handle_trust_recalculate(cli, client, entities).await?
            }
        },
        Commands::Credential { command } => match command {
            credential_lifecycle::CredentialLifecycleCommands::Issue { credential_type } => {
                handle_credential_issue(cli, client, credential_type).await?
            }
            credential_lifecycle::CredentialLifecycleCommands::Present {
                credential_path,
                context,
            } => handle_credential_present(cli, client, credential_path, context).await?,
            credential_lifecycle::CredentialLifecycleCommands::Verify {
                credential_path,
                level,
            } => handle_credential_verify(cli, client, credential_path, level).await?,
            credential_lifecycle::CredentialLifecycleCommands::Anchor {
                disclosure_path,
                metadata,
            } => handle_credential_anchor(cli, client, disclosure_path, metadata).await?,
            credential_lifecycle::CredentialLifecycleCommands::Status { cid } => {
                handle_credential_status(cli, client, cid).await?
            }
            credential_lifecycle::CredentialLifecycleCommands::List {
                holder,
                issuer,
                credential_type,
            } => handle_credential_list(cli, client, holder, issuer, credential_type).await?,
            credential_lifecycle::CredentialLifecycleCommands::Revoke { cid, reason } => {
                handle_credential_revoke(cli, client, cid, reason).await?
            }
            credential_lifecycle::CredentialLifecycleCommands::Example { flow } => {
                handle_credential_example(cli, client, flow).await?
            }
        },
        Commands::Qr { command } => match command {
            QrCommands::ShareIdentity { did, size, output } => {
                handle_qr_share_identity(did, *size, output).await?
            }
            QrCommands::Transfer { token, amount, to, memo, size, output } => {
                handle_qr_transfer(token, *amount, to, memo, *size, output).await?
            }
            QrCommands::Vote { proposal, vote, voter, size, output } => {
                handle_qr_vote(proposal, vote, voter, *size, output).await?
            }
            QrCommands::Join { federation, code, size, output } => {
                handle_qr_join(federation, code, *size, output).await?
            }
            QrCommands::VerifyCredential { credential, challenge, size, output } => {
                handle_qr_verify_credential(credential, challenge, *size, output).await?
            }
            QrCommands::Decode { url } => {
                handle_qr_decode(url).await?
            }
            QrCommands::Encode { url, size, output } => {
                handle_qr_encode(url, *size, output).await?
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
            let req = if let Some(k) = api_key {
                req.header("x-api-key", k)
            } else {
                req
            };
            req.send().await.map_err(|e| anyhow::anyhow!(e))
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
            let req = if let Some(k) = api_key {
                req.header("x-api-key", k)
            } else {
                req
            };
            req.send().await.map_err(|e| anyhow::anyhow!(e))
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
    let response: NodeInfo =
        get_request(&cli.api_url, client, "/info", cli.api_key.as_deref()).await?;
    println!("--- Node Information ---");
    println!("Name:    {}", response.name);
    println!("Version: {}", response.version);
    println!("Status:  {}", response.status_message);
    println!("------------------------");
    Ok(())
}

async fn handle_status(cli: &Cli, client: &Client) -> Result<(), anyhow::Error> {
    let response: NodeStatus =
        get_request(&cli.api_url, client, "/status", cli.api_key.as_deref()).await?;
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
            let req = if let Some(k) = api_key {
                req.header("x-api-key", k)
            } else {
                req
            };
            req.send().await.map_err(|e| anyhow::anyhow!(e))
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
    let response_cid: Cid = post_request(
        &cli.api_url,
        client,
        "/dag/put",
        &block,
        cli.api_key.as_deref(),
    )
    .await?;
    println!(
        "Successfully submitted block. CID: {}",
        serde_json::to_string_pretty(&response_cid)?
    );
    Ok(())
}

async fn handle_dag_get(cli: &Cli, client: &Client, cid_json: &str) -> Result<(), anyhow::Error> {
    let cid: Cid = serde_json::from_str(cid_json)
        .map_err(|e| anyhow::anyhow!("Invalid CID JSON provided: {}. Error: {}", cid_json, e))?;
    let response_block: DagBlock = post_request(
        &cli.api_url,
        client,
        "/dag/get",
        &cid,
        cli.api_key.as_deref(),
    )
    .await?;
    println!("--- Retrieved DAG Block ---");
    println!("{}", serde_json::to_string_pretty(&response_block)?);
    println!("-------------------------");
    Ok(())
}

async fn handle_dag_meta(cli: &Cli, client: &Client, cid_json: &str) -> Result<(), anyhow::Error> {
    let cid: Cid = serde_json::from_str(cid_json)
        .map_err(|e| anyhow::anyhow!("Invalid CID JSON provided: {}. Error: {}", cid_json, e))?;
    let meta: icn_dag::DagBlockMetadata = post_request(
        &cli.api_url,
        client,
        "/dag/meta",
        &cid,
        cli.api_key.as_deref(),
    )
    .await?;
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
    let _: JsonValue = post_request(
        &cli.api_url,
        client,
        "/dag/pin",
        &body,
        cli.api_key.as_deref(),
    )
    .await?;
    println!("Pinned block {cid_json}");
    Ok(())
}

async fn handle_dag_unpin(cli: &Cli, client: &Client, cid_json: &str) -> Result<(), anyhow::Error> {
    let cid: Cid = serde_json::from_str(cid_json)
        .map_err(|e| anyhow::anyhow!("Invalid CID JSON: {cid_json}. Error: {e}"))?;
    let _: JsonValue = post_request(
        &cli.api_url,
        client,
        "/dag/unpin",
        &cid,
        cli.api_key.as_deref(),
    )
    .await?;
    println!("Unpinned block {cid_json}");
    Ok(())
}

async fn handle_dag_prune(cli: &Cli, client: &Client) -> Result<(), anyhow::Error> {
    let _: JsonValue = post_request(
        &cli.api_url,
        client,
        "/dag/prune",
        &(),
        cli.api_key.as_deref(),
    )
    .await?;
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
    let response_proposal_id: ProposalId = post_request(
        &cli.api_url,
        client,
        "/governance/submit",
        &request,
        cli.api_key.as_deref(),
    )
    .await?;
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
    let response: JsonValue = post_request(
        &cli.api_url,
        client,
        "/governance/vote",
        &request,
        cli.api_key.as_deref(),
    )
    .await?;
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
    let result: icn_api::governance_trait::CloseProposalResponse = post_request(
        &cli.api_url,
        client,
        "/governance/close",
        &req,
        cli.api_key.as_deref(),
    )
    .await?;
    println!(
        "Tally result: yes={} no={} abstain={} status={}",
        result.yes, result.no, result.abstain, result.status
    );
    Ok(())
}

async fn handle_gov_list_proposals(cli: &Cli, client: &Client) -> Result<(), anyhow::Error> {
    let proposals: Vec<Proposal> = get_request(
        &cli.api_url,
        client,
        "/governance/proposals",
        cli.api_key.as_deref(),
    )
    .await?;
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
    let proposal: Proposal =
        get_request(&cli.api_url, client, &path, cli.api_key.as_deref()).await?;
    println!("--- Proposal Details (ID: {}) ---", proposal_id);
    println!("{}", serde_json::to_string_pretty(&proposal)?);
    println!("-----------------------------------");
    Ok(())
}

async fn handle_mesh_jobs(cli: &Cli, client: &Client) -> Result<(), anyhow::Error> {
    let response: serde_json::Value =
        get_request(&cli.api_url, client, "/mesh/jobs", cli.api_key.as_deref()).await?;
    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}

async fn handle_mesh_status(cli: &Cli, client: &Client, job_id: &str) -> Result<(), anyhow::Error> {
    let path = format!("/mesh/jobs/{}", job_id);
    let response: serde_json::Value =
        get_request(&cli.api_url, client, &path, cli.api_key.as_deref()).await?;
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
        let cid: Cid = post_request(
            &cli.api_url,
            client,
            "/dag/put",
            &payload,
            cli.api_key.as_deref(),
        )
        .await?;

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

    let response: serde_json::Value = post_request(
        &cli.api_url,
        client,
        "/mesh/submit",
        &request_value,
        cli.api_key.as_deref(),
    )
    .await?;
    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}

/// Get progress information for a mesh job
async fn handle_mesh_progress(
    cli: &Cli,
    client: &Client,
    job_id: &str,
) -> Result<(), anyhow::Error> {
    let path = format!("/mesh/jobs/{}/progress", job_id);
    let response: serde_json::Value =
        get_request(&cli.api_url, client, &path, cli.api_key.as_deref()).await?;

    // Pretty print progress information
    if let Some(progress) = response.get("progress") {
        println!("Job Progress for: {}", job_id);
        println!("================");

        if let Some(stage) = progress.get("current_stage") {
            println!("Current Stage: {}", stage.as_str().unwrap_or("unknown"));
        }

        if let Some(percent) = progress.get("progress_percent") {
            println!("Progress: {:.1}%", percent.as_f64().unwrap_or(0.0));
        }

        if let Some(eta) = progress.get("eta_seconds") {
            if !eta.is_null() {
                let eta_secs = eta.as_u64().unwrap_or(0);
                let minutes = eta_secs / 60;
                let seconds = eta_secs % 60;
                println!("ETA: {}m {}s", minutes, seconds);
            }
        }

        if let Some(message) = progress.get("message") {
            println!("Status: {}", message.as_str().unwrap_or(""));
        }

        if let Some(completed) = progress.get("completed_stages") {
            if let Some(stages) = completed.as_array() {
                if !stages.is_empty() {
                    println!(
                        "Completed Stages: {}",
                        stages
                            .iter()
                            .filter_map(|s| s.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                }
            }
        }

        if let Some(remaining) = progress.get("remaining_stages") {
            if let Some(stages) = remaining.as_array() {
                if !stages.is_empty() {
                    println!(
                        "Remaining Stages: {}",
                        stages
                            .iter()
                            .filter_map(|s| s.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                }
            }
        }
    }

    // Show checkpoints count
    if let Some(checkpoints) = response.get("checkpoints") {
        if let Some(cp_array) = checkpoints.as_array() {
            println!("Checkpoints: {}", cp_array.len());
        }
    }

    // Show partial outputs count
    if let Some(outputs) = response.get("partial_outputs") {
        if let Some(out_array) = outputs.as_array() {
            println!("Partial Outputs: {}", out_array.len());
        }
    }

    Ok(())
}

/// Stream/tail output from a mesh job
async fn handle_mesh_tail(
    cli: &Cli,
    client: &Client,
    job_id: &str,
    follow: bool,
    from: Option<u64>,
) -> Result<(), anyhow::Error> {
    let mut current_sequence = from.unwrap_or(0);

    loop {
        let path = format!("/mesh/jobs/{}/stream?from={}", job_id, current_sequence);
        let response: serde_json::Value =
            get_request(&cli.api_url, client, &path, cli.api_key.as_deref()).await?;

        if let Some(chunks) = response.as_array() {
            for chunk in chunks {
                if let (Some(sequence), Some(stage), Some(data_b64), Some(timestamp)) = (
                    chunk.get("sequence").and_then(|s| s.as_u64()),
                    chunk.get("stage").and_then(|s| s.as_str()),
                    chunk.get("data").and_then(|d| d.as_str()),
                    chunk.get("timestamp").and_then(|t| t.as_u64()),
                ) {
                    // Decode base64 data
                    if let Ok(data_bytes) =
                        base64::engine::general_purpose::STANDARD.decode(data_b64)
                    {
                        if let Ok(data_str) = String::from_utf8(data_bytes) {
                            let dt = chrono::DateTime::from_timestamp(timestamp as i64, 0)
                                .unwrap_or_else(|| chrono::Utc::now());
                            println!(
                                "[{}] [{}] #{}: {}",
                                dt.format("%H:%M:%S"),
                                stage,
                                sequence,
                                data_str
                            );
                        }
                    }

                    current_sequence = sequence + 1;

                    // Check if this is the final chunk
                    if chunk
                        .get("is_final")
                        .and_then(|f| f.as_bool())
                        .unwrap_or(false)
                    {
                        if !follow {
                            return Ok(());
                        }
                    }
                }
            }
        }

        if !follow {
            break;
        }

        // Wait before checking for more output
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }

    Ok(())
}

/// Cancel a running mesh job
async fn handle_mesh_cancel(cli: &Cli, client: &Client, job_id: &str) -> Result<(), anyhow::Error> {
    let path = format!("/mesh/jobs/{}/cancel", job_id);
    let response: serde_json::Value = post_request(
        &cli.api_url,
        client,
        &path,
        &serde_json::json!({}),
        cli.api_key.as_deref(),
    )
    .await?;

    if let Some(success) = response.get("success").and_then(|s| s.as_bool()) {
        if success {
            println!("✅ Job {} cancellation requested", job_id);
        } else {
            println!("❌ Failed to cancel job {}", job_id);
            if let Some(message) = response.get("message").and_then(|m| m.as_str()) {
                println!("Reason: {}", message);
            }
        }
    }

    Ok(())
}

/// Resume a failed mesh job from checkpoint
async fn handle_mesh_resume(cli: &Cli, client: &Client, job_id: &str) -> Result<(), anyhow::Error> {
    let path = format!("/mesh/jobs/{}/resume", job_id);
    let response: serde_json::Value = post_request(
        &cli.api_url,
        client,
        &path,
        &serde_json::json!({}),
        cli.api_key.as_deref(),
    )
    .await?;

    if let Some(success) = response.get("success").and_then(|s| s.as_bool()) {
        if success {
            println!("✅ Job {} resume initiated", job_id);
        } else {
            println!("❌ Failed to resume job {}", job_id);
            if let Some(message) = response.get("message").and_then(|m| m.as_str()) {
                println!("Reason: {}", message);
            }
        }
    }

    Ok(())
}

/// Get mesh execution metrics
async fn handle_mesh_metrics(cli: &Cli, client: &Client) -> Result<(), anyhow::Error> {
    let response: serde_json::Value = get_request(
        &cli.api_url,
        client,
        "/mesh/metrics",
        cli.api_key.as_deref(),
    )
    .await?;

    println!("Mesh Execution Metrics");
    println!("=====================");

    if let Some(total) = response.get("total_jobs").and_then(|t| t.as_u64()) {
        println!("Total Jobs: {}", total);
    }

    if let Some(running) = response.get("running_jobs").and_then(|r| r.as_u64()) {
        println!("Running Jobs: {}", running);
    }

    if let Some(long_running) = response.get("long_running_jobs").and_then(|l| l.as_u64()) {
        println!("Long-Running Jobs: {}", long_running);
    }

    if let Some(completed) = response.get("completed_jobs").and_then(|c| c.as_u64()) {
        println!("Completed Jobs: {}", completed);
    }

    if let Some(failed) = response.get("failed_jobs").and_then(|f| f.as_u64()) {
        println!("Failed Jobs: {}", failed);
    }

    if let Some(avg_time) = response
        .get("avg_execution_time_secs")
        .and_then(|a| a.as_f64())
    {
        println!("Average Execution Time: {:.2}s", avg_time);
    }

    if let Some(custom) = response.get("custom_metrics").and_then(|c| c.as_object()) {
        println!("\nCustom Metrics:");
        for (key, value) in custom {
            if let Some(val) = value.as_f64() {
                println!("  {}: {}", key, val);
            }
        }
    }

    Ok(())
}

async fn handle_network_stats(cli: &Cli, client: &Client) -> Result<(), anyhow::Error> {
    let status: NodeStatus =
        get_request(&cli.api_url, client, "/status", cli.api_key.as_deref()).await?;
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
    let cid: Cid = post_request(
        &cli.api_url,
        client,
        "/dag/put",
        &payload,
        cli.api_key.as_deref(),
    )
    .await?;
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
    let peers: Vec<String> = get_request(
        &cli.api_url,
        client,
        "/federation/peers",
        cli.api_key.as_deref(),
    )
    .await?;
    println!("{}", serde_json::to_string_pretty(&peers)?);
    Ok(())
}

#[derive(Serialize)]
struct PeerReq<'a> {
    peer: &'a str,
}

async fn handle_fed_join(cli: &Cli, client: &Client, peer: &str) -> Result<(), anyhow::Error> {
    let req = PeerReq { peer };
    let resp: serde_json::Value = post_request(
        &cli.api_url,
        client,
        "/federation/join",
        &req,
        cli.api_key.as_deref(),
    )
    .await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

async fn handle_fed_leave(cli: &Cli, client: &Client, peer: &str) -> Result<(), anyhow::Error> {
    let req = PeerReq { peer };
    let resp: serde_json::Value = post_request(
        &cli.api_url,
        client,
        "/federation/leave",
        &req,
        cli.api_key.as_deref(),
    )
    .await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

async fn handle_fed_status(cli: &Cli, client: &Client) -> Result<(), anyhow::Error> {
    let status: serde_json::Value = get_request(
        &cli.api_url,
        client,
        "/federation/status",
        cli.api_key.as_deref(),
    )
    .await?;
    println!("{}", serde_json::to_string_pretty(&status)?);
    Ok(())
}

async fn handle_fed_init(cli: &Cli, client: &Client) -> Result<(), anyhow::Error> {
    let resp: serde_json::Value = post_request(
        &cli.api_url,
        client,
        "/federation/init",
        &(),
        cli.api_key.as_deref(),
    )
    .await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

async fn handle_fed_sync(cli: &Cli, client: &Client) -> Result<(), anyhow::Error> {
    let resp: serde_json::Value = post_request(
        &cli.api_url,
        client,
        "/federation/sync",
        &(),
        cli.api_key.as_deref(),
    )
    .await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

async fn handle_fed_discover(cli: &Cli, client: &Client) -> Result<(), anyhow::Error> {
    let resp: serde_json::Value = get_request(
        &cli.api_url,
        client,
        "/federation/discover",
        cli.api_key.as_deref(),
    )
    .await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

async fn handle_account_balance(
    cli: &Cli,
    client: &Client,
    did: &str,
) -> Result<(), anyhow::Error> {
    let path = format!("/account/{}/mana", did);
    let v: serde_json::Value =
        get_request(&cli.api_url, client, &path, cli.api_key.as_deref()).await?;
    println!("{}", serde_json::to_string_pretty(&v)?);
    Ok(())
}

async fn handle_keys_show(cli: &Cli, client: &Client) -> Result<(), anyhow::Error> {
    let v: serde_json::Value =
        get_request(&cli.api_url, client, "/keys", cli.api_key.as_deref()).await?;
    println!("{}", serde_json::to_string_pretty(&v)?);
    Ok(())
}

async fn handle_reputation_get(cli: &Cli, client: &Client, did: &str) -> Result<(), anyhow::Error> {
    let path = format!("/reputation/{}", did);
    let v: serde_json::Value =
        get_request(&cli.api_url, client, &path, cli.api_key.as_deref()).await?;
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

    let resp: serde_json::Value = post_request(
        &cli.api_url,
        client,
        "/identity/verify",
        &proof,
        cli.api_key.as_deref(),
    )
    .await?;
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

    let resp: serde_json::Value = post_request(
        &cli.api_url,
        client,
        "/identity/verify/revocation",
        &proof,
        cli.api_key.as_deref(),
    )
    .await?;
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
    let resp: BatchVerificationResponse = post_request(
        &cli.api_url,
        client,
        "/identity/verify/batch",
        &req,
        cli.api_key.as_deref(),
    )
    .await?;
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
            println!(
                "⚠️  Job submission failed with status: {}",
                response.status()
            );
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
            println!(
                "⚠️  Proposal submission failed with status: {}",
                response.status()
            );
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
    print!(
        "Node name (default: {}-node): ",
        coop_name.replace(' ', "-")
    );
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
    println!(
        "2. Start your node: cargo run -p icn-node -- --config {}",
        config
    );
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

async fn handle_coop_profile(cli: &Cli, client: &Client, did: &str) -> Result<(), anyhow::Error> {
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
    let global =
        global_input.trim().to_lowercase() == "y" || global_input.trim().to_lowercase() == "yes";

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
    let node_info: JsonValue =
        get_request(&cli.api_url, client, "/keys", cli.api_key.as_deref()).await?;
    let did = node_info["did"]
        .as_str()
        .unwrap_or("did:key:unknown")
        .to_string();

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
    let metrics =
        get_request::<String>(&cli.api_url, client, "/metrics", cli.api_key.as_deref()).await?;
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
    let v: serde_json::Value = get_request(
        &cli.api_url,
        client,
        "/emergency/requests",
        cli.api_key.as_deref(),
    )
    .await?;
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
    let _: serde_json::Value = post_request(
        &cli.api_url,
        client,
        "/emergency/request",
        &body,
        cli.api_key.as_deref(),
    )
    .await?;
    println!("Aid request submitted");
    Ok(())
}

async fn handle_aid_list(cli: &Cli, client: &Client) -> Result<(), anyhow::Error> {
    let v: serde_json::Value = get_request(
        &cli.api_url,
        client,
        "/aid/resources",
        cli.api_key.as_deref(),
    )
    .await?;
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
    let _: serde_json::Value = post_request(
        &cli.api_url,
        client,
        "/aid/resource",
        &body,
        cli.api_key.as_deref(),
    )
    .await?;
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
    let path = format!(
        "/trust/relationship?from={}&to={}&context={}",
        from, to, context
    );
    let resp: serde_json::Value =
        get_request(&cli.api_url, client, &path, cli.api_key.as_deref()).await?;
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
    path.push_str(&format!(
        "&include_inherited={}&include_cross_federation={}",
        include_inherited, include_cross_federation
    ));

    let resp: serde_json::Value =
        get_request(&cli.api_url, client, &path, cli.api_key.as_deref()).await?;
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
    let mut path = format!(
        "/trust/paths?from={}&to={}&context={}&max_length={}&max_paths={}",
        from, to, context, max_length, max_paths
    );
    if let Some(level) = min_level {
        path.push_str(&format!("&min_level={}", level));
    }

    let resp: serde_json::Value =
        get_request(&cli.api_url, client, &path, cli.api_key.as_deref()).await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

async fn handle_trust_score(cli: &Cli, client: &Client, entity: &str) -> Result<(), anyhow::Error> {
    let path = format!("/trust/score?entity={}", entity);
    let resp: serde_json::Value =
        get_request(&cli.api_url, client, &path, cli.api_key.as_deref()).await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

async fn handle_trust_scores(
    cli: &Cli,
    client: &Client,
    entities: &str,
) -> Result<(), anyhow::Error> {
    let path = format!("/trust/scores?entities={}", entities);
    let resp: serde_json::Value =
        get_request(&cli.api_url, client, &path, cli.api_key.as_deref()).await?;
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
    )
    .await?;
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
    )
    .await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

async fn handle_trust_stats(cli: &Cli, client: &Client) -> Result<(), anyhow::Error> {
    let resp: serde_json::Value =
        get_request(&cli.api_url, client, "/trust/stats", cli.api_key.as_deref()).await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

async fn handle_trust_federation_stats(
    cli: &Cli,
    client: &Client,
    federation: &str,
) -> Result<(), anyhow::Error> {
    let path = format!("/trust/federation-stats?federation={}", federation);
    let resp: serde_json::Value =
        get_request(&cli.api_url, client, &path, cli.api_key.as_deref()).await?;
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

    let resp: serde_json::Value =
        get_request(&cli.api_url, client, &path, cli.api_key.as_deref()).await?;
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
    let path = format!(
        "/trust/validate?actor={}&target={}&context={}&operation={}",
        actor, target, context, operation
    );
    let resp: serde_json::Value =
        get_request(&cli.api_url, client, &path, cli.api_key.as_deref()).await?;
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
    let mut path = format!(
        "/trust/neighbors?entity={}&max_distance={}",
        entity, max_distance
    );
    if let Some(level) = min_level {
        path.push_str(&format!("&min_level={}", level));
    }

    let resp: serde_json::Value =
        get_request(&cli.api_url, client, &path, cli.api_key.as_deref()).await?;
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
    )
    .await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

// === Token Command Handlers ===

async fn handle_token_mint(
    cli: &Cli,
    client: &Client,
    class_id: &str,
    to_did: &str,
    amount: u64,
    issuer: &Option<String>,
    scope: &Option<String>,
) -> Result<(), anyhow::Error> {
    let issuer_did = if let Some(issuer) = issuer {
        issuer.clone()
    } else {
        // Get node DID as default issuer
        let node_info: serde_json::Value =
            get_request(&cli.api_url, client, "/keys", cli.api_key.as_deref()).await?;
        node_info["did"]
            .as_str()
            .unwrap_or("did:key:unknown")
            .to_string()
    };

    let request = serde_json::json!({
        "class_id": class_id,
        "to_did": to_did,
        "amount": amount,
        "issuer_did": issuer_did,
        "scope": scope
    });

    let response: serde_json::Value = post_request(
        &cli.api_url,
        client,
        "/tokens/mint",
        &request,
        cli.api_key.as_deref(),
    )
    .await?;

    println!("✅ Tokens minted successfully!");
    println!("Class: {}", class_id);
    println!("Recipient: {}", to_did);
    println!("Amount: {}", amount);
    println!("Issuer: {}", issuer_did);
    if let Some(scope) = scope {
        println!("Scope: {}", scope);
    }
    println!("Response: {}", serde_json::to_string_pretty(&response)?);
    Ok(())
}

async fn handle_token_transfer(
    cli: &Cli,
    client: &Client,
    class_id: &str,
    from_did: &str,
    to_did: &str,
    amount: u64,
    issuer: &Option<String>,
    scope: &Option<String>,
) -> Result<(), anyhow::Error> {
    let issuer_did = if let Some(issuer) = issuer {
        issuer.clone()
    } else {
        // Get node DID as default issuer
        let node_info: serde_json::Value =
            get_request(&cli.api_url, client, "/keys", cli.api_key.as_deref()).await?;
        node_info["did"]
            .as_str()
            .unwrap_or("did:key:unknown")
            .to_string()
    };

    let request = serde_json::json!({
        "class_id": class_id,
        "from_did": from_did,
        "to_did": to_did,
        "amount": amount,
        "issuer_did": issuer_did,
        "scope": scope
    });

    let response: serde_json::Value = post_request(
        &cli.api_url,
        client,
        "/tokens/transfer",
        &request,
        cli.api_key.as_deref(),
    )
    .await?;

    println!("✅ Tokens transferred successfully!");
    println!("Class: {}", class_id);
    println!("From: {}", from_did);
    println!("To: {}", to_did);
    println!("Amount: {}", amount);
    println!("Issuer: {}", issuer_did);
    if let Some(scope) = scope {
        println!("Scope: {}", scope);
    }
    println!("Response: {}", serde_json::to_string_pretty(&response)?);
    Ok(())
}

async fn handle_token_balance(
    cli: &Cli,
    client: &Client,
    class_id: &str,
    did: &str,
) -> Result<(), anyhow::Error> {
    let path = format!("/tokens/balance?class_id={}&did={}", class_id, did);
    let response: serde_json::Value =
        get_request(&cli.api_url, client, &path, cli.api_key.as_deref()).await?;

    println!("💰 Token Balance");
    println!("Class: {}", class_id);
    println!("Account: {}", did);
    if let Some(balance) = response.get("balance") {
        println!("Balance: {}", balance);
    } else {
        println!("Balance: 0");
    }
    println!("Details: {}", serde_json::to_string_pretty(&response)?);
    Ok(())
}

async fn handle_token_burn(
    cli: &Cli,
    client: &Client,
    class_id: &str,
    from_did: &str,
    amount: u64,
    issuer: &Option<String>,
    scope: &Option<String>,
) -> Result<(), anyhow::Error> {
    let issuer_did = if let Some(issuer) = issuer {
        issuer.clone()
    } else {
        // Get node DID as default issuer
        let node_info: serde_json::Value =
            get_request(&cli.api_url, client, "/keys", cli.api_key.as_deref()).await?;
        node_info["did"]
            .as_str()
            .unwrap_or("did:key:unknown")
            .to_string()
    };

    let request = serde_json::json!({
        "class_id": class_id,
        "from_did": from_did,
        "amount": amount,
        "issuer_did": issuer_did,
        "scope": scope
    });

    let response: serde_json::Value = post_request(
        &cli.api_url,
        client,
        "/tokens/burn",
        &request,
        cli.api_key.as_deref(),
    )
    .await?;

    println!("🔥 Tokens burned successfully!");
    println!("Class: {}", class_id);
    println!("From: {}", from_did);
    println!("Amount: {}", amount);
    println!("Issuer: {}", issuer_did);
    if let Some(scope) = scope {
        println!("Scope: {}", scope);
    }
    println!("Response: {}", serde_json::to_string_pretty(&response)?);
    Ok(())
}

// Missing federation trust functions
async fn handle_fed_trust_configure(
    _cli: &Cli,
    _client: &Client,
    _federation_id: &str,
    _policy_json_or_stdin: &str,
) -> Result<(), anyhow::Error> {
    println!("🚧 Federation trust configuration not yet implemented");
    Ok(())
}

async fn handle_fed_trust_add(
    _cli: &Cli,
    _client: &Client,
    _trust_json_or_stdin: &str,
) -> Result<(), anyhow::Error> {
    println!("🚧 Federation trust addition not yet implemented");
    Ok(())
}

async fn handle_fed_trust_remove(
    _cli: &Cli,
    _client: &Client,
    _from: &str,
    _to: &str,
    _context: &str,
    _federation: &str,
) -> Result<(), anyhow::Error> {
    println!("🚧 Federation trust removal not yet implemented");
    Ok(())
}

async fn handle_fed_trust_list(
    _cli: &Cli,
    _client: &Client,
    _federation_id: &str,
    _context: &Option<String>,
    _min_level: &Option<String>,
) -> Result<(), anyhow::Error> {
    println!("🚧 Federation trust listing not yet implemented");
    Ok(())
}

async fn handle_fed_trust_validate(
    _cli: &Cli,
    _client: &Client,
    _actor: &str,
    _target: &str,
    _context: &str,
    _operation: &str,
    _federation: &str,
) -> Result<(), anyhow::Error> {
    println!("🚧 Federation trust validation not yet implemented");
    Ok(())
}

async fn handle_fed_trust_bridge(
    _cli: &Cli,
    _client: &Client,
    _bridge_json_or_stdin: &str,
) -> Result<(), anyhow::Error> {
    println!("🚧 Federation trust bridging not yet implemented");
    Ok(())
}

async fn handle_fed_trust_bootstrap(
    _cli: &Cli,
    _client: &Client,
    _peer: &str,
    _contexts: &String,
    _trust_level: &str,
) -> Result<(), anyhow::Error> {
    println!("🚧 Federation trust bootstrap not yet implemented");
    Ok(())
}

// Missing federation metadata functions
async fn handle_fed_metadata_get(
    _cli: &Cli,
    _client: &Client,
    _federation_id: &str,
) -> Result<(), anyhow::Error> {
    println!("🚧 Federation metadata get not yet implemented");
    Ok(())
}

async fn handle_fed_metadata_set(
    _cli: &Cli,
    _client: &Client,
    _federation_id: &str,
    _metadata_json_or_stdin: &str,
) -> Result<(), anyhow::Error> {
    println!("🚧 Federation metadata set not yet implemented");
    Ok(())
}

async fn handle_fed_metadata_scope(
    _cli: &Cli,
    _client: &Client,
    _federation_id: &str,
    _scope_json_or_stdin: &str,
) -> Result<(), anyhow::Error> {
    println!("🚧 Federation metadata scope not yet implemented");
    Ok(())
}

async fn handle_fed_metadata_quorum(
    _cli: &Cli,
    _client: &Client,
    _federation_id: &str,
    _policy_json_or_stdin: &str,
) -> Result<(), anyhow::Error> {
    println!("🚧 Federation metadata quorum not yet implemented");
    Ok(())
}

async fn handle_fed_metadata_members(
    _cli: &Cli,
    _client: &Client,
    _federation_id: &str,
) -> Result<(), anyhow::Error> {
    println!("🚧 Federation metadata members not yet implemented");
    Ok(())
}

async fn handle_fed_metadata_add_member(
    _cli: &Cli,
    _client: &Client,
    _federation_id: &str,
    _member_did: &str,
    _profile_json_or_stdin: &str,
) -> Result<(), anyhow::Error> {
    println!("🚧 Federation metadata add member not yet implemented");
    Ok(())
}

async fn handle_fed_metadata_remove_member(
    _cli: &Cli,
    _client: &Client,
    _federation_id: &str,
    _member_did: &str,
) -> Result<(), anyhow::Error> {
    println!("🚧 Federation metadata remove member not yet implemented");
    Ok(())
}

// Missing federation DID functions
async fn handle_fed_did_generate(
    _cli: &Cli,
    _client: &Client,
    _federation_id: &str,
    _output: &Option<String>,
) -> Result<(), anyhow::Error> {
    println!("🚧 Federation DID generation not yet implemented");
    Ok(())
}

async fn handle_fed_did_verify(
    _cli: &Cli,
    _client: &Client,
    _document_json_or_stdin: &str,
) -> Result<(), anyhow::Error> {
    println!("🚧 Federation DID verification not yet implemented");
    Ok(())
}

async fn handle_fed_did_publish(
    _cli: &Cli,
    _client: &Client,
    _federation_id: &str,
    _document_json_or_stdin: &str,
) -> Result<(), anyhow::Error> {
    println!("🚧 Federation DID publishing not yet implemented");
    Ok(())
}

async fn handle_fed_did_resolve(
    _cli: &Cli,
    _client: &Client,
    _did: &str,
    _federation: &Option<String>,
) -> Result<(), anyhow::Error> {
    println!("🚧 Federation DID resolution not yet implemented");
    Ok(())
}

// Missing credential functions
async fn handle_credential_issue(
    _cli: &Cli,
    _client: &Client,
    _credential_type: &crate::credential_lifecycle::IssueCommands,
) -> Result<(), anyhow::Error> {
    println!("🚧 Credential issuance not yet implemented");
    Ok(())
}

async fn handle_credential_present(
    _cli: &Cli,
    _client: &Client,
    _credential_path: &str,
    _context: &str,
) -> Result<(), anyhow::Error> {
    println!("🚧 Credential presentation not yet implemented");
    Ok(())
}

async fn handle_credential_verify(
    _cli: &Cli,
    _client: &Client,
    _credential_path: &str,
    _level: &Option<String>,
) -> Result<(), anyhow::Error> {
    println!("🚧 Credential verification not yet implemented");
    Ok(())
}

async fn handle_credential_anchor(
    _cli: &Cli,
    _client: &Client,
    _disclosure_path: &str,
    _metadata: &Option<String>,
) -> Result<(), anyhow::Error> {
    println!("🚧 Credential anchoring not yet implemented");
    Ok(())
}

async fn handle_credential_status(
    _cli: &Cli,
    _client: &Client,
    _cid: &str,
) -> Result<(), anyhow::Error> {
    println!("🚧 Credential status check not yet implemented");
    Ok(())
}

async fn handle_credential_list(
    _cli: &Cli,
    _client: &Client,
    _holder: &Option<String>,
    _issuer: &Option<String>,
    _credential_type: &Option<String>,
) -> Result<(), anyhow::Error> {
    println!("🚧 Credential listing not yet implemented");
    Ok(())
}

async fn handle_credential_revoke(
    cli: &Cli,
    client: &Client,
    cid: &str,
    _reason: &str, // Note: reason parameter not used by current API endpoint
) -> Result<(), anyhow::Error> {
    let cid = icn_common::parse_cid_from_string(cid)?;

    let request = icn_api::identity_trait::RevokeCredentialRequest { cid };

    let resp: serde_json::Value = post_request(
        &cli.api_url,
        client,
        "/identity/credentials/revoke",
        &request,
        cli.api_key.as_deref(),
    )
    .await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

async fn handle_credential_example(
    _cli: &Cli,
    _client: &Client,
    _flow: &crate::credential_lifecycle::ExampleFlows,
) -> Result<(), anyhow::Error> {
    println!("🚧 Credential examples not yet implemented");
    Ok(())
}

// --- QR Code Command Handlers ---

async fn handle_qr_share_identity(
    did: &str,
    size: u32,
    output: &Option<String>,
) -> Result<(), anyhow::Error> {
    let did = Did::from_str(did).map_err(|_| anyhow::anyhow!("Invalid DID format"))?;
    let action = Action::ShareIdentity { did };
    generate_and_display_qr(&action, size, output).await
}

async fn handle_qr_transfer(
    token: &str,
    amount: u64,
    to: &str,
    memo: &Option<String>,
    size: u32,
    output: &Option<String>,
) -> Result<(), anyhow::Error> {
    let to = Did::from_str(to).map_err(|_| anyhow::anyhow!("Invalid recipient DID format"))?;
    let action = Action::TransferToken {
        token: token.to_string(),
        amount,
        to,
        memo: memo.clone(),
    };
    generate_and_display_qr(&action, size, output).await
}

async fn handle_qr_vote(
    proposal: &str,
    vote: &str,
    voter: &Option<String>,
    size: u32,
    output: &Option<String>,
) -> Result<(), anyhow::Error> {
    let proposal = Cid::from_str(proposal).map_err(|_| anyhow::anyhow!("Invalid proposal CID format"))?;
    let vote = VoteChoice::from_str(vote)?;
    let voter = if let Some(voter_str) = voter {
        Some(Did::from_str(voter_str).map_err(|_| anyhow::anyhow!("Invalid voter DID format"))?)
    } else {
        None
    };
    
    let action = Action::Vote {
        proposal,
        vote,
        voter,
    };
    generate_and_display_qr(&action, size, output).await
}

async fn handle_qr_join(
    federation: &str,
    code: &Option<String>,
    size: u32,
    output: &Option<String>,
) -> Result<(), anyhow::Error> {
    let action = Action::JoinFederation {
        federation_id: federation.to_string(),
        invitation_code: code.clone(),
    };
    generate_and_display_qr(&action, size, output).await
}

async fn handle_qr_verify_credential(
    credential: &str,
    challenge: &Option<String>,
    size: u32,
    output: &Option<String>,
) -> Result<(), anyhow::Error> {
    let credential = Cid::from_str(credential).map_err(|_| anyhow::anyhow!("Invalid credential CID format"))?;
    let action = Action::VerifyCredential {
        credential,
        challenge: challenge.clone(),
    };
    generate_and_display_qr(&action, size, output).await
}

async fn handle_qr_decode(url: &str) -> Result<(), anyhow::Error> {
    let action = ActionEncoder::decode(url)?;
    println!("Decoded ICN Action:");
    println!("{}", serde_json::to_string_pretty(&action)?);
    
    match &action {
        Action::ShareIdentity { did } => {
            println!("\n📱 Action: Share Identity");
            println!("   DID: {}", did);
        }
        Action::TransferToken { token, amount, to, memo } => {
            println!("\n💰 Action: Transfer Token");
            println!("   Token: {}", token);
            println!("   Amount: {}", amount);
            println!("   To: {}", to);
            if let Some(memo) = memo {
                println!("   Memo: {}", memo);
            }
        }
        Action::Vote { proposal, vote, voter } => {
            println!("\n🗳️ Action: Vote on Proposal");
            println!("   Proposal: {}", proposal);
            println!("   Vote: {}", vote);
            if let Some(voter) = voter {
                println!("   Voter: {}", voter);
            }
        }
        Action::JoinFederation { federation_id, invitation_code } => {
            println!("\n🤝 Action: Join Federation");
            println!("   Federation: {}", federation_id);
            if let Some(code) = invitation_code {
                println!("   Invitation Code: {}", code);
            }
        }
        Action::VerifyCredential { credential, challenge } => {
            println!("\n🔐 Action: Verify Credential");
            println!("   Credential: {}", credential);
            if let Some(challenge) = challenge {
                println!("   Challenge: {}", challenge);
            }
        }
        _ => {
            println!("\n🔧 Action: Custom or Advanced");
        }
    }
    
    Ok(())
}

async fn handle_qr_encode(
    url: &str,
    size: u32,
    output: &Option<String>,
) -> Result<(), anyhow::Error> {
    // Validate that it's a proper ICN URL
    let _action = ActionEncoder::decode(url)?;
    
    if let Some(output_path) = output {
        let qr_data = QrGenerator::generate_png(url, size)?;
        std::fs::write(output_path, qr_data)?;
        println!("✅ QR code saved to: {}", output_path);
    } else {
        let terminal_qr = QrGenerator::display_terminal(url)?;
        println!("🔗 ICN Action URL: {}", url);
        println!("\n📱 QR Code:");
        println!("{}", terminal_qr);
    }
    
    Ok(())
}

async fn generate_and_display_qr(
    action: &Action,
    size: u32,
    output: &Option<String>,
) -> Result<(), anyhow::Error> {
    let url = ActionEncoder::encode(action)?;
    
    if let Some(output_path) = output {
        let qr_data = QrGenerator::generate_for_action(action, &icn_action::QrMetadata::default())?;
        std::fs::write(output_path, qr_data)?;
        println!("✅ QR code saved to: {}", output_path);
        println!("🔗 ICN Action URL: {}", url);
    } else {
        let terminal_qr = QrGenerator::display_terminal(&url)?;
        println!("🔗 ICN Action URL: {}", url);
        println!("\n📱 QR Code:");
        println!("{}", terminal_qr);
        println!("\n💡 Tip: Use --output <file.png> to save as image");
    }
    
    Ok(())
}
