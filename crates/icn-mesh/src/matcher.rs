use crate::Resources;
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
    pub id: String,
    pub description: String,
    pub resources: Resources,
}

pub fn match_unfilled_requests<'a>(
    requests: &'a [AidRequest],
    templates: &'a [JobTemplate],
) -> Vec<(&'a AidRequest, &'a JobTemplate)> {
    let mut matches = Vec::new();
    for req in requests.iter().filter(|r| !r.filled) {
        if let Some(tmpl) = templates.iter().find(|t| {
            t.resources.cpu_cores >= req.required_resources.cpu_cores
                && t.resources.memory_mb >= req.required_resources.memory_mb
        }) {
            matches.push((req, tmpl));
        }
    }
    matches
}
