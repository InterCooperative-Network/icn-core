use ed25519_dalek::{SigningKey, VerifyingKey};
use icn_common::{
    compute_merkle_cid, Cid, DagBlock, DagLink, Did, DidDocument, Signable, Transaction,
};
use rand_core::OsRng;

#[test]
fn transaction_sign_verify() {
    let sk = SigningKey::generate(&mut OsRng);
    let pk: VerifyingKey = sk.verifying_key();
    let tx = Transaction {
        id: "tx1".to_string(),
        timestamp: 1,
        sender_did: Did::new("key", "a"),
        recipient_did: None,
        payload_type: "test".to_string(),
        payload: b"hello".to_vec(),
        nonce: 0,
        gas_limit: 100,
        gas_price: 1,
        signature: None,
    };

    let sig = tx.sign(&sk).unwrap();
    assert!(tx.verify(&sig, &pk).is_ok());

    let mut bad_tx = tx.clone();
    bad_tx.payload.push(1);
    assert!(bad_tx.verify(&sig, &pk).is_err());
}

#[test]
fn transaction_serialization_roundtrip() {
    let tx = Transaction {
        id: "tx2".to_string(),
        timestamp: 42,
        sender_did: Did::new("key", "alice"),
        recipient_did: Some(Did::new("key", "bob")),
        payload_type: "test".to_string(),
        payload: b"hello".to_vec(),
        nonce: 7,
        gas_limit: 200,
        gas_price: 2,
        signature: None,
    };

    let json = serde_json::to_string(&tx).unwrap();
    let parsed: Transaction = serde_json::from_str(&json).unwrap();
    assert_eq!(tx, parsed);
}

#[test]
fn dagblock_sign_verify() {
    let sk = SigningKey::generate(&mut OsRng);
    let pk = sk.verifying_key();

    let link_cid = Cid::new_v1_sha256(0x71, b"child");
    let link = DagLink {
        cid: link_cid,
        name: "child".into(),
        size: 5,
    };
    let data = b"parent".to_vec();
    let timestamp = 0u64;
    let author = Did::new("key", "tester");
    let sig_opt = None;
    let cid = compute_merkle_cid(
        0x71,
        &data,
        std::slice::from_ref(&link),
        timestamp,
        &author,
        &sig_opt,
    );
    let block = DagBlock {
        cid,
        data,
        links: vec![link],
        timestamp,
        author_did: author,
        signature: sig_opt,
    };

    let sig = block.sign(&sk).unwrap();
    assert!(block.verify(&sig, &pk).is_ok());

    let mut bad_block = block.clone();
    bad_block.data.push(0);
    assert!(bad_block.verify(&sig, &pk).is_err());
}

#[test]
fn did_document_sign_verify() {
    let sk = SigningKey::generate(&mut OsRng);
    let pk = sk.verifying_key();
    let did = Did::new("key", "abc");
    let doc = DidDocument {
        id: did.clone(),
        public_key: pk.as_bytes().to_vec(),
    };

    let sig = doc.sign(&sk).unwrap();
    assert!(doc.verify(&sig, &pk).is_ok());
}
