use crate::{JobSpec, Resources};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AidRequest {
    pub id: String,
    pub description: String,
    pub required_resources: Resources,
    pub filled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobTemplate {
    pub name: String,
    pub spec: JobSpec,
}

pub fn match_template<'a>(
    req: &AidRequest,
    templates: &'a [JobTemplate],
) -> Option<&'a JobTemplate> {
    templates.iter().find(|t| {
        t.spec.required_resources.cpu_cores <= req.required_resources.cpu_cores
            && t.spec.required_resources.memory_mb <= req.required_resources.memory_mb
    })
}
