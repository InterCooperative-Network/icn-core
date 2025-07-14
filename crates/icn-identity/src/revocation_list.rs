use dashmap::DashSet;
use icn_common::Cid;
use std::sync::Arc;

/// In-memory list of revoked credential identifiers.
#[derive(Clone, Default)]
pub struct RevocationList {
    revoked: Arc<DashSet<Cid>>,
}

impl RevocationList {
    /// Create an empty revocation list.
    pub fn new() -> Self {
        Self {
            revoked: Arc::new(DashSet::new()),
        }
    }

    /// Add a credential CID to the revocation list.
    pub fn add(&self, cid: Cid) {
        self.revoked.insert(cid);
    }

    /// Check if a credential has been revoked.
    pub fn contains(&self, cid: &Cid) -> bool {
        self.revoked.contains(cid)
    }

    /// Return all revoked credential CIDs.
    pub fn all(&self) -> Vec<Cid> {
        self.revoked.iter().map(|c| c.clone()).collect()
    }
}
