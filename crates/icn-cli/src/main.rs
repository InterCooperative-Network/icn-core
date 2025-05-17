#![doc = include_str!("../README.md")]

//! # ICN CLI Crate
//! This crate provides a command-line interface (CLI) for interacting with the InterCooperative Network (ICN).
//! It allows users and administrators to manage nodes, interact with the network, and perform administrative tasks.
//! The CLI aims for usability, discoverability, and scriptability.

// Use the icn_api crate
use icn_api::{get_node_info, get_node_status, submit_dag_block, retrieve_dag_block, discover_peers_api, send_network_message_api};
// Use icn_common for types if needed, though get_node_info wraps them.
// use icn_common::NodeInfo;
use icn_common::{CommonError, DagBlock, Cid};
use std::process::exit;

fn handle_info_command() {
    println!("Requesting node info via icn_api...");
    match get_node_info() {
        Ok(info) => {
            println!("--- Node Information ---");
            // Adjusting to actual fields in NodeInfo: name, version, status_message
            println!("Name:    {}", info.name);
            println!("Version: {}", info.version);
            println!("Status:  {}", info.status_message);
            println!("------------------------");
        }
        Err(e) => {
            eprintln!("Error: Failed to retrieve node info: {:?}", e);
            exit(1);
        }
    }
}

fn handle_status_command(args: &[String]) {
    let simulate_online = if args.len() > 2 && args[2] == "offline" {
        println!("Requesting node status (simulating offline)...");
        false
    } else {
        println!("Requesting node status (simulating online)...");
        true
    };

    match get_node_status(simulate_online) {
        Ok(status) => {
            println!("--- Node Status ---");
            // Adjusting to actual fields in NodeStatus: is_online, peer_count, current_block_height, version
            println!("Online:         {}", status.is_online);
            println!("Peer Count:     {}", status.peer_count);
            println!("Block Height:   {}", status.current_block_height);
            println!("Version:        {}", status.version);
            println!("-------------------");
        }
        Err(CommonError::NodeOffline(msg)) => {
            eprintln!("Error: Node is offline - {}", msg);
            exit(1);
        }
        Err(e) => {
            eprintln!("Error: Failed to retrieve node status: {:?}", e);
            exit(1);
        }
    }
}

fn handle_dag_put_command(args: &[String]) {
    if args.len() > 3 {
        let block_data_json = &args[3];
        println!("Attempting to submit DAG block via API...");
        match submit_dag_block(block_data_json.to_string()) {
            Ok(cid) => {
                println!("Successfully submitted block. CID: {}", cid.to_string_approx());
            }
            Err(CommonError::DeserializationError(msg)) => {
                 eprintln!("Error: Invalid DagBlock JSON provided: {}. Please provide a valid JSON string for the DagBlock.", msg);
                 exit(1);
            }
            Err(e) => {
                eprintln!("Error: Failed to submit DAG block: {:?}", e);
                exit(1);
            }
        }
    } else {
        eprintln!("Usage: icn-cli dag put <DAG_BLOCK_JSON_STRING>");
        eprintln!(r#"Example: icn-cli dag put '{{"cid":{{"version":1,"codec":113,"hash_alg":18,"hash_bytes":[104,101,108,108,111]}},"data":[104,101,108,108,111],"links":[]}}'"#);
        exit(1);
    }
}

fn handle_dag_get_command(args: &[String]) {
    if args.len() > 3 {
        let cid_json = &args[3];
        println!("Attempting to retrieve DAG block via API...");
        match retrieve_dag_block(cid_json.to_string()) {
            Ok(Some(block)) => {
                println!("--- Retrieved DAG Block ---");
                println!("{:#?}", block); // Pretty print
                println!("-------------------------");
            }
            Ok(None) => {
                println!("Block not found for the given CID: {}", cid_json);
            }
            Err(CommonError::DeserializationError(msg)) => {
                 eprintln!("Error: Invalid CID JSON provided: {}. Please provide a valid JSON string for the CID.", msg);
                 exit(1);
            }
            Err(e) => {
                eprintln!("Error: Failed to retrieve DAG block: {:?}", e);
                exit(1);
            }
        }
    } else {
        eprintln!("Usage: icn-cli dag get <CID_JSON_STRING>");
        eprintln!(r#"Example: icn-cli dag get '{{"version":1,"codec":113,"hash_alg":18,"hash_bytes":[104,101,108,108,111]}}'"#);
        exit(1);
    }
}

fn handle_network_discover_peers_command(_args: &[String]) {
    println!("Requesting network peer discovery via icn_api...");
    // Currently, discover_peers_api in stub ignores bootstrap nodes, so passing an empty vec.
    // A real CLI might take bootstrap nodes as arguments.
    match discover_peers_api(Vec::new()) {
        Ok(peers) => {
            if peers.is_empty() {
                println!("No peers discovered (stubbed network service might return a fixed list or be configured).");
            } else {
                println!("--- Discovered Peers ---");
                for peer in peers {
                    println!("  Peer ID: {}", peer.0); // Assuming PeerId is a tuple struct PeerId(String)
                }
                println!("----------------------");
            }
        }
        Err(e) => {
            eprintln!("Error: Failed to discover network peers: {:?}", e);
            exit(1);
        }
    }
}

fn handle_network_send_message_command(args: &[String]) {
    if args.len() > 4 { // icn-cli network send-message <PEER_ID> <MESSAGE_JSON>
        let peer_id_str = args[3].clone();
        let message_json = args[4].clone();
        println!("Attempting to send network message to peer {} via icn_api...", peer_id_str);
        
        match send_network_message_api(peer_id_str, message_json) {
            Ok(_) => {
                println!("Successfully sent message.");
            }
            Err(CommonError::DeserializationError(msg)) => {
                eprintln!("Error: Invalid NetworkMessage JSON provided: {}.", msg);
                eprintln!("Please ensure the message is a valid JSON representation of a NetworkMessage variant (e.g., RequestBlock, AnnounceBlock).");
                exit(1);
            }
            Err(CommonError::PeerNotFound(msg)) => {
                eprintln!("Error: Peer not found: {}.", msg);
                exit(1);
            }
             Err(CommonError::ApiError(msg)) if msg.to_lowercase().contains("peer not found") => { // More robust check for wrapped PeerNotFound
                eprintln!("Error: Peer not found (reported by API): {}.", msg);
                exit(1);
            }
            Err(e) => {
                eprintln!("Error: Failed to send network message: {:?}", e);
                exit(1);
            }
        }
    } else {
        eprintln!("Usage: icn-cli network send-message <PEER_ID> <MESSAGE_JSON>");
        eprintln!("  <PEER_ID>: String identifier of the target peer.");
        eprintln!("  <MESSAGE_JSON>: JSON string of the NetworkMessage.");
        eprintln!(r#"Example: icn-cli network send-message mock_peer_1 '{{"RequestBlock":{{"version":1,"codec":112,"hash_alg":18,"hash_bytes":[100,97,116,97]}}}}'"#);
        exit(1);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() <= 1 {
        print_usage();
        exit(0);
    }

    match args[1].as_str() {
        "info" => handle_info_command(),
        "status" => handle_status_command(&args),
        "hello" => println!("Hello from ICN CLI!"),
        "dag" => {
            if args.len() > 2 {
                match args[2].as_str() {
                    "put" => handle_dag_put_command(&args),
                    "get" => handle_dag_get_command(&args),
                    _ => {
                        eprintln!("Error: Unknown DAG command: {}. Try 'put' or 'get'.", args[2]);
                        print_dag_usage();
                        exit(1);
                    }
                }
            } else {
                print_dag_usage();
                exit(1);
            }
        }
        "network" => {
            if args.len() > 2 {
                match args[2].as_str() {
                    "discover-peers" => handle_network_discover_peers_command(&args),
                    "send-message" => handle_network_send_message_command(&args),
                    _ => {
                        eprintln!("Error: Unknown network command: {}. Try 'discover-peers' or 'send-message'.", args[2]);
                        print_network_usage();
                        exit(1);
                    }
                }
            } else {
                print_network_usage();
                exit(1);
            }
        }
        "--help" | "-h" | "help" => {
            print_usage();
            exit(0);
        }
        _ => {
            eprintln!("Error: Unknown command: {}", args[1]);
            print_usage();
            exit(1);
        }
    }
}

fn print_usage() {
    println!("ICN CLI - InterCooperative Network Command Line Interface");
    println!("Usage: icn-cli <command> [options]");
    println!("");
    println!("Available commands:");
    println!("  info                     Display node information.");
    println!("  status [offline]         Display node status. Optionally simulate offline.");
    println!("  dag put <JSON_BLOCK>     Submit a DAG block (as a JSON string).");
    println!("  dag get <JSON_CID>       Retrieve a DAG block by its CID (as a JSON string).");
    println!("  network discover-peers   Discover network peers (stubbed).");
    println!("  network send-message <PEER_ID> <JSON_MESSAGE>  Send a message to a peer (stubbed).");
    println!("  hello                    A friendly greeting.");
    println!("  help, -h, --help         Show this help message.");
    println!("");
}

fn print_dag_usage() {
    println!("ICN CLI - DAG Subcommands");
    println!("Usage: icn-cli dag <subcommand> [arguments]");
    println!("");
    println!("Available DAG subcommands:");
    println!("  put <DAG_BLOCK_JSON_STRING>    Submit a DAG block. The block must be a valid JSON string.");
    println!(r#"                                     Example: icn-cli dag put '{{"...block_data..."}}'"#);
    println!("  get <CID_JSON_STRING>            Retrieve a DAG block by its CID. The CID must be a valid JSON string.");
    println!(r#"                                     Example: icn-cli dag get '{{"...cid_data..."}}'"#);
    println!("");
}

fn print_network_usage() {
    println!("ICN CLI - Network Subcommands");
    println!("Usage: icn-cli network <subcommand> [arguments]");
    println!("");
    println!("Available network subcommands:");
    println!("  discover-peers                       Discover network peers (uses stubbed service).");
    println!("  send-message <PEER_ID> <MESSAGE_JSON>  Send a message to a specific peer (uses stubbed service).");
    println!("    <PEER_ID>:      The string identifier of the target peer (e.g., \"mock_peer_1\").");
    println!("    <MESSAGE_JSON>: The message to send, as a JSON string. Examples:");
    println!(r#"      '{{"RequestBlock":{{"version":1,"codec":112,"hash_alg":18,"hash_bytes":[100,97,116,97]}}}}' (for RequestBlock)"#);
    println!(r#"      '{{"AnnounceBlock":{{"cid":{{"version":1,...}},"data":[...],"links":[]}}}}' (for AnnounceBlock)"#);
    println!("");
}

#[cfg(test)]
mod tests {
    // For CLI, tests often involve running the binary with arguments and checking output,
    // or testing argument parsing logic directly.
    // The `it_compiles` test is a basic check.
    #[test]
    fn it_compiles() {
        assert!(true);
    }

    // Example of how you might test a function if logic was extracted:
    // fn process_info_command() -> Result<String, String> {
    //     // ... logic of info command ...
    //     Ok("Processed".to_string())
    // }
    // #[test]
    // fn test_info_logic() {
    //     assert!(process_info_command().is_ok());
    // }
}
