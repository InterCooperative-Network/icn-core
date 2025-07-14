use icn_mesh::{
    mutual_aid::{match_template, AidRequest, JobTemplate},
    JobKind, JobSpec, Resources,
};

#[test]
fn template_matches() {
    let req = AidRequest {
        id: "1".into(),
        description: "basic".into(),
        required_resources: Resources {
            cpu_cores: 2,
            memory_mb: 1024,
        },
        filled: false,
    };
    let template = JobTemplate {
        name: "echo".into(),
        spec: JobSpec {
            kind: JobKind::Echo {
                payload: "hi".into(),
            },
            inputs: vec![],
            outputs: vec![],
            required_resources: Resources {
                cpu_cores: 1,
                memory_mb: 512,
            },
        },
    };
    let result = match_template(&req, &[template.clone()]);
    assert!(result.is_some());
    assert_eq!(result.unwrap().name, "echo");
}
