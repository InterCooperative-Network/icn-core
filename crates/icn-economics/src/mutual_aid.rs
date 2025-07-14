use crate::{ResourceLedger, ResourceRepositoryAdapter};
use icn_common::{CommonError, Did, NodeScope};
use serde::{Deserialize, Serialize};

/// Non-transferable mutual aid token event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutualAidToken {
    /// Class identifier for this aid token type.
    pub class_id: String,
    /// Amount of aid credits represented.
    pub amount: u64,
    /// Issuer of the aid token.
    pub issuer: Did,
    /// Recipient who may redeem the aid.
    pub recipient: Did,
    /// Optional node scope for localized aid distribution.
    pub scope: Option<NodeScope>,
}

/// Manager for non-transferable mutual aid tokens.
pub struct MutualAidManager<L: ResourceLedger> {
    repo: ResourceRepositoryAdapter<L>,
}

impl<L: ResourceLedger> MutualAidManager<L> {
    /// Create a new manager around the provided ledger adapter.
    pub fn new(repo: ResourceRepositoryAdapter<L>) -> Self {
        Self { repo }
    }

    /// Issue mutual aid tokens to a recipient. Tokens cannot be transferred.
    pub fn issue(
        &self,
        issuer: &Did,
        class_id: &str,
        amount: u64,
        recipient: &Did,
        scope: Option<NodeScope>,
    ) -> Result<(), CommonError> {
        self.repo.mint(issuer, class_id, amount, recipient, scope)
    }

    /// Consume mutual aid tokens from the owner's balance.
    pub fn consume(
        &self,
        issuer: &Did,
        class_id: &str,
        amount: u64,
        owner: &Did,
        scope: Option<NodeScope>,
    ) -> Result<(), CommonError> {
        self.repo.burn(issuer, class_id, amount, owner, scope)
    }
}
