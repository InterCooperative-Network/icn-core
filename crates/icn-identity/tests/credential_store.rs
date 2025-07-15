use icn_common::{Cid, Did};
use icn_identity::credential::CredentialIssuer;
use icn_identity::credential_store::InMemoryCredentialStore;
use icn_identity::generate_ed25519_keypair;

#[test]
fn store_insert_get_revoke() {
    let (sk, _pk) = generate_ed25519_keypair();
    let issuer_did = Did::new("key", "issuer");
    let holder_did = Did::new("key", "holder");
    let mut claims = std::collections::HashMap::new();
    claims.insert("birth_year".to_string(), "2000".to_string());
    let issuer = CredentialIssuer::new(issuer_did.clone(), sk);
    let (cred, _) = issuer
        .issue(
            holder_did,
            claims,
            Some(Cid::new_v1_sha256(0x55, b"schema")),
            None,
            None,
            None,
        )
        .unwrap();
    let cid = Cid::new_v1_sha256(0x55, b"cred");
    let store = InMemoryCredentialStore::new();
    store.insert(cid.clone(), cred.clone());
    assert_eq!(store.get(&cid), Some(cred.clone()));
    let schemas = store.list_schemas();
    assert!(schemas.contains(&cred.schema.clone().unwrap()));
    assert!(store.revoke(&cid));
    assert!(store.get(&cid).is_none());
}
