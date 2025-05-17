use icn_api::{get_node_info, get_node_status, retrieve_dag_block, submit_dag_block};
use icn_common::block::{Cid, DagBlock, DagLink};
use icn_common::node::{NodeInfo, NodeStatus};
use icn_common::utils::ICN_CORE_VERSION;
use icn_network::service::{NetworkMessage, NetworkService, StubNetworkService};
use serde_json;

fn main() {
    println!("--- ICN Node ({}) ---", ICN_CORE_VERSION);

    // Get and print node info
    println!("\n--- ICN Node: Info ---");
    match get_node_info() {
        Ok(node_info) => {
            println!(
                "Node Info: ID={}, Version={}, Uptime={}, Status={:?}",
                node_info.id, node_info.version, node_info.uptime_seconds, node_info.status
            );
        }
        Err(e) => {
            println!("Error getting node info: {:?}", e);
        }
    }

    // Get and print node status (simulating online)
    println!("\n--- ICN Node: Status (Online) ---");
    match get_node_status(true) {
        Ok(status) => {
            println!(
                "Node Status: Peers={}, Mempool Size={}, Chain Height={}, Is Syncing={}",
                status.connected_peers,
                status.mempool_size,
                status.current_chain_height,
                status.is_syncing
            );
        }
        Err(e) => {
            println!("Error getting node status: {:?}", e);
        }
    }

    // Get and print node status (simulating offline)
    println!("\n--- ICN Node: Status (Offline) ---");
    match get_node_status(false) {
        // Simulating node being offline
        Ok(status) => {
            // This case should ideally not be reached if is_simulated_online is false and get_node_status handles it
            println!(
                "Node Status (unexpected): Peers={}, Mempool Size={}, Chain Height={}, Is Syncing={}",
                status.connected_peers,
                status.mempool_size,
                status.current_chain_height,
                status.is_syncing
            );
        }
        Err(e) => {
            println!("Correctly handled simulated offline status: {:?}", e);
        }
    }

    // Demonstrate DagBlock submission and retrieval
    println!("\n--- ICN Node: DAG Operations ---");
    let link_demo = DagLink::new(Cid::new_v1_dummy("demo_link_cid"), "demo_link", 50);
    let sample_block_demo = DagBlock::new(
        Cid::new_v1_dummy("demo_block_cid"),
        "some_node_payload_data".as_bytes().to_vec(),
        vec![link_demo],
    );
    let block_data_json_demo =
        serde_json::to_string(&sample_block_demo).expect("Failed to serialize demo block");

    println!(
        "Submitting DagBlock (Demo) to local API: {}",
        block_data_json_demo
    );

    match submit_dag_block(block_data_json_demo.clone()) {
        Ok(cid_json_result_demo) => {
            println!(
                "Successfully submitted DagBlock (Demo). CID: {}",
                cid_json_result_demo
            );
            println!(
                "Retrieving DagBlock (Demo) with CID: {}",
                cid_json_result_demo
            );
            match retrieve_dag_block(cid_json_result_demo) {
                Ok(retrieved_block_json_result_demo) => {
                    println!(
                        "Retrieved DagBlock (Demo): {}",
                        retrieved_block_json_result_demo
                    );
                }
                Err(e) => {
                    println!("Error retrieving DagBlock (Demo): {:?}", e);
                }
            }
        }
        Err(e) => {
            println!("Error submitting DagBlock (Demo): {:?}", e);
        }
    }

    // Instantiate and use the network service
    let mut network_service = StubNetworkService::new();
    println!("\n--- ICN Node: Network Operations ---");
    match network_service.discover_peers() {
        Ok(peers) => println!("Discovered peers: {:?}", peers),
        Err(e) => println!("Failed to discover peers: {:?}", e),
    }

    // Simulate submitting another DagBlock and then broadcasting its announcement
    let link_net = DagLink::new(Cid::new_v1_dummy("net_link_cid_1"), "net_link1", 10);
    let sample_block_net = DagBlock::new(
        Cid::new_v1_dummy("net_block_cid_for_node"),
        "network_payload_data".as_bytes().to_vec(),
        vec![link_net],
    );

    let block_data_json_net =
        serde_json::to_string(&sample_block_net).expect("Failed to serialize net block");
    println!(
        "Submitting DagBlock (Net) to local API: {}",
        block_data_json_net
    );

    match submit_dag_block(block_data_json_net.clone()) {
        Ok(cid_json_result_net) => {
            println!(
                "Successfully submitted DagBlock (Net). CID: {}",
                cid_json_result_net
            );
            // Broadcast the block announcement
            let announcement_message = NetworkMessage::AnnounceBlock {
                cid: sample_block_net.cid().clone(),
            };
            match network_service.broadcast_message(announcement_message) {
                Ok(_) => println!("Successfully broadcasted block announcement (Net)."),
                Err(e) => println!("Failed to broadcast block announcement (Net): {:?}", e),
            }

            // Retrieve the block to verify
            println!(
                "Retrieving DagBlock (Net) with CID: {}",
                cid_json_result_net
            );
            match retrieve_dag_block(cid_json_result_net) {
                Ok(retrieved_block_json_result_net) => {
                    println!(
                        "Retrieved DagBlock (Net): {}",
                        retrieved_block_json_result_net
                    );
                }
                Err(e) => {
                    println!("Error retrieving DagBlock (Net): {:?}", e);
                }
            }
        }
        Err(e) => {
            println!("Error submitting DagBlock (Net): {:?}", e);
        }
    }

    // Demonstrate sending a specific message to a peer (if any discovered)
    if let Ok(peers) = network_service.discover_peers() {
        if let Some(first_peer) = peers.first() {
            let request_message = NetworkMessage::RequestBlock {
                cid: Cid::new_v1_dummy("some_other_block_cid_for_request"),
            };
            match network_service.send_message(first_peer.clone(), request_message) {
                Ok(_) => println!(
                    "Successfully sent RequestBlock message to peer: {}",
                    first_peer.0
                ),
                Err(e) => println!(
                    "Failed to send RequestBlock message to peer {}: {:?}",
                    first_peer.0, e
                ),
            }
        } else {
            println!("No peers discovered to send a direct message to.");
        }
    } else {
        println!("Could not discover peers to test sending a direct message.");
    }
} 