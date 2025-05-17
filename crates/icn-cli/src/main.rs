#![doc = include_str!("../README.md")]

//! # ICN CLI Crate
//! This crate provides a command-line interface (CLI) for interacting with the InterCooperative Network (ICN).
//! It allows users and administrators to manage nodes, interact with the network, and perform administrative tasks.
//! The CLI aims for usability, discoverability, and scriptability.

// Use the icn_api crate
use icn_api::{get_node_info, get_node_status};
// Use icn_common for types if needed, though get_node_info wraps them.
// use icn_common::NodeInfo;
use icn_common::CommonError;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "info" => { // Basic info command
                println!("Requesting node info via icn_api...");
                match get_node_info() {
                    Ok(info) => {
                        println!("--- Node Information ---");
                        println!("Name:    {}", info.name);
                        println!("Version: {}", info.version);
                        println!("Status:  {}", info.status_message);
                        println!("------------------------");
                    }
                    Err(e) => {
                        eprintln!("Error retrieving node info: {:?}", e);
                    }
                }
            }
            "status" => { // Node status command
                // Allow simulating offline for CLI testing: `icn-cli status offline`
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
                        println!("Online:         {}", status.is_online);
                        println!("Peer Count:     {}", status.peer_count);
                        println!("Block Height:   {}", status.current_block_height);
                        println!("Version:        {}", status.version);
                        println!("-------------------");
                    }
                    Err(CommonError::NodeOffline(msg)) => {
                        eprintln!("Error: Node is offline - {}", msg);
                    }
                    Err(e) => {
                        eprintln!("Error retrieving node status: {:?}", e);
                    }
                }
            }
            "hello" => { // Simple hello command
                println!("Hello from ICN CLI!");
            }
            _ => {
                println!("Unknown command: {}. Try 'info', 'status', or 'hello'.", args[1]);
            }
        }
    } else {
        println!("ICN CLI. Available commands: info, status, hello");
    }
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
