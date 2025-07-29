//! Economic Dispute Resolution
//!
//! This module implements dispute resolution for economic conflicts such as:
//! - Mana distribution disputes
//! - Resource allocation conflicts
//! - Token transfer disputes
//! - Marketplace transaction disputes
//! - Mutual credit disagreements

use crate::{ManaLedger, ResourceLedger};
use icn_common::{CommonError, Did, NodeScope};
use icn_core_traits::economics::ManaTransaction;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

/// Types of economic disputes that can occur
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EconomicDisputeType {
    /// Disputed mana distribution or allocation
    ManaDispute,
    /// Conflicting resource allocation claims
    ResourceAllocationConflict,
    /// Disputed token transfers or balances
    TokenTransferDispute,
    /// Marketplace transaction disputes
    MarketplaceDispute,
    /// Mutual credit disagreements
    MutualCreditDispute,
    /// Price manipulation or unfair pricing
    PricingDispute,
    /// Double spending or accounting errors
    DoubleSpending,
}

/// A specific economic dispute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicDispute {
    /// Unique identifier for this dispute
    pub dispute_id: String,
    /// Type of economic dispute
    pub dispute_type: EconomicDisputeType,
    /// Parties involved in the dispute
    pub parties: Vec<Did>,
    /// Amount in dispute (if applicable)
    pub disputed_amount: Option<u64>,
    /// Token class or resource type involved
    pub disputed_asset: String,
    /// Timestamp when dispute was filed
    pub filed_at: u64,
    /// Current resolution status
    pub resolution_status: EconomicResolutionStatus,
    /// Evidence supporting the dispute
    pub evidence: Vec<EconomicEvidence>,
    /// Description of the dispute
    pub description: String,
    /// Severity level of the dispute
    pub severity: DisputeSeverity,
    /// Node scope where dispute occurred
    pub scope: Option<NodeScope>,
}

/// Current status of economic dispute resolution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EconomicResolutionStatus {
    /// Dispute filed but not yet reviewed
    Filed,
    /// Under investigation by economic authorities
    UnderInvestigation,
    /// Mediation in progress
    Mediation,
    /// Arbitration process initiated
    Arbitration { arbitrator: Did, deadline: u64 },
    /// Community voting on resolution
    CommunityVoting {
        voting_deadline: u64,
        votes_for: u64,
        votes_against: u64,
    },
    /// Dispute resolved
    Resolved {
        resolution: EconomicResolution,
        applied_at: u64,
    },
    /// Resolution rejected or failed
    Failed { reason: String },
    /// Escalated to governance level
    EscalatedToGovernance,
}

/// Evidence supporting an economic dispute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EconomicEvidence {
    /// Transaction records as evidence
    TransactionRecords { transactions: Vec<String> },
    /// Balance discrepancies
    BalanceDiscrepancy {
        account: Did,
        expected_balance: u64,
        actual_balance: u64,
        asset_type: String,
    },
    /// Resource allocation logs
    ResourceAllocationLog {
        allocation_id: String,
        disputed_allocation: String,
    },
    /// Price manipulation evidence
    PriceManipulation {
        asset: String,
        suspicious_prices: Vec<(u64, u64)>, // (timestamp, price)
    },
    /// Double spending evidence
    DoubleSpendingEvidence {
        conflicting_transactions: Vec<String>,
    },
    /// Witness testimony or external verification
    ExternalVerification {
        verifier: Did,
        verification_details: String,
    },
}

/// Severity levels for economic disputes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum DisputeSeverity {
    /// Low severity - minor discrepancies
    Low,
    /// Medium severity - significant economic impact
    Medium,
    /// High severity - major economic loss or systemic issues
    High,
    /// Critical severity - economic system integrity at risk
    Critical,
}

/// Resolution actions for economic disputes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EconomicResolution {
    /// Reverse disputed transactions
    ReverseTransactions { transaction_ids: Vec<String> },
    /// Adjust account balances
    AdjustBalances { adjustments: Vec<BalanceAdjustment> },
    /// Redistribute resources
    RedistributeResources {
        redistributions: Vec<ResourceRedistribution>,
    },
    /// Impose penalties or fees
    ImposePenalties { penalties: Vec<EconomicPenalty> },
    /// Provide compensation
    Compensation { compensations: Vec<Compensation> },
    /// Freeze disputed assets pending further investigation
    FreezeAssets { frozen_assets: Vec<AssetFreeze> },
    /// No action required (dispute invalid)
    NoActionRequired,
    /// Escalate to governance for policy decision
    EscalateToGovernance { reason: String },
}

/// Balance adjustment as part of dispute resolution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BalanceAdjustment {
    pub account: Did,
    pub asset_type: String,
    pub adjustment_amount: i64, // Positive for credit, negative for debit
    pub reason: String,
}

/// Resource redistribution as part of dispute resolution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResourceRedistribution {
    pub from_account: Did,
    pub to_account: Did,
    pub resource_type: String,
    pub amount: u64,
    pub reason: String,
}

/// Economic penalty imposed as dispute resolution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EconomicPenalty {
    pub penalized_party: Did,
    pub penalty_type: PenaltyType,
    pub amount: u64,
    pub reason: String,
}

/// Types of economic penalties
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PenaltyType {
    /// Mana penalty
    ManaDeduction,
    /// Token confiscation
    TokenConfiscation { token_class: String },
    /// Trading restrictions
    TradingRestriction { duration_seconds: u64 },
    /// Fee imposition
    Fee { fee_type: String },
}

/// Compensation provided as dispute resolution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Compensation {
    pub recipient: Did,
    pub compensation_type: CompensationType,
    pub amount: u64,
    pub reason: String,
}

/// Types of compensation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CompensationType {
    /// Mana compensation
    ManaCredit,
    /// Token compensation
    TokenCredit { token_class: String },
    /// Resource allocation priority
    ResourcePriority { resource_type: String },
    /// Fee waiver
    FeeWaiver { fee_type: String },
}

/// Asset freeze as dispute resolution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AssetFreeze {
    pub account: Did,
    pub asset_type: String,
    pub frozen_amount: u64,
    pub freeze_duration: Option<u64>, // None for indefinite
    pub reason: String,
}

/// Configuration for economic dispute resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicDisputeConfig {
    /// Enable automatic dispute detection
    pub auto_detection: bool,
    /// Minimum severity for automatic resolution
    pub auto_resolution_threshold: DisputeSeverity,
    /// Maximum time for dispute investigation (seconds)
    pub investigation_timeout: u64,
    /// Mediation timeout (seconds)
    pub mediation_timeout: u64,
    /// Arbitration timeout (seconds)
    pub arbitration_timeout: u64,
    /// Community voting period (seconds)
    pub voting_period: u64,
    /// Minimum amount threshold for disputes (below this auto-resolve)
    pub minimum_dispute_amount: u64,
    /// Enable reputation-based arbitrator selection
    pub reputation_based_arbitration: bool,
    /// Maximum number of concurrent disputes per account
    pub max_disputes_per_account: usize,
}

impl Default for EconomicDisputeConfig {
    fn default() -> Self {
        Self {
            auto_detection: true,
            auto_resolution_threshold: DisputeSeverity::Medium,
            investigation_timeout: 172800, // 48 hours
            mediation_timeout: 86400,      // 24 hours
            arbitration_timeout: 86400,    // 24 hours
            voting_period: 259200,         // 72 hours
            minimum_dispute_amount: 10,
            reputation_based_arbitration: true,
            max_disputes_per_account: 5,
        }
    }
}

/// Manages economic dispute detection and resolution
pub struct EconomicDisputeResolver {
    config: EconomicDisputeConfig,
    active_disputes: HashMap<String, EconomicDispute>,
    resolution_history: Vec<EconomicDispute>,
    economic_authorities: HashSet<Did>,
    qualified_arbitrators: HashSet<Did>,
    reputation_provider: Option<Box<dyn ReputationProvider>>,
}

/// Trait for providing reputation scores for arbitrator selection
pub trait ReputationProvider: Send + Sync {
    /// Get the reputation score for a given DID
    fn get_reputation(&self, did: &Did) -> f64;
    /// Check if a DID is qualified as an arbitrator
    fn is_qualified_arbitrator(&self, did: &Did) -> bool;
}

impl EconomicDisputeResolver {
    /// Create a new economic dispute resolver
    pub fn new(config: EconomicDisputeConfig) -> Self {
        Self {
            config,
            active_disputes: HashMap::new(),
            resolution_history: Vec::new(),
            economic_authorities: HashSet::new(),
            qualified_arbitrators: HashSet::new(),
            reputation_provider: None,
        }
    }

    /// Create a new resolver with reputation provider
    pub fn new_with_reputation(
        config: EconomicDisputeConfig,
        reputation_provider: Box<dyn ReputationProvider>,
    ) -> Self {
        Self {
            config,
            active_disputes: HashMap::new(),
            resolution_history: Vec::new(),
            economic_authorities: HashSet::new(),
            qualified_arbitrators: HashSet::new(),
            reputation_provider: Some(reputation_provider),
        }
    }

    /// Add an economic authority who can resolve disputes
    pub fn add_economic_authority(&mut self, authority: Did) {
        self.economic_authorities.insert(authority);
    }

    /// Add a qualified arbitrator
    pub fn add_arbitrator(&mut self, arbitrator: Did) {
        self.qualified_arbitrators.insert(arbitrator);
    }

    /// File a new economic dispute
    pub fn file_dispute(&mut self, dispute: EconomicDispute) -> Result<String, CommonError> {
        // Validate the dispute
        self.validate_dispute(&dispute)?;

        // Check if account has too many active disputes
        let account_disputes = self
            .active_disputes
            .values()
            .filter(|d| {
                d.parties
                    .iter()
                    .any(|party| dispute.parties.contains(party))
            })
            .count();

        if account_disputes >= self.config.max_disputes_per_account {
            return Err(CommonError::PolicyDenied(
                "Too many active disputes for this account".to_string(),
            ));
        }

        // Auto-resolve trivial disputes
        if let Some(auto_resolution) = self.check_auto_resolution(&dispute)? {
            let mut resolved_dispute = dispute;
            resolved_dispute.resolution_status = EconomicResolutionStatus::Resolved {
                resolution: auto_resolution,
                applied_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            };
            let dispute_id = resolved_dispute.dispute_id.clone();
            self.resolution_history.push(resolved_dispute);
            return Ok(dispute_id);
        }

        // Add to active disputes
        let dispute_id = dispute.dispute_id.clone();
        self.active_disputes.insert(dispute_id.clone(), dispute);

        Ok(dispute_id)
    }

    /// Validate a dispute before processing
    fn validate_dispute(&self, dispute: &EconomicDispute) -> Result<(), CommonError> {
        // Check minimum amount threshold
        if let Some(amount) = dispute.disputed_amount {
            if amount < self.config.minimum_dispute_amount {
                return Err(CommonError::PolicyDenied(
                    "Dispute amount below minimum threshold".to_string(),
                ));
            }
        }

        // Check that parties are not empty
        if dispute.parties.is_empty() {
            return Err(CommonError::InvalidInputError(
                "Dispute must involve at least one party".to_string(),
            ));
        }

        // Check that evidence is provided
        if dispute.evidence.is_empty() {
            return Err(CommonError::InvalidInputError(
                "Dispute must include supporting evidence".to_string(),
            ));
        }

        Ok(())
    }

    /// Check if a dispute can be automatically resolved
    fn check_auto_resolution(
        &self,
        dispute: &EconomicDispute,
    ) -> Result<Option<EconomicResolution>, CommonError> {
        // Only auto-resolve low severity disputes below threshold
        if dispute.severity > self.config.auto_resolution_threshold {
            return Ok(None);
        }

        match &dispute.dispute_type {
            EconomicDisputeType::DoubleSpending => {
                // Auto-resolve clear double spending cases
                for evidence in &dispute.evidence {
                    if let EconomicEvidence::DoubleSpendingEvidence {
                        conflicting_transactions,
                    } = evidence
                    {
                        if conflicting_transactions.len() >= 2 {
                            return Ok(Some(EconomicResolution::ReverseTransactions {
                                transaction_ids: conflicting_transactions.clone(),
                            }));
                        }
                    }
                }
            }
            EconomicDisputeType::TokenTransferDispute => {
                // Auto-resolve small token discrepancies
                if let Some(amount) = dispute.disputed_amount {
                    if amount <= self.config.minimum_dispute_amount * 2 {
                        return Ok(Some(EconomicResolution::Compensation {
                            compensations: dispute
                                .parties
                                .iter()
                                .map(|party| Compensation {
                                    recipient: party.clone(),
                                    compensation_type: CompensationType::ManaCredit,
                                    amount: amount / dispute.parties.len() as u64,
                                    reason: "Auto-resolved token dispute".to_string(),
                                })
                                .collect(),
                        }));
                    }
                }
            }
            _ => {}
        }

        Ok(None)
    }

    /// Detect economic disputes from transaction patterns
    pub fn detect_disputes<M: ManaLedger, R: ResourceLedger>(
        &mut self,
        mana_ledger: &M,
        resource_ledger: &R,
        recent_transactions: &[ManaTransaction],
    ) -> Result<Vec<EconomicDispute>, CommonError> {
        if !self.config.auto_detection {
            return Ok(Vec::new());
        }

        let mut detected_disputes = Vec::new();

        // Detect double spending
        detected_disputes.extend(self.detect_double_spending(mana_ledger, recent_transactions)?);

        // Detect balance discrepancies
        detected_disputes
            .extend(self.detect_balance_discrepancies(mana_ledger, recent_transactions)?);

        // Detect price manipulation
        detected_disputes.extend(self.detect_price_manipulation(recent_transactions)?);

        // Add detected disputes to active tracking
        for dispute in &detected_disputes {
            self.active_disputes
                .insert(dispute.dispute_id.clone(), dispute.clone());
        }

        Ok(detected_disputes)
    }

    /// Detect double spending patterns
    fn detect_double_spending<M: ManaLedger>(
        &self,
        mana_ledger: &M,
        transactions: &[ManaTransaction],
    ) -> Result<Vec<EconomicDispute>, CommonError> {
        let mut disputes = Vec::new();
        let mut transaction_map: HashMap<String, Vec<&ManaTransaction>> = HashMap::new();

        // Group transactions by a composite key to detect potential double spending
        for tx in transactions {
            let key = format!("{}_{}", tx.did, tx.timestamp);
            transaction_map.entry(key).or_default().push(tx);
        }

        // Look for suspicious patterns
        for (key, txs) in transaction_map {
            if txs.len() > 1 {
                // Check if multiple transactions from same account at same time
                let total_amount: i64 = txs.iter().map(|tx| tx.amount).sum();
                let account_balance = mana_ledger.get_balance(&txs[0].did);

                if total_amount.unsigned_abs() > account_balance {
                    let dispute_id = format!(
                        "double_spend_{}_{}",
                        key,
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs()
                    );

                    disputes.push(EconomicDispute {
                        dispute_id,
                        dispute_type: EconomicDisputeType::DoubleSpending,
                        parties: vec![txs[0].did.clone()],
                        disputed_amount: Some(total_amount.unsigned_abs()),
                        disputed_asset: "mana".to_string(),
                        filed_at: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                        resolution_status: EconomicResolutionStatus::Filed,
                        evidence: vec![EconomicEvidence::DoubleSpendingEvidence {
                            conflicting_transactions: txs
                                .iter()
                                .map(|tx| tx.transaction_id.clone())
                                .collect(),
                        }],
                        description: "Potential double spending detected".to_string(),
                        severity: DisputeSeverity::High,
                        scope: None,
                    });
                }
            }
        }

        Ok(disputes)
    }

    /// Detect balance discrepancies by looking for inconsistent transaction patterns
    ///
    /// Since we don't have access to historical balances, this function detects
    /// potential discrepancies by looking for suspicious transaction patterns:
    /// 1. Negative balances that shouldn't exist
    /// 2. Transactions that would exceed reasonable balance limits
    /// 3. Inconsistent transaction flows
    fn detect_balance_discrepancies<M: ManaLedger>(
        &self,
        mana_ledger: &M,
        transactions: &[ManaTransaction],
    ) -> Result<Vec<EconomicDispute>, CommonError> {
        let mut disputes = Vec::new();
        let mut account_transactions: HashMap<Did, Vec<&ManaTransaction>> = HashMap::new();

        // Group transactions by account
        for tx in transactions {
            account_transactions
                .entry(tx.did.clone())
                .or_default()
                .push(tx);
        }

        // Check each account for suspicious patterns
        for (did, txs) in account_transactions {
            let current_balance = mana_ledger.get_balance(&did);
            let net_change: i64 = txs.iter().map(|tx| tx.amount).sum();
            let total_debits: u64 = txs
                .iter()
                .filter(|tx| tx.amount < 0)
                .map(|tx| (-tx.amount) as u64)
                .sum();

            // Pattern 1: Check if total debits exceed what seems reasonable for current balance
            // This could indicate transactions were processed when balance was insufficient
            if total_debits > current_balance + (self.config.minimum_dispute_amount * 10) {
                let dispute_id = format!(
                    "excessive_debits_{}_{}",
                    did,
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                );

                disputes.push(EconomicDispute {
                    dispute_id,
                    dispute_type: EconomicDisputeType::ManaDispute,
                    parties: vec![did.clone()],
                    disputed_amount: Some(total_debits - current_balance),
                    disputed_asset: "mana".to_string(),
                    filed_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    resolution_status: EconomicResolutionStatus::Filed,
                    evidence: vec![EconomicEvidence::BalanceDiscrepancy {
                        account: did.clone(),
                        expected_balance: current_balance.saturating_sub(total_debits),
                        actual_balance: current_balance,
                        asset_type: "mana".to_string(),
                    }],
                    description: format!("Suspicious transaction pattern: total debits ({total_debits}) exceed reasonable balance limit for account with balance {current_balance}"),
                    severity: DisputeSeverity::High,
                    scope: None,
                });
                continue;
            }

            // Pattern 2: Check for accounts with zero balance but positive net changes
            // This might indicate missing credit transactions
            if current_balance == 0 && net_change > 0 {
                let dispute_id = format!(
                    "missing_credits_{}_{}",
                    did,
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                );

                disputes.push(EconomicDispute {
                    dispute_id,
                    dispute_type: EconomicDisputeType::ManaDispute,
                    parties: vec![did.clone()],
                    disputed_amount: Some(net_change as u64),
                    disputed_asset: "mana".to_string(),
                    filed_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    resolution_status: EconomicResolutionStatus::Filed,
                    evidence: vec![EconomicEvidence::BalanceDiscrepancy {
                        account: did.clone(),
                        expected_balance: net_change as u64,
                        actual_balance: current_balance,
                        asset_type: "mana".to_string(),
                    }],
                    description: format!("Account has zero balance but recent transactions show net positive change of {net_change}"),
                    severity: DisputeSeverity::Medium,
                    scope: None,
                });
                continue;
            }

            // Pattern 3: Check for extremely large negative net changes compared to current balance
            // This might indicate unauthorized debits or double-spending
            if net_change < 0 && (-net_change) as u64 > current_balance * 2 && current_balance > 0 {
                let dispute_id = format!(
                    "excessive_negative_change_{}_{}",
                    did,
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                );

                disputes.push(EconomicDispute {
                    dispute_id,
                    dispute_type: EconomicDisputeType::ManaDispute,
                    parties: vec![did.clone()],
                    disputed_amount: Some((-net_change) as u64),
                    disputed_asset: "mana".to_string(),
                    filed_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    resolution_status: EconomicResolutionStatus::Filed,
                    evidence: vec![EconomicEvidence::BalanceDiscrepancy {
                        account: did,
                        expected_balance: current_balance,
                        actual_balance: current_balance,
                        asset_type: "mana".to_string(),
                    }],
                    description: format!("Excessive negative transaction flow: {} debits for account with balance {}", (-net_change), current_balance),
                    severity: DisputeSeverity::High,
                    scope: None,
                });
            }
        }

        Ok(disputes)
    }

    /// Detect price manipulation patterns
    fn detect_price_manipulation(
        &self,
        transactions: &[ManaTransaction],
    ) -> Result<Vec<EconomicDispute>, CommonError> {
        let mut disputes = Vec::new();

        // Simplified price manipulation detection
        // In a real implementation, this would analyze market transactions for suspicious pricing patterns
        // For now, we'll look for unusual transaction amounts that might indicate manipulation

        let amounts: Vec<i64> = transactions.iter().map(|tx| tx.amount).collect();
        if amounts.len() < 10 {
            return Ok(disputes); // Need enough data for pattern detection
        }

        let mean: f64 = amounts.iter().map(|&x| x as f64).sum::<f64>() / amounts.len() as f64;
        let variance: f64 = amounts
            .iter()
            .map(|&x| {
                let diff = x as f64 - mean;
                diff * diff
            })
            .sum::<f64>()
            / amounts.len() as f64;
        let std_dev = variance.sqrt();

        // Look for transactions that are unusually large (potential manipulation)
        for tx in transactions {
            let z_score = (tx.amount as f64 - mean).abs() / std_dev;
            if z_score > 3.0 && tx.amount.unsigned_abs() > self.config.minimum_dispute_amount * 5 {
                let dispute_id = format!(
                    "price_manipulation_{}_{}",
                    tx.transaction_id,
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                );

                disputes.push(EconomicDispute {
                    dispute_id,
                    dispute_type: EconomicDisputeType::PricingDispute,
                    parties: vec![tx.did.clone()],
                    disputed_amount: Some(tx.amount.unsigned_abs()),
                    disputed_asset: "mana".to_string(),
                    filed_at: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    resolution_status: EconomicResolutionStatus::Filed,
                    evidence: vec![EconomicEvidence::PriceManipulation {
                        asset: "mana".to_string(),
                        suspicious_prices: vec![(tx.timestamp, tx.amount.unsigned_abs())],
                    }],
                    description: "Potential price manipulation detected".to_string(),
                    severity: DisputeSeverity::Medium,
                    scope: None,
                });

                // Only flag one per detection cycle
                break;
            }
        }

        Ok(disputes)
    }

    /// Resolve a dispute
    pub fn resolve_dispute(
        &mut self,
        dispute_id: &str,
        resolver: &Did,
    ) -> Result<EconomicResolutionStatus, CommonError> {
        // Verify resolver is authorized
        if !self.economic_authorities.contains(resolver)
            && !self.qualified_arbitrators.contains(resolver)
        {
            return Err(CommonError::PolicyDenied(
                "Not authorized to resolve economic disputes".to_string(),
            ));
        }

        let dispute = self
            .active_disputes
            .get(dispute_id)
            .ok_or_else(|| {
                CommonError::ResourceNotFound(format!("Dispute {dispute_id} not found"))
            })?
            .clone();

        // Determine resolution based on dispute type and evidence
        let resolution = self.determine_resolution(&dispute)?;

        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Update the dispute status
        if let Some(active_dispute) = self.active_disputes.get_mut(dispute_id) {
            active_dispute.resolution_status = EconomicResolutionStatus::Resolved {
                resolution: resolution.clone(),
                applied_at: current_time,
            };
        }

        // Move to history
        let resolved_dispute = self.active_disputes.remove(dispute_id).unwrap();
        self.resolution_history.push(resolved_dispute);

        Ok(EconomicResolutionStatus::Resolved {
            resolution,
            applied_at: current_time,
        })
    }

    /// Determine appropriate resolution for a dispute
    fn determine_resolution(
        &self,
        dispute: &EconomicDispute,
    ) -> Result<EconomicResolution, CommonError> {
        match &dispute.dispute_type {
            EconomicDisputeType::DoubleSpending => {
                // Reverse the conflicting transactions
                let transaction_ids = dispute
                    .evidence
                    .iter()
                    .filter_map(|e| match e {
                        EconomicEvidence::DoubleSpendingEvidence {
                            conflicting_transactions,
                        } => Some(conflicting_transactions.clone()),
                        _ => None,
                    })
                    .flatten()
                    .collect();

                Ok(EconomicResolution::ReverseTransactions { transaction_ids })
            }
            EconomicDisputeType::ManaDispute => {
                // Adjust balances based on evidence
                let adjustments = dispute
                    .evidence
                    .iter()
                    .filter_map(|e| match e {
                        EconomicEvidence::BalanceDiscrepancy {
                            account,
                            expected_balance,
                            actual_balance,
                            ..
                        } => {
                            let adjustment = *expected_balance as i64 - *actual_balance as i64;
                            Some(BalanceAdjustment {
                                account: account.clone(),
                                asset_type: "mana".to_string(),
                                adjustment_amount: adjustment,
                                reason: "Balance correction".to_string(),
                            })
                        }
                        _ => None,
                    })
                    .collect();

                Ok(EconomicResolution::AdjustBalances { adjustments })
            }
            EconomicDisputeType::PricingDispute => {
                // Provide compensation for price manipulation victims
                let compensations = dispute
                    .parties
                    .iter()
                    .map(|party| {
                        Compensation {
                            recipient: party.clone(),
                            compensation_type: CompensationType::ManaCredit,
                            amount: dispute.disputed_amount.unwrap_or(0) / 2, // Split the disputed amount
                            reason: "Price manipulation compensation".to_string(),
                        }
                    })
                    .collect();

                Ok(EconomicResolution::Compensation { compensations })
            }
            _ => {
                // For other dispute types, escalate to governance
                Ok(EconomicResolution::EscalateToGovernance {
                    reason: format!(
                        "Complex dispute type requires governance decision: {:?}",
                        dispute.dispute_type
                    ),
                })
            }
        }
    }

    /// Apply a resolution to the economic system
    pub fn apply_resolution<M: ManaLedger, R: ResourceLedger>(
        &self,
        resolution: &EconomicResolution,
        mana_ledger: &M,
        _resource_ledger: &R,
    ) -> Result<(), CommonError> {
        match resolution {
            EconomicResolution::ReverseTransactions { transaction_ids } => {
                // TODO: Implement transaction reversal logic
                // This is a critical operation that needs proper implementation
                todo!(
                    "Reversing transactions is not yet implemented. Transaction IDs: {:?}",
                    transaction_ids
                );
            }
            EconomicResolution::AdjustBalances { adjustments } => {
                for adjustment in adjustments {
                    if adjustment.adjustment_amount > 0 {
                        mana_ledger
                            .credit(&adjustment.account, adjustment.adjustment_amount as u64)?;
                    } else if adjustment.adjustment_amount < 0 {
                        mana_ledger
                            .spend(&adjustment.account, (-adjustment.adjustment_amount) as u64)?;
                    }
                }
                Ok(())
            }
            EconomicResolution::Compensation { compensations } => {
                for compensation in compensations {
                    match &compensation.compensation_type {
                        CompensationType::ManaCredit => {
                            mana_ledger.credit(&compensation.recipient, compensation.amount)?;
                        }
                        CompensationType::TokenCredit { token_class } => {
                            // Would credit tokens in real implementation
                            println!(
                                "Would credit {} tokens of class {} to {}",
                                compensation.amount, token_class, compensation.recipient
                            );
                        }
                        _ => {
                            // Handle other compensation types
                            println!("Would apply compensation: {compensation:?}");
                        }
                    }
                }
                Ok(())
            }
            EconomicResolution::NoActionRequired => Ok(()),
            _ => {
                println!("Resolution requires manual implementation: {resolution:?}");
                Ok(())
            }
        }
    }

    /// Get all active disputes
    pub fn get_active_disputes(&self) -> &HashMap<String, EconomicDispute> {
        &self.active_disputes
    }

    /// Get dispute resolution history
    pub fn get_resolution_history(&self) -> &Vec<EconomicDispute> {
        &self.resolution_history
    }

    /// Process periodic maintenance tasks
    pub fn process_periodic_tasks(&mut self) -> Result<Vec<String>, CommonError> {
        let mut timed_out_disputes = Vec::new();
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Check for investigation timeouts
        let dispute_ids: Vec<String> = self.active_disputes.keys().cloned().collect();

        for dispute_id in dispute_ids {
            if let Some(dispute) = self.active_disputes.get_mut(&dispute_id) {
                let dispute_age = current_time - dispute.filed_at;

                match &dispute.resolution_status {
                    EconomicResolutionStatus::UnderInvestigation => {
                        if dispute_age > self.config.investigation_timeout {
                            dispute.resolution_status =
                                EconomicResolutionStatus::EscalatedToGovernance;
                            timed_out_disputes.push(dispute_id);
                        }
                    }
                    EconomicResolutionStatus::Mediation => {
                        if dispute_age > self.config.mediation_timeout {
                            dispute.resolution_status =
                                EconomicResolutionStatus::EscalatedToGovernance;
                            timed_out_disputes.push(dispute_id);
                        }
                    }
                    EconomicResolutionStatus::Arbitration { deadline, .. } => {
                        if current_time >= *deadline {
                            dispute.resolution_status = EconomicResolutionStatus::Failed {
                                reason: "Arbitration timed out".to_string(),
                            };
                            timed_out_disputes.push(dispute_id);
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(timed_out_disputes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ManaLedger;
    use std::collections::HashMap;
    use std::sync::Mutex;

    #[derive(Default)]
    struct MockManaLedger {
        balances: Mutex<HashMap<Did, u64>>,
    }

    impl ManaLedger for MockManaLedger {
        fn get_balance(&self, did: &Did) -> u64 {
            *self.balances.lock().unwrap().get(did).unwrap_or(&0)
        }

        fn set_balance(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
            self.balances.lock().unwrap().insert(did.clone(), amount);
            Ok(())
        }

        fn spend(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
            let mut balances = self.balances.lock().unwrap();
            let current = *balances.get(did).unwrap_or(&0);
            if current < amount {
                return Err(CommonError::PolicyDenied(
                    "Insufficient balance".to_string(),
                ));
            }
            balances.insert(did.clone(), current - amount);
            Ok(())
        }

        fn credit(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
            let mut balances = self.balances.lock().unwrap();
            let current = *balances.get(did).unwrap_or(&0);
            balances.insert(did.clone(), current + amount);
            Ok(())
        }
    }

    #[test]
    fn test_economic_dispute_resolver_creation() {
        let config = EconomicDisputeConfig::default();
        let resolver = EconomicDisputeResolver::new(config);

        assert_eq!(resolver.active_disputes.len(), 0);
        assert_eq!(resolver.resolution_history.len(), 0);
    }

    #[test]
    fn test_file_dispute() {
        let mut resolver = EconomicDisputeResolver::new(EconomicDisputeConfig::default());

        let dispute = EconomicDispute {
            dispute_id: "test_dispute".to_string(),
            dispute_type: EconomicDisputeType::ManaDispute,
            parties: vec![Did::default()],
            disputed_amount: Some(100),
            disputed_asset: "mana".to_string(),
            filed_at: 1000,
            resolution_status: EconomicResolutionStatus::Filed,
            evidence: vec![EconomicEvidence::BalanceDiscrepancy {
                account: Did::default(),
                expected_balance: 100,
                actual_balance: 50,
                asset_type: "mana".to_string(),
            }],
            description: "Test dispute".to_string(),
            severity: DisputeSeverity::Medium,
            scope: None,
        };

        let result = resolver.file_dispute(dispute).unwrap();
        assert_eq!(result, "test_dispute");
        assert_eq!(resolver.active_disputes.len(), 1);
    }

    #[test]
    fn test_auto_resolution() {
        let mut resolver = EconomicDisputeResolver::new(EconomicDisputeConfig::default());

        // Create a small dispute that should be auto-resolved
        let dispute = EconomicDispute {
            dispute_id: "auto_resolve_test".to_string(),
            dispute_type: EconomicDisputeType::TokenTransferDispute,
            parties: vec![Did::default()],
            disputed_amount: Some(5), // Below threshold
            disputed_asset: "test_token".to_string(),
            filed_at: 1000,
            resolution_status: EconomicResolutionStatus::Filed,
            evidence: vec![EconomicEvidence::TransactionRecords {
                transactions: vec!["tx1".to_string()],
            }],
            description: "Small dispute".to_string(),
            severity: DisputeSeverity::Low,
            scope: None,
        };

        let result = resolver.file_dispute(dispute).unwrap();
        assert_eq!(result, "auto_resolve_test");

        // Should be auto-resolved and moved to history
        assert_eq!(resolver.active_disputes.len(), 0);
        assert_eq!(resolver.resolution_history.len(), 1);
    }

    #[test]
    fn test_double_spending_detection() {
        let resolver = EconomicDisputeResolver::new(EconomicDisputeConfig::default());
        let ledger = MockManaLedger::default();

        let transactions = vec![
            ManaTransaction {
                transaction_id: "tx1".to_string(),
                did: Did::default(),
                amount: -100,
                transaction_type: TransactionType::ManaTransfer,
                timestamp: 1000,
                context: HashMap::new(),
            },
            ManaTransaction {
                transaction_id: "tx2".to_string(),
                did: Did::default(),
                amount: -100,
                transaction_type: TransactionType::ManaTransfer,
                timestamp: 1000, // Same timestamp - suspicious
                context: HashMap::new(),
            },
        ];

        let disputes = resolver
            .detect_double_spending(&ledger, &transactions)
            .unwrap();
        assert_eq!(disputes.len(), 1);
        assert_eq!(
            disputes[0].dispute_type,
            EconomicDisputeType::DoubleSpending
        );
    }

    #[test]
    fn test_balance_discrepancy_detection() {
        let resolver = EconomicDisputeResolver::new(EconomicDisputeConfig::default());
        let ledger = MockManaLedger::default();
        let test_did = Did::default();

        // Set up account with current balance of 50
        ledger.set_balance(&test_did, 50).unwrap();

        // Test pattern 1: Excessive debits (total debits exceed reasonable balance limit)
        let transactions = vec![ManaTransaction {
            transaction_id: "tx1".to_string(),
            did: test_did.clone(),
            amount: -200, // Large debit that exceeds balance + threshold
            transaction_type: TransactionType::ManaTransfer,
            timestamp: 1000,
            context: HashMap::new(),
        }];

        let disputes = resolver
            .detect_balance_discrepancies(&ledger, &transactions)
            .unwrap();
        assert_eq!(disputes.len(), 1);
        assert_eq!(disputes[0].dispute_type, EconomicDisputeType::ManaDispute);
        assert!(disputes[0].description.contains("excessive debits"));

        // Test pattern 2: Zero balance with positive net change
        // Create a different DID by using a different default instance
        let zero_balance_did = Did::default();
        ledger.set_balance(&zero_balance_did, 0).unwrap();

        let transactions2 = vec![ManaTransaction {
            transaction_id: "tx2".to_string(),
            did: zero_balance_did.clone(),
            amount: 100, // Positive change but zero balance suggests missing credits
            transaction_type: TransactionType::ManaTransfer,
            timestamp: 1000,
            context: HashMap::new(),
        }];

        let disputes2 = resolver
            .detect_balance_discrepancies(&ledger, &transactions2)
            .unwrap();
        assert_eq!(disputes2.len(), 1);
        assert!(disputes2[0]
            .description
            .contains("zero balance but recent transactions"));

        // Test pattern 3: Excessive negative change
        let high_balance_did = Did::default();
        ledger.set_balance(&high_balance_did, 100).unwrap();

        let transactions3 = vec![ManaTransaction {
            transaction_id: "tx3".to_string(),
            did: high_balance_did.clone(),
            amount: -300, // Net change way exceeds current balance (100 * 2 = 200)
            transaction_type: TransactionType::ManaTransfer,
            timestamp: 1000,
            context: HashMap::new(),
        }];

        let disputes3 = resolver
            .detect_balance_discrepancies(&ledger, &transactions3)
            .unwrap();
        assert_eq!(disputes3.len(), 1);
        assert!(disputes3[0]
            .description
            .contains("Excessive negative transaction flow"));
    }

    #[test]
    fn test_dispute_resolution() {
        let mut resolver = EconomicDisputeResolver::new(EconomicDisputeConfig::default());
        let authority = Did::default();
        resolver.add_economic_authority(authority.clone());

        let dispute = EconomicDispute {
            dispute_id: "resolve_test".to_string(),
            dispute_type: EconomicDisputeType::ManaDispute,
            parties: vec![Did::default()],
            disputed_amount: Some(100),
            disputed_asset: "mana".to_string(),
            filed_at: 1000,
            resolution_status: EconomicResolutionStatus::Filed,
            evidence: vec![EconomicEvidence::BalanceDiscrepancy {
                account: Did::default(),
                expected_balance: 100,
                actual_balance: 50,
                asset_type: "mana".to_string(),
            }],
            description: "Test dispute".to_string(),
            severity: DisputeSeverity::Medium,
            scope: None,
        };

        resolver
            .active_disputes
            .insert("resolve_test".to_string(), dispute);

        let result = resolver
            .resolve_dispute("resolve_test", &authority)
            .unwrap();
        assert!(matches!(result, EconomicResolutionStatus::Resolved { .. }));
        assert_eq!(resolver.active_disputes.len(), 0);
        assert_eq!(resolver.resolution_history.len(), 1);
    }
}
