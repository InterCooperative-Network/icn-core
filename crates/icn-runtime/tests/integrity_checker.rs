use icn_runtime::context::RuntimeContext;
use icn_common::{compute_merkle_cid, DagBlock, Did};

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

#[tokio::test]
async fn integrity_checker_detects_corruption() {
    let ctx = RuntimeContext::new_with_stubs_and_mana("did:icn:test:intcheck", 0).unwrap();
    let block = create_block("good");
    {
        let mut store = ctx.dag_store.lock().await;
        #[cfg(feature = "async")]
        store.put(&block).await.unwrap();
        #[cfg(not(feature = "async"))]
        store.put(&block).unwrap();
    }
    assert!(ctx.integrity_check_once().await.is_ok());
    {
        let mut store = ctx.dag_store.lock().await;
        let stub = store
            .as_any_mut()
            .downcast_mut::<icn_runtime::context::StubDagStore>()
            .unwrap();
        if let Some(b) = stub.get_mut(&block.cid) {
            b.data[0] ^= 0xFF;
        }
    }
    let result = ctx.integrity_check_once().await;
    assert!(result.is_err());
}
