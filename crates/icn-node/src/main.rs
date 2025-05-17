#![doc = include_str!("../README.md")]

//! # ICN Node Crate
//! This crate provides the main binary for running a long-lived InterCooperative Network (ICN) daemon.
//! It integrates various core components to operate a functional ICN node, handling initialization,
//! lifecycle, configuration, service hosting, and persistence.

// Use the icn_api crate
use icn_api::get_node_info;
use icn_common::NodeInfo; // For type reference if needed, though get_node_info returns it.

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

    println!("ICN Node initialized. (Placeholder - press Ctrl+C to exit)");
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
