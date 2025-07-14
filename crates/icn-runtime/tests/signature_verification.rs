use async_trait::async_trait;
use icn_common::Cid;
use icn_identity::{ExecutionReceipt, SignatureBytes};
use icn_mesh::{JobId, MeshJobBid, Resources};
use icn_network::{MeshNetworkError, NetworkService, NetworkStats, PeerId};
use icn_protocol::{
    ExecutionMetadata, MeshBidSubmissionMessage, MeshReceiptSubmissionMessage, MessagePayload,
    ProtocolMessage, ResourceRequirements,
};
use icn_runtime::context::{DefaultMeshNetworkService, MeshNetworkService as _, Signer};
use libp2p::Multiaddr;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration as StdDuration;
use tokio::sync::{mpsc, Mutex};

#[derive(Debug, Default)]
struct ChannelNetworkService {
    subscribers: Arc<Mutex<Vec<mpsc::Sender<ProtocolMessage>>>>,
}

impl ChannelNetworkService {
    fn new() -> Self {
        Self {
            subscribers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    async fn push(&self, msg: ProtocolMessage) {
        let mut subs = self.subscribers.lock().await;
        subs.retain(|tx| tx.try_send(msg.clone()).is_ok());
    }
}

#[derive(Debug)]
struct TestSigner {
    sk: icn_identity::SigningKey,
    pk: icn_identity::VerifyingKey,
    did: icn_common::Did,
}

impl TestSigner {
    fn new() -> Self {
        let (sk, pk) = icn_identity::generate_ed25519_keypair();
        let did_str = icn_identity::did_key_from_verifying_key(&pk);
        let did = icn_common::Did::from_str(&did_str).unwrap();
        Self { sk, pk, did }
    }
}

impl Signer for TestSigner {
    fn sign(&self, payload: &[u8]) -> Result<Vec<u8>, icn_runtime::context::HostAbiError> {
        Ok(icn_identity::sign_message(&self.sk, payload)
            .to_bytes()
            .to_vec())
    }

    fn verify(
        &self,
        payload: &[u8],
        signature: &[u8],
        public_key_bytes: &[u8],
    ) -> Result<bool, icn_runtime::context::HostAbiError> {
        let vk =
            icn_identity::VerifyingKey::from_bytes(public_key_bytes.try_into().map_err(|_| {
                icn_runtime::context::HostAbiError::SignatureError("Invalid public key".into())
            })?)
            .map_err(|e| icn_runtime::context::HostAbiError::SignatureError(format!("{e}")))?;
        let sig = icn_identity::EdSignature::from_bytes(signature.try_into().map_err(|_| {
            icn_runtime::context::HostAbiError::SignatureError("Invalid signature".into())
        })?);
        Ok(icn_identity::verify_signature(&vk, payload, &sig))
    }

    fn public_key_bytes(&self) -> Vec<u8> {
        self.pk.to_bytes().to_vec()
    }
    fn did(&self) -> icn_common::Did {
        self.did.clone()
    }
    fn verifying_key_ref(&self) -> &icn_identity::VerifyingKey {
        &self.pk
    }
}

#[async_trait]
impl NetworkService for ChannelNetworkService {
    async fn discover_peers(&self, _: Option<String>) -> Result<Vec<PeerId>, MeshNetworkError> {
        Ok(vec![])
    }
    async fn send_message(
        &self,
        _peer: &PeerId,
        msg: ProtocolMessage,
    ) -> Result<(), MeshNetworkError> {
        self.push(msg).await;
        Ok(())
    }
    async fn broadcast_message(&self, msg: ProtocolMessage) -> Result<(), MeshNetworkError> {
        self.push(msg).await;
        Ok(())
    }
    async fn subscribe(&self) -> Result<mpsc::Receiver<ProtocolMessage>, MeshNetworkError> {
        let (tx, rx) = mpsc::channel(10);
        self.subscribers.lock().await.push(tx);
        Ok(rx)
    }
    async fn get_network_stats(&self) -> Result<NetworkStats, MeshNetworkError> {
        Ok(NetworkStats::default())
    }
    async fn store_record(&self, _key: String, _value: Vec<u8>) -> Result<(), MeshNetworkError> {
        Ok(())
    }
    async fn get_record(&self, _key: String) -> Result<Option<Vec<u8>>, MeshNetworkError> {
        Ok(None)
    }
    async fn connect_peer(&self, _addr: Multiaddr) -> Result<(), MeshNetworkError> {
        Ok(())
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[tokio::test]
async fn collect_bids_signature_validation() {
    let net = Arc::new(ChannelNetworkService::new());
    let signer = Arc::new(TestSigner::new());
    let service = DefaultMeshNetworkService::new(net.clone(), signer.clone());

    let job_id = JobId(Cid::new_v1_sha256(0x55, b"bid_job"));

    let unsigned = MeshJobBid {
        job_id: job_id.clone(),
        executor_did: signer.did(),
        price_mana: 5,
        resources: Resources {
            cpu_cores: 1,
            memory_mb: 1,
        },
        signature: SignatureBytes(vec![]),
    };
    let sig = signer.sign(&unsigned.to_signable_bytes().unwrap()).unwrap();
    let _signed_bid = MeshJobBid {
        signature: SignatureBytes(sig.clone()),
        ..unsigned
    };

    let bid_message = MeshBidSubmissionMessage {
        job_id: Cid::from(job_id.clone()),
        executor_did: signer.did(),
        cost_mana: 5,
        estimated_duration_secs: 60,
        offered_resources: ResourceRequirements {
            cpu_cores: 1,
            memory_mb: 1,
            storage_mb: 0,
            max_execution_time_secs: 300,
        },
        reputation_score: 0,
    };

    let protocol_msg = ProtocolMessage {
        version: 1,
        payload: MessagePayload::MeshBidSubmission(bid_message.clone()),
        sender: signer.did(),
        recipient: None,
        timestamp: 0,
        signature: SignatureBytes(sig.clone()),
    };

    let net_clone = net.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        net_clone.push(protocol_msg).await;
    });

    let bids = service
        .collect_bids_for_job(&job_id, StdDuration::from_millis(50))
        .await
        .unwrap();
    assert_eq!(bids.len(), 1);
    assert_eq!(bids[0].executor_did, signer.did());

    // Invalid signature case
    let mut bad_sig = sig.clone();
    bad_sig[0] ^= 0xFF;
    let bad_msg = ProtocolMessage {
        version: 1,
        payload: MessagePayload::MeshBidSubmission(bid_message),
        sender: signer.did(),
        recipient: None,
        timestamp: 0,
        signature: SignatureBytes(bad_sig),
    };
    let net_clone = net.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        net_clone.push(bad_msg).await;
    });
    let bids = service
        .collect_bids_for_job(&job_id, StdDuration::from_millis(50))
        .await
        .unwrap();
    assert_eq!(bids.len(), 0);
}

#[tokio::test]
async fn receive_receipt_signature_validation() {
    let net = Arc::new(ChannelNetworkService::new());
    let signer = Arc::new(TestSigner::new());
    let service = DefaultMeshNetworkService::new(net.clone(), signer.clone());

    let job_id = JobId(Cid::new_v1_sha256(0x55, b"rcpt_job"));
    let mut receipt = ExecutionReceipt {
        job_id: Cid::from(job_id.clone()),
        executor_did: signer.did(),
        result_cid: Cid::new_v1_sha256(0x55, b"res"),
        cpu_ms: 1,
        success: true,
        sig: SignatureBytes(vec![]),
    };
    let sig_bytes = signer.sign(&receipt.to_signable_bytes().unwrap()).unwrap();
    receipt.sig = SignatureBytes(sig_bytes.clone());

    let receipt_message = MeshReceiptSubmissionMessage {
        receipt: receipt.clone(),
        execution_metadata: ExecutionMetadata {
            wall_time_ms: 1,
            peak_memory_mb: 1,
            exit_code: 0,
            execution_logs: None,
        },
    };
    let protocol_msg = ProtocolMessage {
        version: 1,
        payload: MessagePayload::MeshReceiptSubmission(receipt_message.clone()),
        sender: signer.did(),
        recipient: None,
        timestamp: 0,
        signature: SignatureBytes(vec![]),
    };
    let net_clone = net.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        net_clone.push(protocol_msg).await;
    });
    let got = service
        .try_receive_receipt(&job_id, &signer.did(), StdDuration::from_millis(50))
        .await
        .unwrap();
    assert!(got.is_some());

    // Invalid signature case
    let mut bad_receipt = receipt.clone();
    bad_receipt.sig.0[0] ^= 0xFF;
    let bad_message = MeshReceiptSubmissionMessage {
        receipt: bad_receipt,
        execution_metadata: ExecutionMetadata {
            wall_time_ms: 1,
            peak_memory_mb: 1,
            exit_code: 0,
            execution_logs: None,
        },
    };
    let bad_protocol = ProtocolMessage {
        version: 1,
        payload: MessagePayload::MeshReceiptSubmission(bad_message),
        sender: signer.did(),
        recipient: None,
        timestamp: 0,
        signature: SignatureBytes(vec![]),
    };
    let net_clone = net.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        net_clone.push(bad_protocol).await;
    });
    let got = service
        .try_receive_receipt(&job_id, &signer.did(), StdDuration::from_millis(50))
        .await
        .unwrap();
    assert!(got.is_none());
}
