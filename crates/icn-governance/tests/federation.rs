use icn_governance::request_federation_sync;
use icn_network::{PeerId, StubNetworkService};

#[cfg(feature = "federation")]
#[tokio::test]
async fn send_sync_request() {
    let service = StubNetworkService::default();
    let peer = PeerId("mock_peer_1".to_string());
    let result = request_federation_sync(&service, &peer, None).await;
    assert!(result.is_ok());
}
