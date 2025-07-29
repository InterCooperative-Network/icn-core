//! Economic automation for ICN
//!
//! This module provides automated economic management including mana allocation,
//! policy enforcement, dynamic pricing, and economic health monitoring.

use crate::{ManaLedger, ResourceLedger};
use icn_common::{CommonError, Did, TimeProvider};
use icn_reputation::ReputationStore;
use std::str::FromStr;
// Temporarily simplified to avoid circular dependencies
// use icn_governance::{GovernanceModule, Proposal};
// use icn_mesh::{MeshJob, JobBid};
use icn_common::DagBlock;
use icn_dag::StorageService;

// Simplified types to avoid circular dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshJob {
    pub job_id: String,
    pub job_type: Option<String>,
    pub command: String,
    pub estimated_cost: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobBid {
    pub bidder: Did,
    pub cost_bid: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: String,
    pub title: String,
    pub description: String,
}

pub trait GovernanceModule: Send + Sync {
    fn get_proposal(&self, id: &str) -> Result<Option<Proposal>, CommonError>;
}
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Mutex as TokioMutex};

/// Resource pricing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePricing {
    pub base_price: u64,
    pub current_price: u64,
    pub demand_multiplier: f64,
    pub last_updated: u64,
}

/// Allocation optimization suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationOptimization {
    pub resource_type: String,
    pub optimization_type: OptimizationType,
    pub current_allocation: u64,
    pub suggested_allocation: u64,
    pub efficiency_gain: f64,
}

/// Types of allocation optimizations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationType {
    ReduceAllocation,
    IncreaseAllocation,
    Redistribute,
}

/// Resource allocation metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationMetrics {
    pub allocated_amount: u64,
    pub utilization_rate: f64,
    pub efficiency_score: f64,
}

/// Resource access restriction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRestriction {
    /// The resource being restricted
    pub restricted_resource: String,
    /// Severity of the restriction (0.0 to 1.0)
    pub severity: f64,
    /// When the restriction expires (None for permanent, Unix timestamp)
    pub end_time: Option<u64>,
    /// Reason for the restriction
    pub reason: String,
}

/// Configuration for economic automation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicAutomationConfig {
    /// Enable automatic mana regeneration
    pub enable_mana_regeneration: bool,
    /// Base mana regeneration rate per second
    pub base_regeneration_rate: f64,
    /// Maximum mana capacity multiplier
    pub max_capacity_multiplier: f64,
    /// Enable dynamic pricing
    pub enable_dynamic_pricing: bool,
    /// Price adjustment speed (0.0 to 1.0)
    pub price_adjustment_speed: f64,
    /// Enable automatic resource allocation
    pub enable_resource_allocation: bool,
    /// Resource allocation optimization interval
    pub allocation_optimization_interval: Duration,
    /// Enable economic policy enforcement
    pub enable_policy_enforcement: bool,
    /// Policy enforcement strictness (0.0 to 1.0)
    pub enforcement_strictness: f64,
    /// Enable predictive economic modeling
    pub enable_predictive_modeling: bool,
    /// Economic health check interval
    pub health_check_interval: Duration,
    /// Enable automatic market making
    pub enable_market_making: bool,
    /// Market making spread percentage
    pub market_making_spread: f64,
}

impl Default for EconomicAutomationConfig {
    fn default() -> Self {
        Self {
            enable_mana_regeneration: true,
            base_regeneration_rate: 0.01, // 1% per second base rate
            max_capacity_multiplier: 2.0,
            enable_dynamic_pricing: true,
            price_adjustment_speed: 0.1,
            enable_resource_allocation: true,
            allocation_optimization_interval: Duration::from_secs(300), // 5 minutes
            enable_policy_enforcement: true,
            enforcement_strictness: 0.8,
            enable_predictive_modeling: true,
            health_check_interval: Duration::from_secs(60), // 1 minute
            enable_market_making: true,
            market_making_spread: 0.02, // 2% spread
        }
    }
}

/// Types of economic events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EconomicEvent {
    /// Mana regeneration occurred
    ManaRegenerated {
        account: Did,
        amount: u64,
        new_balance: u64,
        regeneration_rate: f64,
        timestamp: u64,
    },
    /// Dynamic price adjustment
    PriceAdjusted {
        resource_type: String,
        old_price: f64,
        new_price: f64,
        adjustment_reason: PriceAdjustmentReason,
        timestamp: u64,
    },
    /// Resource allocation optimized
    ResourceAllocated {
        allocation_id: String,
        resource_type: String,
        amount: u64,
        recipient: Did,
        allocation_strategy: AllocationStrategy,
        timestamp: u64,
    },
    /// Economic policy violation
    PolicyViolation {
        violator: Did,
        policy_id: String,
        violation_type: ViolationType,
        penalty_applied: Option<EconomicPenalty>,
        timestamp: u64,
    },
    /// Economic threshold reached
    ThresholdReached {
        threshold_type: ThresholdType,
        current_value: f64,
        threshold_value: f64,
        action_taken: Option<AutomaticAction>,
        timestamp: u64,
    },
    /// Market transaction executed
    MarketTransaction {
        transaction_id: String,
        buyer: Did,
        seller: Did,
        resource_type: String,
        amount: u64,
        price: f64,
        timestamp: u64,
    },
}

/// Reasons for price adjustments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PriceAdjustmentReason {
    /// Supply and demand imbalance
    SupplyDemandImbalance { supply: f64, demand: f64 },
    /// Network congestion
    NetworkCongestion { congestion_level: f64 },
    /// Quality of service changes
    QualityChange { old_quality: f64, new_quality: f64 },
    /// Competition adjustment
    Competition { competitor_prices: Vec<f64> },
    /// Economic policy directive
    PolicyDirective { policy_id: String },
}

/// Resource allocation strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllocationStrategy {
    /// Fair allocation based on need
    FairAllocation,
    /// Merit-based allocation using reputation
    MeritBased { reputation_weight: f64 },
    /// Contribution-based allocation
    ContributionBased { contribution_weight: f64 },
    /// Lottery-based random allocation
    Lottery { lottery_weight: f64 },
    /// Hybrid strategy combining multiple approaches
    Hybrid { strategies: Vec<AllocationStrategy> },
}

/// Types of policy violations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationType {
    /// Excessive resource consumption
    ExcessiveConsumption { consumed: u64, limit: u64 },
    /// Unfair pricing practices
    UnfairPricing { price: f64, fair_range: (f64, f64) },
    /// Market manipulation
    MarketManipulation { evidence: String },
    /// Hoarding resources
    ResourceHoarding { hoarded: u64, threshold: u64 },
    /// Anti-competitive behavior
    AntiCompetitive { behavior_type: String },
}

/// Economic penalties for violations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicPenalty {
    /// Type of penalty
    pub penalty_type: PenaltyType,
    /// Severity level (0.0 to 1.0)
    pub severity: f64,
    /// Duration of penalty
    pub duration: Option<Duration>,
    /// Amount of penalty (mana, tokens, etc.)
    pub amount: Option<u64>,
    /// Additional restrictions
    pub restrictions: Vec<String>,
}

/// Types of economic penalties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PenaltyType {
    /// Mana penalty
    ManaPenalty,
    /// Token confiscation
    TokenConfiscation,
    /// Resource access restriction
    ResourceRestriction,
    /// Market participation ban
    MarketBan,
    /// Reputation penalty
    ReputationPenalty,
    /// Warning only
    Warning,
}

/// Economic threshold types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThresholdType {
    /// Total mana supply threshold
    TotalManaSupply,
    /// Average transaction volume
    TransactionVolume,
    /// Resource utilization rate
    ResourceUtilization,
    /// Price volatility level
    PriceVolatility,
    /// Economic inequality measure
    EconomicInequality,
    /// Network fee level
    NetworkFees,
}

/// Automatic actions taken when thresholds are reached
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AutomaticAction {
    /// Adjust mana regeneration rates
    AdjustRegenerationRates { new_rate: f64 },
    /// Implement emergency resource allocation
    EmergencyAllocation { allocation_amount: u64 },
    /// Activate price controls
    PriceControls { max_price: f64, min_price: f64 },
    /// Trigger governance proposal
    GovernanceProposal { proposal_type: String },
    /// Implement circuit breakers
    CircuitBreaker { duration: Duration },
}

/// Economic health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicHealthMetrics {
    /// Overall economic health score (0.0 to 1.0)
    pub overall_health: f64,
    /// Mana distribution inequality (Gini coefficient)
    pub mana_inequality: f64,
    /// Resource utilization efficiency
    pub resource_efficiency: f64,
    /// Market liquidity level
    pub market_liquidity: f64,
    /// Price stability index
    pub price_stability: f64,
    /// Economic activity level
    pub activity_level: f64,
    /// Last update timestamp
    pub last_updated: u64,
}

/// Dynamic pricing model
#[derive(Debug, Clone)]
pub struct DynamicPricingModel {
    /// Base price for the resource
    pub base_price: f64,
    /// Current market price
    pub current_price: f64,
    /// Price history
    pub price_history: VecDeque<(Instant, f64)>,
    /// Supply and demand factors
    pub supply_demand_ratio: f64,
    /// Quality adjustment factor
    pub quality_factor: f64,
    /// Competition factor
    pub competition_factor: f64,
    /// Last price update
    pub last_updated: Instant,
}

/// Resource allocation plan
#[derive(Debug, Clone)]
pub struct ResourceAllocationPlan {
    /// Allocation identifier
    pub allocation_id: String,
    /// Resource type being allocated
    pub resource_type: String,
    /// Total amount available for allocation
    pub total_available: u64,
    /// Allocation strategy used
    pub strategy: AllocationStrategy,
    /// Individual allocations
    pub allocations: HashMap<Did, AllocationEntry>,
    /// Creation timestamp
    pub created_at: Instant,
    /// Execution status
    pub status: AllocationStatus,
}

/// Individual allocation entry
#[derive(Debug, Clone)]
pub struct AllocationEntry {
    /// Recipient DID
    pub recipient: Did,
    /// Allocated amount
    pub amount: u64,
    /// Allocation score/priority
    pub score: f64,
    /// Justification for allocation
    pub justification: String,
    /// Allocation conditions
    pub conditions: Vec<String>,
}

/// Status of resource allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllocationStatus {
    /// Planning phase
    Planning,
    /// Ready for execution
    Ready,
    /// Currently executing
    Executing,
    /// Successfully completed
    Completed,
    /// Failed during execution
    Failed { reason: String },
    /// Cancelled before execution
    Cancelled { reason: String },
}

/// Market making configuration
#[derive(Debug, Clone)]
pub struct MarketMakingConfig {
    /// Target spread percentage
    pub target_spread: f64,
    /// Maximum position size
    pub max_position_size: u64,
    /// Inventory target levels
    pub inventory_targets: HashMap<String, u64>,
    /// Risk management parameters
    pub risk_parameters: RiskParameters,
}

/// Risk management parameters for market making
#[derive(Debug, Clone)]
pub struct RiskParameters {
    /// Maximum loss per trade
    pub max_loss_per_trade: f64,
    /// Maximum daily loss
    pub max_daily_loss: f64,
    /// Stop loss threshold
    pub stop_loss_threshold: f64,
    /// Position size limits
    pub position_limits: HashMap<String, u64>,
}

/// Comprehensive economic automation engine
pub struct EconomicAutomationEngine {
    config: EconomicAutomationConfig,
    mana_ledger: Arc<dyn ManaLedger>,
    resource_ledger: Arc<dyn ResourceLedger>,
    reputation_store: Arc<dyn ReputationStore>,
    #[allow(dead_code)]
    governance_module: Arc<TokioMutex<dyn GovernanceModule>>,
    #[allow(dead_code)]
    dag_store: Arc<TokioMutex<dyn StorageService<DagBlock>>>,
    time_provider: Arc<dyn TimeProvider>,

    // Economic state
    mana_accounts: Arc<RwLock<HashMap<Did, ManaAccountState>>>,
    pricing_models: Arc<RwLock<HashMap<String, DynamicPricingModel>>>,
    allocation_plans: Arc<RwLock<HashMap<String, ResourceAllocationPlan>>>,
    economic_policies: Arc<RwLock<HashMap<String, EconomicPolicy>>>,
    health_metrics: Arc<RwLock<EconomicHealthMetrics>>,
    market_making_state: Arc<RwLock<MarketMakingState>>,

    // Event handling
    event_tx: mpsc::UnboundedSender<EconomicEvent>,
    event_rx: Option<mpsc::UnboundedReceiver<EconomicEvent>>,

    // Resource restrictions tracking
    resource_restrictions: Arc<RwLock<HashMap<(Did, String), ResourceRestriction>>>,

    // Background tasks
    automation_handles: Arc<RwLock<Vec<tokio::task::JoinHandle<()>>>>,
}

/// State of a mana account
#[derive(Debug, Clone)]
pub struct ManaAccountState {
    /// Account DID
    pub account: Did,
    /// Current mana balance
    pub balance: u64,
    /// Maximum mana capacity
    pub capacity: u64,
    /// Current regeneration rate
    pub regeneration_rate: f64,
    /// Last regeneration time
    pub last_regeneration: Instant,
    /// Reputation-based bonuses
    pub reputation_bonus: f64,
    /// Usage history
    pub usage_history: VecDeque<(Instant, u64, String)>,
}

/// Economic policy definition
#[derive(Debug, Clone)]
pub struct EconomicPolicy {
    /// Policy identifier
    pub policy_id: String,
    /// Policy type
    pub policy_type: PolicyType,
    /// Policy parameters
    pub parameters: HashMap<String, f64>,
    /// Enforcement level
    pub enforcement_level: f64,
    /// Last update time
    pub last_updated: Instant,
    /// Policy status
    pub status: PolicyStatus,
}

/// Types of economic policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyType {
    /// Mana regeneration policy
    ManaRegeneration,
    /// Resource allocation policy
    ResourceAllocation,
    /// Pricing policy
    Pricing,
    /// Market behavior policy
    MarketBehavior,
    /// Anti-manipulation policy
    AntiManipulation,
}

/// Status of economic policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyStatus {
    /// Policy is active
    Active,
    /// Policy is suspended
    Suspended { reason: String },
    /// Policy is being updated
    Updating,
    /// Policy is deprecated
    Deprecated { replacement: Option<String> },
}

/// Market making state
#[derive(Debug, Clone)]
pub struct MarketMakingState {
    /// Active market making positions
    pub positions: HashMap<String, MarketPosition>,
    /// Inventory levels
    pub inventory: HashMap<String, u64>,
    /// Performance metrics
    pub performance: MarketMakingPerformance,
    /// Risk metrics
    pub risk_metrics: RiskMetrics,
}

/// Market making position
#[derive(Debug, Clone)]
pub struct MarketPosition {
    /// Position identifier
    pub position_id: String,
    /// Resource type
    pub resource_type: String,
    /// Position size
    pub size: u64,
    /// Entry price
    pub entry_price: f64,
    /// Current market price
    pub current_price: f64,
    /// Unrealized P&L
    pub unrealized_pnl: f64,
    /// Position timestamp
    pub timestamp: Instant,
}

/// Market making performance metrics
#[derive(Debug, Clone, Default)]
pub struct MarketMakingPerformance {
    /// Total trades executed
    pub total_trades: u64,
    /// Total volume traded
    pub total_volume: u64,
    /// Total profit/loss
    pub total_pnl: f64,
    /// Success rate
    pub success_rate: f64,
    /// Average spread captured
    pub avg_spread_captured: f64,
    /// Inventory turnover rate
    pub inventory_turnover: f64,
}

/// Risk metrics for market making
#[derive(Debug, Clone, Default)]
pub struct RiskMetrics {
    /// Current value at risk
    pub value_at_risk: f64,
    /// Maximum drawdown
    pub max_drawdown: f64,
    /// Position concentration risk
    pub concentration_risk: f64,
    /// Liquidity risk
    pub liquidity_risk: f64,
}

impl EconomicAutomationEngine {
    /// Create a new economic automation engine
    pub fn new(
        config: EconomicAutomationConfig,
        mana_ledger: Arc<dyn ManaLedger>,
        resource_ledger: Arc<dyn ResourceLedger>,
        reputation_store: Arc<dyn ReputationStore>,
        governance_module: Arc<TokioMutex<dyn GovernanceModule>>,
        dag_store: Arc<TokioMutex<dyn StorageService<DagBlock>>>,
        time_provider: Arc<dyn TimeProvider>,
    ) -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        Self {
            config,
            mana_ledger,
            resource_ledger,
            reputation_store,
            governance_module,
            dag_store,
            time_provider,
            mana_accounts: Arc::new(RwLock::new(HashMap::new())),
            pricing_models: Arc::new(RwLock::new(HashMap::new())),
            allocation_plans: Arc::new(RwLock::new(HashMap::new())),
            economic_policies: Arc::new(RwLock::new(HashMap::new())),
            health_metrics: Arc::new(RwLock::new(EconomicHealthMetrics {
                overall_health: 1.0,
                mana_inequality: 0.3,
                resource_efficiency: 0.8,
                market_liquidity: 0.7,
                price_stability: 0.9,
                activity_level: 0.6,
                last_updated: 0,
            })),
            market_making_state: Arc::new(RwLock::new(MarketMakingState {
                positions: HashMap::new(),
                inventory: HashMap::new(),
                performance: MarketMakingPerformance::default(),
                risk_metrics: RiskMetrics::default(),
            })),
            event_tx,
            event_rx: Some(event_rx),
            resource_restrictions: Arc::new(RwLock::new(HashMap::new())),
            automation_handles: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Start the economic automation engine
    pub async fn start(&mut self) -> Result<(), CommonError> {
        log::info!("Starting economic automation engine");

        // Start mana regeneration if enabled
        let mana_handle = if self.config.enable_mana_regeneration {
            Some(self.start_mana_regeneration().await?)
        } else {
            None
        };

        // Start dynamic pricing if enabled
        let pricing_handle = if self.config.enable_dynamic_pricing {
            Some(self.start_dynamic_pricing().await?)
        } else {
            None
        };

        // Start resource allocation if enabled
        let allocation_handle = if self.config.enable_resource_allocation {
            Some(self.start_resource_allocation().await?)
        } else {
            None
        };

        // Start policy enforcement if enabled
        let policy_handle = if self.config.enable_policy_enforcement {
            Some(self.start_policy_enforcement().await?)
        } else {
            None
        };

        // Start economic health monitoring
        let health_handle = self.start_health_monitoring().await?;

        // Start market making if enabled
        let market_handle = if self.config.enable_market_making {
            Some(self.start_market_making().await?)
        } else {
            None
        };

        // Start predictive modeling if enabled
        let modeling_handle = if self.config.enable_predictive_modeling {
            Some(self.start_predictive_modeling().await?)
        } else {
            None
        };

        // Store handles
        let mut handles = self.automation_handles.write().unwrap();
        if let Some(handle) = mana_handle {
            handles.push(handle);
        }
        if let Some(handle) = pricing_handle {
            handles.push(handle);
        }
        if let Some(handle) = allocation_handle {
            handles.push(handle);
        }
        if let Some(handle) = policy_handle {
            handles.push(handle);
        }
        handles.push(health_handle);
        if let Some(handle) = market_handle {
            handles.push(handle);
        }
        if let Some(handle) = modeling_handle {
            handles.push(handle);
        }

        log::info!("Economic automation engine started successfully");
        Ok(())
    }

    /// Stop the economic automation engine
    pub async fn stop(&self) -> Result<(), CommonError> {
        log::info!("Stopping economic automation engine");

        let handles = self.automation_handles.write().unwrap();
        for handle in handles.iter() {
            handle.abort();
        }

        log::info!("Economic automation engine stopped");
        Ok(())
    }

    /// Get event receiver for economic events
    pub fn take_event_receiver(&mut self) -> Option<mpsc::UnboundedReceiver<EconomicEvent>> {
        self.event_rx.take()
    }

    /// Calculate optimal mana price for a job
    pub async fn calculate_optimal_mana_price(&self, job: &MeshJob) -> Result<u64, CommonError> {
        // Get current pricing model
        let pricing_models = self.pricing_models.read().unwrap();
        let job_type = job
            .job_type
            .clone()
            .unwrap_or_else(|| "default".to_string());

        if let Some(model) = pricing_models.get(&job_type) {
            // Calculate price based on demand, quality, and market conditions
            let base_price = model.base_price;
            let demand_multiplier = self.calculate_demand_multiplier(&job_type).await?;
            let quality_multiplier = model.quality_factor;
            let competition_multiplier = model.competition_factor;

            let optimal_price =
                base_price * demand_multiplier * quality_multiplier * competition_multiplier;
            Ok(optimal_price as u64)
        } else {
            // Fallback to basic calculation
            self.calculate_basic_mana_price(job).await
        }
    }

    /// Execute resource allocation plan
    pub async fn execute_allocation_plan(
        &self,
        plan_id: &str,
    ) -> Result<AllocationExecutionResult, CommonError> {
        let mut allocation_plans = self.allocation_plans.write().unwrap();

        if let Some(plan) = allocation_plans.get_mut(plan_id) {
            plan.status = AllocationStatus::Executing;

            let mut successful_allocations = 0;
            let mut failed_allocations = 0;
            let mut total_allocated = 0;

            for (did, allocation) in &plan.allocations {
                match self.execute_individual_allocation(did, allocation).await {
                    Ok(amount) => {
                        successful_allocations += 1;
                        total_allocated += amount;

                        // Emit allocation event
                        let _ = self.event_tx.send(EconomicEvent::ResourceAllocated {
                            allocation_id: plan.allocation_id.clone(),
                            resource_type: plan.resource_type.clone(),
                            amount,
                            recipient: did.clone(),
                            allocation_strategy: plan.strategy.clone(),
                            timestamp: self.time_provider.unix_seconds(),
                        });
                    }
                    Err(e) => {
                        log::error!("Failed to allocate to {did}: {e}");
                        failed_allocations += 1;
                    }
                }
            }

            // Update plan status
            plan.status = if failed_allocations == 0 {
                AllocationStatus::Completed
            } else if successful_allocations > 0 {
                AllocationStatus::Completed // Partial success
            } else {
                AllocationStatus::Failed {
                    reason: "All allocations failed".to_string(),
                }
            };

            Ok(AllocationExecutionResult {
                plan_id: plan_id.to_string(),
                successful_allocations,
                failed_allocations,
                total_allocated,
                execution_time: Instant::now().duration_since(plan.created_at),
            })
        } else {
            Err(CommonError::InternalError(format!(
                "Allocation plan {plan_id} not found"
            )))
        }
    }

    /// Apply economic penalty for policy violation
    pub async fn apply_economic_penalty(
        &self,
        violator: &Did,
        penalty: &EconomicPenalty,
    ) -> Result<(), CommonError> {
        match penalty.penalty_type {
            PenaltyType::ManaPenalty => {
                if let Some(amount) = penalty.amount {
                    self.mana_ledger.spend(violator, amount)?;
                    log::info!("Applied mana penalty of {amount} to {violator}");
                }
            }
            PenaltyType::TokenConfiscation => {
                // Implement token confiscation
                if let Some(amount) = penalty.amount {
                    // Get system treasury account for confiscated tokens
                    // Use a well-formed DID that should always parse successfully
                    let system_did = Did::from_str("did:icn:treasury").map_err(|e| {
                        CommonError::InternalError(format!(
                            "Failed to create system treasury DID: {e}"
                        ))
                    })?;

                    // Verify system DID is different from violator to prevent self-transfer
                    if system_did == *violator {
                        return Err(CommonError::InternalError(
                            "Cannot confiscate tokens: system treasury DID matches violator DID"
                                .to_string(),
                        ));
                    }

                    // Get all token classes and check balances
                    let token_classes = self.resource_ledger.list_classes();
                    let mut total_confiscated = 0u64;

                    for (token_class, _class_metadata) in token_classes.iter() {
                        let balance = self.resource_ledger.get_balance(token_class, violator);
                        if balance > 0 {
                            let confiscate_amount =
                                amount.saturating_sub(total_confiscated).min(balance);
                            if confiscate_amount > 0 {
                                // Transfer to system treasury account for confiscated tokens
                                self.resource_ledger.transfer(
                                    token_class,
                                    violator,
                                    &system_did,
                                    confiscate_amount,
                                )?;
                                total_confiscated += confiscate_amount;
                                log::info!(
                                    "Confiscated {confiscate_amount} tokens of type {token_class:?} from {violator} to treasury {system_did}"
                                );

                                // If we've confiscated enough, stop
                                if total_confiscated >= amount {
                                    break;
                                }
                            }
                        }
                    }

                    if total_confiscated == 0 {
                        log::warn!(
                            "Token confiscation penalty: No tokens available to confiscate from {violator}"
                        );
                    } else {
                        log::info!(
                            "Successfully confiscated {total_confiscated} tokens from {violator} to treasury"
                        );
                    }
                } else {
                    log::warn!("Token confiscation penalty missing amount specification");
                }
            }
            PenaltyType::ResourceRestriction => {
                // Implement resource access restrictions
                let mut restriction_list = self.resource_restrictions.write().unwrap();

                // Add time-based restriction
                let end_time = penalty
                    .duration
                    .map(|duration| self.time_provider.unix_seconds() + duration.as_secs());

                // Apply restrictions from penalty
                for restriction in &penalty.restrictions {
                    restriction_list.insert(
                        (violator.clone(), restriction.clone()),
                        ResourceRestriction {
                            restricted_resource: restriction.clone(),
                            severity: penalty.severity,
                            end_time,
                            reason: "Economic policy violation".to_string(),
                        },
                    );
                    log::info!(
                        "Applied resource restriction '{}' to {} (severity: {:.2})",
                        restriction,
                        violator,
                        penalty.severity
                    );
                }

                // If no specific restrictions, apply general restrictions based on severity
                if penalty.restrictions.is_empty() {
                    let general_restrictions = match penalty.severity {
                        s if s >= 0.8 => vec!["compute", "storage", "network"],
                        s if s >= 0.5 => vec!["compute", "storage"],
                        _ => vec!["compute"],
                    };

                    for resource in general_restrictions {
                        restriction_list.insert(
                            (violator.clone(), resource.to_string()),
                            ResourceRestriction {
                                restricted_resource: resource.to_string(),
                                severity: penalty.severity,
                                end_time,
                                reason: "Economic policy violation".to_string(),
                            },
                        );
                        log::info!(
                            "Applied general resource restriction '{}' to {} (severity: {:.2})",
                            resource,
                            violator,
                            penalty.severity
                        );
                    }
                }
            }
            PenaltyType::ReputationPenalty => {
                // Apply reputation penalty by recording negative execution events
                let current_reputation = self.reputation_store.get_reputation(violator);

                if let Some(amount) = penalty.amount {
                    // Record failed executions to reduce reputation by the specified amount
                    let penalty_amount = amount.min(current_reputation); // Don't exceed current reputation

                    // Each failed execution typically reduces reputation, so we simulate multiple failures
                    // This is a workaround since we can't directly set reputation
                    let failures_to_record = (penalty_amount / 10).max(1); // Assume each failure reduces ~10 points

                    for _ in 0..failures_to_record {
                        self.reputation_store
                            .record_execution(violator, false, 1000); // Record failed execution
                    }

                    log::info!(
                        "Applied reputation penalty to {violator} by recording {failures_to_record} failed executions (estimated reduction: {penalty_amount})"
                    );
                } else {
                    // Apply severity-based reputation penalty
                    let penalty_percent = penalty.severity.min(1.0).max(0.0);
                    let estimated_penalty = (current_reputation as f64 * penalty_percent) as u64;

                    // Record failed executions proportional to the severity
                    let failures_to_record = ((penalty_percent * 10.0) as u64).max(1);

                    for _ in 0..failures_to_record {
                        self.reputation_store
                            .record_execution(violator, false, 1000); // Record failed execution
                    }

                    log::info!(
                        "Applied reputation penalty of {:.1}% to {} by recording {} failed executions (estimated reduction: {})",
                        penalty_percent * 100.0, violator, failures_to_record, estimated_penalty
                    );
                }
            }
            PenaltyType::Warning => {
                log::warn!("Economic warning issued to {violator}");
            }
            _ => {
                log::warn!(
                    "Penalty type {:?} not fully implemented",
                    penalty.penalty_type
                );
            }
        }

        Ok(())
    }

    // Background task starter methods
    async fn start_mana_regeneration(&self) -> Result<tokio::task::JoinHandle<()>, CommonError> {
        let mana_accounts = self.mana_accounts.clone();
        let mana_ledger = self.mana_ledger.clone();
        let reputation_store = self.reputation_store.clone();
        let config = self.config.clone();
        let event_tx = self.event_tx.clone();
        let time_provider = self.time_provider.clone();

        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1)); // Every second

            loop {
                interval.tick().await;

                if let Err(e) = Self::process_mana_regeneration(
                    &mana_accounts,
                    &mana_ledger,
                    &reputation_store,
                    &config,
                    &event_tx,
                    &time_provider,
                )
                .await
                {
                    log::error!("Error in mana regeneration: {e}");
                }
            }
        });

        Ok(handle)
    }

    async fn start_dynamic_pricing(&self) -> Result<tokio::task::JoinHandle<()>, CommonError> {
        let pricing_models = self.pricing_models.clone();
        let config = self.config.clone();
        let event_tx = self.event_tx.clone();
        let time_provider = self.time_provider.clone();

        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30)); // Every 30 seconds

            loop {
                interval.tick().await;

                if let Err(e) = Self::update_dynamic_pricing(
                    &pricing_models,
                    &config,
                    &event_tx,
                    &time_provider,
                )
                .await
                {
                    log::error!("Error in dynamic pricing: {e}");
                }
            }
        });

        Ok(handle)
    }

    async fn start_resource_allocation(&self) -> Result<tokio::task::JoinHandle<()>, CommonError> {
        let allocation_plans = self.allocation_plans.clone();
        let resource_ledger = self.resource_ledger.clone();
        let reputation_store = self.reputation_store.clone();
        let config = self.config.clone();
        let event_tx = self.event_tx.clone();

        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(config.allocation_optimization_interval);

            loop {
                interval.tick().await;

                if let Err(e) = Self::optimize_resource_allocation(
                    &allocation_plans,
                    &resource_ledger,
                    &reputation_store,
                    &config,
                    &event_tx,
                )
                .await
                {
                    log::error!("Error in resource allocation: {e}");
                }
            }
        });

        Ok(handle)
    }

    async fn start_policy_enforcement(&self) -> Result<tokio::task::JoinHandle<()>, CommonError> {
        let economic_policies = self.economic_policies.clone();
        let mana_ledger = self.mana_ledger.clone();
        let config = self.config.clone();
        let event_tx = self.event_tx.clone();
        let time_provider = self.time_provider.clone();

        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(120)); // Every 2 minutes

            loop {
                interval.tick().await;

                if let Err(e) = Self::enforce_economic_policies(
                    &economic_policies,
                    &mana_ledger,
                    &config,
                    &event_tx,
                    &time_provider,
                )
                .await
                {
                    log::error!("Error in policy enforcement: {e}");
                }
            }
        });

        Ok(handle)
    }

    async fn start_health_monitoring(&self) -> Result<tokio::task::JoinHandle<()>, CommonError> {
        let health_metrics = self.health_metrics.clone();
        let mana_ledger = self.mana_ledger.clone();
        let resource_ledger = self.resource_ledger.clone();
        let config = self.config.clone();
        let event_tx = self.event_tx.clone();
        let time_provider = self.time_provider.clone();

        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(config.health_check_interval);

            loop {
                interval.tick().await;

                if let Err(e) = Self::monitor_economic_health(
                    &health_metrics,
                    &mana_ledger,
                    &resource_ledger,
                    &config,
                    &event_tx,
                    &time_provider,
                )
                .await
                {
                    log::error!("Error in health monitoring: {e}");
                }
            }
        });

        Ok(handle)
    }

    async fn start_market_making(&self) -> Result<tokio::task::JoinHandle<()>, CommonError> {
        let market_making_state = self.market_making_state.clone();
        let pricing_models = self.pricing_models.clone();
        let config = self.config.clone();
        let event_tx = self.event_tx.clone();

        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(10)); // Every 10 seconds

            loop {
                interval.tick().await;

                if let Err(e) = Self::execute_market_making(
                    &market_making_state,
                    &pricing_models,
                    &config,
                    &event_tx,
                )
                .await
                {
                    log::error!("Error in market making: {e}");
                }
            }
        });

        Ok(handle)
    }

    async fn start_predictive_modeling(&self) -> Result<tokio::task::JoinHandle<()>, CommonError> {
        let health_metrics = self.health_metrics.clone();
        let pricing_models = self.pricing_models.clone();
        let mana_accounts = self.mana_accounts.clone();

        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300)); // Every 5 minutes

            loop {
                interval.tick().await;

                if let Err(e) =
                    Self::run_predictive_models(&health_metrics, &pricing_models, &mana_accounts)
                        .await
                {
                    log::error!("Error in predictive modeling: {e}");
                }
            }
        });

        Ok(handle)
    }

    // Implementation of background task methods (simplified for brevity)
    async fn process_mana_regeneration(
        _mana_accounts: &Arc<RwLock<HashMap<Did, ManaAccountState>>>,
        _mana_ledger: &Arc<dyn ManaLedger>,
        _reputation_store: &Arc<dyn ReputationStore>,
        _config: &EconomicAutomationConfig,
        _event_tx: &mpsc::UnboundedSender<EconomicEvent>,
        _time_provider: &Arc<dyn TimeProvider>,
    ) -> Result<(), CommonError> {
        // Implement mana regeneration logic
        log::info!("Running mana regeneration");

        // Get all active accounts - in production this would query the actual member registry
        let mock_accounts = vec![
            "did:icn:alice".to_string(),
            "did:icn:bob".to_string(),
            "did:icn:charlie".to_string(),
            "did:icn:diana".to_string(),
            "did:icn:eve".to_string(),
        ];

        for account_id in mock_accounts {
            if let Ok(did) = Did::from_str(&account_id) {
                // Calculate regeneration amount based on reputation and base rate
                let reputation = _reputation_store.get_reputation(&did);
                let base_regeneration = 10; // Base mana per regeneration cycle

                // Higher reputation accounts get more regeneration
                let reputation_bonus = (reputation as f64 / 100.0) * 5.0; // Up to 5 bonus mana
                let total_regeneration = base_regeneration + reputation_bonus as u64;

                // Apply regeneration
                if let Err(e) = _mana_ledger.credit(&did, total_regeneration) {
                    log::error!("Failed to regenerate mana for {did}: {e}");
                } else {
                    log::debug!(
                        "Regenerated {total_regeneration} mana for {did} (rep: {reputation})"
                    );

                    // Get updated balance
                    let updated_balance = _mana_ledger.get_balance(&did);

                    // Emit regeneration event
                    let _ = _event_tx.send(EconomicEvent::ManaRegenerated {
                        account: did,
                        amount: total_regeneration,
                        new_balance: updated_balance,
                        regeneration_rate: 1.0 + (reputation_bonus / 100.0),
                        timestamp: _time_provider.unix_seconds() * 1000, // Convert to millis
                    });
                }
            }
        }

        log::info!("Mana regeneration completed");
        Ok(())
    }

    async fn update_dynamic_pricing(
        _pricing_models: &Arc<RwLock<HashMap<String, DynamicPricingModel>>>,
        _config: &EconomicAutomationConfig,
        _event_tx: &mpsc::UnboundedSender<EconomicEvent>,
        _time_provider: &Arc<dyn TimeProvider>,
    ) -> Result<(), CommonError> {
        // Implement dynamic pricing logic
        log::debug!("Running dynamic pricing update");

        // For now, use mock system utilization - in production this would query actual metrics
        let system_utilization = 0.7; // Mock 70% utilization

        // Update pricing for different resource types
        let resource_types = vec!["cpu", "memory", "storage", "network"];

        for resource_type in resource_types {
            let base_price = match resource_type {
                "cpu" => 10,
                "memory" => 5,
                "storage" => 2,
                "network" => 3,
                _ => 1,
            };

            // Adjust price based on system utilization
            let demand_multiplier = 1.0 + (system_utilization * 0.5); // Up to 50% price increase
            let adjusted_price = (base_price as f64 * demand_multiplier) as u64;

            log::debug!(
                "Updated {resource_type} pricing: {base_price} -> {adjusted_price} (demand: {demand_multiplier:.2})"
            );
        }

        log::debug!("Dynamic pricing update completed");
        Ok(())
    }

    async fn optimize_resource_allocation(
        _allocation_plans: &Arc<RwLock<HashMap<String, ResourceAllocationPlan>>>,
        _resource_ledger: &Arc<dyn ResourceLedger>,
        _reputation_store: &Arc<dyn ReputationStore>,
        _config: &EconomicAutomationConfig,
        _event_tx: &mpsc::UnboundedSender<EconomicEvent>,
    ) -> Result<(), CommonError> {
        // Implement resource allocation optimization
        log::debug!("Running resource allocation optimization");

        // Get current resource allocation data - mock data for now
        let mut allocation_metrics = HashMap::new();
        allocation_metrics.insert(
            "cpu".to_string(),
            AllocationMetrics {
                allocated_amount: 100,
                utilization_rate: 0.6,
                efficiency_score: 0.8,
            },
        );
        allocation_metrics.insert(
            "memory".to_string(),
            AllocationMetrics {
                allocated_amount: 200,
                utilization_rate: 0.4, // Low utilization
                efficiency_score: 0.6,
            },
        );

        // Find inefficient allocations and suggest optimizations
        let mut optimization_suggestions = Vec::new();

        for (resource_type, metrics) in allocation_metrics {
            if metrics.utilization_rate < 0.5 {
                // Low utilization - suggest reducing allocation
                optimization_suggestions.push(AllocationOptimization {
                    resource_type: resource_type.clone(),
                    optimization_type: OptimizationType::ReduceAllocation,
                    current_allocation: metrics.allocated_amount,
                    suggested_allocation: (metrics.allocated_amount as f64 * 0.8) as u64,
                    efficiency_gain: 0.2,
                });
            } else if metrics.utilization_rate > 0.9 {
                // High utilization - suggest increasing allocation
                optimization_suggestions.push(AllocationOptimization {
                    resource_type: resource_type.clone(),
                    optimization_type: OptimizationType::IncreaseAllocation,
                    current_allocation: metrics.allocated_amount,
                    suggested_allocation: (metrics.allocated_amount as f64 * 1.2) as u64,
                    efficiency_gain: 0.15,
                });
            }
        }

        // Apply optimizations
        for optimization in &optimization_suggestions {
            log::info!("Applying resource optimization: {optimization:?}");

            // Emit optimization event
            let _ = _event_tx.send(EconomicEvent::ResourceAllocated {
                allocation_id: format!("opt_{}", optimization.resource_type),
                resource_type: optimization.resource_type.clone(),
                amount: optimization.suggested_allocation,
                recipient: Did::from_str("did:icn:system").unwrap_or_default(),
                allocation_strategy: AllocationStrategy::FairAllocation,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
            });
        }

        log::debug!("Resource allocation optimization completed");
        Ok(())
    }

    pub async fn enforce_economic_policies(
        economic_policies: &Arc<RwLock<HashMap<String, EconomicPolicy>>>,
        mana_ledger: &Arc<dyn ManaLedger>,
        _config: &EconomicAutomationConfig,
        event_tx: &mpsc::UnboundedSender<EconomicEvent>,
        time_provider: &Arc<dyn TimeProvider>,
    ) -> Result<(), CommonError> {
        let policies = economic_policies.read().unwrap();
        let accounts = mana_ledger.all_accounts();

        for policy in policies.values() {
            if !matches!(policy.status, PolicyStatus::Active) {
                continue;
            }

            match policy.policy_type {
                PolicyType::ManaRegeneration => {
                    let min_balance =
                        policy.parameters.get("min_balance").cloned().unwrap_or(0.0) as u64;
                    for did in &accounts {
                        let bal = mana_ledger.get_balance(did);
                        if bal < min_balance {
                            let diff = min_balance - bal;
                            mana_ledger.credit(did, diff)?;
                            let _ = event_tx.send(EconomicEvent::ManaRegenerated {
                                account: did.clone(),
                                amount: diff,
                                new_balance: min_balance,
                                regeneration_rate: policy.enforcement_level,
                                timestamp: time_provider.unix_seconds() * 1000,
                            });
                        }
                    }
                }
                PolicyType::AntiManipulation => {
                    let max_balance = policy
                        .parameters
                        .get("max_balance")
                        .cloned()
                        .unwrap_or(u64::MAX as f64) as u64;
                    for did in &accounts {
                        let bal = mana_ledger.get_balance(did);
                        if bal > max_balance {
                            let diff = bal - max_balance;
                            mana_ledger.spend(did, diff)?;
                            let penalty = EconomicPenalty {
                                penalty_type: PenaltyType::ManaPenalty,
                                severity: diff as f64 / bal as f64,
                                duration: None,
                                amount: Some(diff),
                                restrictions: Vec::new(),
                            };
                            let _ = event_tx.send(EconomicEvent::PolicyViolation {
                                violator: did.clone(),
                                policy_id: policy.policy_id.clone(),
                                violation_type: ViolationType::ExcessiveConsumption {
                                    consumed: bal,
                                    limit: max_balance,
                                },
                                penalty_applied: Some(penalty),
                                timestamp: time_provider.unix_seconds() * 1000,
                            });
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub async fn monitor_economic_health(
        health_metrics: &Arc<RwLock<EconomicHealthMetrics>>,
        mana_ledger: &Arc<dyn ManaLedger>,
        _resource_ledger: &Arc<dyn ResourceLedger>,
        _config: &EconomicAutomationConfig,
        event_tx: &mpsc::UnboundedSender<EconomicEvent>,
        time_provider: &Arc<dyn TimeProvider>,
    ) -> Result<(), CommonError> {
        let accounts = mana_ledger.all_accounts();
        let mut balances: Vec<u64> = accounts
            .iter()
            .map(|d| mana_ledger.get_balance(d))
            .collect();
        balances.sort_unstable();
        let total: u64 = balances.iter().sum();
        let n = balances.len() as f64;
        let gini = if total == 0 || balances.is_empty() {
            0.0
        } else {
            let mut cum = 0.0;
            for (i, v) in balances.iter().enumerate() {
                cum += (i as f64 + 1.0) * *v as f64;
            }
            (2.0 * cum) / (n * total as f64) - (n + 1.0) / n
        };

        let mut metrics = health_metrics.write().unwrap();
        metrics.mana_inequality = gini;
        metrics.activity_level = (accounts.len() as f64 / 100.0).min(1.0);
        metrics.overall_health = 1.0 - gini;
        metrics.last_updated = time_provider.unix_seconds();

        if metrics.overall_health < 0.2 {
            let _ = event_tx.send(EconomicEvent::ThresholdReached {
                threshold_type: ThresholdType::EconomicInequality,
                current_value: gini,
                threshold_value: 0.2,
                action_taken: None,
                timestamp: metrics.last_updated * 1000,
            });
        }

        Ok(())
    }

    pub async fn execute_market_making(
        market_making_state: &Arc<RwLock<MarketMakingState>>,
        pricing_models: &Arc<RwLock<HashMap<String, DynamicPricingModel>>>,
        config: &EconomicAutomationConfig,
        event_tx: &mpsc::UnboundedSender<EconomicEvent>,
    ) -> Result<(), CommonError> {
        let mut state = market_making_state.write().unwrap();
        let models = pricing_models.read().unwrap();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        for (resource, model) in models.iter() {
            let buy_price = model.current_price * (1.0 - config.market_making_spread / 2.0);
            let sell_price = model.current_price * (1.0 + config.market_making_spread / 2.0);

            state.performance.total_trades += 2;
            state.performance.total_volume += 2;
            state.performance.total_pnl += sell_price - buy_price;
            state.performance.avg_spread_captured =
                state.performance.total_pnl / state.performance.total_trades as f64;

            let maker = Did::from_str("did:icn:maker").unwrap_or_default();
            let pool = Did::from_str("did:icn:pool").unwrap_or_default();

            let _ = event_tx.send(EconomicEvent::MarketTransaction {
                transaction_id: format!("{resource}_buy"),
                buyer: maker.clone(),
                seller: pool.clone(),
                resource_type: resource.clone(),
                amount: 1,
                price: buy_price,
                timestamp,
            });

            let _ = event_tx.send(EconomicEvent::MarketTransaction {
                transaction_id: format!("{resource}_sell"),
                buyer: pool,
                seller: maker,
                resource_type: resource.clone(),
                amount: 1,
                price: sell_price,
                timestamp,
            });
        }

        Ok(())
    }

    pub async fn run_predictive_models(
        health_metrics: &Arc<RwLock<EconomicHealthMetrics>>,
        pricing_models: &Arc<RwLock<HashMap<String, DynamicPricingModel>>>,
        mana_accounts: &Arc<RwLock<HashMap<Did, ManaAccountState>>>,
    ) -> Result<(), CommonError> {
        let mut models = pricing_models.write().unwrap();
        for model in models.values_mut() {
            if model.price_history.len() >= 3 {
                let avg: f64 = model
                    .price_history
                    .iter()
                    .rev()
                    .take(3)
                    .map(|(_, p)| *p)
                    .sum::<f64>()
                    / 3.0;
                model.current_price = (model.current_price + avg) / 2.0;
            }
        }

        let account_count = mana_accounts.read().unwrap().len();
        let mut metrics = health_metrics.write().unwrap();
        metrics.activity_level = (account_count as f64 / 100.0).min(1.0);
        Ok(())
    }

    // Helper methods
    async fn calculate_demand_multiplier(&self, resource_type: &str) -> Result<f64, CommonError> {
        // Implement demand calculation based on usage patterns and market activity
        let base_multiplier = 1.0;

        // Check current resource allocation plans for demand signals
        let allocation_plans = self.allocation_plans.read().unwrap();
        let demand_signal = if let Some(plan) = allocation_plans.get(resource_type) {
            // Calculate demand based on allocation utilization
            let total_allocated: u64 = plan.allocations.values().map(|entry| entry.amount).sum();
            let utilization = if plan.total_available > 0 {
                total_allocated as f64 / plan.total_available as f64
            } else {
                0.0
            };

            match utilization {
                u if u > 0.9 => 2.5, // Very high demand
                u if u > 0.8 => 2.0, // High demand
                u if u > 0.6 => 1.5, // Medium demand
                u if u > 0.4 => 1.2, // Low demand
                _ => 1.0,            // Normal demand
            }
        } else {
            base_multiplier
        };

        // Factor in market-making activity
        let market_state = self.market_making_state.read().unwrap();
        let market_multiplier = if let Some(inventory) = market_state.inventory.get(resource_type) {
            // Low inventory = higher demand multiplier
            if *inventory < 1000 {
                1.3
            } else if *inventory < 5000 {
                1.1
            } else {
                1.0
            }
        } else {
            1.0
        };

        // Factor in recent pricing trends
        let pricing_models = self.pricing_models.read().unwrap();
        let trend_multiplier = if let Some(model) = pricing_models.get(resource_type) {
            // Calculate price trend over recent history
            let recent_prices: Vec<f64> = model
                .price_history
                .iter()
                .rev()
                .take(10)
                .map(|(_, price)| *price)
                .collect();

            if recent_prices.len() >= 2 {
                let recent_avg = recent_prices.iter().sum::<f64>() / recent_prices.len() as f64;
                let old_avg = model
                    .price_history
                    .iter()
                    .take(model.price_history.len().saturating_sub(10))
                    .map(|(_, price)| *price)
                    .sum::<f64>()
                    / (model.price_history.len().saturating_sub(10).max(1)) as f64;

                if recent_avg > old_avg * 1.1 {
                    1.2 // Increasing prices suggest growing demand
                } else if recent_avg < old_avg * 0.9 {
                    0.9 // Decreasing prices suggest falling demand
                } else {
                    1.0
                }
            } else {
                1.0
            }
        } else {
            1.0
        };

        let final_multiplier: f64 =
            base_multiplier * demand_signal * market_multiplier * trend_multiplier;
        Ok(final_multiplier.min(5.0).max(0.1)) // Cap between 0.1x and 5.0x
    }

    async fn calculate_basic_mana_price(&self, job: &MeshJob) -> Result<u64, CommonError> {
        // Implement basic price calculation based on job characteristics
        let base_cost = job.estimated_cost;

        // Factor in job type complexity
        let complexity_multiplier = match job.job_type.as_deref() {
            Some("compute_intensive") => 1.5,
            Some("memory_intensive") => 1.3,
            Some("network_intensive") => 1.2,
            Some("storage_intensive") => 1.1,
            Some("gpu_compute") => 2.0,
            Some("machine_learning") => 1.8,
            _ => 1.0, // Default/unknown job types
        };

        // Factor in current system load
        let health_metrics = self.health_metrics.read().unwrap();
        let load_multiplier = match health_metrics.resource_efficiency {
            e if e < 0.3 => 2.0, // System overloaded
            e if e < 0.5 => 1.5, // System under stress
            e if e < 0.7 => 1.2, // System moderately loaded
            _ => 1.0,            // System operating normally
        };

        // Factor in market liquidity
        let liquidity_multiplier = match health_metrics.market_liquidity {
            l if l < 0.3 => 1.5, // Low liquidity - higher prices
            l if l < 0.6 => 1.2, // Medium liquidity
            _ => 1.0,            // Good liquidity
        };

        // Factor in recent price volatility
        let volatility_multiplier = if health_metrics.price_stability < 0.5 {
            1.3 // High volatility - add premium for uncertainty
        } else {
            1.0
        };

        // Calculate final price
        let calculated_price = (base_cost as f64
            * complexity_multiplier
            * load_multiplier
            * liquidity_multiplier
            * volatility_multiplier) as u64;

        // Apply reasonable bounds
        let min_price = base_cost / 2; // Never less than 50% of estimated cost
        let max_price = base_cost * 5; // Never more than 500% of estimated cost

        Ok(calculated_price.max(min_price).min(max_price))
    }

    async fn execute_individual_allocation(
        &self,
        recipient: &Did,
        allocation: &AllocationEntry,
    ) -> Result<u64, CommonError> {
        // Implement individual allocation logic based on recipient characteristics
        let base_amount = allocation.amount;

        // Factor in recipient's reputation
        let reputation_multiplier = {
            let rep = self.reputation_store.get_reputation(recipient);
            match rep {
                r if r >= 90 => 1.5, // Excellent reputation
                r if r >= 75 => 1.2, // Good reputation
                r if r >= 50 => 1.0, // Average reputation
                r if r >= 25 => 0.8, // Below average
                _ => 0.5,            // Poor reputation
            }
        };

        // Factor in recipient's current mana balance to prevent over-allocation
        let balance_factor = match self.mana_ledger.get_balance(recipient) {
            current_balance => {
                let mana_accounts = self.mana_accounts.read().unwrap();
                if let Some(account) = mana_accounts.get(recipient) {
                    let capacity_ratio = current_balance as f64 / account.capacity as f64;
                    match capacity_ratio {
                        r if r > 0.9 => 0.1, // Near capacity - minimal allocation
                        r if r > 0.7 => 0.5, // High balance - reduced allocation
                        r if r > 0.3 => 1.0, // Normal allocation
                        _ => 1.2,            // Low balance - increased allocation
                    }
                } else {
                    1.0 // Default if no account state tracked
                }
            }
        };

        // Factor in allocation priority based on score
        let priority_multiplier = match allocation.score {
            s if s >= 0.9 => 1.5, // High priority (score 90%+)
            s if s >= 0.7 => 1.3, // Medium-high priority
            s if s >= 0.5 => 1.0, // Normal priority
            s if s >= 0.3 => 0.8, // Low priority
            _ => 0.5,             // Very low priority
        };

        // Factor in allocation conditions - more conditions may indicate special handling
        let conditions_multiplier = match allocation.conditions.len() {
            0 => 1.0,     // No special conditions
            1..=2 => 1.1, // Few conditions - slight boost
            3..=5 => 1.2, // Several conditions - moderate boost
            _ => 1.3,     // Many conditions - higher boost for complexity
        };

        // Calculate final allocation amount
        let final_amount = (base_amount as f64
            * reputation_multiplier
            * balance_factor
            * priority_multiplier
            * conditions_multiplier) as u64;

        // Ensure allocation stays within reasonable bounds
        let min_allocation = base_amount / 10; // At least 10% of requested
        let max_allocation = base_amount * 3; // At most 300% of requested

        let bounded_amount = final_amount.max(min_allocation).min(max_allocation);

        // Execute the actual allocation
        match self.mana_ledger.credit(recipient, bounded_amount) {
            Ok(()) => {
                log::info!(
                    "Allocated {bounded_amount} mana to {recipient} (requested: {base_amount}, factors: rep={reputation_multiplier:.2}, balance={balance_factor:.2}, priority={priority_multiplier:.2}, conditions={conditions_multiplier:.2})"
                );
                Ok(bounded_amount)
            }
            Err(e) => {
                log::error!("Failed to allocate {bounded_amount} mana to {recipient}: {e}");
                Err(e)
            }
        }
    }

    /// Get economic automation statistics
    pub fn get_automation_stats(&self) -> EconomicAutomationStats {
        let mana_accounts = self.mana_accounts.read().unwrap();
        let pricing_models = self.pricing_models.read().unwrap();
        let allocation_plans = self.allocation_plans.read().unwrap();
        let policies = self.economic_policies.read().unwrap();
        let health_metrics = self.health_metrics.read().unwrap();

        EconomicAutomationStats {
            managed_accounts: mana_accounts.len(),
            active_pricing_models: pricing_models.len(),
            pending_allocations: allocation_plans
                .values()
                .filter(|p| {
                    matches!(
                        p.status,
                        AllocationStatus::Ready | AllocationStatus::Planning
                    )
                })
                .count(),
            active_policies: policies
                .values()
                .filter(|p| matches!(p.status, PolicyStatus::Active))
                .count(),
            economic_health_score: health_metrics.overall_health,
            total_mana_managed: mana_accounts.values().map(|a| a.balance).sum::<u64>(),
        }
    }

    /// Calculate system utilization for pricing adjustments
    #[allow(dead_code)]
    async fn calculate_system_utilization(
        &self,
        _mana_ledger: &Arc<dyn ManaLedger>,
    ) -> Result<f64, CommonError> {
        // Calculate system-wide utilization metrics
        // This would normally examine resource usage, transaction volumes, etc.

        // Mock calculation based on current state
        let utilization = 0.7; // 70% utilization
        Ok(utilization)
    }

    /// Get allocation metrics for optimization
    #[allow(dead_code)]
    async fn get_allocation_metrics(&self) -> HashMap<String, AllocationMetrics> {
        let mut metrics = HashMap::new();

        // Mock metrics for different resource types
        metrics.insert(
            "cpu".to_string(),
            AllocationMetrics {
                allocated_amount: 1000,
                utilization_rate: 0.4, // Low utilization
                efficiency_score: 0.6,
            },
        );

        metrics.insert(
            "memory".to_string(),
            AllocationMetrics {
                allocated_amount: 2000,
                utilization_rate: 0.95, // High utilization
                efficiency_score: 0.8,
            },
        );

        metrics.insert(
            "storage".to_string(),
            AllocationMetrics {
                allocated_amount: 5000,
                utilization_rate: 0.75, // Good utilization
                efficiency_score: 0.9,
            },
        );

        metrics
    }
}

/// Result of allocation plan execution
#[derive(Debug, Clone)]
pub struct AllocationExecutionResult {
    /// Plan identifier
    pub plan_id: String,
    /// Number of successful allocations
    pub successful_allocations: usize,
    /// Number of failed allocations
    pub failed_allocations: usize,
    /// Total amount allocated
    pub total_allocated: u64,
    /// Execution time
    pub execution_time: Duration,
}

/// Statistics about economic automation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicAutomationStats {
    /// Number of managed mana accounts
    pub managed_accounts: usize,
    /// Number of active pricing models
    pub active_pricing_models: usize,
    /// Number of pending allocations
    pub pending_allocations: usize,
    /// Number of active policies
    pub active_policies: usize,
    /// Current economic health score
    pub economic_health_score: f64,
    /// Total mana under management
    pub total_mana_managed: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::SystemTimeProvider;

    #[test]
    fn test_economic_automation_config() {
        let config = EconomicAutomationConfig::default();
        assert!(config.enable_mana_regeneration);
        assert!(config.base_regeneration_rate > 0.0);
        assert!(config.enable_dynamic_pricing);
    }

    #[test]
    fn test_economic_penalty() {
        let penalty = EconomicPenalty {
            penalty_type: PenaltyType::ManaPenalty,
            severity: 0.5,
            duration: Some(Duration::from_secs(3600)),
            amount: Some(100),
            restrictions: vec!["market_participation".to_string()],
        };

        assert!(matches!(penalty.penalty_type, PenaltyType::ManaPenalty));
        assert_eq!(penalty.severity, 0.5);
        assert_eq!(penalty.amount, Some(100));
    }

    #[test]
    fn test_allocation_strategy() {
        let strategy = AllocationStrategy::MeritBased {
            reputation_weight: 0.7,
        };

        match strategy {
            AllocationStrategy::MeritBased { reputation_weight } => {
                assert_eq!(reputation_weight, 0.7);
            }
            _ => panic!("Unexpected allocation strategy"),
        }
    }
}
