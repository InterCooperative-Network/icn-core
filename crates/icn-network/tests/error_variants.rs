#![allow(clippy::field_reassign_with_default, clippy::uninlined_format_args)]

#[cfg(feature = "libp2p")]
mod error_variants {
    use icn_network::libp2p_service::{Libp2pNetworkService, NetworkConfig};
    use icn_network::{decode_network_message, MeshNetworkError};

    #[tokio::test]
    async fn handshake_error_on_zero_timeout() {
        let mut config = NetworkConfig::default();
        config.connection_timeout = std::time::Duration::from_secs(0);
        match Libp2pNetworkService::new(config).await {
            Err(MeshNetworkError::HandshakeFailed(_)) => {}
            other => panic!("unexpected result: {other:?}"),
        }
    }

    #[test]
    fn decode_message_error() {
        let bytes = vec![1u8, 2, 3];
        match decode_network_message(&bytes) {
            Err(MeshNetworkError::MessageDecodeFailed(_)) => {}
            other => panic!("unexpected result: {other:?}"),
        }
    }
}
