use icn_mesh::{match_job_template, JobKind, JobSpec, JobTemplate, Resources};

#[test]
fn template_match_basic() {
    let template = JobTemplate {
        name: "basic".into(),
        spec: JobSpec {
            kind: JobKind::Echo {
                payload: "hi".into(),
            },
            required_resources: Resources {
                cpu_cores: 2,
                memory_mb: 512,
            },
            ..Default::default()
        },
    };
    let spec = JobSpec {
        kind: JobKind::Echo {
            payload: "hi".into(),
        },
        required_resources: Resources {
            cpu_cores: 1,
            memory_mb: 256,
        },
        ..Default::default()
    };
    let templates = [template.clone()];
    let found = match_job_template(&spec, &templates);
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "basic");
}
