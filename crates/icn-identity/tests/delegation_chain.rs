use icn_common::Did;
use icn_identity::{
    delegated_credential::{verify_delegation_chain, DelegatedCredential},
    did_key_from_verifying_key, generate_ed25519_keypair, KeyDidResolver,
};
use std::str::FromStr;

#[test]
fn delegation_chain_a_b_c() {
    let (sk_a, vk_a) = generate_ed25519_keypair();
    let did_a = Did::from_str(&did_key_from_verifying_key(&vk_a)).unwrap();

    let (sk_b, vk_b) = generate_ed25519_keypair();
    let did_b = Did::from_str(&did_key_from_verifying_key(&vk_b)).unwrap();

    let (sk_c, vk_c) = generate_ed25519_keypair();
    let did_c = Did::from_str(&did_key_from_verifying_key(&vk_c)).unwrap();

    let d1 = DelegatedCredential::new(did_a.clone(), did_b.clone(), &sk_a);
    let d2 = DelegatedCredential::new(did_b.clone(), did_c.clone(), &sk_b);

    let resolver = KeyDidResolver;
    let result = verify_delegation_chain(&did_a, &[d1, d2], &resolver).unwrap();
    assert_eq!(result, did_c);
}
