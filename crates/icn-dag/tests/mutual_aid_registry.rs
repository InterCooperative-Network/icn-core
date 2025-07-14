use icn_common::{Did, NodeScope};
use icn_dag::mutual_aid::{list_resources, register_resource};
use icn_dag::StorageService;
use icn_runtime::context::StubDagStore;
use std::str::FromStr;

#[test]
fn register_and_list() {
    let mut store = StubDagStore::new();
    let provider = Did::from_str("did:example:prov").unwrap();
    let cid = register_resource(
        &mut store,
        provider.clone(),
        "water".into(),
        "bottled".into(),
        10,
        Some(NodeScope("scope".into())),
    )
    .unwrap();
    let res = list_resources(&store).unwrap();
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].0, cid);
    assert_eq!(res[0].1.provider, provider);
}
