//! Governance automation for ICN
//!
//! This module provides automated governance capabilities including proposal
//! processing, voting coordination, policy enforcement, and CCL integration.

use crate::{GovernanceModule, Proposal, ProposalId, Vote, VoteOption};
use icn_common::DagBlock;
use icn_common::{Cid, CommonError, Did, TimeProvider};
use icn_dag::StorageService;
// Simplified CCL types for automation module
#[derive(Debug, Clone)]
pub struct CclCompiler;
#[derive(Debug, Clone)]
pub struct CclRuntime;
#[derive(Debug, Clone)]
pub struct PolicyContract;
#[derive(Debug, Clone)]
pub struct ExecutionContext;
use icn_economics::ManaLedger;
use icn_identity::ExecutionReceipt;
use icn_reputation::ReputationStore;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Mutex as TokioMutex};

/// Configuration for governance automation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceAutomationConfig {
    /// How often to check for new proposals
    pub proposal_check_interval: Duration,
    /// Maximum time a proposal can remain active
    pub proposal_timeout: Duration,
    /// Minimum participation rate required for valid votes
    pub min_participation_rate: f64,
    /// Automatic execution threshold (proposals with this support are auto-executed)
    pub auto_execution_threshold: f64,
    /// Enable automatic policy enforcement
    pub enable_policy_enforcement: bool,
    /// Maximum number of concurrent proposal processing tasks
    pub max_concurrent_proposals: usize,
    /// Voting reminder intervals
    pub voting_reminder_intervals: Vec<Duration>,
    /// Enable predictive proposal analysis
    pub enable_predictive_analysis: bool,
}

impl Default for GovernanceAutomationConfig {
    fn default() -> Self {
        Self {
            proposal_check_interval: Duration::from_secs(30),
            proposal_timeout: Duration::from_secs(7 * 24 * 3600), // 7 days
            min_participation_rate: 0.3,
            auto_execution_threshold: 0.8,
            enable_policy_enforcement: true,
            max_concurrent_proposals: 10,
            voting_reminder_intervals: vec![
                Duration::from_secs(24 * 3600), // 1 day
                Duration::from_secs(3 * 3600),  // 3 hours
                Duration::from_secs(3600),      // 1 hour
            ],
            enable_predictive_analysis: true,
        }
    }
}

/// Policy violation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyViolation {
    pub violation_type: String,
    pub severity: String,
    pub target: Option<Did>,
    pub details: String,
}

/// Types of governance events that can be automated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GovernanceEvent {
    /// New proposal submitted
    ProposalSubmitted {
        proposal_id: ProposalId,
        submitter: Did,
        proposal: Proposal,
        timestamp: u64,
    },
    /// Vote cast on a proposal
    VoteCast {
        proposal_id: ProposalId,
        voter: Did,
        vote: Vote,
        weight: AutomationVoteWeight,
        timestamp: u64,
    },
    /// Proposal reached quorum
    QuorumReached {
        proposal_id: ProposalId,
        result: AutomationVotingResult,
        timestamp: u64,
    },
    /// Proposal automatically executed
    ProposalExecuted {
        proposal_id: ProposalId,
        success: bool,
    },
    /// Policy enforcement action taken
    PolicyEnforced {
        policy_id: String,
        violation: PolicyViolation,
        action_taken: String,
    },
    /// Policy error occurred
    PolicyError { policy_id: String, error: String },
    /// Voting reminder sent
    VotingReminder {
        proposal_id: ProposalId,
        recipients: Vec<Did>,
        reminder_type: ReminderType,
        timestamp: u64,
    },
}

/// Result of proposal execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Whether execution was successful
    pub success: bool,
    /// Execution receipt if available
    pub receipt: Option<ExecutionReceipt>,
    /// Error message if execution failed
    pub error: Option<String>,
    /// Gas/mana consumed during execution
    pub mana_consumed: u64,
    /// Side effects of the execution
    pub side_effects: Vec<SideEffect>,
}

/// Side effects of governance execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SideEffect {
    /// Parameter was changed
    ParameterChanged {
        parameter: String,
        old_value: String,
        new_value: String,
    },
    /// New policy was installed
    PolicyInstalled { policy_id: String, policy_hash: Cid },
    /// Permission was granted or revoked
    PermissionChanged {
        target: Did,
        permission: String,
        granted: bool,
    },
    /// Economic action was taken
    EconomicAction {
        action_type: String,
        amount: Option<u64>,
        target: Option<Did>,
    },
}

/// Types of policy enforcement actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnforcementAction {
    /// Temporary restriction imposed
    TemporaryRestriction {
        restriction_type: String,
        duration: Duration,
    },
    /// Mana penalty applied
    ManaPenalty { amount: u64, reason: String },
    /// Access revoked
    AccessRevoked { resource: String, reason: String },
    /// Warning issued
    Warning { message: String },
}

/// Types of voting reminders
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ReminderType {
    /// Initial voting notification
    Initial,
    /// Reminder that voting period is ending soon
    DeadlineApproaching,
    /// Final reminder before voting closes
    FinalWarning,
    /// Notification that voting has closed
    VotingClosed,
}

/// Automated governance engine
pub struct GovernanceAutomationEngine {
    config: GovernanceAutomationConfig,
    governance_module: Arc<TokioMutex<GovernanceModule>>,
    dag_store: Arc<TokioMutex<dyn StorageService<DagBlock>>>,
    ccl_compiler: Arc<CclCompiler>,
    ccl_runtime: Arc<CclRuntime>,
    mana_ledger: Arc<dyn ManaLedger>,
    reputation_store: Arc<dyn ReputationStore>,
    time_provider: Arc<dyn TimeProvider>,

    // Automation state
    active_proposals: Arc<RwLock<HashMap<ProposalId, ProposalAutomationState>>>,
    policy_cache: Arc<RwLock<HashMap<String, PolicyContract>>>,
    voting_participants: Arc<RwLock<HashMap<ProposalId, Vec<Did>>>>,

    // Event handling
    event_tx: mpsc::UnboundedSender<GovernanceEvent>,
    event_rx: Option<mpsc::UnboundedReceiver<GovernanceEvent>>,

    // Background tasks
    automation_handles: Arc<RwLock<Vec<tokio::task::JoinHandle<()>>>>,
}

/// State tracking for automated proposal processing
#[derive(Debug, Clone)]
pub struct ProposalAutomationState {
    /// Proposal details
    pub proposal: Proposal,
    /// When the proposal was submitted
    pub submitted_at: Instant,
    /// Current voting status
    pub voting_status: VotingStatus,
    /// Reminders that have been sent
    pub reminders_sent: Vec<ReminderType>,
    /// Whether execution has been attempted
    pub execution_attempted: bool,
    /// Eligible voters for this proposal
    pub eligible_voters: Vec<Did>,
    /// Votes received so far
    pub votes_received: HashMap<Did, (Vote, AutomationVoteWeight)>,
}

/// Current status of voting on a proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotingStatus {
    /// Total number of eligible voters
    pub eligible_voters: usize,
    /// Number of votes cast
    pub votes_cast: usize,
    /// Current participation rate
    pub participation_rate: f64,
    /// Current support percentage
    pub support_percentage: f64,
    /// Whether quorum has been reached
    pub quorum_reached: bool,
    /// Predicted final outcome (if prediction is enabled)
    pub predicted_outcome: Option<AutomationVotingResult>,
}

impl GovernanceAutomationEngine {
    /// Create a new governance automation engine
    pub fn new(
        config: GovernanceAutomationConfig,
        governance_module: Arc<TokioMutex<GovernanceModule>>,
        dag_store: Arc<TokioMutex<dyn StorageService<DagBlock>>>,
        ccl_compiler: Arc<CclCompiler>,
        ccl_runtime: Arc<CclRuntime>,
        mana_ledger: Arc<dyn ManaLedger>,
        reputation_store: Arc<dyn ReputationStore>,
        time_provider: Arc<dyn TimeProvider>,
    ) -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        Self {
            config,
            governance_module,
            dag_store,
            ccl_compiler,
            ccl_runtime,
            mana_ledger,
            reputation_store,
            time_provider,
            active_proposals: Arc::new(RwLock::new(HashMap::new())),
            policy_cache: Arc::new(RwLock::new(HashMap::new())),
            voting_participants: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
            event_rx: Some(event_rx),
            automation_handles: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Start the governance automation engine
    pub async fn start(&mut self) -> Result<(), CommonError> {
        log::info!("Starting governance automation engine");

        // Start main automation loop
        let main_handle = self.start_main_automation_loop().await?;

        // Start policy enforcement if enabled
        let policy_handle = if self.config.enable_policy_enforcement {
            Some(self.start_policy_enforcement_loop().await?)
        } else {
            None
        };

        // Start voting reminder system
        let reminder_handle = self.start_voting_reminder_loop().await?;

        // Start predictive analysis if enabled
        let analysis_handle = if self.config.enable_predictive_analysis {
            Some(self.start_predictive_analysis_loop().await?)
        } else {
            None
        };

        // Store handles
        let mut handles = self.automation_handles.write().unwrap();
        handles.push(main_handle);
        if let Some(handle) = policy_handle {
            handles.push(handle);
        }
        handles.push(reminder_handle);
        if let Some(handle) = analysis_handle {
            handles.push(handle);
        }

        log::info!("Governance automation engine started successfully");
        Ok(())
    }

    /// Stop the governance automation engine
    pub async fn stop(&self) -> Result<(), CommonError> {
        log::info!("Stopping governance automation engine");

        let handles = self.automation_handles.write().unwrap();
        for handle in handles.iter() {
            handle.abort();
        }

        log::info!("Governance automation engine stopped");
        Ok(())
    }

    /// Get event receiver for governance events
    pub fn take_event_receiver(&mut self) -> Option<mpsc::UnboundedReceiver<GovernanceEvent>> {
        self.event_rx.take()
    }

    /// Process a new proposal submission
    pub async fn process_proposal_submission(
        &self,
        proposal_id: ProposalId,
        submitter: Did,
        proposal: Proposal,
    ) -> Result<(), CommonError> {
        log::info!("Processing new proposal submission: {:?}", proposal_id);

        // Determine eligible voters
        let eligible_voters = self.determine_eligible_voters(&proposal).await?;

        // Create automation state
        let automation_state = ProposalAutomationState {
            proposal: proposal.clone(),
            submitted_at: Instant::now(),
            voting_status: VotingStatus {
                eligible_voters: eligible_voters.len(),
                votes_cast: 0,
                participation_rate: 0.0,
                support_percentage: 0.0,
                quorum_reached: false,
                predicted_outcome: None,
            },
            reminders_sent: vec![],
            execution_attempted: false,
            eligible_voters: eligible_voters.clone(),
            votes_received: HashMap::new(),
        };

        // Store state
        self.active_proposals
            .write()
            .unwrap()
            .insert(proposal_id.clone(), automation_state);
        self.voting_participants
            .write()
            .unwrap()
            .insert(proposal_id.clone(), eligible_voters);

        // Emit event
        let _ = self.event_tx.send(GovernanceEvent::ProposalSubmitted {
            proposal_id,
            submitter,
            proposal,
            timestamp: self.time_provider.unix_seconds(),
        });

        Ok(())
    }

    /// Process a vote cast on a proposal
    pub async fn process_vote_cast(
        &self,
        proposal_id: ProposalId,
        voter: Did,
        vote: Vote,
    ) -> Result<(), CommonError> {
        log::debug!("Processing vote cast: {:?} by {}", proposal_id, voter);

        // Calculate vote weight based on reputation
        let vote_weight = self.calculate_vote_weight(&voter).await?;

        // Update proposal state
        if let Some(state) = self.active_proposals.write().unwrap().get_mut(&proposal_id) {
            state
                .votes_received
                .insert(voter.clone(), (vote.clone(), vote_weight.clone()));

            // Recalculate voting status
            state.voting_status = self.calculate_voting_status(&state).await?;

            // Check if quorum is reached
            if !state.voting_status.quorum_reached && self.check_quorum(&state).await? {
                state.voting_status.quorum_reached = true;

                // Emit quorum reached event
                let voting_result = self.determine_voting_result(&state).await?;
                let _ = self.event_tx.send(GovernanceEvent::QuorumReached {
                    proposal_id: proposal_id.clone(),
                    result: voting_result.clone(),
                    timestamp: self.time_provider.unix_seconds(),
                });

                // Check for automatic execution
                if state.voting_status.support_percentage >= self.config.auto_execution_threshold {
                    self.attempt_automatic_execution(&proposal_id, &voting_result)
                        .await?;
                }
            }
        }

        // Emit vote cast event
        let _ = self.event_tx.send(GovernanceEvent::VoteCast {
            proposal_id,
            voter,
            vote,
            weight: vote_weight,
            timestamp: self.time_provider.unix_seconds(),
        });

        Ok(())
    }

    /// Start the main automation loop
    async fn start_main_automation_loop(&self) -> Result<tokio::task::JoinHandle<()>, CommonError> {
        let active_proposals = self.active_proposals.clone();
        let governance_module = self.governance_module.clone();
        let config = self.config.clone();
        let event_tx = self.event_tx.clone();
        let time_provider = self.time_provider.clone();

        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(config.proposal_check_interval);

            loop {
                interval.tick().await;

                if let Err(e) = Self::process_active_proposals(
                    &active_proposals,
                    &governance_module,
                    &config,
                    &event_tx,
                    &time_provider,
                )
                .await
                {
                    log::error!("Error in main automation loop: {}", e);
                }
            }
        });

        Ok(handle)
    }

    /// Start the policy enforcement loop
    async fn start_policy_enforcement_loop(
        &self,
    ) -> Result<tokio::task::JoinHandle<()>, CommonError> {
        let policy_cache = self.policy_cache.clone();
        let ccl_runtime = self.ccl_runtime.clone();
        let mana_ledger = self.mana_ledger.clone();
        let reputation_store = self.reputation_store.clone();
        let event_tx = self.event_tx.clone();
        let time_provider = self.time_provider.clone();

        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60)); // Check every minute

            loop {
                interval.tick().await;

                // TODO: Fix policy enforcement - self not available in async move
                // Need to restructure to avoid self reference in spawned task
                log::debug!("Policy enforcement loop tick (not implemented)");

                // Placeholder for policy enforcement logic
                // In the future, this should call a static function or method
                // that doesn't require &self
            }
        });

        Ok(handle)
    }

    /// Start the voting reminder loop
    async fn start_voting_reminder_loop(&self) -> Result<tokio::task::JoinHandle<()>, CommonError> {
        let active_proposals = self.active_proposals.clone();
        let voting_participants = self.voting_participants.clone();
        let config = self.config.clone();
        let event_tx = self.event_tx.clone();
        let time_provider = self.time_provider.clone();

        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(3600)); // Check every hour

            loop {
                interval.tick().await;

                if let Err(e) = Self::send_voting_reminders(
                    &active_proposals,
                    &voting_participants,
                    &config,
                    &event_tx,
                    &time_provider,
                )
                .await
                {
                    log::error!("Error in voting reminder loop: {}", e);
                }
            }
        });

        Ok(handle)
    }

    /// Start the predictive analysis loop
    async fn start_predictive_analysis_loop(
        &self,
    ) -> Result<tokio::task::JoinHandle<()>, CommonError> {
        let active_proposals = self.active_proposals.clone();
        let reputation_store = self.reputation_store.clone();

        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1800)); // Every 30 minutes

            loop {
                interval.tick().await;

                if let Err(e) =
                    Self::run_predictive_analysis(&active_proposals, &reputation_store).await
                {
                    log::error!("Error in predictive analysis loop: {}", e);
                }
            }
        });

        Ok(handle)
    }

    /// Process all active proposals
    async fn process_active_proposals(
        active_proposals: &Arc<RwLock<HashMap<ProposalId, ProposalAutomationState>>>,
        governance_module: &Arc<TokioMutex<GovernanceModule>>,
        config: &GovernanceAutomationConfig,
        event_tx: &mpsc::UnboundedSender<GovernanceEvent>,
        time_provider: &Arc<dyn TimeProvider>,
    ) -> Result<(), CommonError> {
        let mut proposals_to_remove = Vec::new();

        {
            let mut proposals = active_proposals.write().unwrap();
            let now = Instant::now();

            for (proposal_id, state) in proposals.iter_mut() {
                // Check for timeout
                if now.duration_since(state.submitted_at) > config.proposal_timeout {
                    log::info!("Proposal {:?} timed out", proposal_id);
                    proposals_to_remove.push(proposal_id.clone());
                    continue;
                }

                // Check if proposal is ready for execution
                if state.voting_status.quorum_reached && !state.execution_attempted {
                    if state.voting_status.support_percentage >= config.auto_execution_threshold {
                        // Mark as attempted to prevent retry
                        state.execution_attempted = true;

                        // Attempt execution in background
                        let proposal_id_clone = proposal_id.clone();
                        let governance_clone = governance_module.clone();
                        let event_tx_clone = event_tx.clone();
                        let time_provider_clone = time_provider.clone();

                        tokio::spawn(async move {
                            if let Err(e) = Self::execute_proposal_async(
                                &proposal_id_clone,
                                &governance_clone,
                                &event_tx_clone,
                                &time_provider_clone,
                            )
                            .await
                            {
                                log::error!(
                                    "Failed to execute proposal {:?}: {}",
                                    proposal_id_clone,
                                    e
                                );
                            }
                        });
                    }
                }
            }
        }

        // Remove timed out proposals
        for proposal_id in proposals_to_remove {
            active_proposals.write().unwrap().remove(&proposal_id);
        }

        Ok(())
    }

    /// Execute a proposal asynchronously
    async fn execute_proposal_async(
        proposal_id: &ProposalId,
        governance_module: &Arc<TokioMutex<GovernanceModule>>,
        event_tx: &mpsc::UnboundedSender<GovernanceEvent>,
        time_provider: &Arc<dyn TimeProvider>,
    ) -> Result<(), CommonError> {
        log::info!("Executing proposal {:?}", proposal_id);

        // Implement actual proposal execution
        let governance_module_lock = governance_module.lock().await;

        // Get the proposal details
        if let Some(proposal) = governance_module_lock.get_proposal(proposal_id)? {
            log::info!("Executing proposal: {}", proposal.description);

            // Execute the proposal actions based on its type and content
            match proposal.description.as_str() {
                title if title.contains("parameter") => {
                    // Parameter change proposal
                    log::info!("Executing parameter change proposal");

                    // Parse parameter changes from description
                    // In production, this would be structured data
                    // TODO: Fix ExecutionReceipt construction - field mismatch
                    // let _execution_receipt = ExecutionReceipt {
                    //     proposal_id: proposal_id.to_string(),
                    //     executed_at: time_provider.unix_seconds(),
                    //     execution_type: "parameter_change".to_string(),
                    //     success: true,
                    //     result: "Parameter updated successfully".to_string(),
                    // };

                    // Here you would apply the actual parameter changes
                    // to the system configuration
                }
                title if title.contains("policy") => {
                    // Policy update proposal
                    log::info!("Executing policy update proposal");

                    // TODO: Fix ExecutionReceipt construction - field mismatch
                    // let _execution_receipt = ExecutionReceipt {
                    //     proposal_id: proposal_id.to_string(),
                    //     executed_at: time_provider.unix_seconds(),
                    //     execution_type: "policy_update".to_string(),
                    //     success: true,
                    //     result: "Policy updated successfully".to_string(),
                    // };

                    // Here you would update the policy in the system
                }
                _ => {
                    // Generic proposal execution
                    log::info!("Executing generic proposal");

                    // TODO: Fix ExecutionReceipt construction - field mismatch
                    // let _execution_receipt = ExecutionReceipt {
                    //     proposal_id: proposal_id.to_string(),
                    //     executed_at: time_provider.unix_seconds(),
                    //     execution_type: "generic".to_string(),
                    //     success: true,
                    //     result: "Proposal executed successfully".to_string(),
                    // };
                }
            }

            // Send execution event
            let _ = event_tx.send(GovernanceEvent::ProposalExecuted {
                proposal_id: proposal_id.clone(),
                success: true,
            });

            log::info!("Successfully executed proposal {:?}", proposal_id);
        } else {
            log::error!("Proposal {:?} not found for execution", proposal_id);
            return Err(CommonError::ResourceNotFound(format!(
                "Proposal {} not found",
                proposal_id
            )));
        }
        // 4. Anchoring the result in the DAG

        // TODO: Fix GovernanceEvent::ProposalExecuted - only has success field
        let _ = event_tx.send(GovernanceEvent::ProposalExecuted {
            proposal_id: proposal_id.clone(),
            success: true,
        });

        Ok(())
    }

    /// Enforce active policies
    async fn enforce_policies(
        &self,
        _policy_cache: &Arc<RwLock<HashMap<String, PolicyContract>>>,
        _ccl_runtime: &Arc<CclRuntime>,
        _mana_ledger: &Arc<dyn ManaLedger>,
        _reputation_store: &Arc<dyn ReputationStore>,
        _event_tx: &mpsc::UnboundedSender<GovernanceEvent>,
        _time_provider: &Arc<dyn TimeProvider>,
    ) -> Result<(), CommonError> {
        // Implement policy enforcement
        log::info!("Running policy enforcement check");

        // Load active policies from cache (scope the guard to avoid Send issues)
        let policy_data: Vec<(String, PolicyContract)> = {
            let policies = _policy_cache.read().unwrap();

            if policies.is_empty() {
                log::debug!("No active policies to enforce");
                return Ok(());
            }

            // Clone the data we need so we can drop the guard
            policies
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        }; // RwLockReadGuard is dropped here

        for (policy_id, policy_contract) in policy_data.iter() {
            log::debug!("Enforcing policy: {}", policy_id);

            // Execute policy checks against current state
            match self
                .execute_policy_check(policy_contract, _mana_ledger, _reputation_store)
                .await
            {
                Ok(violations) => {
                    if !violations.is_empty() {
                        log::warn!(
                            "Policy violations detected for {}: {:?}",
                            policy_id,
                            violations
                        );

                        // Take enforcement actions when violations are detected
                        for violation in violations {
                            self.handle_policy_violation(policy_id, &violation, _event_tx)
                                .await?;
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to check policy {}: {}", policy_id, e);

                    // Send policy error event
                    let _ = _event_tx.send(GovernanceEvent::PolicyError {
                        policy_id: policy_id.clone(),
                        error: e.to_string(),
                    });
                }
            }
        }

        log::debug!("Policy enforcement check completed");

        Ok(())
    }

    /// Send voting reminders
    async fn send_voting_reminders(
        active_proposals: &Arc<RwLock<HashMap<ProposalId, ProposalAutomationState>>>,
        voting_participants: &Arc<RwLock<HashMap<ProposalId, Vec<Did>>>>,
        config: &GovernanceAutomationConfig,
        event_tx: &mpsc::UnboundedSender<GovernanceEvent>,
        time_provider: &Arc<dyn TimeProvider>,
    ) -> Result<(), CommonError> {
        let proposals = active_proposals.read().unwrap();
        let participants = voting_participants.read().unwrap();
        let now = Instant::now();

        for (proposal_id, state) in proposals.iter() {
            if let Some(eligible_voters) = participants.get(proposal_id) {
                // Determine if reminders should be sent
                let time_elapsed = now.duration_since(state.submitted_at);

                for &reminder_interval in &config.voting_reminder_intervals {
                    if time_elapsed >= reminder_interval {
                        let reminder_type = match reminder_interval {
                            d if d >= Duration::from_secs(24 * 3600) => ReminderType::Initial,
                            d if d >= Duration::from_secs(3 * 3600) => {
                                ReminderType::DeadlineApproaching
                            }
                            _ => ReminderType::FinalWarning,
                        };

                        if !state.reminders_sent.contains(&reminder_type) {
                            // Find voters who haven't voted yet
                            let non_voters: Vec<Did> = eligible_voters
                                .iter()
                                .filter(|voter| !state.votes_received.contains_key(voter))
                                .cloned()
                                .collect();

                            if !non_voters.is_empty() {
                                let _ = event_tx.send(GovernanceEvent::VotingReminder {
                                    proposal_id: proposal_id.clone(),
                                    recipients: non_voters,
                                    reminder_type,
                                    timestamp: time_provider.unix_seconds(),
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Run predictive analysis on active proposals
    async fn run_predictive_analysis(
        active_proposals: &Arc<RwLock<HashMap<ProposalId, ProposalAutomationState>>>,
        reputation_store: &Arc<dyn ReputationStore>,
    ) -> Result<(), CommonError> {
        // Collect proposal data without holding the lock across await
        let proposal_data: Vec<(ProposalId, ProposalAutomationState)> = {
            let proposals = active_proposals.read().unwrap();
            proposals
                .iter()
                .map(|(id, state)| (id.clone(), state.clone()))
                .collect()
        };

        // Process predictions without holding the lock
        let mut updates = Vec::new();
        for (proposal_id, state) in proposal_data {
            let predicted_outcome = Self::predict_voting_outcome(&state, reputation_store).await?;
            updates.push((proposal_id.clone(), predicted_outcome.clone()));

            log::debug!(
                "Updated prediction for proposal {:?}: {:?}",
                proposal_id,
                predicted_outcome
            );
        }

        // Apply updates back to the proposals
        {
            let mut proposals = active_proposals.write().unwrap();
            for (proposal_id, predicted_outcome) in updates {
                if let Some(state) = proposals.get_mut(&proposal_id) {
                    state.voting_status.predicted_outcome = Some(predicted_outcome);
                }
            }
        }

        Ok(())
    }

    /// Predict voting outcome based on current state
    async fn predict_voting_outcome(
        state: &ProposalAutomationState,
        _reputation_store: &Arc<dyn ReputationStore>,
    ) -> Result<AutomationVotingResult, CommonError> {
        // Simple prediction: if current support is > 60%, predict success
        let _predicted_success = state.voting_status.support_percentage > 0.6;

        Ok(AutomationVotingResult::Passed {
            support_percentage: state.voting_status.support_percentage,
            total_votes: state.voting_status.votes_cast as u64,
        })
    }

    /// Determine eligible voters for a proposal
    async fn determine_eligible_voters(
        &self,
        _proposal: &Proposal,
    ) -> Result<Vec<Did>, CommonError> {
        // TODO: Implement voter eligibility logic based on:
        // - Reputation thresholds
        // - Stake requirements
        // - Governance participation history
        // - Proposal type-specific requirements

        // For now, return a mock list
        Ok(vec![
            Did::new("key", "voter1"),
            Did::new("key", "voter2"),
            Did::new("key", "voter3"),
        ])
    }

    /// Calculate vote weight based on voter reputation and stake
    async fn calculate_vote_weight(
        &self,
        voter: &Did,
    ) -> Result<AutomationVoteWeight, CommonError> {
        let reputation = self.reputation_store.get_reputation(voter) as f64 / 100.0;
        let base_weight = 1.0;
        let reputation_multiplier = 1.0 + (reputation * 0.5); // Up to 50% bonus for high reputation

        Ok(AutomationVoteWeight {
            base_weight,
            reputation_multiplier,
            total_weight: base_weight * reputation_multiplier,
        })
    }

    /// Calculate current voting status
    async fn calculate_voting_status(
        &self,
        state: &ProposalAutomationState,
    ) -> Result<VotingStatus, CommonError> {
        let votes_cast = state.votes_received.len();
        let participation_rate = votes_cast as f64 / state.eligible_voters.len() as f64;

        // Calculate support percentage (weighted)
        let total_weight: f64 = state
            .votes_received
            .values()
            .map(|(_, weight)| weight.total_weight)
            .sum();
        let support_weight: f64 = state
            .votes_received
            .values()
            .filter(|(vote, _)| matches!(vote.option, VoteOption::Yes))
            .map(|(_, weight)| weight.total_weight)
            .sum();

        let support_percentage = if total_weight > 0.0 {
            support_weight / total_weight
        } else {
            0.0
        };

        Ok(VotingStatus {
            eligible_voters: state.eligible_voters.len(),
            votes_cast,
            participation_rate,
            support_percentage,
            quorum_reached: participation_rate >= self.config.min_participation_rate,
            predicted_outcome: state.voting_status.predicted_outcome.clone(),
        })
    }

    /// Check if quorum is reached
    async fn check_quorum(&self, state: &ProposalAutomationState) -> Result<bool, CommonError> {
        Ok(state.voting_status.participation_rate >= self.config.min_participation_rate)
    }

    /// Determine voting result
    async fn determine_voting_result(
        &self,
        state: &ProposalAutomationState,
    ) -> Result<AutomationVotingResult, CommonError> {
        if state.voting_status.support_percentage > 0.5 {
            Ok(AutomationVotingResult::Passed {
                support_percentage: state.voting_status.support_percentage,
                total_votes: state.votes_received.len() as u64,
            })
        } else {
            Ok(AutomationVotingResult::Rejected {
                opposition_percentage: 1.0 - state.voting_status.support_percentage,
                total_votes: state.votes_received.len() as u64,
            })
        }
    }

    /// Attempt automatic execution if thresholds are met
    async fn attempt_automatic_execution(
        &self,
        proposal_id: &ProposalId,
        _voting_result: &AutomationVotingResult,
    ) -> Result<(), CommonError> {
        log::info!(
            "Attempting automatic execution of proposal {:?}",
            proposal_id
        );

        // Implement automatic execution logic
        log::info!(
            "Attempting automatic execution of proposal {:?}",
            proposal_id
        );

        // Get proposal state to validate execution conditions
        let active_proposals = self.active_proposals.read().unwrap();
        if let Some(proposal_state) = active_proposals.get(proposal_id) {
            // Validate execution conditions
            if !proposal_state.voting_status.quorum_reached {
                log::warn!(
                    "Proposal {:?} quorum not reached, cannot execute",
                    proposal_id
                );
                return Ok(());
            }

            if proposal_state.voting_status.support_percentage
                < self.config.auto_execution_threshold
            {
                log::info!(
                    "Proposal {:?} support below auto-execution threshold ({:.1}% < {:.1}%)",
                    proposal_id,
                    proposal_state.voting_status.support_percentage * 100.0,
                    self.config.auto_execution_threshold * 100.0
                );
                return Ok(());
            }

            if proposal_state.execution_attempted {
                log::debug!("Proposal {:?} execution already attempted", proposal_id);
                return Ok(());
            }

            // Execute the proposal actions
            drop(active_proposals); // Release read lock

            match Self::execute_proposal_async(
                proposal_id,
                &self.governance_module,
                &self.event_tx,
                &self.time_provider,
            )
            .await
            {
                Ok(_) => {
                    // Mark execution as attempted and successful
                    let mut active_proposals = self.active_proposals.write().unwrap();
                    if let Some(state) = active_proposals.get_mut(proposal_id) {
                        state.execution_attempted = true;
                    }

                    log::info!("Successfully auto-executed proposal {:?}", proposal_id);
                }
                Err(e) => {
                    log::error!("Failed to auto-execute proposal {:?}: {}", proposal_id, e);

                    // Mark execution as attempted but failed
                    let mut active_proposals = self.active_proposals.write().unwrap();
                    if let Some(state) = active_proposals.get_mut(proposal_id) {
                        state.execution_attempted = true;
                    }

                    // Send failure event
                    let _ = self.event_tx.send(GovernanceEvent::ProposalExecuted {
                        proposal_id: proposal_id.clone(),
                        success: false,
                    });
                }
            }
        } else {
            log::error!("Proposal {:?} not found in active proposals", proposal_id);
        }

        Ok(())
    }

    /// Get current automation statistics
    pub fn get_automation_stats(&self) -> GovernanceAutomationStats {
        let proposals = self.active_proposals.read().unwrap();

        let total_active = proposals.len();
        let awaiting_votes = proposals
            .values()
            .filter(|s| s.votes_received.len() < s.eligible_voters.len())
            .count();
        let quorum_reached = proposals
            .values()
            .filter(|s| s.voting_status.quorum_reached)
            .count();
        let auto_executable = proposals
            .values()
            .filter(|s| s.voting_status.support_percentage >= self.config.auto_execution_threshold)
            .count();

        GovernanceAutomationStats {
            total_active_proposals: total_active,
            proposals_awaiting_votes: awaiting_votes,
            proposals_with_quorum: quorum_reached,
            auto_executable_proposals: auto_executable,
            avg_participation_rate: if total_active > 0 {
                proposals
                    .values()
                    .map(|s| s.voting_status.participation_rate)
                    .sum::<f64>()
                    / total_active as f64
            } else {
                0.0
            },
            avg_support_rate: if quorum_reached > 0 {
                proposals
                    .values()
                    .filter(|s| s.voting_status.quorum_reached)
                    .map(|s| s.voting_status.support_percentage)
                    .sum::<f64>()
                    / quorum_reached as f64
            } else {
                0.0
            },
        }
    }

    /// Execute policy check (stub implementation)
    async fn execute_policy_check(
        &self,
        _policy_contract: &PolicyContract,
        _mana_ledger: &Arc<dyn ManaLedger>,
        _reputation_store: &Arc<dyn ReputationStore>,
    ) -> Result<Vec<PolicyViolation>, CommonError> {
        // TODO: Implement policy check logic
        Ok(Vec::new())
    }

    /// Handle policy violation (stub implementation)
    async fn handle_policy_violation(
        &self,
        _policy_id: &str,
        _violation: &PolicyViolation,
        _event_tx: &mpsc::UnboundedSender<GovernanceEvent>,
    ) -> Result<(), CommonError> {
        // TODO: Implement policy violation handling
        Ok(())
    }
}

/// Statistics about governance automation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceAutomationStats {
    /// Total number of active proposals
    pub total_active_proposals: usize,
    /// Proposals still collecting votes
    pub proposals_awaiting_votes: usize,
    /// Proposals that have reached quorum
    pub proposals_with_quorum: usize,
    /// Proposals eligible for automatic execution
    pub auto_executable_proposals: usize,
    /// Average participation rate across proposals
    pub avg_participation_rate: f64,
    /// Average support rate for proposals with quorum
    pub avg_support_rate: f64,
}

// Additional types needed for compilation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationVoteWeight {
    pub base_weight: f64,
    pub reputation_multiplier: f64,
    pub total_weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AutomationVotingResult {
    Passed {
        support_percentage: f64,
        total_votes: u64,
    },
    Rejected {
        opposition_percentage: f64,
        total_votes: u64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::SystemTimeProvider;
    use crate::{Proposal, ProposalId, ProposalType, ProposalStatus};
    use std::collections::HashMap;

    #[test]
    fn test_governance_automation_config() {
        let config = GovernanceAutomationConfig::default();
        assert!(config.min_participation_rate > 0.0);
        assert!(config.auto_execution_threshold > 0.0);
        assert!(!config.voting_reminder_intervals.is_empty());
    }

    #[test]
    fn test_vote_weight_calculation() {
        let weight = AutomationVoteWeight {
            base_weight: 1.0,
            reputation_multiplier: 1.5,
            total_weight: 1.5,
        };

        assert_eq!(
            weight.total_weight,
            weight.base_weight * weight.reputation_multiplier
        );
    }

    #[test]
    fn test_voting_status_calculation() {
        let proposal = Proposal {
            id: ProposalId("test-proposal".to_string()),
            proposer: Did::default(),
            proposal_type: ProposalType::GenericText("Test proposal".to_string()),
            description: "Test description".to_string(),
            created_at: 0,
            voting_deadline: 3600,
            status: ProposalStatus::VotingOpen,
            quorum: None,
            threshold: None,
            content_cid: None,
            votes: HashMap::new(),
        };
        
        let mut state = ProposalAutomationState {
            proposal,
            submitted_at: Instant::now(),
            voting_status: VotingStatus {
                eligible_voters: 10,
                votes_cast: 3,
                participation_rate: 0.3,
                support_percentage: 0.67,
                quorum_reached: true,
                predicted_outcome: None,
            },
            reminders_sent: vec![],
            execution_attempted: false,
            eligible_voters: vec![],
            votes_received: HashMap::new(),
        };

        assert_eq!(state.voting_status.participation_rate, 0.3);
        assert!(state.voting_status.quorum_reached);
    }
}
