#![allow(
    unused_imports,
    unused_variables,
    dead_code,
    clippy::field_reassign_with_default,
    clippy::uninlined_format_args,
    clippy::clone_on_copy
)]

use anyhow::Result;
use icn_common::{Cid, Did};
use icn_identity::{generate_ed25519_keypair, ExecutionReceipt, SignatureBytes};
use icn_mesh::{ActualMeshJob as Job, JobId, JobSpec, MeshJobBid as Bid, Resources};
use icn_network::libp2p_service::{Libp2pNetworkService, NetworkConfig};
use icn_network::{NetworkMessage, NetworkService};
use icn_runtime::executor::{JobExecutor, SimpleExecutor};
use libp2p::PeerId as Libp2pPeerId;
use std::str::FromStr;
use tokio::sync::mpsc::Receiver;
use tokio::time::{sleep, timeout, Duration};

/// Represents a test node with networking capabilities
pub struct TestNode {
    pub service: Libp2pNetworkService,
    pub peer_id: String,
    pub receiver: Receiver<NetworkMessage>,
}

/// Test job configuration
pub struct TestJobConfig {
    pub id_suffix: String,
    pub creator_did: Did,
    pub cost_mana: u64,
    pub payload: String,
}

impl Default for TestJobConfig {
    fn default() -> Self {
        Self {
            id_suffix: "test_job".to_string(),
            creator_did: Did::from_str("did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuias7ux1jEZ6KATp8")
                .unwrap(),
            cost_mana: 100,
            payload: "hello world".to_string(),
        }
    }
}

/// Creates two connected test nodes with real libp2p networking
pub async fn setup_connected_nodes() -> Result<(TestNode, TestNode)> {
    println!("🔧 [TEST-UTILS] Setting up connected test nodes...");

    // Create Node A
    let config_a = NetworkConfig::default();
    let node_a_service = Libp2pNetworkService::new(config_a).await?;
    let node_a_peer_id = node_a_service.local_peer_id().to_string();

    // Wait for Node A to establish listeners
    let mut node_a_addrs = Vec::new();
    for _attempt in 1..=5 {
        tokio::time::sleep(Duration::from_secs(1)).await;
        node_a_addrs = node_a_service.listening_addresses();
        if !node_a_addrs.is_empty() {
            break;
        }
    }
    if node_a_addrs.is_empty() {
        return Err(anyhow::anyhow!(
            "Node A failed to establish listening addresses"
        ));
    }

    // Create Node B with bootstrap to Node A
    let node_a_libp2p_peer_id = Libp2pPeerId::from_str(&node_a_peer_id)?;
    let mut config_b = NetworkConfig::default();
    config_b.bootstrap_peers = vec![(node_a_libp2p_peer_id, node_a_addrs[0].clone())];

    let node_b_service = Libp2pNetworkService::new(config_b).await?;
    let node_b_peer_id = node_b_service.local_peer_id().to_string();

    // Allow time for peer discovery
    tokio::time::sleep(Duration::from_secs(8)).await;

    // Verify connectivity
    let node_a_stats = node_a_service.get_network_stats().await?;
    let node_b_stats = node_b_service.get_network_stats().await?;

    if node_a_stats.peer_count == 0 || node_b_stats.peer_count == 0 {
        return Err(anyhow::anyhow!(
            "Nodes failed to connect. A peers: {}, B peers: {}",
            node_a_stats.peer_count,
            node_b_stats.peer_count
        ));
    }

    // Set up message subscriptions
    let node_a_receiver = node_a_service.subscribe().await?;
    let node_b_receiver = node_b_service.subscribe().await?;

    println!(
        "✅ [TEST-UTILS] Nodes connected - A: {}, B: {}",
        node_a_peer_id, node_b_peer_id
    );

    let node_a = TestNode {
        service: node_a_service,
        peer_id: node_a_peer_id,
        receiver: node_a_receiver,
    };

    let node_b = TestNode {
        service: node_b_service,
        peer_id: node_b_peer_id,
        receiver: node_b_receiver,
    };

    Ok((node_a, node_b))
}

/// Creates a test job with the given configuration
pub fn create_test_job(config: &TestJobConfig) -> Job {
    let job_id_cid = Cid::new_v1_dummy(0x55, 0x13, config.id_suffix.as_bytes());
    let job_id = JobId::from(job_id_cid);
    let manifest_cid = Cid::new_v1_dummy(0x71, 0x12, b"dummy_manifest_data");
    let job_spec = JobSpec::Echo {
        payload: config.payload.clone(),
    };

    Job {
        id: job_id,
        creator_did: config.creator_did.clone(),
        manifest_cid,
        spec: job_spec,
        cost_mana: config.cost_mana,
        signature: SignatureBytes(vec![]),
    }
}

/// Creates a test bid for the given job
pub fn create_test_bid(job_id: &JobId, executor_did: &Did, price_mana: u64) -> Bid {
    Bid {
        job_id: job_id.clone(),
        executor_did: executor_did.clone(),
        price_mana,
        resources: Resources::default(),
    }
}

/// Executes a job using SimpleExecutor and returns a signed receipt
pub async fn execute_job_with_simple_executor(
    job: &Job,
    executor_did: &Did,
) -> Result<ExecutionReceipt> {
    let (executor_signing_key, _executor_verifying_key) = generate_ed25519_keypair();
    let executor = SimpleExecutor::new(executor_did.clone(), executor_signing_key);

    let receipt = executor
        .execute_job(job)
        .await
        .map_err(|e| anyhow::anyhow!("Job execution failed: {}", e))?;

    Ok(receipt)
}

/// Verifies a receipt has a valid signature structure
pub fn verify_receipt_signature_format(receipt: &ExecutionReceipt) -> Result<()> {
    if receipt.sig.0.is_empty() {
        return Err(anyhow::anyhow!("Receipt signature is empty"));
    }

    if receipt.sig.0.len() < 32 {
        return Err(anyhow::anyhow!(
            "Receipt signature too short: {} bytes",
            receipt.sig.0.len()
        ));
    }

    Ok(())
}

/// Waits for a specific message type with timeout
pub async fn wait_for_message<F, T>(
    receiver: &mut Receiver<NetworkMessage>,
    timeout_secs: u64,
    matcher: F,
) -> Result<T>
where
    F: Fn(&NetworkMessage) -> Option<T>,
{
    timeout(Duration::from_secs(timeout_secs), async {
        loop {
            if let Some(message) = receiver.recv().await {
                if let Some(result) = matcher(&message) {
                    return Ok(result);
                }
            }
        }
    })
    .await?
}

/// Mock function to anchor receipt to DAG
pub fn mock_anchor_receipt_to_dag(receipt: &ExecutionReceipt) -> Result<Cid> {
    let receipt_data = format!("receipt_for_job_{}", receipt.job_id);
    Ok(Cid::new_v1_dummy(0x71, 0x12, receipt_data.as_bytes()))
}
