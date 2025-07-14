use crate::{mint_tokens_with_reputation, ManaLedger, ResourceLedger, ResourceRepositoryAdapter};
use icn_common::{CommonError, Did, NodeScope};
use icn_reputation::ReputationStore;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Bounty {
    pub id: u64,
    pub description: String,
    pub amount: u64,
    pub class_id: String,
    pub issuer: Did,
    pub claimant: Option<Did>,
    pub paid: bool,
}

pub struct BountyManager<L: ManaLedger, R: ResourceLedger> {
    pub bounties: HashMap<u64, Bounty>,
    pub repo: ResourceRepositoryAdapter<R>,
    pub mana: L,
    next_id: u64,
}

impl<L: ManaLedger, R: ResourceLedger> BountyManager<L, R> {
    pub fn new(mana: L, repo: ResourceRepositoryAdapter<R>) -> Self {
        Self {
            bounties: HashMap::new(),
            repo,
            mana,
            next_id: 0,
        }
    }

    pub fn create_bounty(
        &mut self,
        issuer: Did,
        class_id: &str,
        amount: u64,
        description: &str,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.bounties.insert(
            id,
            Bounty {
                id,
                description: description.into(),
                amount,
                class_id: class_id.into(),
                issuer,
                claimant: None,
                paid: false,
            },
        );
        id
    }

    pub fn claim_bounty(&mut self, id: u64, claimant: Did) -> Result<(), CommonError> {
        let b = self
            .bounties
            .get_mut(&id)
            .ok_or_else(|| CommonError::ResourceNotFound("bounty".into()))?;
        if b.paid {
            return Err(CommonError::PolicyDenied("already paid".into()));
        }
        b.claimant = Some(claimant);
        Ok(())
    }

    pub fn payout_bounty(
        &mut self,
        id: u64,
        rep: &dyn ReputationStore,
        scope: Option<NodeScope>,
    ) -> Result<(), CommonError> {
        let b = self
            .bounties
            .get_mut(&id)
            .ok_or_else(|| CommonError::ResourceNotFound("bounty".into()))?;
        let claimant = b
            .claimant
            .clone()
            .ok_or_else(|| CommonError::InvalidInputError("no claimant".into()))?;
        if b.paid {
            return Err(CommonError::PolicyDenied("already paid".into()));
        }
        mint_tokens_with_reputation(
            &self.repo,
            &self.mana,
            rep,
            &b.issuer,
            &b.class_id,
            b.amount,
            &claimant,
            scope,
        )?;
        b.paid = true;
        Ok(())
    }
}
