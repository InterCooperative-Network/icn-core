use crate::{ResourceLedger, TokenClassId, TokenType};
use icn_common::{CommonError, Did, SystemTimeProvider, TimeProvider};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Specialized functionality for mutual credit systems.
/// Mutual credit allows communities to create money by extending credit to members.
///
/// Represents a credit line extended to a community member.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditLine {
    /// Unique identifier for this credit line.
    pub credit_id: String,
    /// DID of the account receiving credit.
    pub account: Did,
    /// Token class this credit line applies to.
    pub token_class: TokenClassId,
    /// Maximum credit limit.
    pub credit_limit: u64,
    /// Current credit used.
    pub credit_used: u64,
    /// Interest rate (basis points, e.g., 500 = 5%).
    pub interest_rate: u16,
    /// Unix timestamp when credit line was created.
    pub created_at: u64,
    /// Unix timestamp when credit line expires (None = no expiration).
    pub expires_at: Option<u64>,
    /// Status of the credit line.
    pub status: CreditLineStatus,
    /// Credit scoring factors.
    pub credit_score: CreditScore,
    /// Optional metadata.
    pub metadata: HashMap<String, String>,
}

/// Status of a credit line.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CreditLineStatus {
    /// Credit line is active and available.
    Active,
    /// Credit line is temporarily suspended.
    Suspended,
    /// Credit line has been closed.
    Closed,
    /// Credit line is in default.
    Default,
}

/// Credit scoring information for mutual credit systems.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditScore {
    /// Overall creditworthiness score (0-1000).
    pub score: u16,
    /// Community reputation score.
    pub community_reputation: u16,
    /// Payment history score.
    pub payment_history: u16,
    /// Network trust score.
    pub network_trust: u16,
    /// Economic activity score.
    pub economic_activity: u16,
    /// Last updated timestamp.
    pub last_updated: u64,
}

/// Record of a mutual credit transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutualCreditTransaction {
    /// Unique identifier for this transaction.
    pub transaction_id: String,
    /// Account extending credit (creditor).
    pub creditor: Did,
    /// Account receiving credit (debtor).
    pub debtor: Did,
    /// Token class involved.
    pub token_class: TokenClassId,
    /// Amount of credit extended.
    pub amount: u64,
    /// Interest rate applied.
    pub interest_rate: u16,
    /// Purpose of the credit.
    pub purpose: String,
    /// Unix timestamp when credit was extended.
    pub created_at: u64,
    /// Unix timestamp when repayment is due.
    pub due_date: u64,
    /// Current status of the transaction.
    pub status: CreditTransactionStatus,
    /// Repayment history.
    pub repayments: Vec<RepaymentRecord>,
}

/// Status of a mutual credit transaction.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CreditTransactionStatus {
    /// Credit is active and being used.
    Active,
    /// Credit has been fully repaid.
    Repaid,
    /// Credit is overdue.
    Overdue,
    /// Credit is in default.
    Default,
    /// Credit has been forgiven.
    Forgiven,
}

/// Record of a payment made towards mutual credit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepaymentRecord {
    /// Amount repaid.
    pub amount: u64,
    /// Unix timestamp of repayment.
    pub repaid_at: u64,
    /// Method of repayment.
    pub method: RepaymentMethod,
}

/// Methods for repaying mutual credit.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RepaymentMethod {
    /// Direct token transfer.
    TokenTransfer,
    /// Work performed for creditor.
    WorkPerformed { hours: f64, work_type: String },
    /// Goods or services provided.
    GoodsProvided { description: String },
    /// Other community arrangement.
    CommunityArrangement { description: String },
}

/// Trait for managing mutual credit systems.
pub trait MutualCreditStore: Send + Sync {
    /// Create a new credit line.
    fn create_credit_line(&self, credit_line: CreditLine) -> Result<(), CommonError>;
    /// Get a credit line by ID.
    fn get_credit_line(&self, credit_id: &str) -> Option<CreditLine>;
    /// Update a credit line.
    fn update_credit_line(&self, credit_line: CreditLine) -> Result<(), CommonError>;
    /// Get all credit lines for an account.
    fn get_account_credit_lines(&self, account: &Did) -> Vec<CreditLine>;
    /// Record a mutual credit transaction.
    fn record_credit_transaction(
        &self,
        transaction: MutualCreditTransaction,
    ) -> Result<(), CommonError>;
    /// Get a credit transaction by ID.
    fn get_credit_transaction(&self, transaction_id: &str) -> Option<MutualCreditTransaction>;
    /// Update a credit transaction.
    fn update_credit_transaction(
        &self,
        transaction: MutualCreditTransaction,
    ) -> Result<(), CommonError>;
    /// Get credit transaction history for an account.
    fn get_credit_history(&self, account: &Did) -> Vec<MutualCreditTransaction>;
    /// Store a mutual credit agreement.
    fn store_agreement(&self, agreement: &MutualCreditAgreement) -> Result<(), CommonError>;
}

/// In-memory mutual credit store for development and testing.
#[derive(Default)]
pub struct InMemoryMutualCreditStore {
    credit_lines: std::sync::Mutex<HashMap<String, CreditLine>>,
    transactions: std::sync::Mutex<HashMap<String, MutualCreditTransaction>>,
    agreements: std::sync::Mutex<HashMap<String, MutualCreditAgreement>>,
}

impl InMemoryMutualCreditStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl MutualCreditStore for InMemoryMutualCreditStore {
    fn create_credit_line(&self, credit_line: CreditLine) -> Result<(), CommonError> {
        let mut credit_lines = self.credit_lines.lock().unwrap();
        if credit_lines.contains_key(&credit_line.credit_id) {
            return Err(CommonError::InvalidInputError(format!(
                "Credit line {} already exists",
                credit_line.credit_id
            )));
        }
        credit_lines.insert(credit_line.credit_id.clone(), credit_line);
        Ok(())
    }

    fn get_credit_line(&self, credit_id: &str) -> Option<CreditLine> {
        let credit_lines = self.credit_lines.lock().unwrap();
        credit_lines.get(credit_id).cloned()
    }

    fn update_credit_line(&self, credit_line: CreditLine) -> Result<(), CommonError> {
        let mut credit_lines = self.credit_lines.lock().unwrap();
        credit_lines.insert(credit_line.credit_id.clone(), credit_line);
        Ok(())
    }

    fn get_account_credit_lines(&self, account: &Did) -> Vec<CreditLine> {
        let credit_lines = self.credit_lines.lock().unwrap();
        credit_lines
            .values()
            .filter(|cl| &cl.account == account)
            .cloned()
            .collect()
    }

    fn record_credit_transaction(
        &self,
        transaction: MutualCreditTransaction,
    ) -> Result<(), CommonError> {
        let mut transactions = self.transactions.lock().unwrap();
        if transactions.contains_key(&transaction.transaction_id) {
            return Err(CommonError::InvalidInputError(format!(
                "Credit transaction {} already exists",
                transaction.transaction_id
            )));
        }
        transactions.insert(transaction.transaction_id.clone(), transaction);
        Ok(())
    }

    fn get_credit_transaction(&self, transaction_id: &str) -> Option<MutualCreditTransaction> {
        let transactions = self.transactions.lock().unwrap();
        transactions.get(transaction_id).cloned()
    }

    fn update_credit_transaction(
        &self,
        transaction: MutualCreditTransaction,
    ) -> Result<(), CommonError> {
        let mut transactions = self.transactions.lock().unwrap();
        transactions.insert(transaction.transaction_id.clone(), transaction);
        Ok(())
    }

    fn get_credit_history(&self, account: &Did) -> Vec<MutualCreditTransaction> {
        let transactions = self.transactions.lock().unwrap();
        let mut results: Vec<MutualCreditTransaction> = transactions
            .values()
            .filter(|tx| &tx.creditor == account || &tx.debtor == account)
            .cloned()
            .collect();

        // Sort by creation date (newest first)
        results.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        results
    }

    fn store_agreement(&self, agreement: &MutualCreditAgreement) -> Result<(), CommonError> {
        let mut agreements = self.agreements.lock().unwrap();
        if agreements.contains_key(&agreement.agreement_id) {
            return Err(CommonError::InvalidInputError(format!(
                "Mutual credit agreement {} already exists",
                agreement.agreement_id
            )));
        }
        agreements.insert(agreement.agreement_id.clone(), agreement.clone());
        Ok(())
    }
}

/// Status of a mutual credit agreement
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MutualCreditStatus {
    /// Agreement is active
    Active,
    /// Agreement has been fully repaid
    Repaid,
    /// Agreement is in default
    Defaulted,
    /// Agreement has been cancelled
    Cancelled,
}

/// A mutual credit agreement between two parties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutualCreditAgreement {
    /// Unique identifier for this agreement
    pub agreement_id: String,
    /// DID of the creditor (lender)
    pub creditor: Did,
    /// DID of the debtor (borrower)
    pub debtor: Did,
    /// Amount of credit extended
    pub amount: u64,
    /// Token class being used
    pub token_class: TokenClassId,
    /// Purpose or description of the credit
    pub purpose: String,
    /// When the agreement was issued (Unix timestamp)
    pub issue_date: u64,
    /// When repayment is due (Unix timestamp)
    pub repayment_deadline: u64,
    /// Current status of the agreement
    pub status: MutualCreditStatus,
    /// History of repayments made
    pub repayment_history: Vec<String>,
}

/// Configuration for extending mutual credit
#[derive(Debug, Clone)]
pub struct MutualCreditConfig {
    pub creditor: Did,
    pub debtor: Did,
    pub token_class: TokenClassId,
    pub amount: u64,
    pub purpose: String,
    pub repayment_period_days: u64,
}

/// Extend mutual credit between two entities
pub fn extend_mutual_credit<L: ResourceLedger, C: MutualCreditStore>(
    resource_ledger: &L,
    credit_store: &C,
    config: MutualCreditConfig,
    time_provider: &dyn TimeProvider,
) -> Result<String, CommonError> {
    // Validate that the token class is for mutual credit
    let token_class_info = resource_ledger.get_class(&config.token_class).ok_or_else(|| {
        CommonError::InvalidInputError(format!("Token class {} not found", config.token_class))
    })?;

    if token_class_info.token_type != TokenType::MutualCredit {
        return Err(CommonError::InvalidInputError(
            "Token class is not for mutual credit".into(),
        ));
    }

    // Check if creditor has sufficient balance to extend credit
    let creditor_balance = resource_ledger.get_balance(&config.token_class, &config.creditor);
    if creditor_balance < config.amount {
        return Err(CommonError::InsufficientFunds("Creditor has insufficient balance".to_string()));
    }

    // Generate credit agreement ID
    let agreement_id = format!(
        "mc_{}_{}_{}_{}",
        config.creditor.to_string().replace(':', "_"),
        config.debtor.to_string().replace(':', "_"),
        config.amount,
        time_provider.unix_seconds()
    );

    // Create the mutual credit agreement
    let agreement = MutualCreditAgreement {
        agreement_id: agreement_id.clone(),
        creditor: config.creditor.clone(),
        debtor: config.debtor.clone(),
        amount: config.amount,
        token_class: config.token_class.clone(),
        purpose: config.purpose,
        issue_date: time_provider.unix_seconds(),
        repayment_deadline: time_provider.unix_seconds() + (config.repayment_period_days * 24 * 60 * 60),
        status: MutualCreditStatus::Active,
        repayment_history: Vec::new(),
    };

    // Store the agreement
    credit_store.store_agreement(&agreement)?;

    // Transfer tokens from creditor to debtor
    resource_ledger.transfer(
        &config.token_class,
        &config.creditor,
        &config.debtor,
        config.amount,
    )?;

    log::info!(
        "Extended mutual credit: {} tokens from {} to {} (Agreement: {})",
        config.amount,
        config.creditor,
        config.debtor,
        agreement_id
    );

    Ok(agreement_id)
}

/// Repay mutual credit by burning tokens.
pub fn repay_mutual_credit<L: ResourceLedger, C: MutualCreditStore>(
    resource_ledger: &L,
    credit_store: &C,
    transaction_id: &str,
    repayer: &Did,
    amount: u64,
    method: RepaymentMethod,
) -> Result<(), CommonError> {
    let mut transaction = credit_store
        .get_credit_transaction(transaction_id)
        .ok_or_else(|| {
            CommonError::InvalidInputError(format!("Credit transaction {transaction_id} not found"))
        })?;

    // Validate repayer
    if &transaction.debtor != repayer {
        return Err(CommonError::PolicyDenied(
            "Only debtor can repay credit".into(),
        ));
    }

    // Calculate remaining amount
    let total_repaid: u64 = transaction.repayments.iter().map(|r| r.amount).sum();
    let remaining = transaction.amount.saturating_sub(total_repaid);

    if amount > remaining {
        return Err(CommonError::InvalidInputError(
            "Repayment amount exceeds remaining debt".into(),
        ));
    }

    // If repayment is via token transfer, burn the tokens
    if method == RepaymentMethod::TokenTransfer {
        resource_ledger.burn(&transaction.token_class, repayer, amount)?;
    }

    // Record repayment
    transaction.repayments.push(RepaymentRecord {
        amount,
        repaid_at: SystemTimeProvider.unix_seconds(),
        method,
    });

    // Update transaction status
    let new_total_repaid = total_repaid + amount;
    if new_total_repaid >= transaction.amount {
        transaction.status = CreditTransactionStatus::Repaid;
    }

    credit_store.update_credit_transaction(transaction)?;

    Ok(())
}

/// Calculate credit score for an account based on activity.
pub fn calculate_credit_score<C: MutualCreditStore>(
    credit_store: &C,
    account: &Did,
    community_reputation: u16,
) -> CreditScore {
    let credit_history = credit_store.get_credit_history(account);

    // Calculate payment history score
    let total_transactions = credit_history.len() as f64;
    let repaid_transactions = credit_history
        .iter()
        .filter(|tx| tx.status == CreditTransactionStatus::Repaid)
        .count() as f64;

    let payment_history = if total_transactions > 0.0 {
        ((repaid_transactions / total_transactions) * 1000.0) as u16
    } else {
        500 // Default neutral score
    };

    // Calculate network trust (simplified - based on number of different creditors/debtors)
    let mut unique_partners = std::collections::HashSet::new();
    for tx in &credit_history {
        if &tx.creditor == account {
            unique_partners.insert(&tx.debtor);
        } else {
            unique_partners.insert(&tx.creditor);
        }
    }
    let network_trust = std::cmp::min(unique_partners.len() as u16 * 100, 1000);

    // Economic activity score (based on transaction volume)
    let total_volume: u64 = credit_history
        .iter()
        .filter(|tx| tx.created_at > SystemTimeProvider.unix_seconds() - (90 * 24 * 60 * 60)) // Last 90 days
        .map(|tx| tx.amount)
        .sum();
    let economic_activity = std::cmp::min((total_volume / 100) as u16, 1000);

    // Calculate overall score
    let score = (payment_history + community_reputation + network_trust + economic_activity) / 4;

    CreditScore {
        score,
        community_reputation,
        payment_history,
        network_trust,
        economic_activity,
        last_updated: SystemTimeProvider.unix_seconds(),
    }
}

/// Create a credit line for a community member.
pub fn create_credit_line<C: MutualCreditStore>(
    credit_store: &C,
    account: Did,
    token_class: TokenClassId,
    credit_limit: u64,
    community_reputation: u16,
) -> Result<String, CommonError> {
    let credit_id = format!("credit_{}_{}", account, SystemTimeProvider.unix_seconds());
    let credit_score = calculate_credit_score(credit_store, &account, community_reputation);

    let credit_line = CreditLine {
        credit_id: credit_id.clone(),
        account,
        token_class,
        credit_limit,
        credit_used: 0,
        interest_rate: 0, // Mutual credit typically has no interest
        created_at: SystemTimeProvider.unix_seconds(),
        expires_at: None,
        status: CreditLineStatus::Active,
        credit_score,
        metadata: HashMap::new(),
    };

    credit_store.create_credit_line(credit_line)?;
    Ok(credit_id)
}

/// Get community credit statistics.
pub fn get_community_credit_stats<C: MutualCreditStore>(
    credit_store: &C,
    community_members: &[Did],
) -> CommunityStats {
    let mut total_credit_limit = 0u64;
    let mut total_credit_used = 0u64;
    let mut active_credit_lines = 0usize;
    let mut total_transactions = 0usize;
    let mut successful_repayments = 0usize;

    for member in community_members {
        let credit_lines = credit_store.get_account_credit_lines(member);
        for cl in credit_lines {
            if cl.status == CreditLineStatus::Active {
                total_credit_limit += cl.credit_limit;
                total_credit_used += cl.credit_used;
                active_credit_lines += 1;
            }
        }

        let credit_history = credit_store.get_credit_history(member);
        total_transactions += credit_history.len();
        successful_repayments += credit_history
            .iter()
            .filter(|tx| tx.status == CreditTransactionStatus::Repaid)
            .count();
    }

    let repayment_rate = if total_transactions > 0 {
        (successful_repayments as f64 / total_transactions as f64) * 100.0
    } else {
        0.0
    };

    CommunityStats {
        total_credit_limit,
        total_credit_used,
        active_credit_lines,
        total_transactions,
        successful_repayments,
        repayment_rate,
        community_members: community_members.len(),
    }
}

/// Statistics about a mutual credit community.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityStats {
    /// Total credit limit across all members.
    pub total_credit_limit: u64,
    /// Total credit currently in use.
    pub total_credit_used: u64,
    /// Number of active credit lines.
    pub active_credit_lines: usize,
    /// Total number of credit transactions.
    pub total_transactions: usize,
    /// Number of successfully repaid transactions.
    pub successful_repayments: usize,
    /// Repayment rate as a percentage.
    pub repayment_rate: f64,
    /// Number of community members.
    pub community_members: usize,
}
