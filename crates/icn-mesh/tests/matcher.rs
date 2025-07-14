use icn_mesh::{match_unfilled_requests, AidRequest, JobTemplate, Resources};

#[test]
fn basic_matching_works() {
    let requests = vec![AidRequest {
        id: "r1".into(),
        description: "need help".into(),
        required_resources: Resources { cpu_cores: 1, memory_mb: 1 },
        filled: false,
    }];
    let templates = vec![JobTemplate {
        id: "t1".into(),
        description: "generic".into(),
        resources: Resources { cpu_cores: 2, memory_mb: 2 },
    }];

    let m = match_unfilled_requests(&requests, &templates);
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].0.id, "r1");
    assert_eq!(m[0].1.id, "t1");
}
