use crate::{
    burn_tokens, mint_tokens, ManaLedger, NodeScope, ResourceLedger, ResourceRepositoryAdapter,
};
use icn_common::{CommonError, Did};

/// Resource class identifier for mutual aid tokens.
pub const MUTUAL_AID_CLASS: &str = "mutual_aid";

/// Grant non-transferable mutual aid tokens to a recipient.
///
/// These tokens represent community support and cannot be transferred between
/// accounts. They may be minted by authorized issuers and burned when
/// consumed. The caller is expected to enforce any relevant policies.
pub fn grant_mutual_aid<L: ResourceLedger, M: ManaLedger>(
    repo: &ResourceRepositoryAdapter<L>,
    mana_ledger: &M,
    issuer: &Did,
    recipient: &Did,
    amount: u64,
    scope: Option<NodeScope>,
) -> Result<(), CommonError> {
    mint_tokens(
        repo,
        mana_ledger,
        issuer,
        MUTUAL_AID_CLASS,
        amount,
        recipient,
        scope,
    )
}

/// Consume mutual aid tokens from the owner's balance.
pub fn use_mutual_aid<L: ResourceLedger, M: ManaLedger>(
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
        MUTUAL_AID_CLASS,
        amount,
        owner,
        scope,
    )
}
