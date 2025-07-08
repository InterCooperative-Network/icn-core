use icn_common::{compute_merkle_cid, DagBlock, Did};
use icn_dag::{InMemoryDagStore, StorageService};

fn create_block(id: &str) -> DagBlock {
    let data = format!("data {id}").into_bytes();
    let ts = 0u64;
    let author = Did::new("key", "tester");
    let sig = None;
    let cid = compute_merkle_cid(0x71, &data, &[], ts, &author, &sig, &None);
    DagBlock {
        cid,
        data,
        links: vec![],
        timestamp: ts,
        author_did: author,
        signature: sig,
        scope: None,
    }
}

#[test]
fn prune_expired_unpinned() {
    let mut store = InMemoryDagStore::new();
    let block = create_block("b");
    store.put(&block).unwrap();
    store.set_ttl(&block.cid, Some(10)).unwrap();
    assert!(store.prune_expired(5).unwrap().is_empty());
    assert!(store.contains(&block.cid).unwrap());
    let removed = store.prune_expired(11).unwrap();
    assert_eq!(removed.len(), 1);
    assert!(!store.contains(&block.cid).unwrap());
}

#[test]
fn prune_keeps_pinned() {
    let mut store = InMemoryDagStore::new();
    let block = create_block("p");
    store.put(&block).unwrap();
    store.set_ttl(&block.cid, Some(10)).unwrap();
    store.pin_block(&block.cid).unwrap();
    let removed = store.prune_expired(20).unwrap();
    assert!(removed.is_empty());
    assert!(store.contains(&block.cid).unwrap());
}
