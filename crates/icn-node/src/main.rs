#![doc = include_str!("../README.md")]

//! # ICN Node Crate
//! This crate provides the main binary for running a long-lived InterCooperative Network (ICN) daemon.
//! It integrates various core components to operate a functional ICN node, handling initialization,
//! lifecycle, configuration, service hosting, and persistence.

// Use the icn_api crate
use icn_api::{get_node_info, get_node_status};
use icn_common::{NodeInfo, NodeStatus, CommonError};

fn main() {
    println!("ICN Node starting...");

    match get_node_info() {
        Ok(info) => {
            println!("Successfully retrieved node info using icn_api:");
            println!("  Name: {}", info.name);
            println!("  Version: {}", info.version);
            println!("  Status: {}", info.status_message);
        }
        Err(e) => {
            eprintln!("Error retrieving node info: {:?}", e);
        }
    }

    // Simulate node status check (can be online or offline)
    let simulate_online_status = true; // Change to false to test error handling
    println!("\n--- Attempting to get Node Status (API call, simulated online: {}) ---", simulate_online_status);
    match get_node_status(simulate_online_status) {
        Ok(status) => {
            println!("Successfully retrieved node status using icn_api:");
            println!("  Online: {}", status.is_online);
            println!("  Peer Count: {}", status.peer_count);
            println!("  Block Height: {}", status.current_block_height);
            println!("  Version: {}", status.version);
        }
        Err(CommonError::NodeOffline(msg)) => {
            eprintln!("Node is offline: {}", msg);
        }
        Err(e) => {
            eprintln!("Error retrieving node status: {:?}", e);
        }
    }

    println!("\nICN Node initialized. (Placeholder - press Ctrl+C to exit)");
    // Block the main thread or enter a main event loop
    // In a real application, this would be an async runtime with tasks.
    loop {
        std::thread::sleep(std::time::Duration::from_secs(60));
    }
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
