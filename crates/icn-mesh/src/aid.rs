use crate::JobSpec;
use icn_common::Did;
use serde::{Deserialize, Serialize};

/// Request for community aid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AidRequest {
    /// Unique request identifier.
    pub id: String,
    /// DID of the requester.
    pub requester: Did,
    /// Tags describing needed resources.
    pub tags: Vec<String>,
}

/// Template describing a standard aid job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AidJobTemplate {
    /// Tags covered by this template.
    pub tags: Vec<String>,
    /// Job specification to execute when matched.
    pub job: JobSpec,
}

/// Match aid requests with job templates.
pub fn match_aid_requests<'a>(
    requests: &'a [AidRequest],
    templates: &'a [AidJobTemplate],
) -> Vec<(&'a AidRequest, &'a AidJobTemplate)> {
    let mut matches = Vec::new();
    for req in requests {
        for tmpl in templates {
            if tmpl.tags.iter().any(|t| req.tags.contains(t)) {
                matches.push((req, tmpl));
                break;
            }
        }
    }
    matches
}
