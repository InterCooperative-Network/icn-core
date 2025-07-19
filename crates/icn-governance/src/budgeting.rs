use icn_common::{CommonError, Did};
use icn_economics::ManaLedger;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Budget allocation proposal targeting a specific recipient.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BudgetProposal {
    /// Account receiving the allocated funds.
    pub recipient: Did,
    /// Amount of mana to allocate.
    pub amount: u64,
    /// Human readable description of the allocation purpose.
    pub purpose: String,
}

/// Apply a [`BudgetProposal`] by crediting the recipient in the provided ledger.
pub fn apply_budget_allocation<M: ManaLedger>(
    ledger: &M,
    proposal: &BudgetProposal,
) -> Result<(), CommonError> {
    ledger.credit(&proposal.recipient, proposal.amount)
}
