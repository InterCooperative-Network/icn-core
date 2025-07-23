use icn_common::{compute_merkle_cid, DagBlock, Did};
use icn_dag::{FileDagStore, StorageService};
use tempfile::tempdir;

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
fn file_store_sharded_layout() {
    let dir = tempdir().unwrap();
    let mut store = FileDagStore::new(dir.path().to_path_buf()).unwrap();
    let block = create_block("a");
    store.put(&block).unwrap();
    let cid_str = block.cid.to_string();
    let expected = dir
        .path()
        .join(&cid_str[0..2])
        .join(&cid_str[2..4])
        .join(&cid_str);
    assert!(expected.exists());
    assert!(store.contains(&block.cid).unwrap());
    assert_eq!(store.get(&block.cid).unwrap().unwrap().cid, block.cid);
    let blocks = store.list_blocks().unwrap();
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].cid, block.cid);
}

#[cfg(feature = "async")]
#[tokio::test]
async fn tokio_file_store_sharded_layout() {
    let dir = tempdir().unwrap();
    let mut store = TokioFileDagStore::new(dir.path().to_path_buf()).unwrap();
    let block = create_block("b");
    store.put(&block).await.unwrap();
    let cid_str = block.cid.to_string();
    let expected = dir
        .path()
        .join(&cid_str[0..2])
        .join(&cid_str[2..4])
        .join(&cid_str);
    assert!(expected.exists());
    assert!(store.contains(&block.cid).await.unwrap());
    assert_eq!(store.get(&block.cid).await.unwrap().unwrap().cid, block.cid);
    let blocks = store.list_blocks().await.unwrap();
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].cid, block.cid);
}
