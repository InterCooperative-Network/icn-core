//! Enhanced P2P+DAG Integration Tests with Real Distributed Storage
//!
//! This module provides robust end-to-end tests for P2P networking and DAG functionality
//! with actual message handling, distributed block storage, and network resilience testing.

#[cfg(feature = "enable-libp2p")]
mod enhanced_p2p_dag_tests {
    use futures::future::join_all;
    use icn_common::{
        compute_merkle_cid, Cid, DagBlock, DagLink, Did, SystemTimeProvider, TimeProvider,
    };
    use icn_dag::{InMemoryDagStore, StorageService};
    use icn_network::{
        libp2p_service::{Libp2pNetworkService, NetworkConfig},
        NetworkService, NetworkStats, PeerId,
    };
    use icn_protocol::{
        DagBlockAnnouncementMessage, DagBlockRequestMessage, DagBlockResponseMessage,
        MessagePayload, ProtocolMessage,
    };
    use std::collections::{HashMap, HashSet};
    use std::sync::{Arc, Mutex, Once};
    use std::time::{Duration, Instant};
    use tokio::sync::RwLock;
    use tokio::time::{sleep, timeout};
    use tracing::{debug, error, info, warn};

    /// Enhanced test node with proper message handling
    struct EnhancedTestNode {
        peer_id: PeerId,
        did: Did,
        network_service: Arc<Libp2pNetworkService>,
        dag_store: Arc<Mutex<InMemoryDagStore>>,
        message_handler: Arc<MessageHandler>,
        _message_receiver_task: tokio::task::JoinHandle<()>,
    }

    /// Message handler for processing P2P messages
    struct MessageHandler {
        dag_store: Arc<Mutex<InMemoryDagStore>>,
        network_service: Arc<Libp2pNetworkService>,
        pending_requests: Arc<RwLock<HashMap<Cid, Vec<PeerId>>>>,
        known_blocks: Arc<RwLock<HashSet<Cid>>>,
    }

    impl MessageHandler {
        fn new(
            dag_store: Arc<Mutex<InMemoryDagStore>>,
            network_service: Arc<Libp2pNetworkService>,
        ) -> Self {
            Self {
                dag_store,
                network_service,
                pending_requests: Arc::new(RwLock::new(HashMap::new())),
                known_blocks: Arc::new(RwLock::new(HashSet::new())),
            }
        }

        /// Handle incoming protocol messages
        async fn handle_message(
            &self,
            message: ProtocolMessage,
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            match message.payload {
                MessagePayload::DagBlockRequest(req) => {
                    self.handle_block_request(req, message.sender).await
                }
                MessagePayload::DagBlockAnnouncement(announcement) => {
                    self.handle_block_announcement(announcement).await
                }
                MessagePayload::DagBlockResponse(response) => {
                    self.handle_block_response(response).await
                }
                _ => {
                    debug!("Ignoring non-DAG message: {:?}", message.payload);
                    Ok(())
                }
            }
        }

        /// Handle DAG block requests from peers
        async fn handle_block_request(
            &self,
            request: DagBlockRequestMessage,
            requester: Did,
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            debug!(
                "Handling block request for {} from {}",
                request.block_cid, requester
            );

            let block_data = {
                let store = self.dag_store.lock().unwrap();
                store.get(&request.block_cid)?
            };

            let response = if let Some(block) = block_data {
                ProtocolMessage::new(
                    MessagePayload::DagBlockResponse(DagBlockResponseMessage {
                        block_cid: request.block_cid.clone(),
                        block_data: Some(block),
                        error: None,
                    }),
                    requester.clone(),
                    Some(requester.clone()),
                )
            } else {
                ProtocolMessage::new(
                    MessagePayload::DagBlockResponse(DagBlockResponseMessage {
                        block_cid: request.block_cid.clone(),
                        block_data: None,
                        error: Some("Block not found".to_string()),
                    }),
                    requester.clone(),
                    Some(requester.clone()),
                )
            };

            // For now, just broadcast the response (in a real implementation, we'd send directly to requester)
            self.network_service.broadcast_message(response).await?;
            debug!("Sent response for block {}", request.block_cid);

            Ok(())
        }

        /// Handle DAG block announcements
        async fn handle_block_announcement(
            &self,
            announcement: DagBlockAnnouncementMessage,
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            debug!("Received block announcement for {}", announcement.block_cid);

            // Check if we already have this block
            let have_block = {
                let store = self.dag_store.lock().unwrap();
                store.get(&announcement.block_cid)?.is_some()
            };

            if !have_block {
                // Add to known blocks for potential future requests
                {
                    let mut known = self.known_blocks.write().await;
                    known.insert(announcement.block_cid.clone());
                }

                debug!("Added {} to known blocks", announcement.block_cid);
            }

            Ok(())
        }

        /// Handle DAG block responses
        async fn handle_block_response(
            &self,
            response: DagBlockResponseMessage,
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            if let Some(block) = response.block_data {
                debug!("Received block {} in response", block.cid);

                // Store the received block
                {
                    let mut store = self.dag_store.lock().unwrap();
                    store.put(&block)?;
                }

                // Notify any pending requests for this block
                let cid = block.cid.clone();
                let mut pending = self.pending_requests.write().await;
                if let Some(waiters) = pending.remove(&cid) {
                    debug!("Notified {} waiters for block {}", waiters.len(), cid);
                }
            } else if let Some(error) = response.error {
                warn!("Block request failed: {}", error);
            }

            Ok(())
        }

        /// Request a block from the network with timeout
        async fn request_block(
            &self,
            cid: &Cid,
            timeout_duration: Duration,
        ) -> Result<Option<DagBlock>, Box<dyn std::error::Error + Send + Sync>> {
            // Check if we already have it locally
            {
                let store = self.dag_store.lock().unwrap();
                if let Some(block) = store.get(cid)? {
                    return Ok(Some(block));
                }
            }

            // Register this request
            {
                let mut pending = self.pending_requests.write().await;
                pending.entry(cid.clone()).or_insert_with(Vec::new);
            }

            // Send request to network
            let request = ProtocolMessage::new(
                MessagePayload::DagBlockRequest(DagBlockRequestMessage {
                    block_cid: cid.clone(),
                    priority: 128,
                }),
                Did::new("temp", "requester"), // Temporary DID
                None,
            );

            self.network_service.broadcast_message(request).await?;

            // Wait for response with timeout
            let start = Instant::now();
            while start.elapsed() < timeout_duration {
                {
                    let store = self.dag_store.lock().unwrap();
                    if let Some(block) = store.get(cid)? {
                        return Ok(Some(block));
                    }
                }
                sleep(Duration::from_millis(100)).await;
            }

            // Clean up pending request
            {
                let mut pending = self.pending_requests.write().await;
                pending.remove(cid);
            }

            Ok(None)
        }
    }

    impl EnhancedTestNode {
        async fn new(
            name: &str,
            bootstrap_peers: Vec<(libp2p::PeerId, libp2p::Multiaddr)>,
        ) -> Self {
            let mut network_config = NetworkConfig::development();
            network_config.bootstrap_peers = bootstrap_peers;

            let network_service =
                Arc::new(Libp2pNetworkService::new(network_config).await.unwrap());
            let peer_id = PeerId::from(*network_service.local_peer_id());
            let did = Did::new("test", &format!("node_{}", name));
            let dag_store = Arc::new(Mutex::new(InMemoryDagStore::new()));

            // Create message handler
            let message_handler = Arc::new(MessageHandler::new(
                dag_store.clone(),
                network_service.clone(),
            ));

            // Start message receiver task
            let handler_clone = message_handler.clone();
            let mut receiver = network_service.subscribe().await.unwrap();
            let message_receiver_task = tokio::spawn(async move {
                while let Some(message) = receiver.recv().await {
                    if let Err(e) = handler_clone.handle_message(message).await {
                        error!("Error handling message: {}", e);
                    }
                }
            });

            Self {
                peer_id,
                did,
                network_service,
                dag_store,
                message_handler,
                _message_receiver_task: message_receiver_task,
            }
        }

        async fn get_connectivity_stats(
            &self,
        ) -> Result<NetworkStats, Box<dyn std::error::Error + Send + Sync>> {
            self.network_service
                .get_network_stats()
                .await
                .map_err(Into::into)
        }

        fn create_test_block(&self, data: &[u8], link_cids: Vec<Cid>) -> DagBlock {
            let dag_links: Vec<DagLink> = link_cids
                .into_iter()
                .map(|c| DagLink {
                    cid: c,
                    name: "test_link".to_string(),
                    size: 100,
                })
                .collect();

            let timestamp = SystemTimeProvider.unix_seconds();
            let signature = None;
            let scope = None;

            let cid = compute_merkle_cid(
                0x55, data, &dag_links, timestamp, &self.did, &signature, &scope,
            );

            DagBlock {
                cid,
                data: data.to_vec(),
                links: dag_links,
                timestamp,
                author_did: self.did.clone(),
                signature,
                scope,
            }
        }

        async fn store_block(
            &self,
            block: &DagBlock,
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            {
                let mut store = self.dag_store.lock().unwrap();
                store.put(block)?;
            }

            // Announce the block to the network
            let announcement = ProtocolMessage::new(
                MessagePayload::DagBlockAnnouncement(DagBlockAnnouncementMessage {
                    block_cid: block.cid.clone(),
                    block_size: block.data.len() as u64,
                    link_count: block.links.len() as u32,
                    created_at: block.timestamp,
                }),
                self.did.clone(),
                None,
            );

            self.network_service.broadcast_message(announcement).await?;
            info!(
                "Node {:?} stored and announced block {}",
                self.peer_id, block.cid
            );
            Ok(())
        }

        fn get_local_block(&self, cid: &Cid) -> Option<DagBlock> {
            let store = self.dag_store.lock().unwrap();
            store.get(cid).unwrap()
        }

        async fn request_block_from_network(
            &self,
            cid: &Cid,
        ) -> Result<Option<DagBlock>, Box<dyn std::error::Error + Send + Sync>> {
            self.message_handler
                .request_block(cid, Duration::from_secs(5))
                .await
        }
    }

    /// Enhanced test network with real distributed storage
    struct EnhancedTestNetwork {
        nodes: Vec<EnhancedTestNode>,
    }

    impl EnhancedTestNetwork {
        async fn new(node_count: usize) -> Self {
            assert!(node_count >= 2, "Need at least 2 nodes for testing");
            let mut nodes = Vec::new();

            info!("Creating enhanced test network with {} nodes", node_count);

            // Create bootstrap node
            let bootstrap_node = EnhancedTestNode::new("bootstrap", vec![]).await;
            let bootstrap_peer_id = *bootstrap_node.network_service.local_peer_id();

            // Get bootstrap addresses
            let bootstrap_addrs = bootstrap_node.network_service.listening_addresses();
            assert!(
                !bootstrap_addrs.is_empty(),
                "Bootstrap node should have listening addresses"
            );
            let bootstrap_addr = bootstrap_addrs[0].clone();

            info!(
                "Bootstrap node {:?} listening on: {}",
                bootstrap_peer_id, bootstrap_addr
            );
            nodes.push(bootstrap_node);

            // Create remaining nodes
            for i in 1..node_count {
                let node_name = format!("node_{}", i);
                let bootstrap_peers = vec![(bootstrap_peer_id, bootstrap_addr.clone())];
                let node = EnhancedTestNode::new(&node_name, bootstrap_peers).await;
                nodes.push(node);
            }

            // Wait for network to establish
            info!("Waiting for network to establish connections...");
            sleep(Duration::from_secs(5)).await;

            let network = Self { nodes };
            network.verify_connectivity().await;
            network
        }

        async fn verify_connectivity(&self) {
            info!("Verifying enhanced network connectivity...");

            for (i, node) in self.nodes.iter().enumerate() {
                let stats = node
                    .get_connectivity_stats()
                    .await
                    .expect("Failed to get connectivity stats");

                info!("Node {}: {} peers", i, stats.peer_count);

                if i > 0 {
                    assert!(
                        stats.peer_count > 0,
                        "Node {} should be connected to at least one peer",
                        i
                    );
                }
            }

            info!("✅ All nodes are properly connected");
        }

        /// Test real distributed storage with network communication
        async fn test_real_distributed_storage(
            &self,
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            assert!(
                self.nodes.len() >= 2,
                "Need at least 2 nodes for distributed storage test"
            );

            info!("Testing real distributed storage...");

            // Create a test block on node 0
            let test_data = b"real_distributed_storage_test_data";
            let block = self.nodes[0].create_test_block(test_data, vec![]);
            let block_cid = block.cid.clone();

            // Store the block on node 0
            self.nodes[0].store_block(&block).await?;

            // Wait for network propagation
            sleep(Duration::from_secs(1)).await;

            // Verify node 0 has the block
            let local_block = self.nodes[0].get_local_block(&block_cid);
            assert!(
                local_block.is_some(),
                "Node 0 should have the block locally"
            );

            // Request the block from node 1 via network
            info!("Requesting block from network via node 1...");
            let retrieved_block = self.nodes[1].request_block_from_network(&block_cid).await?;

            assert!(
                retrieved_block.is_some(),
                "Node 1 should be able to retrieve the block via network"
            );
            assert_eq!(
                retrieved_block.unwrap().cid,
                block_cid,
                "Retrieved block should match original"
            );

            info!("✅ Real distributed storage test passed");
            Ok(())
        }

        /// Test concurrent operations and network resilience
        async fn test_concurrent_operations(
            &self,
            num_operations: usize,
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            info!(
                "Testing {} concurrent operations across {} nodes",
                num_operations,
                self.nodes.len()
            );

            let start_time = Instant::now();
            let mut tasks = Vec::new();

            for i in 0..num_operations {
                let node_idx = i % self.nodes.len();
                let node = &self.nodes[node_idx];

                let test_data = format!("concurrent_test_data_{}", i).into_bytes();
                let block = node.create_test_block(&test_data, vec![]);
                let block_cid = block.cid.clone();

                // Create concurrent store operations
                let store_task = {
                    let node_store = node.dag_store.clone();
                    let node_network = node.network_service.clone();
                    let node_did = node.did.clone();

                    async move {
                        // Store block
                        {
                            let mut store = node_store.lock().unwrap();
                            store.put(&block)?;
                        }

                        // Announce to network
                        let announcement = ProtocolMessage::new(
                            MessagePayload::DagBlockAnnouncement(DagBlockAnnouncementMessage {
                                block_cid: block.cid.clone(),
                                block_size: block.data.len() as u64,
                                link_count: block.links.len() as u32,
                                created_at: block.timestamp,
                            }),
                            node_did,
                            None,
                        );

                        node_network.broadcast_message(announcement).await?;

                        Ok::<Cid, Box<dyn std::error::Error + Send + Sync>>(block_cid)
                    }
                };

                tasks.push(store_task);
            }

            // Execute all operations concurrently
            let results = join_all(tasks).await;

            // Check results
            let mut successful_operations = 0;
            for result in results {
                match result {
                    Ok(_) => successful_operations += 1,
                    Err(e) => warn!("Concurrent operation failed: {}", e),
                }
            }

            let duration = start_time.elapsed();
            let ops_per_second = successful_operations as f64 / duration.as_secs_f64();

            info!(
                "✅ Completed {}/{} operations in {:?} ({:.2} ops/sec)",
                successful_operations, num_operations, duration, ops_per_second
            );

            assert!(
                successful_operations >= num_operations * 8 / 10,
                "At least 80% of operations should succeed"
            );

            Ok(())
        }
    }

    static INIT: Once = Once::new();

    fn init_tracing() {
        INIT.call_once(|| {
            tracing_subscriber::fmt::init();
        });
    }

    // Tests
    #[tokio::test]
    async fn test_enhanced_network_setup() {
        init_tracing();

        let _network = timeout(Duration::from_secs(30), EnhancedTestNetwork::new(3))
            .await
            .expect("Network setup should not timeout");

        info!("✅ Enhanced network setup test passed");
    }

    #[tokio::test]
    async fn test_real_distributed_storage_enhanced() {
        init_tracing();

        let network = timeout(Duration::from_secs(30), EnhancedTestNetwork::new(3))
            .await
            .expect("Network setup should not timeout");

        timeout(
            Duration::from_secs(30),
            network.test_real_distributed_storage(),
        )
        .await
        .expect("Distributed storage test should not timeout")
        .expect("Distributed storage test should succeed");

        info!("✅ Real distributed storage enhanced test passed");
    }

    #[tokio::test]
    async fn test_concurrent_operations_enhanced() {
        init_tracing();

        let network = timeout(Duration::from_secs(30), EnhancedTestNetwork::new(3))
            .await
            .expect("Network setup should not timeout");

        timeout(
            Duration::from_secs(60),
            network.test_concurrent_operations(20),
        )
        .await
        .expect("Concurrent operations test should not timeout")
        .expect("Concurrent operations test should succeed");

        info!("✅ Concurrent operations enhanced test passed");
    }
}
