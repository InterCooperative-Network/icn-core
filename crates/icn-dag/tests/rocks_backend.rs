#[cfg(feature = "persist-rocksdb")]
mod tests {
    use icn_common::{Cid, DagBlock};
    use icn_dag::rocksdb_store::RocksDagStore;
    use icn_dag::StorageService;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn create_block(id: &str) -> DagBlock {
        let data = format!("data {id}").into_bytes();
        let cid = Cid::new_v1_sha256(0x71, id.as_bytes());
        DagBlock {
            cid,
            data,
            links: vec![],
        }
    }

    fn run_suite<S: StorageService<DagBlock>>(store: &mut S) {
        let b1 = create_block("b1");
        let b2 = create_block("b2");
        assert!(store.put(&b1).is_ok());
        assert!(store.contains(&b1.cid).unwrap());
        assert!(!store.contains(&b2.cid).unwrap());
        assert_eq!(store.get(&b1.cid).unwrap().unwrap().cid, b1.cid);
        assert!(store.get(&b2.cid).unwrap().is_none());
        let mod_block = DagBlock {
            cid: b1.cid.clone(),
            data: b"mod".to_vec(),
            links: vec![],
        };
        assert!(store.put(&mod_block).is_ok());
        assert_eq!(store.get(&b1.cid).unwrap().unwrap().data, b"mod".to_vec());
        assert!(store.delete(&b1.cid).is_ok());
        assert!(!store.contains(&b1.cid).unwrap());
        assert!(store.delete(&b2.cid).is_ok());
    }

    #[test]
    fn rocksdb_round_trip() {
        let dir = tempdir().unwrap();
        let path: PathBuf = dir.path().join("rocks");
        let mut store = RocksDagStore::new(path.clone()).unwrap();
        run_suite(&mut store);
        let persist = create_block("persist");
        store.put(&persist).unwrap();
        drop(store);
        let store2 = RocksDagStore::new(path).unwrap();
        assert!(store2.get(&persist.cid).unwrap().is_some());
    }
}
