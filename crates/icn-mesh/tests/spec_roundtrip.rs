use icn_common::Cid;
use icn_mesh::{Job, JobId, JobKind, JobSpec};

#[test]
fn spec_bytes_roundtrip() {
    let spec = JobSpec {
        kind: JobKind::Echo {
            payload: "hi".into(),
        },
        ..Default::default()
    };
    let bytes = bincode::serialize(&spec).unwrap();
    let job = Job {
        id: JobId(Cid::new_v1_sha256(0x55, b"j")),
        manifest_cid: Cid::new_v1_sha256(0x55, b"m"),
        spec_bytes: bytes.clone(),
        spec_json: None,
        submitter_did: icn_common::Did::new("key", "test"),
        cost_mana: 0,
        submitted_at: 0,
        status: icn_mesh::JobLifecycleStatus::Submitted,
        resource_requirements: Default::default(),
    };
    let decoded = job.decode_spec().unwrap();
    assert_eq!(decoded, spec);
}
