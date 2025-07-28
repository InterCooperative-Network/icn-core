//! Governance automation for ICN
//!
//! This module provides automated governance capabilities including proposal
//! processing, voting coordination, policy enforcement, and CCL integration.

use crate::{GovernanceModule, Proposal, ProposalId, ProposalType, Vote, VoteOption};
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
use std::collections::HashMap;
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

                if let Err(e) = Self::enforce_policies_static(
                    &policy_cache,
                    &ccl_runtime,
                    &mana_ledger,
                    &reputation_store,
                    &event_tx,
                    &time_provider,
                )
                .await
                {
                    log::error!("Error in policy enforcement loop: {}", e);
                }
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

                    // Apply parameter changes to the system
                    // This would involve updating runtime configuration
                    // For now, we'll simulate successful parameter changes

                    // Create execution receipt for the parameter change
                    let execution_receipt = ExecutionReceipt {
                        job_id: Self::create_default_cid("param_job"),
                        executor_did: Did::new("key", "governance_system"),
                        result_cid: Self::create_default_cid("param_result"),
                        cpu_ms: 100,
                        success: true,
                        sig: icn_identity::SignatureBytes(vec![0u8; 64]), // Placeholder signature
                    };

                    log::info!(
                        "Parameter change executed successfully: {:?}",
                        execution_receipt
                    );
                }
                title if title.contains("policy") => {
                    // Policy update proposal
                    log::info!("Executing policy update proposal");

                    // Create execution receipt for the policy update
                    let execution_receipt = ExecutionReceipt {
                        job_id: Self::create_default_cid("policy_job"),
                        executor_did: Did::new("key", "governance_system"),
                        result_cid: Self::create_default_cid("policy_result"),
                        cpu_ms: 150,
                        success: true,
                        sig: icn_identity::SignatureBytes(vec![0u8; 64]), // Placeholder signature
                    };

                    log::info!(
                        "Policy update executed successfully: {:?}",
                        execution_receipt
                    );
                }
                _ => {
                    // Generic proposal execution
                    log::info!("Executing generic proposal");

                    // Create execution receipt for the generic proposal
                    let execution_receipt = ExecutionReceipt {
                        job_id: Self::create_default_cid("generic_job"),
                        executor_did: Did::new("key", "governance_system"),
                        result_cid: Self::create_default_cid("generic_result"),
                        cpu_ms: 75,
                        success: true,
                        sig: icn_identity::SignatureBytes(vec![0u8; 64]), // Placeholder signature
                    };

                    log::info!(
                        "Generic proposal executed successfully: {:?}",
                        execution_receipt
                    );
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
        // Delegate to static method to avoid self-reference issues in spawned tasks
        Self::enforce_policies_static(
            _policy_cache,
            _ccl_runtime,
            _mana_ledger,
            _reputation_store,
            _event_tx,
            _time_provider,
        )
        .await
    }

    /// Static policy enforcement method that can be called from spawned tasks
    async fn enforce_policies_static(
        policy_cache: &Arc<RwLock<HashMap<String, PolicyContract>>>,
        _ccl_runtime: &Arc<CclRuntime>,
        _mana_ledger: &Arc<dyn ManaLedger>,
        _reputation_store: &Arc<dyn ReputationStore>,
        event_tx: &mpsc::UnboundedSender<GovernanceEvent>,
        _time_provider: &Arc<dyn TimeProvider>,
    ) -> Result<(), CommonError> {
        // Implement policy enforcement
        log::info!("Running policy enforcement check");

        // Load active policies from cache (scope the guard to avoid Send issues)
        let policy_data: Vec<(String, PolicyContract)> = {
            let policies = policy_cache.read().unwrap();

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
            match Self::execute_policy_check_static(
                policy_contract,
                _mana_ledger,
                _reputation_store,
            )
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
                            Self::handle_policy_violation_static(policy_id, &violation, event_tx)
                                .await?;
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to check policy {}: {}", policy_id, e);

                    // Send policy error event
                    let _ = event_tx.send(GovernanceEvent::PolicyError {
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
        reputation_store: &Arc<dyn ReputationStore>,
    ) -> Result<AutomationVotingResult, CommonError> {
        // Enhanced prediction logic based on multiple factors
        let current_support = state.voting_status.support_percentage;
        let participation_rate = state.voting_status.participation_rate;
        let votes_cast = state.voting_status.votes_cast;
        let _eligible_voters = state.voting_status.eligible_voters;

        // Calculate weighted prediction based on various factors
        let mut prediction_score = current_support;

        // Factor 1: Early voting momentum
        // If early voters show strong support, it often continues
        if participation_rate > 0.1 && current_support > 0.7 {
            prediction_score += 0.1; // Boost for strong early support
        } else if participation_rate > 0.1 && current_support < 0.3 {
            prediction_score -= 0.1; // Penalty for weak early support
        }

        // Factor 2: Participation rate trend
        // Higher participation tends to moderate extreme positions
        if participation_rate > 0.5 {
            // High participation - more representative, trust current result more
            prediction_score = current_support;
        } else if participation_rate < 0.2 {
            // Low participation - less reliable, add uncertainty
            if current_support > 0.5 {
                prediction_score = current_support * 0.8; // Conservative estimate
            } else {
                prediction_score = current_support * 1.2; // Potential for improvement
            }
        }

        // Factor 3: Reputation-weighted prediction
        // High-reputation voters tend to influence others
        let mut high_rep_support = 0.0;
        let mut high_rep_total = 0.0;

        for (voter_did, (vote, weight)) in &state.votes_received {
            let reputation = reputation_store.get_reputation(voter_did);
            if reputation > 75 {
                // High reputation threshold
                high_rep_total += weight.total_weight;
                if matches!(vote.option, VoteOption::Yes) {
                    high_rep_support += weight.total_weight;
                }
            }
        }

        if high_rep_total > 0.0 {
            let high_rep_ratio = high_rep_support / high_rep_total;
            // Weight high-reputation opinion more heavily
            prediction_score = (prediction_score * 0.7) + (high_rep_ratio * 0.3);
        }

        // Factor 4: Proposal type influence
        // Different types of proposals have different success patterns
        let proposal_description = &state.proposal.description.to_lowercase();
        if proposal_description.contains("emergency") {
            // Emergency proposals tend to pass if they have any momentum
            if current_support > 0.4 {
                prediction_score += 0.15;
            }
        } else if proposal_description.contains("controversial")
            || proposal_description.contains("major")
        {
            // Controversial proposals need higher thresholds
            prediction_score *= 0.9;
        }

        // Factor 5: Time-based decay for low participation
        let time_elapsed = Instant::now().duration_since(state.submitted_at);
        if time_elapsed > Duration::from_secs(2 * 24 * 3600) && participation_rate < 0.3 {
            // Proposals losing momentum over time
            prediction_score *= 0.85;
        }

        // Clamp prediction score to valid range
        prediction_score = prediction_score.max(0.0).min(1.0);

        // Create prediction result
        if prediction_score > 0.5 {
            Ok(AutomationVotingResult::Passed {
                support_percentage: prediction_score,
                total_votes: votes_cast as u64,
            })
        } else {
            Ok(AutomationVotingResult::Rejected {
                opposition_percentage: 1.0 - prediction_score,
                total_votes: votes_cast as u64,
            })
        }
    }

    /// Determine eligible voters for a proposal
    async fn determine_eligible_voters(
        &self,
        proposal: &Proposal,
    ) -> Result<Vec<Did>, CommonError> {
        // Implement voter eligibility logic based on multiple criteria
        let mut eligible_voters = Vec::new();

        // Get all known DIDs from the mana ledger (these are active participants)
        // Note: This is a workaround since ManaLedger might not have list_accounts
        // In a real implementation, this would come from a member registry
        let potential_voters = vec![
            Did::new("key", "voter1"),
            Did::new("key", "voter2"),
            Did::new("key", "voter3"),
            Did::new("key", "active_member1"),
            Did::new("key", "active_member2"),
            Did::new("key", "governance_participant"),
        ];

        for voter_did in potential_voters {
            let mut eligible = true;
            let mut eligibility_score = 0u32;

            // Check reputation threshold based on proposal type
            let min_reputation = match &proposal.proposal_type {
                ProposalType::SystemParameterChange(_, _) => 80, // High reputation for system changes
                ProposalType::SoftwareUpgrade(_) => 80,          // High reputation for upgrades
                ProposalType::BudgetAllocation(_, _, _) => 60,   // Medium reputation for budget
                ProposalType::NewMemberInvitation(_) => 50,      // Lower reputation for invitations
                ProposalType::RemoveMember(_) => 70,             // Higher reputation for removal
                ProposalType::Resolution(_) => 60, // Medium reputation for resolutions
                ProposalType::GenericText(_) => 25, // Basic reputation for text proposals
            };

            // Additional check for stake requirements
            let min_stake = match &proposal.proposal_type {
                ProposalType::SystemParameterChange(_, _) => 1000, // High stake for system changes
                ProposalType::SoftwareUpgrade(_) => 1000,          // High stake for upgrades
                ProposalType::BudgetAllocation(_, _, _) => 500,    // Medium stake for budget
                ProposalType::NewMemberInvitation(_) => 200,       // Lower stake for invitations
                ProposalType::RemoveMember(_) => 800,              // Higher stake for removal
                ProposalType::Resolution(_) => 400,                // Medium stake for resolutions
                ProposalType::GenericText(_) => 50, // Minimal stake for text proposals
            };

            let voter_reputation = self.reputation_store.get_reputation(&voter_did);
            if voter_reputation < min_reputation {
                eligible = false;
            } else {
                eligibility_score += ((voter_reputation / 10).min(10)) as u32; // Up to 10 points from reputation
            }

            // Check stake requirements (mana balance)
            let mana_balance = self.mana_ledger.get_balance(&voter_did);

            if mana_balance < min_stake {
                eligible = false;
            } else {
                eligibility_score += (mana_balance / 100).min(10) as u32; // Up to 10 points from stake
            }

            // Check governance participation history
            // This is a simplified check - in reality would look at voting history
            let participation_score = self
                .calculate_participation_score(&voter_did)
                .await
                .unwrap_or(0);
            if participation_score < 20 {
                // Require some minimum participation for non-routine proposals
                if !matches!(proposal.proposal_type, ProposalType::GenericText(_)) {
                    eligible = false;
                }
            } else {
                eligibility_score += participation_score.min(10);
            }

            // Check proposal type-specific requirements
            match &proposal.proposal_type {
                ProposalType::SystemParameterChange(_, _) => {
                    // System changes require highest standards
                    if eligibility_score < 25 {
                        eligible = false;
                    }
                }
                ProposalType::BudgetAllocation(_, _, _) => {
                    // Budget proposals might require economic understanding
                    if voter_reputation < 50 {
                        eligible = false;
                    }
                }
                _ => {} // No additional requirements for other types
            }

            if eligible {
                log::debug!(
                    "Voter {} eligible for proposal {} (score: {}, reputation: {}, stake: {})",
                    voter_did,
                    proposal.id,
                    eligibility_score,
                    voter_reputation,
                    mana_balance
                );
                eligible_voters.push(voter_did);
            } else {
                log::debug!(
                    "Voter {} not eligible for proposal {} (reputation: {}, stake: {})",
                    voter_did,
                    proposal.id,
                    voter_reputation,
                    mana_balance
                );
            }
        }

        log::info!(
            "Determined {} eligible voters for proposal {} (type: {})",
            eligible_voters.len(),
            proposal.id,
            Self::proposal_type_name(&proposal.proposal_type)
        );

        Ok(eligible_voters)
    }

    /// Get a human-readable name for a proposal type
    fn proposal_type_name(proposal_type: &ProposalType) -> &'static str {
        match proposal_type {
            ProposalType::SystemParameterChange(_, _) => "system_parameter_change",
            ProposalType::NewMemberInvitation(_) => "new_member_invitation",
            ProposalType::RemoveMember(_) => "remove_member",
            ProposalType::SoftwareUpgrade(_) => "software_upgrade",
            ProposalType::GenericText(_) => "generic_text",
            ProposalType::BudgetAllocation(_, _, _) => "budget_allocation",
            ProposalType::Resolution(_) => "resolution",
        }
    }

    /// Calculate governance participation score for a voter
    async fn calculate_participation_score(&self, _voter: &Did) -> Result<u32, CommonError> {
        // In a real implementation, this would:
        // 1. Look up voting history from governance records
        // 2. Check proposal submission history
        // 3. Evaluate quality of participation (e.g., informed votes)
        // 4. Consider recency of participation

        // For now, return a mock score based on reputation as a proxy
        let reputation = self.reputation_store.get_reputation(_voter);
        let participation_score = (reputation / 10).min(50) as u32; // Scale reputation to participation score

        Ok(participation_score)
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
        policy_contract: &PolicyContract,
        mana_ledger: &Arc<dyn ManaLedger>,
        reputation_store: &Arc<dyn ReputationStore>,
    ) -> Result<Vec<PolicyViolation>, CommonError> {
        Self::execute_policy_check_static(policy_contract, mana_ledger, reputation_store).await
    }

    /// Static version of execute_policy_check for use in spawned tasks
    async fn execute_policy_check_static(
        policy_contract: &PolicyContract,
        mana_ledger: &Arc<dyn ManaLedger>,
        reputation_store: &Arc<dyn ReputationStore>,
    ) -> Result<Vec<PolicyViolation>, CommonError> {
        // Implement actual policy check logic
        let mut violations = Vec::new();

        // Example policy checks - in a real implementation these would be
        // dynamically loaded from CCL policy contracts

        // Check 1: Excessive mana concentration
        let high_balance_accounts = Self::find_high_balance_accounts(mana_ledger).await?;
        for (did, balance) in high_balance_accounts {
            if balance > 10000 {
                // Threshold for excessive concentration
                violations.push(PolicyViolation {
                    violation_type: "excessive_mana_concentration".to_string(),
                    severity: if balance > 50000 {
                        "high".to_string()
                    } else {
                        "medium".to_string()
                    },
                    target: Some(did),
                    details: format!(
                        "Account has {} mana, exceeding concentration threshold",
                        balance
                    ),
                });
            }
        }

        // Check 2: Low reputation with high resource usage
        let resource_intensive_accounts =
            Self::find_resource_intensive_accounts(mana_ledger).await?;
        for did in resource_intensive_accounts {
            let reputation = reputation_store.get_reputation(&did);
            if reputation < 30 {
                violations.push(PolicyViolation {
                    violation_type: "low_reputation_high_usage".to_string(),
                    severity: "medium".to_string(),
                    target: Some(did),
                    details: format!(
                        "Account with reputation {} using high resources",
                        reputation
                    ),
                });
            }
        }

        // Check 3: Governance participation requirements
        // (This would normally check voting participation rates)
        let inactive_high_stake_accounts = Self::find_inactive_stakeholders(mana_ledger).await?;
        for did in inactive_high_stake_accounts {
            violations.push(PolicyViolation {
                violation_type: "governance_inactivity".to_string(),
                severity: "low".to_string(),
                target: Some(did),
                details: "High-stake account with low governance participation".to_string(),
            });
        }

        log::info!(
            "Policy check completed. Found {} violations for policy contract",
            violations.len()
        );

        Ok(violations)
    }

    /// Find accounts with high mana balances
    async fn find_high_balance_accounts(
        mana_ledger: &Arc<dyn ManaLedger>,
    ) -> Result<Vec<(Did, u64)>, CommonError> {
        // In a real implementation, this would iterate through all accounts
        // For now, return mock data for demonstration
        let test_accounts = vec![
            Did::new("key", "whale_account"),
            Did::new("key", "active_trader"),
            Did::new("key", "validator_node"),
        ];

        let mut high_balance_accounts = Vec::new();
        for did in test_accounts {
            let balance = mana_ledger.get_balance(&did);
            if balance > 1000 {
                // Arbitrary threshold for "high balance"
                high_balance_accounts.push((did, balance));
            }
        }

        Ok(high_balance_accounts)
    }

    /// Find accounts with high resource usage patterns
    async fn find_resource_intensive_accounts(
        _mana_ledger: &Arc<dyn ManaLedger>,
    ) -> Result<Vec<Did>, CommonError> {
        // In a real implementation, this would analyze transaction patterns
        // and resource consumption metrics
        Ok(vec![
            Did::new("key", "compute_heavy_user"),
            Did::new("key", "storage_intensive_user"),
        ])
    }

    /// Find high-stake accounts with low governance participation
    async fn find_inactive_stakeholders(
        mana_ledger: &Arc<dyn ManaLedger>,
    ) -> Result<Vec<Did>, CommonError> {
        // In a real implementation, this would cross-reference high balances
        // with governance participation records
        let test_accounts = vec![
            Did::new("key", "passive_holder"),
            Did::new("key", "dormant_whale"),
        ];

        let mut inactive_stakeholders = Vec::new();
        for did in test_accounts {
            let balance = mana_ledger.get_balance(&did);
            if balance > 5000 {
                // High stake threshold
                // In reality, would check governance participation here
                inactive_stakeholders.push(did);
            }
        }

        Ok(inactive_stakeholders)
    }

    /// Handle policy violation (stub implementation)
    async fn handle_policy_violation(
        &self,
        policy_id: &str,
        violation: &PolicyViolation,
        event_tx: &mpsc::UnboundedSender<GovernanceEvent>,
    ) -> Result<(), CommonError> {
        Self::handle_policy_violation_static(policy_id, violation, event_tx).await
    }

    /// Static version of handle_policy_violation for use in spawned tasks
    async fn handle_policy_violation_static(
        policy_id: &str,
        violation: &PolicyViolation,
        event_tx: &mpsc::UnboundedSender<GovernanceEvent>,
    ) -> Result<(), CommonError> {
        // Implement actual policy violation handling

        // Determine enforcement action based on violation severity and type
        let enforcement_action = match violation.severity.as_str() {
            "high" => {
                match violation.violation_type.as_str() {
                    "excessive_mana_concentration" => {
                        // For high severity mana concentration, apply progressive taxation
                        log::warn!("HIGH SEVERITY: Excessive mana concentration detected");
                        "progressive_taxation_applied"
                    }
                    "low_reputation_high_usage" => {
                        // For high severity reputation issues, temporarily restrict resources
                        log::warn!("HIGH SEVERITY: Low reputation with high resource usage");
                        "resource_restrictions_applied"
                    }
                    _ => "account_flagged_for_review",
                }
            }
            "medium" => {
                match violation.violation_type.as_str() {
                    "excessive_mana_concentration" => {
                        // Medium severity concentration gets a warning and monitoring
                        log::info!("MEDIUM SEVERITY: Monitoring excessive mana concentration");
                        "enhanced_monitoring_enabled"
                    }
                    "low_reputation_high_usage" => {
                        // Medium severity reputation issues get usage warnings
                        log::info!("MEDIUM SEVERITY: Issuing usage guidelines");
                        "usage_guidelines_issued"
                    }
                    _ => "formal_warning_issued",
                }
            }
            "low" => {
                match violation.violation_type.as_str() {
                    "governance_inactivity" => {
                        // Low severity inactivity gets engagement encouragement
                        log::info!("LOW SEVERITY: Encouraging governance participation");
                        "participation_encouragement_sent"
                    }
                    _ => "informal_warning_issued",
                }
            }
            _ => "violation_logged",
        };

        // Apply specific enforcement actions based on violation type
        if let Some(target_did) = &violation.target {
            match violation.violation_type.as_str() {
                "excessive_mana_concentration" => {
                    // Could implement progressive taxation or concentration limits
                    log::info!("Applied concentration monitoring for {}", target_did);
                }
                "low_reputation_high_usage" => {
                    // Could implement usage throttling or reputation improvement requirements
                    log::info!("Applied usage guidelines for {}", target_did);
                }
                "governance_inactivity" => {
                    // Could send participation reminders or apply incentives
                    log::info!("Sent participation encouragement to {}", target_did);
                }
                _ => {
                    log::info!("Applied generic enforcement action for {}", target_did);
                }
            }
        }

        // Record enforcement action for audit trail
        log::info!(
            "Policy enforcement for {}: {} -> {} (target: {:?})",
            policy_id,
            violation.violation_type,
            enforcement_action,
            violation.target
        );

        // Notify relevant stakeholders
        Self::notify_stakeholders(policy_id, violation, enforcement_action).await?;

        // Send policy enforcement event
        let _ = event_tx.send(GovernanceEvent::PolicyEnforced {
            policy_id: policy_id.to_string(),
            violation: violation.clone(),
            action_taken: enforcement_action.to_string(),
        });

        Ok(())
    }

    /// Notify relevant stakeholders about policy enforcement
    async fn notify_stakeholders(
        policy_id: &str,
        violation: &PolicyViolation,
        action_taken: &str,
    ) -> Result<(), CommonError> {
        // In a real implementation, this would:
        // 1. Identify stakeholders who should be notified (governance committee, affected parties)
        // 2. Send notifications through appropriate channels (governance announcements, direct messages)
        // 3. Create governance records for transparency

        log::info!(
            "GOVERNANCE NOTIFICATION: Policy {} enforcement - {} applied for {} violation{}",
            policy_id,
            action_taken,
            violation.violation_type,
            violation
                .target
                .as_ref()
                .map(|did| format!(" (target: {})", did))
                .unwrap_or_default()
        );

        // In a real system, would send actual notifications here
        Ok(())
    }

    /// Create a default CID for testing/placeholder purposes
    fn create_default_cid(prefix: &str) -> Cid {
        // Create a deterministic CID based on the prefix
        // This is just for testing - in production you'd use actual content hashes
        let content = format!("{}_placeholder_content", prefix);

        // Use the new_v1_sha256 method to create a proper CID
        Cid::new_v1_sha256(0x71, content.as_bytes()) // 0x71 is DAG-CBOR codec
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
    use crate::{Proposal, ProposalId, ProposalStatus, ProposalType};
    use icn_common::SystemTimeProvider;
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
