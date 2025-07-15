use icn_common::Did;
use icn_mesh::aid::{match_aid_requests, AidJobTemplate, AidRequest};
use icn_mesh::{JobKind, JobSpec};

#[test]
fn basic_matching() {
    let request = AidRequest {
        id: "req1".into(),
        requester: Did::default(),
        tags: vec!["medical".into()],
    };

    let template = AidJobTemplate {
        tags: vec!["medical".into(), "supply".into()],
        job: JobSpec {
            kind: JobKind::Echo {
                payload: "hi".into(),
            },
            ..Default::default()
        },
    };

    let requests = [request.clone()];
    let templates = [template.clone()];
    let matches = match_aid_requests(&requests, &templates);
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].0.id, request.id);
    assert_eq!(matches[0].1.tags[0], template.tags[0]);
}
