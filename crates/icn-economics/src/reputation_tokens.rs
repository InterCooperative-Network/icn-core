use crate::{burn_tokens, mint_tokens_with_reputation, ManaLedger, NodeScope, ResourceLedger, ResourceRepositoryAdapter};
use icn_common::{CommonError, Did};

/// Resource class identifier for reputation reward tokens.
pub const REPUTATION_CLASS: &str = "reputation_reward";

/// Grant non-transferable reputation tokens to a recipient.
pub fn grant_reputation_tokens<L: ResourceLedger, M: ManaLedger>(
    repo: &ResourceRepositoryAdapter<L>,
    mana_ledger: &M,
    reputation_store: &dyn icn_reputation::ReputationStore,
    issuer: &Did,
    recipient: &Did,
    amount: u64,
    scope: Option<NodeScope>,
) -> Result<(), CommonError> {
    mint_tokens_with_reputation(
        repo,
        mana_ledger,
        reputation_store,
        issuer,
        REPUTATION_CLASS,
        amount,
        recipient,
        scope,
    )
}

/// Consume reputation tokens from the owner's balance.
pub fn use_reputation_tokens<L: ResourceLedger, M: ManaLedger>(
    repo: &ResourceRepositoryAdapter<L>,
    mana_ledger: &M,
    issuer: &Did,
    owner: &Did,
    amount: u64,
    scope: Option<NodeScope>,
) -> Result<(), CommonError> {
    burn_tokens(
        repo,
        mana_ledger,
        issuer,
        REPUTATION_CLASS,
        amount,
        owner,
        scope,
    )
}
