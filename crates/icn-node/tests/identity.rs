use icn_node::config::NodeConfig;
use icn_node::node::load_or_generate_identity;
use tempfile::tempdir;

#[tokio::test]
async fn identity_persists_between_runs() {
    let dir = tempdir().unwrap();
    let did_path = dir.path().join("node.did");
    let key_path = dir.path().join("node.key");

    let mut cfg1 = NodeConfig {
        node_did_path: did_path.clone(),
        node_private_key_path: key_path.clone(),
        ..Default::default()
    };
    let (_sk1, _pk1, did1) = load_or_generate_identity(&mut cfg1).unwrap();

    let mut cfg2 = NodeConfig {
        node_did_path: did_path.clone(),
        node_private_key_path: key_path.clone(),
        ..Default::default()
    };
    let (_sk2, _pk2, did2) = load_or_generate_identity(&mut cfg2).unwrap();

    assert_eq!(did1, did2);
    assert_eq!(cfg2.node_did.as_deref(), Some(did1.as_str()));
}
