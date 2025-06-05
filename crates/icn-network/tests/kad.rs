#[cfg(feature = "experimental-libp2p")]
mod kademlia_peer_discovery_tests {
    use icn_network::libp2p_service::Libp2pNetworkService;
    use icn_network::PeerId as IcnPeerId; // Renamed to avoid confusion
    use icn_network::NetworkService; // Import the trait
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_kademlia_two_node_peer_discovery() {
        println!("Starting Kademlia two_node_peer_discovery test...");

        // Node 1 Setup
        let node1_service = Libp2pNetworkService::new(None)
            .await
            .expect("Node 1 failed to start");
        let node1_libp2p_peer_id = node1_service.local_peer_id().clone();
        println!("Node 1 Libp2p Peer ID: {}", node1_libp2p_peer_id);

        // Allow Node 1 to establish listeners
        sleep(Duration::from_secs(2)).await; // Increased slightly for stability
        let node1_addrs = node1_service.listening_addresses();
        assert!(!node1_addrs.is_empty(), "Node 1 has no listening addresses");
        
        let node1_listen_addr_for_kad = node1_addrs
            .iter()
            .find(|addr| addr.to_string().contains("127.0.0.1") || addr.to_string().contains("/::1/")) // Prefer loopback
            .unwrap_or_else(|| node1_addrs.first().expect("Node 1 has no listen addresses at all"))
            .clone();
        println!("Node 1 chosen listen address for Kademlia bootstrap: {}", node1_listen_addr_for_kad);

        // Node 2 Setup
        let bootstrap_info_for_node2 = Some(vec![(node1_libp2p_peer_id.clone(), node1_listen_addr_for_kad)]);
        let node2_service = Libp2pNetworkService::new(bootstrap_info_for_node2)
            .await
            .expect("Node 2 failed to start");
        let node2_libp2p_peer_id = node2_service.local_peer_id().clone();
        println!("Node 2 Libp2p Peer ID: {}", node2_libp2p_peer_id);
        
        // Allow time for Node 2 to establish listeners and connect to Node 1
        println!("Allowing Node 2 to initialize and connect (5 seconds)...");
        sleep(Duration::from_secs(5)).await; 

        // Explicitly add Node 2 to Node 1's Kademlia and trigger bootstrap on Node 1
        let node2_addrs = node2_service.listening_addresses();
        assert!(!node2_addrs.is_empty(), "Node 2 has no listening addresses");
        let node2_listen_addr_for_kad = node2_addrs
            .iter()
            .find(|addr| addr.to_string().contains("127.0.0.1") || addr.to_string().contains("/::1/")) // Prefer loopback
            .unwrap_or_else(|| node2_addrs.first().expect("Node 2 has no listen addresses at all"))
            .clone();
        println!("Node 1 adding Node 2 ({}, {}) to its Kademlia.", node2_libp2p_peer_id, node2_listen_addr_for_kad);
        node1_service.add_kad_address(node2_libp2p_peer_id.clone(), node2_listen_addr_for_kad).await
            .expect("Node 1 failed to add Node 2 to Kademlia");
        println!("Node 1 triggering Kademlia bootstrap...");
        node1_service.trigger_kad_bootstrap().await
            .expect("Node 1 failed to trigger Kademlia bootstrap");

        // Node 2 already bootstraps to Node 1 during its init. 
        // We can optionally re-trigger bootstrap on Node 2 as well for good measure.
        println!("Node 2 triggering Kademlia bootstrap (re-trigger)...");
        node2_service.trigger_kad_bootstrap().await
            .expect("Node 2 failed to trigger Kademlia bootstrap");

        // Allow time for Kademlia operations to complete and tables to populate
        println!("Allowing time for Kademlia operations (15 seconds)...");
        sleep(Duration::from_secs(15)).await;

        // --- Verify Kademlia Routing Tables Directly ---
        println!("Verifying Node 1's Kademlia routing table...");
        let node1_routing_table = node1_service.get_routing_table_peers().await
            .expect("Failed to get Node 1's routing table");
        println!("Node 1 Kademlia routing table: {:?}", node1_routing_table.iter().map(|p| &p.0).collect::<Vec<_>>());
        let node2_icn_peer_id_for_rt_check = IcnPeerId(node2_libp2p_peer_id.to_string());
        assert!(
            node1_routing_table.contains(&node2_icn_peer_id_for_rt_check),
            "Node 1's routing table should contain Node 2. Expected: {}, Found in table: {:?}", 
            node2_icn_peer_id_for_rt_check.0, node1_routing_table
        );
        println!("Node 1's Kademlia routing table contains Node 2.");

        println!("Verifying Node 2's Kademlia routing table...");
        let node2_routing_table = node2_service.get_routing_table_peers().await
            .expect("Failed to get Node 2's routing table");
        println!("Node 2 Kademlia routing table: {:?}", node2_routing_table.iter().map(|p| &p.0).collect::<Vec<_>>());
        let node1_icn_peer_id_for_rt_check = IcnPeerId(node1_libp2p_peer_id.to_string());
        assert!(
            node2_routing_table.contains(&node1_icn_peer_id_for_rt_check),
            "Node 2's routing table should contain Node 1. Expected: {}, Found in table: {:?}", 
            node1_icn_peer_id_for_rt_check.0, node2_routing_table
        );
        println!("Node 2's Kademlia routing table contains Node 1.");

        // --- Exercise discover_peers (random Kademlia query) and check for successful completion ---
        println!("Node 1 attempting general peer discovery (random Kademlia query)...");
        let discovered_by_node1_random = node1_service.discover_peers(None).await;
        assert!(discovered_by_node1_random.is_ok(), "Node 1's random Kademlia query should complete successfully, even if empty. Result: {:?}", discovered_by_node1_random);
        println!("Node 1 random Kademlia query completed. Result: {:?}", discovered_by_node1_random.as_ref().unwrap().iter().map(|p| &p.0).collect::<Vec<_>>());

        println!("Node 2 attempting general peer discovery (random Kademlia query)...");
        let discovered_by_node2_random = node2_service.discover_peers(None).await;
        assert!(discovered_by_node2_random.is_ok(), "Node 2's random Kademlia query should complete successfully, even if empty. Result: {:?}", discovered_by_node2_random);
        println!("Node 2 random Kademlia query completed. Result: {:?}", discovered_by_node2_random.as_ref().unwrap().iter().map(|p| &p.0).collect::<Vec<_>>());
        
        // The detailed logging and previous assertions for discover_peers results can be removed or commented out
        // as the primary check is now the routing table content.

        println!("Kademlia two_node_peer_discovery test finished successfully.");
    }
} 

#[cfg(feature = "experimental-libp2p")]
mod kademlia_three_node_tests {
    use icn_network::libp2p_service::Libp2pNetworkService;
    use icn_network::PeerId as IcnPeerId; 
    use icn_network::NetworkService; 
    use std::time::Duration;
    use tokio::time::sleep;
    use libp2p::kad::RecordKey as Libp2pKadKey; // Renamed for clarity, using as Libp2pKadKey
    use futures::Future; // Required for Pin<Box<dyn Future>>

    // Helper function for retrying an async operation until a deadline
    async fn wait_until<F, Fut>(timeout: Duration, interval: Duration, mut condition: F) -> Result<(), String>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<bool, String>>, // Condition can now return an error
    {
        let start = tokio::time::Instant::now();
        loop {
            match condition().await {
                Ok(true) => return Ok(()),
                Ok(false) => { // Condition not met, continue
                }
                Err(e) => return Err(format!("Condition check failed: {}", e)), // Propagate error from condition
            }

            if start.elapsed() > timeout {
                return Err("Timeout waiting for condition".to_string());
            }
            sleep(interval).await;
        }
    }

    // Helper to assert that a node's routing table contains all expected peers
    async fn assert_routing_table_contains(
        service: &Libp2pNetworkService,
        expected_peers: &[IcnPeerId],
        node_name: &str,
    ) {
        println!("Waiting for {}'s routing table to converge... Expected: {:?}", node_name, expected_peers.iter().map(|p|&p.0).collect::<Vec<_>>());
        wait_until(Duration::from_secs(15), Duration::from_millis(500), || async {
            match service.get_routing_table_peers().await {
                Ok(current_peers) => {
                    let current_peer_strings = current_peers.iter().map(|p| &p.0).collect::<Vec<_>>();
                    println!("Current {} routing table: {:?}", node_name, current_peer_strings);
                    let all_found = expected_peers.iter().all(|expected_peer| current_peers.contains(expected_peer));
                    if !all_found {
                         println!("{}'s table missing some peers. Expected: {:?}, Found: {:?}", node_name, expected_peers.iter().map(|p|&p.0).collect::<Vec<_>>(), current_peer_strings);
                    }
                    Ok(all_found)
                }
                Err(e) => {
                    println!("Error getting {} routing table: {:?}", node_name, e);
                    Err(format!("Failed to get {} routing table: {}", node_name, e))
                }
            }
        })
        .await
        .unwrap_or_else(|err| panic!("{}'s routing table did not converge: {}. Expected: {:?}", node_name, err, expected_peers.iter().map(|p|&p.0).collect::<Vec<_>>())
        );
        println!("{}'s Kademlia routing table converged and contains all expected peers.", node_name);
    }

    fn setup_tracing_for_test() {
        // Simplified basic tracing setup for ignored test
        let _ = tracing_subscriber::fmt().with_test_writer().try_init();
    }

    #[tokio::test]
    async fn test_kademlia_three_node_peer_discovery() {
        setup_tracing_for_test(); // Call the setup function

        println!("Starting Kademlia three_node_peer_discovery test...");

        // Node 1 Setup (Bootstrap Node)
        let node1_service = Libp2pNetworkService::new(None)
            .await
            .expect("Node 1 failed to start");
        let node1_libp2p_peer_id = node1_service.local_peer_id().clone();
        println!("Node 1 Libp2p Peer ID: {}", node1_libp2p_peer_id);
        sleep(Duration::from_secs(2)).await; 
        let node1_addrs = node1_service.listening_addresses();
        assert!(!node1_addrs.is_empty(), "Node 1 has no listening addresses");
        let node1_listen_addr_for_kad = node1_addrs
            .iter()
            .find(|addr| addr.to_string().contains("127.0.0.1") || addr.to_string().contains("/::1/"))
            .unwrap_or_else(|| node1_addrs.first().expect("Node 1 has no listen addresses at all"))
            .clone();
        println!("Node 1 chosen listen address for Kademlia bootstrap: {}", node1_listen_addr_for_kad);

        // Node 2 Setup (Bootstraps to Node 1)
        let bootstrap_info_for_node2 = Some(vec![(node1_libp2p_peer_id.clone(), node1_listen_addr_for_kad.clone())]);
        let node2_service = Libp2pNetworkService::new(bootstrap_info_for_node2)
            .await
            .expect("Node 2 failed to start");
        let node2_libp2p_peer_id = node2_service.local_peer_id().clone();
        println!("Node 2 Libp2p Peer ID: {}", node2_libp2p_peer_id);
        sleep(Duration::from_secs(2)).await; 
        let node2_addrs = node2_service.listening_addresses();
        assert!(!node2_addrs.is_empty(), "Node 2 has no listening addresses");
         let node2_listen_addr_for_kad = node2_addrs
            .iter()
            .find(|addr| addr.to_string().contains("127.0.0.1") || addr.to_string().contains("/::1/"))
            .unwrap_or_else(|| node2_addrs.first().expect("Node 2 has no listen addresses at all"))
            .clone();
        println!("Node 2 chosen listen address: {}", node2_listen_addr_for_kad);


        // Node 3 Setup (Bootstraps to Node 1)
        let bootstrap_info_for_node3 = Some(vec![(node1_libp2p_peer_id.clone(), node1_listen_addr_for_kad.clone())]);
        let node3_service = Libp2pNetworkService::new(bootstrap_info_for_node3)
            .await
            .expect("Node 3 failed to start");
        let node3_libp2p_peer_id = node3_service.local_peer_id().clone();
        println!("Node 3 Libp2p Peer ID: {}", node3_libp2p_peer_id);
        sleep(Duration::from_secs(2)).await;
        let node3_addrs = node3_service.listening_addresses();
        assert!(!node3_addrs.is_empty(), "Node 3 has no listening addresses");
        let node3_listen_addr_for_kad = node3_addrs
            .iter()
            .find(|addr| addr.to_string().contains("127.0.0.1") || addr.to_string().contains("/::1/"))
            .unwrap_or_else(|| node3_addrs.first().expect("Node 3 has no listen addresses at all"))
            .clone();
        println!("Node 3 chosen listen address: {}", node3_listen_addr_for_kad);

        println!("Allowing nodes to initialize and connect (5 seconds)..");
        sleep(Duration::from_secs(5)).await;

        // Explicitly add peers to Kademlia and trigger bootstrap for full awareness
        // Node 1
        println!("Node 1 adding Node 2 ({}, {}) to its Kademlia.", node2_libp2p_peer_id, node2_listen_addr_for_kad);
        node1_service.add_kad_address(node2_libp2p_peer_id.clone(), node2_listen_addr_for_kad.clone()).await.expect("Node 1 failed to add Node 2");
        println!("Node 1 adding Node 3 ({}, {}) to its Kademlia.", node3_libp2p_peer_id, node3_listen_addr_for_kad);
        node1_service.add_kad_address(node3_libp2p_peer_id.clone(), node3_listen_addr_for_kad.clone()).await.expect("Node 1 failed to add Node 3");
        println!("Node 1 triggering Kademlia bootstrap...");
        node1_service.trigger_kad_bootstrap().await.expect("Node 1 failed to trigger Kademlia bootstrap");

        // Node 2
        println!("Node 2 adding Node 1 ({}, {}) to its Kademlia.", node1_libp2p_peer_id, node1_listen_addr_for_kad);
        node2_service.add_kad_address(node1_libp2p_peer_id.clone(), node1_listen_addr_for_kad.clone()).await.expect("Node 2 failed to add Node 1");
        println!("Node 2 adding Node 3 ({}, {}) to its Kademlia.", node3_libp2p_peer_id, node3_listen_addr_for_kad);
        node2_service.add_kad_address(node3_libp2p_peer_id.clone(), node3_listen_addr_for_kad.clone()).await.expect("Node 2 failed to add Node 3");
        println!("Node 2 triggering Kademlia bootstrap...");
        node2_service.trigger_kad_bootstrap().await.expect("Node 2 failed to trigger Kademlia bootstrap");

        // Node 3
        println!("Node 3 adding Node 1 ({}, {}) to its Kademlia.", node1_libp2p_peer_id, node1_listen_addr_for_kad);
        node3_service.add_kad_address(node1_libp2p_peer_id.clone(), node1_listen_addr_for_kad.clone()).await.expect("Node 3 failed to add Node 1");
        println!("Node 3 adding Node 2 ({}, {}) to its Kademlia.", node2_libp2p_peer_id, node2_listen_addr_for_kad);
        node3_service.add_kad_address(node2_libp2p_peer_id.clone(), node2_listen_addr_for_kad.clone()).await.expect("Node 3 failed to add Node 2");
        println!("Node 3 triggering Kademlia bootstrap...");
        node3_service.trigger_kad_bootstrap().await.expect("Node 3 failed to trigger Kademlia bootstrap");

        println!("Allowing initial time for Kademlia operations (5 seconds).."); 
        sleep(Duration::from_secs(5)).await;

        // --- Verify Kademlia Routing Tables with retries ---
        let node1_icn_peer_id = IcnPeerId(node1_libp2p_peer_id.to_string());
        let node2_icn_peer_id = IcnPeerId(node2_libp2p_peer_id.to_string());
        let node3_icn_peer_id = IcnPeerId(node3_libp2p_peer_id.to_string());

        // Verify Node 1's table
        assert_routing_table_contains(
            &node1_service, 
            &[node2_icn_peer_id.clone(), node3_icn_peer_id.clone()],
            "Node 1"
        ).await;

        // Verify Node 2's table
        assert_routing_table_contains(
            &node2_service, 
            &[node1_icn_peer_id.clone(), node3_icn_peer_id.clone()],
            "Node 2"
        ).await;

        // Verify Node 3's table
        assert_routing_table_contains(
            &node3_service, 
            &[node1_icn_peer_id.clone(), node2_icn_peer_id.clone()],
            "Node 3"
        ).await;

        // --- Exercise discover_peers (random Kademlia query) ---
        println!("Node 1 attempting general peer discovery...");
        let discovered_by_node1 = node1_service.discover_peers(None).await;
        assert!(discovered_by_node1.is_ok(), "Node 1 discover_peers failed: {:?}", discovered_by_node1);
        println!("Node 1 discovered: {:?}", discovered_by_node1.as_ref().unwrap().iter().map(|p| &p.0).collect::<Vec<_>>());

        println!("Node 2 attempting general peer discovery...");
        let discovered_by_node2 = node2_service.discover_peers(None).await;
        assert!(discovered_by_node2.is_ok(), "Node 2 discover_peers failed: {:?}", discovered_by_node2);
        println!("Node 2 discovered: {:?}", discovered_by_node2.as_ref().unwrap().iter().map(|p| &p.0).collect::<Vec<_>>());

        println!("Node 3 attempting general peer discovery...");
        let discovered_by_node3 = node3_service.discover_peers(None).await;
        assert!(discovered_by_node3.is_ok(), "Node 3 discover_peers failed: {:?}", discovered_by_node3);
        println!("Node 3 discovered: {:?}", discovered_by_node3.as_ref().unwrap().iter().map(|p| &p.0).collect::<Vec<_>>());
        
        // --- Test Record Propagation (Node 1 PUT -> Node 3 GET) ---
        println!("Testing Kademlia Record Propagation (Node 1 PUT -> Node 3 GET)...");
        let test_key = Libp2pKadKey::new(&b"icn_test_kad_key_1");
        let test_value = b"icn_test_kad_value_1".to_vec();

        println!("Node 1 putting record: key={:?}, value={:?}", test_key, test_value);
        node1_service.put_kad_record(test_key.clone(), test_value.clone()).await
            .expect("Node 1 failed to put record");
        println!("Node 1 put_record command issued.");

        // Allow some time for the record to propagate
        // This initial sleep might be generous, wait_until will do the fine-grained retries
        println!("Allowing 5 seconds for initial record propagation...");
        sleep(Duration::from_secs(5)).await; 

        println!("Node 3 attempting to get record with retries (timeout 15s, interval 1s)...");
        let get_record_result = wait_until(Duration::from_secs(15), Duration::from_secs(1), || {
            let node3_service_clone = node3_service.clone(); // Clone for async block
            let test_key_clone = test_key.clone(); // Clone for async block
            let test_value_clone = test_value.clone(); // Clone for async block
            async move {
                println!("Node 3: Attempting get_record for key: {:?}", test_key_clone);
                match node3_service_clone.get_kad_record(test_key_clone.clone()).await {
                    Ok(Some(record)) => {
                        println!("Node 3: Got record: {:?}", record);
                        if record.value == test_value_clone {
                            println!("Node 3: Record value matches!");
                            Ok(true) // Record found and value matches
                        } else {
                            println!("Node 3: Record value mismatch. Expected: {:?}, Got: {:?}", test_value_clone, record.value);
                            Ok(false) // Value mismatch, retry
                        }
                    }
                    Ok(None) => {
                        println!("Node 3: get_record returned Ok(None) for key: {:?}", test_key_clone);
                        Ok(false) // Record not found yet, retry
                    }
                    Err(e) => {
                        println!("Node 3: get_record returned error: {:?}", e);
                        Err(format!("Node 3 get_record failed: {}", e)) // Error during get, stop retrying with error
                    }
                }
            }
        }).await;

        assert!(get_record_result.is_ok(), "Node 3 failed to get record or record did not propagate: {:?}", get_record_result.err());
        println!("Node 3 successfully retrieved and verified the record from Kademlia DHT.");

        println!("Kademlia three_node_peer_discovery test finished successfully.");
    }
} 