use dashmap::DashMap;
use std::sync::Arc;

use icn_common::Cid;

use crate::{credential::Credential, revocation_list::RevocationList};

/// Simple in-memory store for issued credentials.
#[derive(Clone, Default)]
pub struct InMemoryCredentialStore {
    creds: Arc<DashMap<Cid, Credential>>,
    revoked: RevocationList,
}

impl InMemoryCredentialStore {
    pub fn new() -> Self {
        Self {
            creds: Arc::new(DashMap::new()),
            revoked: RevocationList::new(),
        }
    }

    pub fn insert(&self, cid: Cid, cred: Credential) {
        self.creds.insert(cid, cred);
    }

    pub fn get(&self, cid: &Cid) -> Option<Credential> {
        self.creds.get(cid).map(|c| c.clone())
    }

    pub fn revoke(&self, cid: &Cid) -> bool {
        if self.creds.remove(cid).is_some() {
            self.revoked.add(cid.clone());
            true
        } else {
            false
        }
    }

    pub fn is_revoked(&self, cid: &Cid) -> bool {
        self.revoked.contains(cid)
    }

    pub fn revoked_list(&self) -> Vec<Cid> {
        self.revoked.all()
    }

    pub fn list_schemas(&self) -> Vec<Cid> {
        self.creds
            .iter()
            .filter_map(|e| e.value().schema.clone())
            .collect()
    }
}
