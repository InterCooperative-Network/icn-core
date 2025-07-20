use crate::{ResourceLedger, TokenClass, TokenClassId, TokenType};
use icn_common::{CommonError, Did, SystemTimeProvider, TimeProvider};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Specialized functionality for mutual credit systems.
/// Mutual credit allows communities to create money by extending credit to members.

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
}

/// In-memory mutual credit store for development and testing.
#[derive(Default)]
pub struct InMemoryMutualCreditStore {
    credit_lines: std::sync::Mutex<HashMap<String, CreditLine>>,
    transactions: std::sync::Mutex<HashMap<String, MutualCreditTransaction>>,
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
}

/// Extend credit in a mutual credit system by minting tokens.
pub fn extend_mutual_credit<L: ResourceLedger, C: MutualCreditStore>(
    resource_ledger: &L,
    credit_store: &C,
    creditor: &Did,
    debtor: &Did,
    token_class: &TokenClassId,
    amount: u64,
    purpose: String,
    repayment_period_days: u64,
) -> Result<String, CommonError> {
    // Validate that the token class is for mutual credit
    let token_class_info = resource_ledger.get_class(token_class).ok_or_else(|| {
        CommonError::InvalidInputError(format!("Token class {} not found", token_class))
    })?;

    if token_class_info.token_type != TokenType::MutualCredit {
        return Err(CommonError::InvalidInputError(
            "Token class is not for mutual credit".into(),
        ));
    }

    // Check if debtor has available credit
    let credit_lines = credit_store.get_account_credit_lines(debtor);
    let total_credit_limit: u64 = credit_lines
        .iter()
        .filter(|cl| cl.token_class == *token_class && cl.status == CreditLineStatus::Active)
        .map(|cl| cl.credit_limit)
        .sum();

    let total_credit_used: u64 = credit_lines
        .iter()
        .filter(|cl| cl.token_class == *token_class && cl.status == CreditLineStatus::Active)
        .map(|cl| cl.credit_used)
        .sum();

    if total_credit_used + amount > total_credit_limit {
        return Err(CommonError::PolicyDenied("Credit limit exceeded".into()));
    }

    // Create credit transaction
    let transaction_id = format!("credit_{}_{}", debtor, SystemTimeProvider.unix_seconds());
    let now = SystemTimeProvider.unix_seconds();
    let due_date = now + (repayment_period_days * 24 * 60 * 60);

    let transaction = MutualCreditTransaction {
        transaction_id: transaction_id.clone(),
        creditor: creditor.clone(),
        debtor: debtor.clone(),
        token_class: token_class.clone(),
        amount,
        interest_rate: 0, // Default to 0% for mutual credit
        purpose,
        created_at: now,
        due_date,
        status: CreditTransactionStatus::Active,
        repayments: Vec::new(),
    };

    // Record the transaction
    credit_store.record_credit_transaction(transaction)?;

    // Mint tokens to the debtor (this creates the money)
    resource_ledger.mint(token_class, debtor, amount)?;

    // Update credit line usage - distribute across available credit lines
    let mut remaining_amount = amount;
    for credit_line in credit_lines {
        if credit_line.token_class == *token_class
            && credit_line.status == CreditLineStatus::Active
            && remaining_amount > 0
        {
            let available_credit = credit_line.credit_limit - credit_line.credit_used;
            let amount_to_use = std::cmp::min(remaining_amount, available_credit);

            if amount_to_use > 0 {
                let mut updated_credit_line = credit_line;
                updated_credit_line.credit_used += amount_to_use;
                credit_store.update_credit_line(updated_credit_line)?;
                remaining_amount -= amount_to_use;
            }
        }
    }

    // This should never happen if our aggregate limit check is correct
    if remaining_amount > 0 {
        return Err(CommonError::InternalError(
            "Unable to allocate credit across available lines".into(),
        ));
    }

    Ok(transaction_id)
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
            CommonError::InvalidInputError(format!(
                "Credit transaction {} not found",
                transaction_id
            ))
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
