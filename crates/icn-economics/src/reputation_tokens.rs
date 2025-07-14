use crate::{burn_tokens, ManaLedger, NodeScope, ResourceLedger, ResourceRepositoryAdapter};
use icn_common::{CommonError, Did};
use icn_reputation::ReputationStore;

pub const REPUTATION_CREDIT_CLASS: &str = "reputation_credit";

#[allow(clippy::too_many_arguments)]
pub fn grant_reputation_credit<L: ResourceLedger, M: ManaLedger, R: ReputationStore>(
    repo: &ResourceRepositoryAdapter<L>,
    mana_ledger: &M,
    rep_store: &R,
    issuer: &Did,
    recipient: &Did,
    amount: u64,
    scope: Option<NodeScope>,
) -> Result<(), CommonError> {
    let reputation = rep_store.get_reputation(issuer);
    let cost = super::price_by_reputation(amount, reputation);
    super::charge_mana(mana_ledger, issuer, cost)?;
    repo.mint(issuer, REPUTATION_CREDIT_CLASS, amount, recipient, scope)
}

#[allow(clippy::too_many_arguments)]
pub fn use_reputation_credit<L: ResourceLedger, M: ManaLedger, R: ReputationStore>(
    repo: &ResourceRepositoryAdapter<L>,
    mana_ledger: &M,
    _rep_store: &R,
    issuer: &Did,
    owner: &Did,
    amount: u64,
    scope: Option<NodeScope>,
) -> Result<(), CommonError> {
    burn_tokens(
        repo,
        mana_ledger,
        issuer,
        REPUTATION_CREDIT_CLASS,
        amount,
        owner,
        scope,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn mint_tokens_with_reputation<L: ResourceLedger, M: ManaLedger, R: ReputationStore>(
    repo: &ResourceRepositoryAdapter<L>,
    mana_ledger: &M,
    rep_store: &R,
    issuer: &Did,
    class_id: &str,
    amount: u64,
    recipient: &Did,
    scope: Option<NodeScope>,
) -> Result<(), CommonError> {
    let rep = rep_store.get_reputation(issuer);
    let cost = super::price_by_reputation(amount, rep);
    super::charge_mana(mana_ledger, issuer, cost)?;
    repo.mint(issuer, class_id, amount, recipient, scope)
}
