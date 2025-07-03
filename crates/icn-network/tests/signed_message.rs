use icn_common::Did;
use icn_identity::{did_key_from_verifying_key, generate_ed25519_keypair};
use icn_network::{
    sign_message, NetworkMessage, NetworkService, SignedMessage, StubNetworkService,
};
use std::str::FromStr;

#[tokio::test]
async fn stub_service_valid_signature() {
    let service = StubNetworkService::default();
    let (sk, vk) = generate_ed25519_keypair();
    let did_str = did_key_from_verifying_key(&vk);
    let did = Did::from_str(&did_str).unwrap();
    let msg = NetworkMessage::GossipSub("test".into(), b"hello".to_vec());
    let signed = sign_message(&msg, &did, &sk).unwrap();
    service
        .send_signed_message(&icn_network::PeerId("peer1".into()), signed)
        .await
        .unwrap();
}

#[tokio::test]
async fn stub_service_invalid_signature() {
    let service = StubNetworkService::default();
    let (sk, vk) = generate_ed25519_keypair();
    let did_str = did_key_from_verifying_key(&vk);
    let did = Did::from_str(&did_str).unwrap();
    let msg = NetworkMessage::GossipSub("test".into(), b"hello".to_vec());
    let mut signed = sign_message(&msg, &did, &sk).unwrap();
    // Corrupt the signature
    if let Some(byte) = signed.signature.0.first_mut() {
        *byte ^= 0xFF;
    }
    let err = service
        .send_signed_message(&icn_network::PeerId("peer1".into()), signed)
        .await
        .expect_err("should fail");
    match err {
        icn_network::MeshNetworkError::Common(_) => {}
        other => panic!("unexpected error: {other:?}"),
    }
}

#[tokio::test]
async fn stub_service_duplicate_message() {
    let service = StubNetworkService::default();
    let (sk, vk) = generate_ed25519_keypair();
    let did_str = did_key_from_verifying_key(&vk);
    let did = Did::from_str(&did_str).unwrap();
    let msg = NetworkMessage::GossipSub("dup".into(), b"hello".to_vec());
    let signed = sign_message(&msg, &did, &sk).unwrap();
    service
        .send_signed_message(&icn_network::PeerId("peer1".into()), signed.clone())
        .await
        .unwrap();
    let err = service
        .send_signed_message(&icn_network::PeerId("peer1".into()), signed)
        .await
        .expect_err("should fail");
    match err {
        icn_network::MeshNetworkError::DuplicateMessage => {}
        other => panic!("unexpected error: {other:?}"),
    }
}
