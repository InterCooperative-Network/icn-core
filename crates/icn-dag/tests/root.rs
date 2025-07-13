use icn_common::{compute_merkle_cid, DagBlock, DagLink, Did};
use icn_dag::compute_dag_root;

fn make_block(id: &str, links: Vec<DagLink>) -> DagBlock {
    let data = id.as_bytes().to_vec();
    let ts = 0u64;
    let author = Did::new("key", "tester");
    let sig = None;
    let cid = compute_merkle_cid(0x71, &data, &links, ts, &author, &sig, &None);
    DagBlock {
        cid,
        data,
        links,
        timestamp: ts,
        author_did: author,
        signature: sig,
        scope: None,
    }
}

#[test]
fn root_deterministic() {
    let child = make_block("child", vec![]);
    let link = DagLink {
        cid: child.cid.clone(),
        name: "child".into(),
        size: 0,
    };
    let parent = make_block("parent", vec![link]);
    let root1 = compute_dag_root(&[parent.cid.clone()]);
    let root2 = compute_dag_root(&[parent.cid.clone()]);
    assert_eq!(root1, root2);
}
