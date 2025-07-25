// crates/icn-governance/src/advanced_democracy.rs
//! Advanced democracy primitives for sophisticated governance
//!
//! This module implements advanced democratic mechanisms including:
//! - Liquid democracy with delegation chains
//! - Quadratic voting with voice credits
//! - Multi-stage proposals with automated workflows
//! - Weighted voting systems

use crate::{GovernanceModule, Proposal, ProposalId, ProposalStatus, ProposalType, Vote, VoteOption};
use icn_common::{CommonError, Did};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Liquid democracy implementation with delegation support
#[derive(Debug, Clone)]
pub struct LiquidDemocracy {
    delegations: HashMap<Did, Did>,          // delegator -> delegate
    delegation_chains: HashMap<Did, Vec<Did>>, // delegate -> full chain
    max_delegation_depth: usize,
}

impl LiquidDemocracy {
    pub fn new(max_delegation_depth: usize) -> Self {
        Self {
            delegations: HashMap::new(),
            delegation_chains: HashMap::new(),
            max_delegation_depth,
        }
    }

    /// Delegate voting power from one member to another
    pub fn delegate_vote(&mut self, delegator: Did, delegate: Did) -> Result<(), CommonError> {
        if delegator == delegate {
            return Err(CommonError::InvalidInputError(
                "Cannot delegate to self".to_string(),
            ));
        }

        // Check for cycles and depth limits
        if self.would_create_cycle(&delegator, &delegate)? {
            return Err(CommonError::InvalidInputError(
                "Delegation would create a cycle".to_string(),
            ));
        }

        let chain_length = self.get_delegation_chain_length(&delegate);
        if chain_length >= self.max_delegation_depth {
            return Err(CommonError::InvalidInputError(format!(
                "Delegation chain would exceed maximum depth of {}",
                self.max_delegation_depth
            )));
        }

        // Set the delegation
        self.delegations.insert(delegator.clone(), delegate.clone());
        self.update_delegation_chains(&delegator);

        Ok(())
    }

    /// Remove a delegation
    pub fn revoke_delegation(&mut self, delegator: Did) {
        self.delegations.remove(&delegator);
        self.update_delegation_chains(&delegator);
    }

    /// Get the final delegate for a voter (following the chain)
    pub fn resolve_delegation(&self, voter: &Did) -> Did {
        let mut current = voter.clone();
        let mut visited = HashSet::new();

        for _ in 0..self.max_delegation_depth {
            if visited.contains(&current) {
                break; // Cycle protection
            }
            visited.insert(current.clone());

            if let Some(delegate) = self.delegations.get(&current) {
                current = delegate.clone();
            } else {
                break;
            }
        }

        current
    }

    /// Check if a delegation would create a cycle
    fn would_create_cycle(&self, delegator: &Did, delegate: &Did) -> Result<bool, CommonError> {
        let final_delegate = self.resolve_delegation(delegate);
        Ok(final_delegate == *delegator)
    }

    /// Get the length of delegation chain for a delegate
    fn get_delegation_chain_length(&self, delegate: &Did) -> usize {
        let mut current = delegate.clone();
        let mut length = 0;

        for _ in 0..self.max_delegation_depth {
            if let Some(next_delegate) = self.delegations.get(&current) {
                current = next_delegate.clone();
                length += 1;
            } else {
                break;
            }
        }

        length
    }

    /// Update delegation chains cache
    fn update_delegation_chains(&mut self, _voter: &Did) {
        // Simplified implementation - in production this would maintain
        // efficient caching of delegation chains
        self.delegation_chains.clear();
        
        for (delegator, delegate) in &self.delegations {
            let chain = self.build_chain(delegator);
            self.delegation_chains.insert(delegate.clone(), chain);
        }
    }

    /// Build delegation chain for a voter
    fn build_chain(&self, voter: &Did) -> Vec<Did> {
        let mut chain = Vec::new();
        let mut current = voter.clone();

        for _ in 0..self.max_delegation_depth {
            chain.push(current.clone());
            if let Some(delegate) = self.delegations.get(&current) {
                current = delegate.clone();
            } else {
                break;
            }
        }

        chain
    }

    /// Get all delegators for a delegate
    pub fn get_delegators(&self, delegate: &Did) -> Vec<Did> {
        self.delegations
            .iter()
            .filter_map(|(delegator, _del)| {
                if self.resolve_delegation(delegator) == *delegate {
                    Some(delegator.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get voting power for a delegate (including delegated votes)
    pub fn get_voting_power(&self, delegate: &Did) -> usize {
        1 + self.get_delegators(delegate).len()
    }
}

/// Quadratic voting system with voice credits
#[derive(Debug, Clone)]
pub struct QuadraticVoting {
    voice_credits: HashMap<Did, u64>,
    spent_credits: HashMap<(Did, ProposalId), u64>, // (voter, proposal) -> credits spent
    initial_credits: u64,
}

impl QuadraticVoting {
    pub fn new(initial_credits: u64) -> Self {
        Self {
            voice_credits: HashMap::new(),
            spent_credits: HashMap::new(),
            initial_credits,
        }
    }

    /// Initialize voice credits for a voter
    pub fn initialize_voter(&mut self, voter: Did) {
        self.voice_credits.insert(voter, self.initial_credits);
    }

    /// Get available voice credits for a voter
    pub fn get_voice_credits(&self, voter: &Did) -> u64 {
        self.voice_credits.get(voter).copied().unwrap_or(0)
    }

    /// Calculate quadratic cost for vote strength
    pub fn calculate_cost(vote_strength: u32) -> u64 {
        (vote_strength as u64).pow(2)
    }

    /// Cast a quadratic vote
    pub fn cast_quadratic_vote(
        &mut self,
        voter: Did,
        proposal_id: ProposalId,
        vote_strength: u32,
        vote_option: VoteOption,
    ) -> Result<QuadraticVote, CommonError> {
        let cost = Self::calculate_cost(vote_strength);
        let available_credits = self.get_voice_credits(&voter);

        if cost > available_credits {
            return Err(CommonError::PolicyDenied(format!(
                "Insufficient voice credits: need {}, have {}",
                cost, available_credits
            )));
        }

        // Spend credits
        *self.voice_credits.get_mut(&voter).unwrap() -= cost;
        self.spent_credits.insert((voter.clone(), proposal_id.clone()), cost);

        Ok(QuadraticVote {
            voter,
            proposal_id,
            vote_option,
            vote_strength,
            credits_spent: cost,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        })
    }

    /// Refund credits if vote is changed or proposal cancelled
    pub fn refund_credits(&mut self, voter: &Did, proposal_id: &ProposalId) -> Result<(), CommonError> {
        if let Some(spent) = self.spent_credits.remove(&(voter.clone(), proposal_id.clone())) {
            *self.voice_credits.get_mut(voter).unwrap() += spent;
            Ok(())
        } else {
            Err(CommonError::ResourceNotFound(
                "No credits to refund".to_string(),
            ))
        }
    }

    /// Get total quadratic voting power for a proposal
    pub fn tally_quadratic_votes(&self, proposal_id: &ProposalId, votes: &[QuadraticVote]) -> QuadraticTally {
        let mut yes_power = 0u64;
        let mut no_power = 0u64;
        let mut abstain_power = 0u64;

        for vote in votes.iter().filter(|v| v.proposal_id == *proposal_id) {
            let power = vote.vote_strength as u64;
            match vote.vote_option {
                VoteOption::Yes => yes_power += power,
                VoteOption::No => no_power += power,
                VoteOption::Abstain => abstain_power += power,
            }
        }

        QuadraticTally {
            yes_power,
            no_power,
            abstain_power,
            total_power: yes_power + no_power + abstain_power,
        }
    }
}

/// Quadratic vote record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuadraticVote {
    pub voter: Did,
    pub proposal_id: ProposalId,
    pub vote_option: VoteOption,
    pub vote_strength: u32,
    pub credits_spent: u64,
    pub timestamp: u64,
}

/// Quadratic voting tally results
#[derive(Debug, Clone)]
pub struct QuadraticTally {
    pub yes_power: u64,
    pub no_power: u64,
    pub abstain_power: u64,
    pub total_power: u64,
}

/// Multi-stage proposal system
#[derive(Debug, Clone)]
pub struct MultiStageProposal {
    pub stages: Vec<ProposalStage>,
    pub current_stage: usize,
    pub stage_transitions: HashMap<usize, StageTransition>,
}

/// Proposal stage definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalStage {
    pub name: String,
    pub description: String,
    pub duration_secs: u64,
    pub required_actions: Vec<StageAction>,
    pub approval_threshold: f32,
    pub quorum: Option<usize>,
}

/// Actions required in a proposal stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StageAction {
    PublicComment,
    ExpertReview,
    ImpactAssessment,
    BudgetAnalysis,
    LegalReview,
    Voting,
    Implementation,
    Custom(String),
}

/// Stage transition conditions
#[derive(Debug, Clone)]
pub struct StageTransition {
    pub from_stage: usize,
    pub to_stage: usize,
    pub condition: TransitionCondition,
    pub automatic: bool,
}

/// Conditions for stage transitions
#[derive(Debug, Clone)]
pub enum TransitionCondition {
    TimeExpired,
    ThresholdMet,
    AllActionscomplete,
    ManualTrigger,
    Custom(String), // Just store description for custom logic
}

impl MultiStageProposal {
    pub fn new(stages: Vec<ProposalStage>) -> Self {
        Self {
            stages,
            current_stage: 0,
            stage_transitions: HashMap::new(),
        }
    }

    /// Get current stage
    pub fn current_stage(&self) -> Option<&ProposalStage> {
        self.stages.get(self.current_stage)
    }

    /// Check if proposal can advance to next stage
    pub fn can_advance(&self) -> bool {
        if let Some(stage) = self.current_stage() {
            // Simplified check - in production this would evaluate all conditions
            !stage.required_actions.is_empty()
        } else {
            false
        }
    }

    /// Advance to next stage
    pub fn advance_stage(&mut self) -> Result<(), CommonError> {
        if self.current_stage + 1 >= self.stages.len() {
            return Err(CommonError::InvalidInputError(
                "Already at final stage".to_string(),
            ));
        }

        if !self.can_advance() {
            return Err(CommonError::PolicyDenied(
                "Cannot advance stage - conditions not met".to_string(),
            ));
        }

        self.current_stage += 1;
        Ok(())
    }

    /// Check if proposal is complete
    pub fn is_complete(&self) -> bool {
        self.current_stage >= self.stages.len()
    }
}

/// Weighted voting system
#[derive(Debug, Clone)]
pub struct WeightedVoting {
    weights: HashMap<Did, f64>,
    weight_calculation: WeightCalculation,
}

/// Methods for calculating vote weights
#[derive(Debug, Clone)]
pub enum WeightCalculation {
    FixedWeights,
    StakeBasedWeights,
    ReputationBasedWeights,
    TimeBasedWeights,
    Custom(String),
}

impl WeightedVoting {
    pub fn new(weight_calculation: WeightCalculation) -> Self {
        Self {
            weights: HashMap::new(),
            weight_calculation,
        }
    }

    /// Set weight for a voter
    pub fn set_weight(&mut self, voter: Did, weight: f64) {
        self.weights.insert(voter, weight);
    }

    /// Get weight for a voter
    pub fn get_weight(&self, voter: &Did) -> f64 {
        self.weights.get(voter).copied().unwrap_or(1.0)
    }

    /// Calculate weighted vote totals
    pub fn tally_weighted_votes(&self, votes: &[Vote]) -> WeightedTally {
        let mut yes_weight = 0.0;
        let mut no_weight = 0.0;
        let mut abstain_weight = 0.0;

        for vote in votes {
            let weight = self.get_weight(&vote.voter);
            match vote.option {
                VoteOption::Yes => yes_weight += weight,
                VoteOption::No => no_weight += weight,
                VoteOption::Abstain => abstain_weight += weight,
            }
        }

        WeightedTally {
            yes_weight,
            no_weight,
            abstain_weight,
            total_weight: yes_weight + no_weight + abstain_weight,
        }
    }
}

/// Weighted voting tally results
#[derive(Debug, Clone)]
pub struct WeightedTally {
    pub yes_weight: f64,
    pub no_weight: f64,
    pub abstain_weight: f64,
    pub total_weight: f64,
}

/// Advanced governance module extension
pub struct AdvancedGovernanceModule {
    base_governance: GovernanceModule,
    liquid_democracy: LiquidDemocracy,
    quadratic_voting: QuadraticVoting,
    weighted_voting: WeightedVoting,
    multi_stage_proposals: HashMap<ProposalId, MultiStageProposal>,
    quadratic_votes: Vec<QuadraticVote>,
}

impl AdvancedGovernanceModule {
    pub fn new(base_governance: GovernanceModule) -> Self {
        Self {
            base_governance,
            liquid_democracy: LiquidDemocracy::new(5), // Max depth of 5
            quadratic_voting: QuadraticVoting::new(100), // 100 initial credits
            weighted_voting: WeightedVoting::new(WeightCalculation::FixedWeights),
            multi_stage_proposals: HashMap::new(),
            quadratic_votes: Vec::new(),
        }
    }

    /// Cast a liquid democracy vote (with delegation)
    pub fn cast_liquid_vote(
        &mut self,
        voter: Did,
        proposal_id: &ProposalId,
        option: VoteOption,
    ) -> Result<(), CommonError> {
        let final_voter = self.liquid_democracy.resolve_delegation(&voter);
        self.base_governance.cast_vote(final_voter, proposal_id, option)
    }

    /// Cast a quadratic vote
    pub fn cast_quadratic_vote(
        &mut self,
        voter: Did,
        proposal_id: ProposalId,
        vote_strength: u32,
        option: VoteOption,
    ) -> Result<(), CommonError> {
        let quadratic_vote = self.quadratic_voting.cast_quadratic_vote(
            voter.clone(),
            proposal_id.clone(),
            vote_strength,
            option,
        )?;

        self.quadratic_votes.push(quadratic_vote);
        Ok(())
    }

    /// Create a multi-stage proposal
    pub fn create_multi_stage_proposal(
        &mut self,
        proposal_id: ProposalId,
        stages: Vec<ProposalStage>,
    ) -> Result<(), CommonError> {
        let multi_stage = MultiStageProposal::new(stages);
        self.multi_stage_proposals.insert(proposal_id, multi_stage);
        Ok(())
    }

    /// Advance a multi-stage proposal to next stage
    pub fn advance_proposal_stage(&mut self, proposal_id: &ProposalId) -> Result<(), CommonError> {
        if let Some(multi_stage) = self.multi_stage_proposals.get_mut(proposal_id) {
            multi_stage.advance_stage()
        } else {
            Err(CommonError::ResourceNotFound(
                "Multi-stage proposal not found".to_string(),
            ))
        }
    }

    /// Get quadratic voting tally for a proposal
    pub fn get_quadratic_tally(&self, proposal_id: &ProposalId) -> QuadraticTally {
        self.quadratic_voting.tally_quadratic_votes(proposal_id, &self.quadratic_votes)
    }

    /// Delegate vote in liquid democracy
    pub fn delegate_vote(&mut self, delegator: Did, delegate: Did) -> Result<(), CommonError> {
        self.liquid_democracy.delegate_vote(delegator, delegate)
    }

    /// Get voting power for liquid democracy
    pub fn get_liquid_voting_power(&self, delegate: &Did) -> usize {
        self.liquid_democracy.get_voting_power(delegate)
    }

    /// Initialize quadratic voting credits
    pub fn initialize_quadratic_voter(&mut self, voter: Did) {
        self.quadratic_voting.initialize_voter(voter);
    }

    /// Set weighted voting weight
    pub fn set_voting_weight(&mut self, voter: Did, weight: f64) {
        self.weighted_voting.set_weight(voter, weight);
    }

    /// Get current proposal stage
    pub fn get_proposal_stage(&self, proposal_id: &ProposalId) -> Option<&ProposalStage> {
        self.multi_stage_proposals
            .get(proposal_id)
            .and_then(|ms| ms.current_stage())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_liquid_democracy_delegation() {
        let mut liquid = LiquidDemocracy::new(3);
        let alice = Did::default();
        let bob = Did::new("key", "bob");

        assert!(liquid.delegate_vote(alice.clone(), bob.clone()).is_ok());
        assert_eq!(liquid.resolve_delegation(&alice), bob);
    }

    #[test]
    fn test_liquid_democracy_cycle_detection() {
        let mut liquid = LiquidDemocracy::new(3);
        let alice = Did::default();
        let bob = Did::new("key", "bob");

        liquid.delegate_vote(alice.clone(), bob.clone()).unwrap();
        assert!(liquid.delegate_vote(bob, alice).is_err());
    }

    #[test]
    fn test_quadratic_voting() {
        let mut qv = QuadraticVoting::new(100);
        let alice = Did::default();
        let proposal = ProposalId("test".to_string());

        qv.initialize_voter(alice.clone());
        assert_eq!(qv.get_voice_credits(&alice), 100);

        let vote = qv.cast_quadratic_vote(alice.clone(), proposal, 5, VoteOption::Yes);
        assert!(vote.is_ok());
        assert_eq!(qv.get_voice_credits(&alice), 75); // 100 - 25 (5^2)
    }

    #[test]
    fn test_multi_stage_proposal() {
        let stages = vec![
            ProposalStage {
                name: "Discussion".to_string(),
                description: "Public discussion period".to_string(),
                duration_secs: 86400,
                required_actions: vec![StageAction::PublicComment],
                approval_threshold: 0.5,
                quorum: Some(5),
            },
            ProposalStage {
                name: "Voting".to_string(),
                description: "Voting period".to_string(),
                duration_secs: 86400,
                required_actions: vec![StageAction::Voting],
                approval_threshold: 0.6,
                quorum: Some(10),
            },
        ];

        let mut proposal = MultiStageProposal::new(stages);
        assert_eq!(proposal.current_stage, 0);
        assert_eq!(proposal.current_stage().unwrap().name, "Discussion");

        assert!(proposal.advance_stage().is_ok());
        assert_eq!(proposal.current_stage, 1);
        assert_eq!(proposal.current_stage().unwrap().name, "Voting");
    }
}