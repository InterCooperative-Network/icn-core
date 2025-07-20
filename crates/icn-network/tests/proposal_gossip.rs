use async_trait::async_trait;
use icn_common::Did;
use icn_network::{
    gossip_proposal_with_retry, MeshNetworkError, NetworkService, PeerId, StubNetworkService,
};
use icn_protocol::{GossipMessage, MessagePayload, ProtocolMessage};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::Receiver;

#[derive(Debug)]
struct FlakyService {
    inner: StubNetworkService,
    attempts: Arc<Mutex<u32>>,
    fail_for: u32,
}

impl FlakyService {
    fn new(fail_for: u32) -> Self {
        Self {
            inner: StubNetworkService::default(),
            attempts: Arc::new(Mutex::new(0)),
            fail_for,
        }
    }
}

#[async_trait]
impl NetworkService for FlakyService {
    async fn discover_peers(&self, t: Option<String>) -> Result<Vec<PeerId>, MeshNetworkError> {
        self.inner.discover_peers(t).await
    }
    async fn send_message(&self, p: &PeerId, m: ProtocolMessage) -> Result<(), MeshNetworkError> {
        self.inner.send_message(p, m).await
    }
    async fn broadcast_message(&self, m: ProtocolMessage) -> Result<(), MeshNetworkError> {
        let mut count = self.attempts.lock().unwrap();
        *count += 1;
        if *count <= self.fail_for {
            return Err(MeshNetworkError::Libp2p("temporary failure".into()));
        }
        self.inner.broadcast_message(m).await
    }
    async fn subscribe(&self) -> Result<Receiver<ProtocolMessage>, MeshNetworkError> {
        self.inner.subscribe().await
    }
    async fn get_network_stats(&self) -> Result<icn_network::NetworkStats, MeshNetworkError> {
        self.inner.get_network_stats().await
    }
    async fn store_record(&self, k: String, v: Vec<u8>) -> Result<(), MeshNetworkError> {
        self.inner.store_record(k, v).await
    }
    async fn get_record(&self, k: String) -> Result<Option<Vec<u8>>, MeshNetworkError> {
        self.inner.get_record(k).await
    }
    #[cfg(feature = "libp2p")]
    async fn connect_peer(&self, addr: libp2p::Multiaddr) -> Result<(), MeshNetworkError> {
        self.inner.connect_peer(addr).await
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[tokio::test]
async fn proposal_gossip_retries() {
    let service = FlakyService::new(2);
    let msg = ProtocolMessage::new(
        MessagePayload::GossipMessage(GossipMessage {
            topic: "proposal".into(),
            payload: vec![],
            ttl: 1,
        }),
        Did::from_str("did:key:abc").unwrap(),
        None,
    );
    gossip_proposal_with_retry(&service, msg).await.unwrap();
    let attempts = *service.attempts.lock().unwrap();
    assert!(attempts > 2);
}
