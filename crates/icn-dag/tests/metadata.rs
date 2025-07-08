use icn_common::{compute_merkle_cid, DagBlock, Did};
use icn_dag::{block_to_metadata, DagBlockMetadata};

#[test]
fn block_to_metadata_maps_fields() {
    let data = b"hello".to_vec();
    let timestamp = 42u64;
    let author = Did::new("key", "tester");
    let sig = None;
    let cid = compute_merkle_cid(0x71, &data, &[], timestamp, &author, &sig, &None);
    let block = DagBlock {
        cid,
        data: data.clone(),
        links: vec![],
        timestamp,
        author_did: author.clone(),
        signature: sig,
        scope: None,
    };
    let meta = block_to_metadata(&block);
    assert_eq!(meta.size, data.len() as u64);
    assert_eq!(meta.timestamp, timestamp);
    assert_eq!(meta.author_did, author);
    assert!(meta.links.is_empty());
}
