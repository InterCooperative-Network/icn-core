#[cfg(feature = "persist-sled")]
mod tests {
    use icn_common::{compute_merkle_cid, DagBlock, Did};
    use icn_dag::sled_store::SledDagStore;
    use icn_dag::StorageService;
    use tempfile::tempdir;

    fn create_block(id: &str) -> DagBlock {
        let data = format!("data {id}").into_bytes();
        let timestamp = 0u64;
        let author = Did::new("key", "tester");
        let sig = None;
        let cid = compute_merkle_cid(0x71, &data, &[], timestamp, &author, &sig, &None);
        DagBlock {
            cid,
            data,
            links: vec![],
            timestamp,
            author_did: author,
            signature: sig,
            scope: None,
        }
    }

    fn run_suite<S: StorageService<DagBlock>>(store: &mut S) {
        let b1 = create_block("b1");
        let b2 = create_block("b2");
        assert!(store.put(&b1).is_ok());
        assert!(store.contains(&b1.cid).unwrap());
        assert!(!store.contains(&b2.cid).unwrap());
        match store.get(&b1.cid) {
            Ok(Some(block)) => assert_eq!(block.cid, b1.cid),
            _ => panic!("failed get b1"),
        }
        assert!(store.get(&b2.cid).unwrap().is_none());
        let mod_data = b"mod".to_vec();
        let ts = 1u64;
        let author = Did::new("key", "tester");
        let sig = None;
        let cid = compute_merkle_cid(0x71, &mod_data, &[], ts, &author, &sig, &None);
        let mod_block = DagBlock {
            cid,
            data: mod_data,
            links: vec![],
            timestamp: ts,
            author_did: author,
            signature: sig,
            scope: None,
        };
        assert!(store.put(&mod_block).is_ok());
        assert_eq!(store.get(&b1.cid).unwrap().unwrap().data, b"mod".to_vec());
        assert!(store.delete(&b1.cid).is_ok());
        assert!(!store.contains(&b1.cid).unwrap());
        assert!(store.delete(&b2.cid).is_ok());
    }

    #[test]
    fn sled_round_trip() {
        let dir = tempdir().unwrap();
        let mut store = SledDagStore::new(dir.path().to_path_buf()).unwrap();
        run_suite(&mut store);
        let persist = create_block("persist");
        store.put(&persist).unwrap();
        drop(store);
        let store2 = SledDagStore::new(dir.path().to_path_buf()).unwrap();
        assert!(store2.get(&persist.cid).unwrap().is_some());
    }
}
