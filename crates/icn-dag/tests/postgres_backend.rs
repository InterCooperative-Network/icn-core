#[cfg(feature = "persist-postgres")]
mod tests {
    use icn_common::{compute_merkle_cid, DagBlock, Did};
    use icn_dag::postgres_store::PostgresDagStore;
    use icn_dag::AsyncStorageService;

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

    async fn run_suite<S: AsyncStorageService<DagBlock>>(store: &mut S) {
        let b1 = create_block("b1");
        let b2 = create_block("b2");
        store.put(&b1).await.unwrap();
        assert!(store.contains(&b1.cid).await.unwrap());
        assert!(!store.contains(&b2.cid).await.unwrap());
        assert_eq!(store.get(&b1.cid).await.unwrap().unwrap().cid, b1.cid);
        assert!(store.get(&b2.cid).await.unwrap().is_none());
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
        store.put(&mod_block).await.unwrap();
        assert_eq!(
            store.get(&b1.cid).await.unwrap().unwrap().data,
            b"mod".to_vec()
        );
        store.delete(&b1.cid).await.unwrap();
        assert!(!store.contains(&b1.cid).await.unwrap());
        store.delete(&b2.cid).await.unwrap();
    }

    #[tokio::test]
    async fn postgres_round_trip() {
        // Requires a running Postgres instance accessible at $ICN_TEST_PG_URL or default
        let url = std::env::var("ICN_TEST_PG_URL")
            .unwrap_or_else(|_| "postgres://postgres@localhost/icn_test".to_string());
        let mut store = match PostgresDagStore::new(&url).await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Postgres unavailable: {e:?}");
                return;
            }
        };
        run_suite(&mut store).await;
        let persist = create_block("persist");
        store.put(&persist).await.unwrap();
        drop(store);
        let store2 = match PostgresDagStore::new(&url).await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Postgres unavailable on reopen: {e:?}");
                return;
            }
        };
        assert!(store2.get(&persist.cid).await.unwrap().is_some());
    }
}
