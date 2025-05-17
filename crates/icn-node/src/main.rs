#![doc = include_str!("../README.md")]

//! # ICN Node Crate
//! This crate provides the main binary for running a long-lived InterCooperative Network (ICN) daemon.
//! It integrates various core components to operate a functional ICN node, handling initialization,
//! lifecycle, configuration, service hosting, and persistence.

// Use the icn_api crate
use icn_api::{get_node_info, get_node_status, retrieve_dag_block, submit_dag_block};
// Corrected imports for icn_common types
use icn_common::{Cid, DagBlock, DagLink, NodeInfo, NodeStatus, ICN_CORE_VERSION, CommonError};
// Corrected import for icn_network types
use icn_network::{NetworkMessage, NetworkService, StubNetworkService, PeerId};
use icn_dag::{StorageService, InMemoryDagStore, FileDagStore};
use serde_json;
use clap::Parser;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

// Conditionally import Libp2pNetworkService
#[cfg(feature = "with-libp2p")]
use icn_network::libp2p_service::Libp2pNetworkService;

// Import governance types
use icn_governance::{GovernanceModule, ProposalId, ProposalType, VoteOption};
use icn_common::Did;
use icn_api::{submit_proposal_api, cast_vote_api, get_proposal_api, list_proposals_api, SubmitProposalRequest, CastVoteRequest}; // Import API functions and request structs

// --- CLI Arguments --- 

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Option<Commands>,

    #[clap(long, value_enum, default_value = "memory")]
    storage_backend: StorageBackendType,

    #[clap(long, default_value = "./icn_data/file_store")]
    storage_path: PathBuf,

    #[clap(long, value_enum, default_value = "stub")]
    network_backend: NetworkBackendType,
    // TODO: Add args for listen address, bootstrap peers for libp2p
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum StorageBackendType {
    Memory,
    File,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum NetworkBackendType {
    Stub,
    #[cfg(feature = "with-libp2p")]
    Libp2p,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// Run full node demonstrations
    Demo,
    /// Manage governance proposals
    #[clap(subcommand)]
    Proposal(ProposalCommands),
}

#[derive(clap::Subcommand, Debug)]
enum ProposalCommands {
    /// Submit a new proposal
    Submit {
        #[clap(long, help = "Proposer DID (e.g., did:example:123)")]
        proposer_did: String,
        #[clap(long, help = "Type of proposal: e.g., '{"SystemParameterChange": ["param_name", "new_value"]}', '{"NewMemberInvitation": "did:example:new_member"}', '{"SoftwareUpgrade": "v2.0.0"}', or '{"GenericText": "My proposal text"}'")]
        proposal_type_json: String, // JSON string for ProposalType
        #[clap(long)]
        description: String,
        #[clap(long, default_value = "604800", help = "Voting duration in seconds (default: 7 days)")]
        duration_secs: u64,
    },
    /// Cast a vote on a proposal
    Vote {
        #[clap(long, help = "Voter DID (e.g., did:example:456)")]
        voter_did: String,
        #[clap(long, help = "ID of the proposal to vote on")]
        proposal_id: String,
        #[clap(long, value_parser=["yes", "no", "abstain"], help = "Vote option: yes, no, or abstain")]
        option: String,
    },
    /// Get details of a specific proposal
    Get { 
        #[clap(long, help = "ID of the proposal to retrieve")]
        proposal_id: String 
    },
    /// List all proposals
    List,
}


// --- Global Application State (Conceptual) ---
// This is where we'd initialize and hold shared services like the storage backend.
// For simplicity in this refactoring step, we'll create it in main and pass it down.
// A more robust application might use a struct to hold app state or a DI framework.

// Placeholder for a shared storage service. 
// This will be properly initialized in main based on CLI args.
// This static approach for the API to access storage is problematic and needs a deeper refactor.
// For now, the API will continue to use its internal/default storage for `submit_dag_block` and `retrieve_dag_block`
// unless we pass the storage service explicitly through the API layer.
// TODO: Refactor icn_api to accept a StorageService instance.


fn show_node_info() {
    println!("\n--- ICN Node: Info ---");
    match get_node_info() {
        Ok(node_info) => {
            println!(
                "Node Info: Name='{}', Version='{}', Message='{}'",
                node_info.name, node_info.version, node_info.status_message
            );
        }
        Err(e) => {
            eprintln!("Error getting node info: {:?}", e);
        }
    }
}

fn show_node_status() {
    println!("\n--- ICN Node: Status (Simulating Online) ---");
    match get_node_status(true) {
        Ok(status) => {
            println!(
                "Node Status: Online={}, Peers={}, Block Height={}, Version='{}'",
                status.is_online, status.peer_count, status.current_block_height, status.version
            );
        }
        Err(e) => {
            eprintln!("Error getting node status (online): {:?}", e);
        }
    }

    println!("\n--- ICN Node: Status (Simulating Offline) ---");
    match get_node_status(false) {
        Ok(status) => {
            eprintln!(
                "Error: Node status (offline) test unexpectedly returned Ok: Online={}, Peers={}, Block Height={}, Version='{}'",
                status.is_online, status.peer_count, status.current_block_height, status.version
            );
        }
        Err(CommonError::NodeOffline(msg)) => {
            println!("Correctly handled simulated offline status: NodeOffline - {}", msg);
        }
        Err(e) => {
            eprintln!("Error getting node status (offline) - unexpected error: {:?}", e);
        }
    }
}

fn demonstrate_dag_operations(storage: Arc<Mutex<dyn StorageService<DagBlock> + Send + Sync>>) {
    println!("\n--- ICN Node: Local DAG Operations ---");

    // Create a block
    let link_demo = DagLink {
        cid: Cid::new_v1_dummy(0x71,0x12,b"demo_link_cid_data_cli"), 
        name: "demo_link_cli".to_string(), 
        size: 50
    };
    let sample_block_demo = DagBlock {
        cid: Cid::new_v1_dummy(0x71,0x12,b"demo_block_cid_data_cli"),
        data: "some_node_payload_data_cli".as_bytes().to_vec(),
        links: vec![link_demo],
    };

    println!(
        "Attempting to put DagBlock (Demo) with CID: {} into selected store",
        sample_block_demo.cid.to_string_approx()
    );

    // Use the passed-in storage service
    match storage.lock().unwrap().put(&sample_block_demo) {
        Ok(_) => {
            println!(
                "Successfully put DagBlock (Demo) with CID: {}",
                sample_block_demo.cid.to_string_approx()
            );

            println!(
                "Attempting to retrieve DagBlock (Demo) with CID: {} from selected store",
                sample_block_demo.cid.to_string_approx()
            );
            match storage.lock().unwrap().get(&sample_block_demo.cid) {
                Ok(Some(retrieved_block)) => {
                    println!(
                        "Retrieved DagBlock (Demo) from selected store: CID={}, Data='{}'",
                        retrieved_block.cid.to_string_approx(),
                        String::from_utf8_lossy(&retrieved_block.data)
                    );
                    // Further demonstration: delete and check contains
                    assert!(storage.lock().unwrap().delete(&sample_block_demo.cid).is_ok());
                    assert_eq!(storage.lock().unwrap().contains(&sample_block_demo.cid).unwrap(), false);
                    println!("Block {} successfully deleted and verified not present.", sample_block_demo.cid.to_string_approx());
                }
                Ok(None) => {
                     eprintln!(
                        "Error retrieving DagBlock (Demo) from selected store: Block not found for CID {}.", 
                        sample_block_demo.cid.to_string_approx()
                    );
                }
                Err(e) => {
                    eprintln!("Error retrieving DagBlock (Demo) from selected store: {:?}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Error putting DagBlock (Demo) into selected store: {:?}", e);
        }
    }

    // The existing API calls submit_dag_block and retrieve_dag_block will continue to use
    // the global default InMemoryDagStore within icn-dag/icn-api for now.
    // A deeper refactor is needed to plumb the selected storage through the API layer.
    println!("\n--- (Note: The following API calls still use the default global in-memory store) ---");
    let block_data_json_demo =
        serde_json::to_string(&sample_block_demo).expect("Failed to serialize demo block for API");
    match submit_dag_block(Arc::clone(&storage), block_data_json_demo.clone()) {
        Ok(submitted_cid) => {
            println!(
                "Successfully submitted DagBlock (Demo via API). CID: {}",
                submitted_cid.to_string_approx()
            );
            let cid_json_to_retrieve = serde_json::to_string(&submitted_cid)
                .expect("Failed to serialize CID for API retrieval");
            match retrieve_dag_block(Arc::clone(&storage), cid_json_to_retrieve) {
                Ok(Some(retrieved_block_json)) => {
                    println!(
                        "Retrieved DagBlock (Demo via API): {:#?}",
                        retrieved_block_json 
                    );
                }
                Ok(None) => {
                     eprintln!("Error retrieving DagBlock (Demo via API): Block not found for CID {}.", submitted_cid.to_string_approx());
                }
                Err(e) => {
                    eprintln!("Error retrieving DagBlock (Demo via API): {:?}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Error submitting DagBlock (Demo via API): {:?}", e);
        }
    }
}

fn demonstrate_network_operations(
    storage: Arc<Mutex<dyn StorageService<DagBlock> + Send + Sync>>,
    network_service: Arc<dyn NetworkService + Send + Sync>,
) {
    println!("\n--- ICN Node: Network Operations ---");

    match network_service.discover_peers(vec!["/ip4/127.0.0.1/tcp/4001/p2p/QmSomePeer".to_string()]) {
        Ok(peers) => println!("Discovered peers: {:?}", peers),
        Err(e) => eprintln!("Failed to discover peers: {:?}", e),
    }

    let link_net = DagLink {
        cid: Cid::new_v1_dummy(0x71,0x12, b"net_link_cid_1_data"), 
        name: "net_link1".to_string(), 
        size: 10
    };
    let sample_block_net = DagBlock {
        cid: Cid::new_v1_dummy(0x71,0x12, b"net_block_cid_for_node_data_cli"),
        data: "network_payload_data_cli".as_bytes().to_vec(),
        links: vec![link_net],
    };
    
    // Store this block in the selected storage service first
    match storage.lock().unwrap().put(&sample_block_net) {
        Ok(_) => println!("Block {} for network demo stored in selected storage.", sample_block_net.cid.to_string_approx()),
        Err(e) => eprintln!("Failed to store block for network demo: {:?}", e),
    }

    // The API call submit_dag_block for network demo will use the default global store
    println!("\n--- (Note: The following API calls related to network demo still use the default global in-memory store) ---");
    let block_data_json_net =
        serde_json::to_string(&sample_block_net).expect("Failed to serialize net block");
    
    match submit_dag_block(Arc::clone(&storage), block_data_json_net.clone()) {
        Ok(cid_net) => {
            println!(
                "Successfully submitted DagBlock (Net). CID: {}",
                cid_net.to_string_approx()
            );
            
            let block_for_announce = DagBlock {
                cid: cid_net.clone(),
                data: sample_block_net.data.clone(),
                links: sample_block_net.links.clone(),
            };
            let announcement_message = NetworkMessage::AnnounceBlock(block_for_announce);

            match network_service.broadcast_message(announcement_message) {
                Ok(_) => println!("Successfully broadcasted block announcement (Net)."),
                Err(e) => eprintln!("Failed to broadcast block announcement (Net): {:?}", e),
            }

            let cid_to_retrieve_json = serde_json::to_string(&cid_net)
                .expect("Failed to serialize net CID for retrieval");
            println!(
                "Retrieving DagBlock (Net) with CID: {} via API",
                cid_net.to_string_approx()
            );
            match retrieve_dag_block(Arc::clone(&storage), cid_to_retrieve_json) {
                Ok(Some(retrieved_block_net)) => {
                    println!(
                        "Retrieved DagBlock (Net): {:#?}",
                        retrieved_block_net
                    );
                }
                Ok(None) => {
                     eprintln!("Error retrieving DagBlock (Net): Block not found for CID {}.", cid_net.to_string_approx());
                }
                Err(e) => {
                    eprintln!("Error retrieving DagBlock (Net): {:?}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Error submitting DagBlock (Net): {:?}", e);
        }
    }

    match network_service.discover_peers(vec![]) {
        Ok(peers) => {
            if let Some(first_peer) = peers.first() {
                let request_message = NetworkMessage::RequestBlock(
                    Cid::new_v1_dummy(0x71,0x12,b"some_other_block_cid_for_request_data"),
                );
                println!("Attempting to send RequestBlock to peer: {:?}", first_peer);
                match network_service.send_message(first_peer, request_message) {
                    Ok(_) => println!(
                        "Successfully sent RequestBlock message to peer: {}",
                        first_peer.0
                    ),
                    Err(e) => eprintln!(
                        "Failed to send RequestBlock message to peer {}: {:?}",
                        first_peer.0, e
                    ),
                }
            } else {
                println!("No peers discovered to send a direct message to.");
            }
        }
        Err(e) => {
             eprintln!("Could not discover peers to test sending a direct message: {:?}",e);
        }
    }
}

#[cfg(not(feature = "with-libp2p"))]
fn main() {
    let cli = Cli::parse();
    if matches!(cli.network_backend, NetworkBackendType::Libp2p) {
        eprintln!("Error: Libp2p network backend selected, but node was not compiled with 'with-libp2p' feature.");
        std::process::exit(1);
    }
    // Fallback to a simpler main or panic if libp2p is selected without the feature.
    // For now, just run the synchronous part.
    run_node(cli);
}

#[cfg(feature = "with-libp2p")]
#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    run_node_async(cli).await;
}

// Synchronous runner for when libp2p is not enabled or not selected.
fn run_node(cli: Cli) {
    println!("--- ICN Node ({}) Starting (Sync Mode) ---", ICN_CORE_VERSION);
    setup_and_run_demonstrations(cli, None );
    println!("(Node would typically enter an event loop here or await termination)");
}


#[cfg(feature = "with-libp2p")]
async fn run_node_async(cli: Cli) {
    println!("--- ICN Node ({}) Starting (Async Mode) ---", ICN_CORE_VERSION);
    
    let network_service_instance: Option<Arc<dyn NetworkService + Send + Sync>> = if matches!(cli.network_backend, NetworkBackendType::Libp2p) {
        println!("Initializing Libp2pNetworkService...");
        match Libp2pNetworkService::new().await {
            Ok(service) => Some(Arc::new(service)),
            Err(e) => {
                eprintln!("Failed to initialize Libp2pNetworkService: {:?}. Falling back to StubNetworkService.", e);
                Some(Arc::new(StubNetworkService::default()))
            }
        }
    } else {
        None // Will default to stub in setup_and_run_demonstrations
    };

    setup_and_run_demonstrations(cli, network_service_instance);
    println!("(Node would typically enter an event loop here or await termination - async)");
    // If libp2p is running, we might need to keep main alive or await the swarm task.
    // For now, the swarm task is spawned and runs in the background.
    // A proper shutdown mechanism would be needed for a real node.
    if matches!(cli.network_backend, NetworkBackendType::Libp2p) {
        println!("Libp2p service initialized. Swarm is running in a background task.");
        println!("Node will exit once demonstrations are complete. For a persistent node, an event loop or park() is needed here.");
        // tokio::signal::ctrl_c().await.expect("failed to listen for ctrl-c");
        // println!("Ctrl-C received, shutting down.");
    }
}

// Shared logic for setting up storage and running demos
fn setup_and_run_demonstrations(cli: Cli, specific_network_service: Option<Arc<dyn NetworkService + Send + Sync>>) {
    println!("Selected storage backend: {:?}", cli.storage_backend);
    if matches!(cli.storage_backend, StorageBackendType::File) {
        println!("Storage path: {:?}", cli.storage_path);
    }
     println!("Selected network backend: {:?}", cli.network_backend);

    let storage_service: Arc<Mutex<dyn StorageService<DagBlock> + Send + Sync>> = match cli.storage_backend {
        StorageBackendType::Memory => {
            println!("Using InMemoryDagStore.");
            Arc::new(Mutex::new(InMemoryDagStore::new()))
        }
        StorageBackendType::File => {
            println!("Using FileDagStore at path: {:?}", cli.storage_path);
            match FileDagStore::new(cli.storage_path.clone()) {
                Ok(store) => Arc::new(Mutex::new(store)),
                Err(e) => {
                    eprintln!("Failed to initialize FileDagStore at {:?}: {:?}. Falling back to InMemoryDagStore.", cli.storage_path, e);
                    Arc::new(Mutex::new(InMemoryDagStore::new()))
                }
            }
        }
    };

    let network_service: Arc<dyn NetworkService + Send + Sync> = specific_network_service.unwrap_or_else(|| {
        println!("Using StubNetworkService.");
        Arc::new(StubNetworkService::default())
    });

    // Initialize Governance Module
    let governance_module = Arc::new(Mutex::new(GovernanceModule::new()));

    match cli.command {
        Some(Commands::Demo) | None => {
            println!("Running full demonstrations...");
            show_node_info();
            show_node_status();
            demonstrate_dag_operations(Arc::clone(&storage_service));
            demonstrate_network_operations(Arc::clone(&storage_service), Arc::clone(&network_service));
            demonstrate_governance_operations(Arc::clone(&governance_module)); // Add governance demo
            println!("\n--- ICN Node Demonstrations Complete ---");
        }
        Some(Commands::Proposal(proposal_cmd)) => {
            handle_proposal_command(proposal_cmd, Arc::clone(&governance_module));
        }
    }
}

fn handle_proposal_command(command: ProposalCommands, gov_module: Arc<Mutex<GovernanceModule>>) {
    match command {
        ProposalCommands::Submit { proposer_did, proposal_type_json, description, duration_secs } => {
            let parsed_proposal_type: serde_json::Value = serde_json::from_str(&proposal_type_json).unwrap_or_else(|e| {
                eprintln!("Error parsing proposal_type_json: {}. Please provide valid JSON.", e);
                // Provide a default or exit; here, using GenericText as a fallback for demonstration
                serde_json::json!({ "GenericText": "Error: Invalid proposal type JSON provided" })
            });

            let request = SubmitProposalRequest {
                proposer_did,
                proposal_type_json: parsed_proposal_type,
                description,
                duration_secs,
            };
            let request_json = serde_json::to_string(&request).expect("Failed to serialize SubmitProposalRequest");
            match submit_proposal_api(gov_module, request_json) {
                Ok(proposal_id) => println!("Successfully submitted proposal. ID: {}", proposal_id),
                Err(e) => eprintln!("Error submitting proposal: {:?}", e),
            }
        }
        ProposalCommands::Vote { voter_did, proposal_id, option } => {
            let request = CastVoteRequest {
                voter_did,
                proposal_id,
                vote_option: option,
            };
            let request_json = serde_json::to_string(&request).expect("Failed to serialize CastVoteRequest");
            match cast_vote_api(gov_module, request_json) {
                Ok(_) => println!("Successfully cast vote."),
                Err(e) => eprintln!("Error casting vote: {:?}", e),
            }
        }
        ProposalCommands::Get { proposal_id } => {
            let proposal_id_json = serde_json::to_string(&proposal_id).expect("Failed to serialize proposal_id for get");
            match get_proposal_api(gov_module, proposal_id_json) {
                Ok(Some(proposal)) => println!("Proposal details: {:#?}", proposal),
                Ok(None) => println!("Proposal with ID '{}' not found.", proposal_id),
                Err(e) => eprintln!("Error getting proposal: {:?}", e),
            }
        }
        ProposalCommands::List => {
            match list_proposals_api(gov_module) {
                Ok(proposals) => {
                    if proposals.is_empty() {
                        println!("No proposals found.");
                    } else {
                        println!("Current proposals:");
                        for proposal in proposals {
                            println!("- ID: {}, Description: \"{}\", Status: {:?}, Proposer: {}", 
                                     proposal.id.0, proposal.description, proposal.status, proposal.proposer.0);
                        }
                    }
                }
                Err(e) => eprintln!("Error listing proposals: {:?}", e),
            }
        }
    }
}

fn demonstrate_governance_operations(gov_module: Arc<Mutex<GovernanceModule>>) {
    println!("\n--- ICN Node: Governance Operations (Demonstration) ---");

    let proposer_did_str = "did:example:demoproposer001".to_string();
    let voter1_did_str = "did:example:demovoter001".to_string();
    let voter2_did_str = "did:example:demovoter002".to_string();

    // 1. Submit a new proposal via API
    println!("\n1. Submitting a new proposal (GenericText):");
    let proposal_type_generic_json = serde_json::json!({
        "GenericText": "Should we adopt standard X for data exchange?"
    });
    let submit_req1 = SubmitProposalRequest {
        proposer_did: proposer_did_str.clone(),
        proposal_type_json: proposal_type_generic_json,
        description: "Discussion on adopting standard X".to_string(),
        duration_secs: 300, // 5 minutes for demo
    };
    let submit_req1_json = serde_json::to_string(&submit_req1).unwrap();
    let proposal1_id = match submit_proposal_api(Arc::clone(&gov_module), submit_req1_json) {
        Ok(id) => {
            println!("Proposal submitted successfully. ID: {}", id);
            id
        }
        Err(e) => {
            eprintln!("Failed to submit proposal: {:?}", e);
            return;
        }
    };

    // 2. List proposals
    println!("\n2. Listing all proposals:");
    if let Ok(proposals) = list_proposals_api(Arc::clone(&gov_module)) {
        for proposal in proposals {
             println!("- ID: {}, Desc: \"{}\", Status: {:?}, Proposer: {}, Deadline: {}", 
                 proposal.id.0, proposal.description, proposal.status, proposal.proposer.0, proposal.voting_deadline);
        }
    } else {
        eprintln!("Failed to list proposals.");
    }

    // 3. Cast votes
    println!("\n3. Casting votes for proposal ID: {}", proposal1_id);
    let vote_yes_req = CastVoteRequest {
        voter_did: voter1_did_str.clone(),
        proposal_id: proposal1_id.clone(),
        vote_option: "yes".to_string(),
    };
    let vote_yes_req_json = serde_json::to_string(&vote_yes_req).unwrap();
    if let Err(e) = cast_vote_api(Arc::clone(&gov_module), vote_yes_req_json) {
        eprintln!("Voter 1 failed to vote: {:?}", e);
    } else {
        println!("Voter {} voted successfully.", voter1_did_str);
    }

    let vote_no_req = CastVoteRequest {
        voter_did: voter2_did_str.clone(),
        proposal_id: proposal1_id.clone(),
        vote_option: "no".to_string(),
    };
    let vote_no_req_json = serde_json::to_string(&vote_no_req).unwrap();
    if let Err(e) = cast_vote_api(Arc::clone(&gov_module), vote_no_req_json) {
        eprintln!("Voter 2 failed to vote: {:?}", e);
    } else {
        println!("Voter {} voted successfully.", voter2_did_str);
    }
    
    // 4. Get proposal details to see votes
    println!("\n4. Retrieving proposal {} details after voting:", proposal1_id);
    let proposal1_id_json = serde_json::to_string(&proposal1_id).unwrap();
    if let Ok(Some(proposal)) = get_proposal_api(Arc::clone(&gov_module), proposal1_id_json) {
        println!("Proposal Details: {:#?}", proposal);
        println!("Votes cast: {}", proposal.votes.len());
        for (voter, vote_details) in proposal.votes {
            println!("  - Voter: {}, Vote: {:?}", voter.0, vote_details.option);
        }
    } else {
        eprintln!("Failed to retrieve proposal details or proposal not found.");
    }
    
    // Simulate propagation (Conceptual)
    // In a real system, new proposals and votes would be gossiped to other nodes.
    // Nodes would update their local GovernanceModule state upon receiving these messages.
    // For now, we just call the stubbed federation sync API if available.
    println!("\n5. Simulating federation sync request (conceptual):");
    if cfg!(feature = "with-libp2p") { // Only if network features might imply actual sync later
        match icn_governance::request_federation_sync(&icn_common::PeerId("some_federation_peer_id".to_string()), None) {
            Ok(msg) => println!("Federation sync simulation: {}", msg),
            Err(e) => eprintln!("Federation sync simulation error: {:?}", e),
        }
    } else {
        println!("(Skipping federation sync simulation as network features for it are not active)");
    }

    println!("\n--- Governance Demonstration Complete ---");
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_compiles() {
        // This test primarily ensures the binary crate compiles successfully.
        // For a real node, you might add basic startup/shutdown tests here, or integration tests
        // in a separate tests/ directory.
        assert!(true);
    }
}
