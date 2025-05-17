#![doc = include_str!("../README.md")]

//! # ICN Node Crate
//! This crate provides the main binary for running a long-lived InterCooperative Network (ICN) daemon.
//! It integrates various core components to operate a functional ICN node, handling initialization,
//! lifecycle, configuration, service hosting, and persistence.

// Use the icn_api crate
use icn_api::{get_node_info, get_node_status, submit_dag_block, retrieve_dag_block};
use icn_common::{NodeInfo, NodeStatus, CommonError, DagBlock, Cid, DagLink, ICN_CORE_VERSION};

fn main() {
    println!("ICN Node starting (version: {})...", ICN_CORE_VERSION);

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

    println!("\n--- Attempting to submit and retrieve a DAG block (via API) ---");
    let test_data = b"Hello from icn-node DAG test!".to_vec();
    let test_cid = Cid::new_v1_dummy(0x71, 0x12, &test_data); // dag-cbor, sha2-256
    let test_link_cid = Cid::new_v1_dummy(0x71, 0x12, b"node link");
    let test_link = DagLink { cid: test_link_cid, name: "nodelink".to_string(), size: 9 };

    let block_to_submit = DagBlock {
        cid: test_cid.clone(),
        data: test_data.clone(),
        links: vec![test_link],
    };

    let block_json = serde_json::to_string(&block_to_submit).expect("Failed to serialize block for node test");

    match submit_dag_block(block_json) {
        Ok(submitted_cid) => {
            println!("Successfully submitted DAG block. Submitted CID: {}", submitted_cid.to_string_approx());
            println!("Attempting to retrieve block with CID: {}", test_cid.to_string_approx());
            
            let cid_to_retrieve_json = serde_json::to_string(&test_cid).expect("Failed to serialize CID for node retrieval test");
            match retrieve_dag_block(cid_to_retrieve_json) {
                Ok(Some(retrieved_block)) => {
                    println!("Successfully retrieved block!");
                    assert_eq!(retrieved_block.cid, test_cid, "Retrieved block CID mismatch");
                    assert_eq!(retrieved_block.data, test_data, "Retrieved block data mismatch");
                    println!("Retrieved block data (first 10 bytes): {:?}", retrieved_block.data.iter().take(10).collect::<Vec<_>>());
                }
                Ok(None) => {
                    eprintln!("Error: Submitted block not found during retrieval.");
                }
                Err(e) => {
                    eprintln!("Error retrieving block: {:?}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Error submitting DAG block: {:?}", e);
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
