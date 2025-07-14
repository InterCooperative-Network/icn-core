use crate::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutualAidResource {
    pub id: String,
    pub description: String,
    pub quantity: u64,
    pub provider: Did,
    pub location: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MutualAidRegistry {
    pub resources: HashMap<String, MutualAidResource>,
}

impl MutualAidRegistry {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    pub fn register(&mut self, resource: MutualAidResource) {
        self.resources.insert(resource.id.clone(), resource);
    }

    pub fn get(&self, id: &str) -> Option<&MutualAidResource> {
        self.resources.get(id)
    }
}
