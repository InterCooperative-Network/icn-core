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

#[test]
fn canonical_root_prefers_highest_height() {
    let a = icn_common::Cid::new_v1_dummy(0x71, 0x12, b"A");
    let b = icn_common::Cid::new_v1_dummy(0x71, 0x12, b"B");
    let chosen = icn_dag::choose_canonical_root(vec![(a.clone(), 1), (b.clone(), 2)]).unwrap();
    assert_eq!(chosen, b);
}

#[test]
fn canonical_root_tiebreaks_lexicographically() {
    let a = icn_common::Cid::new_v1_dummy(0x71, 0x12, b"A");
    let b = icn_common::Cid::new_v1_dummy(0x71, 0x12, b"B");
    let chosen = icn_dag::choose_canonical_root(vec![(b.clone(), 1), (a.clone(), 1)]).unwrap();
    if a.to_string() < b.to_string() {
        assert_eq!(chosen, a);
    } else {
        assert_eq!(chosen, b);
    }
}
