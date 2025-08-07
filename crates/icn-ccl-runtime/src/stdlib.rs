//! Standard library contracts for democratic governance and economic coordination

use icn_common::{Did, Cid, SystemTimeProvider, TimeProvider};
use icn_governance::{ProposalType, Vote, ProposalStatus as ProposalState};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BTreeMap, HashSet};
use std::time::SystemTime;

/// Contract event emitted during execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractEvent {
    pub contract: String, // Contract address
    pub event_type: String,
    pub data: Vec<u8>,
    pub block_height: u64,
    pub timestamp: u64,
}

/// Error type for contract operations
#[derive(Debug)]
pub enum CclRuntimeError {
    ExecutionError(String),
    PermissionDenied(String),
    InvalidContractState,
}

/// Capability enum for security
#[derive(Debug, Clone)]
pub enum Capability {
    CreateProposal,
    ModifyState,
}

/// Proposal identifier
pub type ProposalId = String;

/// Job identifier for marketplace
pub type JobId = String;

/// Credit line identifier
pub type CreditLineId = String;

/// Epoch for time-based operations
pub type Epoch = u64;

/// Democratic governance contract implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemocraticGovernanceContract {
    /// Active proposals
    proposals: HashMap<ProposalId, Proposal>,
    /// Vote records: (proposal_id, voter_did) -> vote
    votes: HashMap<(ProposalId, Did), Vote>,
    /// Member registry
    members: HashSet<Did>,
    /// Governance configuration
    config: GovernanceConfig,
    /// Contract state
    state: ContractState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: ProposalId,
    pub proposer: Did,
    pub title: String,
    pub description: String,
    pub actions: Vec<ProposalAction>,
    pub voting_ends: Epoch,
    pub state: ProposalState,
    pub votes_for: u32,
    pub votes_against: u32,
    pub votes_abstain: u32,
    pub created_at: Epoch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalAction {
    TransferFunds { to: Did, amount: u64 },
    UpdateParameter { key: String, value: String },
    AddMember { did: Did },
    RemoveMember { did: Did },
    CustomAction { data: Vec<u8> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceConfig {
    pub quorum: f64,              // e.g., 0.25 for 25%
    pub voting_period: Epoch,     // Duration in epochs
    pub proposal_threshold: f64,  // Minimum support to create proposal
    pub execution_delay: Epoch,   // Time-lock delay
    pub max_proposals_per_member: u32,
}

impl Default for GovernanceConfig {
    fn default() -> Self {
        Self {
            quorum: 0.25,
            voting_period: 7, // 7 epochs (days)
            proposal_threshold: 0.05, // 5% of members needed to propose
            execution_delay: 1, // 1 epoch delay
            max_proposals_per_member: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContractState {
    Active,
    Paused,
    Deprecated,
}

impl DemocraticGovernanceContract {
    /// Create a new governance contract
    pub fn new(initial_members: Vec<Did>) -> Self {
        Self {
            proposals: HashMap::new(),
            votes: HashMap::new(),
            members: initial_members.into_iter().collect(),
            config: GovernanceConfig::default(),
            state: ContractState::Active,
        }
    }
    
    /// Submit a new proposal
    pub fn propose(
        &mut self,
        proposer: Did,
        title: String,
        description: String,
        actions: Vec<ProposalAction>,
    ) -> Result<ProposalId, CclRuntimeError> {
        // Check if contract is active
        if !matches!(self.state, ContractState::Active) {
            return Err(CclRuntimeError::InvalidContractState);
        }
        
        // Check membership
        if !self.members.contains(&proposer) {
            return Err(CclRuntimeError::PermissionDenied(
                crate::security::Capability::CreateProposal
            ));
        }
        
        // Check proposal threshold
        let member_count = self.members.len() as f64;
        let required_support = (member_count * self.config.proposal_threshold).ceil() as u32;
        
        if required_support > 1 {
            // TODO: Check if proposer has enough support/endorsements
        }
        
        // Generate proposal ID
        let proposal_id = format!("prop_{}", uuid::Uuid::new_v4());
        
        // Create proposal
        let proposal = Proposal {
            id: proposal_id.clone(),
            proposer: proposer.clone(),
            title,
            description,
            actions,
            voting_ends: self.current_epoch() + self.config.voting_period,
            state: ProposalState::Active,
            votes_for: 0,
            votes_against: 0,
            votes_abstain: 0,
            created_at: self.current_epoch(),
        };
        
        self.proposals.insert(proposal_id.clone(), proposal);
        
        // Emit event
        self.emit_event("ProposalCreated", &proposal_id);
        
        Ok(proposal_id)
    }
    
    /// Cast a vote on a proposal
    pub fn vote(
        &mut self,
        voter: Did,
        proposal_id: ProposalId,
        vote: Vote,
    ) -> Result<(), CclRuntimeError> {
        // Check membership
        if !self.members.contains(&voter) {
            return Err(CclRuntimeError::PermissionDenied(
                crate::security::Capability::CreateProposal
            ));
        }
        
        // Get proposal
        let proposal = self.proposals.get_mut(&proposal_id)
            .ok_or_else(|| CclRuntimeError::ExecutionError("Proposal not found".to_string()))?;
        
        // Check if proposal is still active
        if proposal.state != ProposalState::Active {
            return Err(CclRuntimeError::ExecutionError("Proposal not active".to_string()));
        }
        
        // Check if voting period is still open
        if self.current_epoch() > proposal.voting_ends {
            return Err(CclRuntimeError::ExecutionError("Voting period ended".to_string()));
        }
        
        // Record vote (overwrite if already voted)
        let vote_key = (proposal_id.clone(), voter.clone());
        
        // Remove previous vote if exists
        if let Some(old_vote) = self.votes.get(&vote_key) {
            match old_vote {
                Vote::Yes => proposal.votes_for -= 1,
                Vote::No => proposal.votes_against -= 1,
                Vote::Abstain => proposal.votes_abstain -= 1,
            }
        }
        
        // Add new vote
        self.votes.insert(vote_key, vote.clone());
        match vote {
            Vote::Yes => proposal.votes_for += 1,
            Vote::No => proposal.votes_against += 1,
            Vote::Abstain => proposal.votes_abstain += 1,
        }
        
        // Emit event
        self.emit_event("VoteCast", &format!("{}:{:?}", proposal_id, vote));
        
        Ok(())
    }
    
    /// Finalize a proposal after voting period
    pub fn finalize_proposal(&mut self, proposal_id: ProposalId) -> Result<(), CclRuntimeError> {
        let proposal = self.proposals.get_mut(&proposal_id)
            .ok_or_else(|| CclRuntimeError::ExecutionError("Proposal not found".to_string()))?;
        
        // Check if voting period has ended
        if self.current_epoch() <= proposal.voting_ends {
            return Err(CclRuntimeError::ExecutionError("Voting period not ended".to_string()));
        }
        
        // Check if proposal is still active
        if proposal.state != ProposalState::Active {
            return Err(CclRuntimeError::ExecutionError("Proposal already finalized".to_string()));
        }
        
        let total_votes = proposal.votes_for + proposal.votes_against + proposal.votes_abstain;
        let member_count = self.members.len() as f64;
        let required_quorum = (member_count * self.config.quorum).ceil() as u32;
        
        // Check quorum
        if total_votes < required_quorum {
            proposal.state = ProposalState::Failed;
            self.emit_event("ProposalFailed", &format!("{}:quorum_not_met", proposal_id));
            return Ok(());
        }
        
        // Check if proposal passed (simple majority of votes cast)
        if proposal.votes_for > (total_votes / 2) {
            proposal.state = ProposalState::Passed;
            self.emit_event("ProposalPassed", &proposal_id);
            
            // Auto-execute safe actions
            if proposal.actions.iter().all(|a| self.is_safe_action(a)) {
                self.execute_proposal_actions(&proposal.actions)?;
                proposal.state = ProposalState::Executed;
                self.emit_event("ProposalExecuted", &proposal_id);
            }
        } else {
            proposal.state = ProposalState::Failed;
            self.emit_event("ProposalFailed", &format!("{}:rejected", proposal_id));
        }
        
        Ok(())
    }
    
    /// Execute proposal actions (governance admin only)
    pub fn execute_proposal(
        &mut self,
        proposal_id: ProposalId,
        executor: Did,
    ) -> Result<(), CclRuntimeError> {
        // TODO: Check execution permissions
        
        let proposal = self.proposals.get_mut(&proposal_id)
            .ok_or_else(|| CclRuntimeError::ExecutionError("Proposal not found".to_string()))?;
        
        if proposal.state != ProposalState::Passed {
            return Err(CclRuntimeError::ExecutionError("Proposal not passed".to_string()));
        }
        
        // Check execution delay
        let execution_time = proposal.voting_ends + self.config.execution_delay;
        if self.current_epoch() < execution_time {
            return Err(CclRuntimeError::ExecutionError("Execution delay not met".to_string()));
        }
        
        self.execute_proposal_actions(&proposal.actions)?;
        proposal.state = ProposalState::Executed;
        
        self.emit_event("ProposalExecuted", &proposal_id);
        
        Ok(())
    }
    
    /// Add a new member (via governance)
    pub fn add_member(&mut self, new_member: Did) -> Result<(), CclRuntimeError> {
        self.members.insert(new_member.clone());
        self.emit_event("MemberAdded", &new_member.to_string());
        Ok(())
    }
    
    /// Remove a member (via governance)
    pub fn remove_member(&mut self, member: Did) -> Result<(), CclRuntimeError> {
        self.members.remove(&member);
        self.emit_event("MemberRemoved", &member.to_string());
        Ok(())
    }
    
    /// Check if an action is safe for auto-execution
    fn is_safe_action(&self, action: &ProposalAction) -> bool {
        match action {
            ProposalAction::AddMember { .. } | ProposalAction::RemoveMember { .. } => true,
            ProposalAction::UpdateParameter { .. } => true,
            ProposalAction::TransferFunds { amount, .. } => *amount < 1000, // Small transfers only
            ProposalAction::CustomAction { .. } => false, // Never auto-execute custom actions
        }
    }
    
    /// Execute proposal actions
    fn execute_proposal_actions(&mut self, actions: &[ProposalAction]) -> Result<(), CclRuntimeError> {
        for action in actions {
            match action {
                ProposalAction::AddMember { did } => {
                    self.add_member(did.clone())?;
                }
                ProposalAction::RemoveMember { did } => {
                    self.remove_member(did.clone())?;
                }
                ProposalAction::UpdateParameter { key, value } => {
                    // TODO: Update contract parameters
                    self.emit_event("ParameterUpdated", &format!("{}:{}", key, value));
                }
                ProposalAction::TransferFunds { to, amount } => {
                    // TODO: Integrate with economics protocol for actual transfer
                    self.emit_event("FundsTransferred", &format!("{}:{}", to, amount));
                }
                ProposalAction::CustomAction { data } => {
                    // TODO: Handle custom actions
                    self.emit_event("CustomActionExecuted", &hex::encode(data));
                }
            }
        }
        Ok(())
    }
    
    /// Get current epoch (placeholder)
    fn current_epoch(&self) -> Epoch {
        SystemTimeProvider.unix_seconds() / 86400 // Simple day-based epochs
    }
    
    /// Emit contract event (placeholder)
    fn emit_event(&self, event_type: &str, data: &str) {
        log::info!("Contract event: {} - {}", event_type, data);
        // TODO: Integrate with actual event system
    }
    
    /// Get proposal by ID
    pub fn get_proposal(&self, proposal_id: &ProposalId) -> Option<&Proposal> {
        self.proposals.get(proposal_id)
    }
    
    /// List all active proposals
    pub fn list_active_proposals(&self) -> Vec<&Proposal> {
        self.proposals.values()
            .filter(|p| p.state == ProposalState::Active)
            .collect()
    }
    
    /// Get member count
    pub fn member_count(&self) -> usize {
        self.members.len()
    }
    
    /// Check if DID is a member
    pub fn is_member(&self, did: &Did) -> bool {
        self.members.contains(did)
    }
}

/// Mutual credit system contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutualCreditContract {
    /// Credit lines between members
    credit_lines: HashMap<CreditLineId, CreditLine>,
    /// Account balances (can be negative)
    balances: HashMap<Did, i64>,
    /// Trust relationships
    trust_matrix: HashMap<(Did, Did), f64>,
    /// System configuration
    config: MutualCreditConfig,
    /// Member registry
    members: HashSet<Did>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditLine {
    pub id: CreditLineId,
    pub creditor: Did,
    pub debtor: Did,
    pub limit: u64,
    pub used: u64,
    pub interest_rate: f64, // Always 0 for mutual credit
    pub created_at: Epoch,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutualCreditConfig {
    pub default_credit_limit: u64,
    pub max_credit_limit: u64,
    pub trust_decay_rate: f64,
    pub min_trust_score: f64,
}

impl Default for MutualCreditConfig {
    fn default() -> Self {
        Self {
            default_credit_limit: 1000,
            max_credit_limit: 10000,
            trust_decay_rate: 0.01, // 1% per epoch
            min_trust_score: 0.1,
        }
    }
}

impl MutualCreditContract {
    /// Create new mutual credit contract
    pub fn new(initial_members: Vec<Did>) -> Self {
        let mut balances = HashMap::new();
        for member in &initial_members {
            balances.insert(member.clone(), 0);
        }
        
        Self {
            credit_lines: HashMap::new(),
            balances,
            trust_matrix: HashMap::new(),
            config: MutualCreditConfig::default(),
            members: initial_members.into_iter().collect(),
        }
    }
    
    /// Extend credit to another member
    pub fn extend_credit(
        &mut self,
        creditor: Did,
        debtor: Did,
        limit: u64,
    ) -> Result<CreditLineId, CclRuntimeError> {
        // Check membership
        if !self.members.contains(&creditor) || !self.members.contains(&debtor) {
            return Err(CclRuntimeError::PermissionDenied(
                crate::security::Capability::ModifyState
            ));
        }
        
        // Check limits
        if limit > self.config.max_credit_limit {
            return Err(CclRuntimeError::ExecutionError(
                "Credit limit exceeds maximum".to_string()
            ));
        }
        
        // Create credit line
        let credit_line_id = format!("credit_{}_{}", creditor, debtor);
        let credit_line = CreditLine {
            id: credit_line_id.clone(),
            creditor: creditor.clone(),
            debtor: debtor.clone(),
            limit,
            used: 0,
            interest_rate: 0.0, // No interest in mutual credit
            created_at: self.current_epoch(),
            active: true,
        };
        
        self.credit_lines.insert(credit_line_id.clone(), credit_line);
        
        // Update trust matrix
        self.trust_matrix.insert((creditor.clone(), debtor.clone()), 1.0);
        
        self.emit_event("CreditExtended", &format!("{}:{}:{}", creditor, debtor, limit));
        
        Ok(credit_line_id)
    }
    
    /// Transfer credits through the network
    pub fn transfer(
        &mut self,
        from: Did,
        to: Did,
        amount: u64,
    ) -> Result<(), CclRuntimeError> {
        // Find credit path
        let path = self.find_credit_path(&from, &to, amount)?;
        
        // Execute transfers along the path
        for i in 0..path.len()-1 {
            let creditor = &path[i];
            let debtor = &path[i+1];
            
            // Find credit line
            let credit_line_id = format!("credit_{}_{}", creditor, debtor);
            let credit_line = self.credit_lines.get_mut(&credit_line_id)
                .ok_or_else(|| CclRuntimeError::ExecutionError("Credit line not found".to_string()))?;
            
            // Check credit limit
            if credit_line.used + amount > credit_line.limit {
                return Err(CclRuntimeError::ExecutionError("Insufficient credit".to_string()));
            }
            
            // Update credit line usage
            credit_line.used += amount;
            
            // Update balances
            *self.balances.entry(creditor.clone()).or_insert(0) += amount as i64;
            *self.balances.entry(debtor.clone()).or_insert(0) -= amount as i64;
        }
        
        self.emit_event("TransferCompleted", &format!("{}:{}:{}", from, to, amount));
        
        Ok(())
    }
    
    /// Find credit path between two members
    fn find_credit_path(
        &self,
        from: &Did,
        to: &Did,
        amount: u64,
    ) -> Result<Vec<Did>, CclRuntimeError> {
        // Simple direct path check first
        let direct_credit_id = format!("credit_{}_{}", from, to);
        if let Some(credit_line) = self.credit_lines.get(&direct_credit_id) {
            if credit_line.active && credit_line.used + amount <= credit_line.limit {
                return Ok(vec![from.clone(), to.clone()]);
            }
        }
        
        // TODO: Implement more sophisticated path finding using trust network
        // For now, return error if no direct path
        Err(CclRuntimeError::ExecutionError(
            "No credit path found".to_string()
        ))
    }
    
    /// Get balance for a member
    pub fn get_balance(&self, member: &Did) -> i64 {
        self.balances.get(member).copied().unwrap_or(0)
    }
    
    /// Get credit line information
    pub fn get_credit_line(&self, credit_line_id: &CreditLineId) -> Option<&CreditLine> {
        self.credit_lines.get(credit_line_id)
    }
    
    /// List active credit lines for a member
    pub fn list_credit_lines(&self, member: &Did) -> Vec<&CreditLine> {
        self.credit_lines.values()
            .filter(|cl| cl.active && (cl.creditor == *member || cl.debtor == *member))
            .collect()
    }
    
    fn current_epoch(&self) -> Epoch {
        icn_common::current_timestamp() / 86400
    }
    
    fn emit_event(&self, event_type: &str, data: &str) {
        log::info!("MutualCredit event: {} - {}", event_type, data);
    }
}

/// Job marketplace contract for coordinating work
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobMarketplaceContract {
    jobs: HashMap<JobId, Job>,
    bids: HashMap<JobId, Vec<Bid>>,
    executions: HashMap<JobId, JobExecution>,
    members: HashSet<Did>,
    config: MarketplaceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: JobId,
    pub poster: Did,
    pub title: String,
    pub description: String,
    pub requirements: Vec<String>,
    pub budget: u64,
    pub deadline: Epoch,
    pub status: JobStatus,
    pub created_at: Epoch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobStatus {
    Open,
    Assigned,
    InProgress,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bid {
    pub bidder: Did,
    pub amount: u64,
    pub proposal: String,
    pub estimated_duration: u64,
    pub submitted_at: Epoch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobExecution {
    pub job_id: JobId,
    pub executor: Did,
    pub started_at: Epoch,
    pub completed_at: Option<Epoch>,
    pub deliverables: Vec<Cid>,
    pub status: ExecutionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionStatus {
    InProgress,
    Completed,
    Disputed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceConfig {
    pub min_bid_amount: u64,
    pub max_job_duration: Epoch,
    pub dispute_resolution_period: Epoch,
    pub platform_fee_percent: f64,
}

impl Default for MarketplaceConfig {
    fn default() -> Self {
        Self {
            min_bid_amount: 1,
            max_job_duration: 30, // 30 epochs
            dispute_resolution_period: 7, // 7 epochs
            platform_fee_percent: 0.05, // 5%
        }
    }
}

impl JobMarketplaceContract {
    pub fn new(initial_members: Vec<Did>) -> Self {
        Self {
            jobs: HashMap::new(),
            bids: HashMap::new(),
            executions: HashMap::new(),
            members: initial_members.into_iter().collect(),
            config: MarketplaceConfig::default(),
        }
    }
    
    /// Post a new job
    pub fn post_job(
        &mut self,
        poster: Did,
        title: String,
        description: String,
        requirements: Vec<String>,
        budget: u64,
        deadline: Epoch,
    ) -> Result<JobId, CclRuntimeError> {
        // Check membership
        if !self.members.contains(&poster) {
            return Err(CclRuntimeError::PermissionDenied(
                crate::security::Capability::ModifyState
            ));
        }
        
        // Validate job parameters
        if deadline > self.current_epoch() + self.config.max_job_duration {
            return Err(CclRuntimeError::ExecutionError(
                "Job deadline exceeds maximum duration".to_string()
            ));
        }
        
        let job_id = format!("job_{}", uuid::Uuid::new_v4());
        let job = Job {
            id: job_id.clone(),
            poster,
            title,
            description,
            requirements,
            budget,
            deadline,
            status: JobStatus::Open,
            created_at: self.current_epoch(),
        };
        
        self.jobs.insert(job_id.clone(), job);
        self.bids.insert(job_id.clone(), Vec::new());
        
        self.emit_event("JobPosted", &job_id);
        
        Ok(job_id)
    }
    
    /// Submit a bid for a job
    pub fn submit_bid(
        &mut self,
        job_id: JobId,
        bidder: Did,
        amount: u64,
        proposal: String,
        estimated_duration: u64,
    ) -> Result<(), CclRuntimeError> {
        // Check membership
        if !self.members.contains(&bidder) {
            return Err(CclRuntimeError::PermissionDenied(
                crate::security::Capability::ModifyState
            ));
        }
        
        // Check if job exists and is open
        let job = self.jobs.get(&job_id)
            .ok_or_else(|| CclRuntimeError::ExecutionError("Job not found".to_string()))?;
        
        if !matches!(job.status, JobStatus::Open) {
            return Err(CclRuntimeError::ExecutionError("Job not open for bidding".to_string()));
        }
        
        // Validate bid
        if amount < self.config.min_bid_amount {
            return Err(CclRuntimeError::ExecutionError("Bid amount too low".to_string()));
        }
        
        let bid = Bid {
            bidder: bidder.clone(),
            amount,
            proposal,
            estimated_duration,
            submitted_at: self.current_epoch(),
        };
        
        self.bids.get_mut(&job_id).unwrap().push(bid);
        
        self.emit_event("BidSubmitted", &format!("{}:{}", job_id, bidder));
        
        Ok(())
    }
    
    /// Accept a bid and assign the job
    pub fn accept_bid(
        &mut self,
        job_id: JobId,
        bidder: Did,
        accepter: Did,
    ) -> Result<(), CclRuntimeError> {
        // Check if job exists
        let job = self.jobs.get_mut(&job_id)
            .ok_or_else(|| CclRuntimeError::ExecutionError("Job not found".to_string()))?;
        
        // Check if accepter is the job poster
        if job.poster != accepter {
            return Err(CclRuntimeError::PermissionDenied(
                crate::security::Capability::ModifyState
            ));
        }
        
        // Check if job is open
        if !matches!(job.status, JobStatus::Open) {
            return Err(CclRuntimeError::ExecutionError("Job not open".to_string()));
        }
        
        // Check if bid exists
        let bids = self.bids.get(&job_id).unwrap();
        let _bid = bids.iter()
            .find(|b| b.bidder == bidder)
            .ok_or_else(|| CclRuntimeError::ExecutionError("Bid not found".to_string()))?;
        
        // Update job status
        job.status = JobStatus::Assigned;
        
        // Create execution record
        let execution = JobExecution {
            job_id: job_id.clone(),
            executor: bidder.clone(),
            started_at: self.current_epoch(),
            completed_at: None,
            deliverables: Vec::new(),
            status: ExecutionStatus::InProgress,
        };
        
        self.executions.insert(job_id.clone(), execution);
        
        self.emit_event("JobAssigned", &format!("{}:{}", job_id, bidder));
        
        Ok(())
    }
    
    /// Complete job execution
    pub fn complete_job(
        &mut self,
        job_id: JobId,
        executor: Did,
        deliverables: Vec<Cid>,
    ) -> Result<(), CclRuntimeError> {
        // Get execution record
        let execution = self.executions.get_mut(&job_id)
            .ok_or_else(|| CclRuntimeError::ExecutionError("Job execution not found".to_string()))?;
        
        // Check if executor matches
        if execution.executor != executor {
            return Err(CclRuntimeError::PermissionDenied(
                crate::security::Capability::ModifyState
            ));
        }
        
        // Update execution
        let current_time = self.current_epoch();
        execution.completed_at = Some(current_time);
        execution.deliverables = deliverables;
        execution.status = ExecutionStatus::Completed;
        
        // Update job status
        if let Some(job) = self.jobs.get_mut(&job_id) {
            job.status = JobStatus::Completed;
        }
        
        self.emit_event("JobCompleted", &format!("{}:{}", job_id, executor));
        
        Ok(())
    }
    
    /// Get job information
    pub fn get_job(&self, job_id: &JobId) -> Option<&Job> {
        self.jobs.get(job_id)
    }
    
    /// List open jobs
    pub fn list_open_jobs(&self) -> Vec<&Job> {
        self.jobs.values()
            .filter(|job| matches!(job.status, JobStatus::Open))
            .collect()
    }
    
    /// Get bids for a job
    pub fn get_bids(&self, job_id: &JobId) -> Option<&Vec<Bid>> {
        self.bids.get(job_id)
    }
    
    fn current_epoch(&self) -> Epoch {
        icn_common::current_timestamp() / 86400
    }
    
    fn emit_event(&self, event_type: &str, data: &str) {
        log::info!("JobMarketplace event: {} - {}", event_type, data);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::Did;
    
    #[test]
    fn test_democratic_governance_creation() {
        let members = vec![
            Did::new("key", "alice"),
            Did::new("key", "bob"),
            Did::new("key", "charlie"),
        ];
        
        let governance = DemocraticGovernanceContract::new(members.clone());
        assert_eq!(governance.member_count(), 3);
        assert!(governance.is_member(&members[0]));
    }
    
    #[test]
    fn test_proposal_creation() {
        let members = vec![
            Did::new("key", "alice"),
            Did::new("key", "bob"),
        ];
        
        let mut governance = DemocraticGovernanceContract::new(members.clone());
        
        let proposal_id = governance.propose(
            members[0].clone(),
            "Test Proposal".to_string(),
            "A test proposal".to_string(),
            vec![ProposalAction::AddMember { did: Did::new("key", "charlie") }],
        ).unwrap();
        
        let proposal = governance.get_proposal(&proposal_id).unwrap();
        assert_eq!(proposal.title, "Test Proposal");
        assert_eq!(proposal.state, ProposalState::Active);
    }
    
    #[test]
    fn test_mutual_credit_creation() {
        let members = vec![
            Did::new("key", "alice"),
            Did::new("key", "bob"),
        ];
        
        let credit = MutualCreditContract::new(members.clone());
        assert_eq!(credit.get_balance(&members[0]), 0);
        assert_eq!(credit.get_balance(&members[1]), 0);
    }
    
    #[test]
    fn test_credit_extension() {
        let members = vec![
            Did::new("key", "alice"),
            Did::new("key", "bob"),
        ];
        
        let mut credit = MutualCreditContract::new(members.clone());
        
        let credit_line_id = credit.extend_credit(
            members[0].clone(),
            members[1].clone(),
            1000,
        ).unwrap();
        
        let credit_line = credit.get_credit_line(&credit_line_id).unwrap();
        assert_eq!(credit_line.limit, 1000);
        assert_eq!(credit_line.used, 0);
    }
    
    #[test]
    fn test_job_marketplace_creation() {
        let members = vec![
            Did::new("key", "alice"),
            Did::new("key", "bob"),
        ];
        
        let marketplace = JobMarketplaceContract::new(members);
        assert_eq!(marketplace.list_open_jobs().len(), 0);
    }
    
    #[test]
    fn test_job_posting() {
        let members = vec![
            Did::new("key", "alice"),
            Did::new("key", "bob"),
        ];
        
        let mut marketplace = JobMarketplaceContract::new(members.clone());
        
        let job_id = marketplace.post_job(
            members[0].clone(),
            "Test Job".to_string(),
            "A test job".to_string(),
            vec!["skill1".to_string()],
            1000,
            100, // deadline
        ).unwrap();
        
        let job = marketplace.get_job(&job_id).unwrap();
        assert_eq!(job.title, "Test Job");
        assert!(matches!(job.status, JobStatus::Open));
    }
}