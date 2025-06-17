#![doc = include_str!("../README.md")]
#![allow(clippy::uninlined_format_args)]

//! # ICN CLI Crate
//! This crate provides a command-line interface (CLI) for interacting with an ICN HTTP node.

use clap::{Parser, Subcommand};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue; // For generic JSON data if needed
use std::io::{self, Read};
use std::process::exit; // Added for reading from stdin

// Types from our ICN crates that CLI will interact with (serialize/deserialize)
// These types are expected to be sent to/received from the icn-node HTTP API.
use icn_common::{Cid, DagBlock, NodeInfo, NodeStatus};
// Using aliased request structs from icn-api for clarity, these are what the node expects
use icn_api::governance_trait::{
    CastVoteRequest as ApiCastVoteRequest, SubmitProposalRequest as ApiSubmitProposalRequest,
};
use icn_governance::{Proposal, ProposalId};

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
    /// Submit a new mesh job (JSON string or '-' for stdin)
    Submit {
        #[clap(help = "Mesh job request JSON, or '-' to read from stdin")]
        job_json_or_stdin: String,
    },
}

// --- Main CLI Logic ---

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let client = Client::new();

    if let Err(e) = run_command(&cli, &client).await {
        eprintln!("Error: {}", e);
        exit(1);
    }
}

async fn run_command(cli: &Cli, client: &Client) -> Result<(), anyhow::Error> {
    match &cli.command {
        Commands::Info => handle_info(cli, client).await?,
        Commands::Status => handle_status(cli, client).await?,
        Commands::Dag { command } => match command {
            DagCommands::Put {
                block_json_or_stdin,
            } => handle_dag_put(cli, client, block_json_or_stdin).await?,
            DagCommands::Get { cid_json } => handle_dag_get(cli, client, cid_json).await?,
        },
        Commands::Governance { command } => match command {
            GovernanceCommands::Submit {
                proposal_request_json_or_stdin,
            } => handle_gov_submit(cli, client, proposal_request_json_or_stdin).await?,
            GovernanceCommands::Vote { vote_request_json } => {
                handle_gov_vote(cli, client, vote_request_json).await?
            }
            GovernanceCommands::Proposals => handle_gov_list_proposals(cli, client).await?,
            GovernanceCommands::Proposal { id } => handle_gov_get_proposal(cli, client, id).await?,
        },
        Commands::Mesh { command } => match command {
            MeshCommands::Submit { job_json_or_stdin } => {
                handle_mesh_submit(cli, client, job_json_or_stdin).await?
            }
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
    let res = client.get(&url).send().await?;

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
    let res = client.post(&url).json(body).send().await?;

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

async fn handle_mesh_submit(
    cli: &Cli,
    client: &Client,
    job_json_or_stdin: &str,
) -> Result<(), anyhow::Error> {
    let job_content = if job_json_or_stdin == "-" {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else {
        job_json_or_stdin.to_string()
    };

    let job_value: serde_json::Value = serde_json::from_str(&job_content)
        .map_err(|e| anyhow::anyhow!("Invalid job JSON provided. Error: {}", e))?;

    let response: serde_json::Value =
        post_request(&cli.api_url, client, "/mesh/submit", &job_value).await?;
    println!(
        "Successfully submitted job. Response: {}",
        serde_json::to_string_pretty(&response)?
    );
    Ok(())
}

// CLI command behavior is covered by tests in `crates/icn-cli/tests`.
