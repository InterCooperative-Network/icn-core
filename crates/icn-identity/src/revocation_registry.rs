use std::sync::Arc;
use dashmap::DashSet;
use icn_common::Cid;

/// Registry of issued credentials used to check revocation status.
pub trait RevocationRegistry: Send + Sync {
    /// Record a newly issued credential by CID.
    fn record(&self, cid: Cid);
    /// Mark a credential as revoked. Returns `true` if the credential was known.
    fn revoke(&self, cid: &Cid) -> bool;
    /// Check if a credential has been revoked.
    fn is_revoked(&self, cid: &Cid) -> bool;
}

/// Simple in-memory implementation of [`RevocationRegistry`].
#[derive(Clone, Default)]
pub struct InMemoryRevocationRegistry {
    issued: Arc<DashSet<Cid>>,
    revoked: Arc<DashSet<Cid>>,
}

impl InMemoryRevocationRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            issued: Arc::new(DashSet::new()),
            revoked: Arc::new(DashSet::new()),
        }
    }
}

impl RevocationRegistry for InMemoryRevocationRegistry {
    fn record(&self, cid: Cid) {
        self.issued.insert(cid);
    }

    fn revoke(&self, cid: &Cid) -> bool {
        if self.issued.contains(cid) {
            self.revoked.insert(cid.clone());
            true
        } else {
            false
        }
    }

    fn is_revoked(&self, cid: &Cid) -> bool {
        self.revoked.contains(cid)
    }
}
