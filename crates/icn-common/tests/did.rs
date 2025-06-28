use icn_common::Did;
use std::str::FromStr;

#[test]
fn parse_web_did_url() {
    let url = "did:web:example.com:user:alice/profile#key-1";
    let did = Did::from_str(url).expect("parse failed");
    assert_eq!(did.method, "web");
    assert_eq!(did.id_string, "example.com:user:alice");
    assert_eq!(did.path.as_deref(), Some("/profile"));
    assert!(did.query.is_none());
    assert_eq!(did.fragment.as_deref(), Some("key-1"));
    assert_eq!(did.to_string(), url);
}

#[test]
fn parse_key_did_url_with_query() {
    let url = "did:key:z6MkjExample/service?foo=bar#frag";
    let did = Did::from_str(url).expect("parse failed");
    assert_eq!(did.method, "key");
    assert_eq!(did.id_string, "z6MkjExample");
    assert_eq!(did.path.as_deref(), Some("/service"));
    assert_eq!(did.query.as_deref(), Some("foo=bar"));
    assert_eq!(did.fragment.as_deref(), Some("frag"));
    assert_eq!(did.to_string(), url);
}
