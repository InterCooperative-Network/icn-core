#![doc = include_str!("../README.md")]
#![cfg_attr(
    not(feature = "allow-nondeterminism"),
    deny(clippy::disallowed_methods)
)]

//! # ICN Economics Crate
//! This crate handles the economic protocols of the InterCooperative Network (ICN).
//! It manages token models, ledger interactions, transaction logic, and incentive mechanisms,
//! aiming for security, accuracy, and interoperability.

use icn_common::{
    compute_merkle_cid, CommonError, DagBlock, Did, NodeInfo, NodeScope, SystemTimeProvider,
    TimeProvider,
};
use icn_dag::StorageService;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
pub mod crdt_ledger;
pub mod economic_dispute_resolver;
pub mod explorer;
pub mod ledger;
pub mod marketplace;
pub mod metrics;
pub mod mutual_aid;
pub mod mutual_credit;
pub mod reputation_tokens;
pub mod time_banking;

/// Comprehensive economic automation and policy enforcement
pub mod automation;
pub use automation::{
    EconomicAutomationConfig, EconomicAutomationEngine, EconomicAutomationStats, EconomicEvent,
    EconomicHealthMetrics,
};
pub use crdt_ledger::{CRDTManaLedger, CRDTManaLedgerConfig, CRDTManaLedgerStats};
pub use economic_dispute_resolver::{
    AssetFreeze, BalanceAdjustment, Compensation, CompensationType, DisputeSeverity,
    EconomicDispute, EconomicDisputeConfig, EconomicDisputeResolver, EconomicDisputeType,
    EconomicEvidence, EconomicPenalty, EconomicResolution, EconomicResolutionStatus, PenaltyType,
    ReputationProvider as DisputeReputationProvider, ResourceRedistribution,
};
pub use explorer::{FlowStats, LedgerExplorer};
pub use ledger::FileResourceLedger;
pub use ledger::{
    FileManaLedger, ResourceLedger, ScopingRules, TokenClass, TokenClassId, TokenType,
    TransferRecord, TransferabilityRule,
};
#[cfg(feature = "persist-rocksdb")]
pub use ledger::{RocksdbManaLedger, RocksdbResourceLedger};
#[cfg(feature = "persist-sled")]
pub use ledger::{SledManaLedger, SledResourceLedger};
#[cfg(feature = "persist-sqlite")]
pub use ledger::{SqliteManaLedger, SqliteResourceLedger};
pub use marketplace::{
    BidStatus, FulfillmentDetails, FulfillmentMethod, InMemoryMarketplaceStore, ItemType,
    MarketplaceBid, MarketplaceOffer, MarketplaceStore, MarketplaceTransaction, OfferFilter,
    OfferStatus, TransactionStatus,
};
pub use mutual_aid::{grant_mutual_aid, use_mutual_aid, MUTUAL_AID_CLASS};
pub use mutual_credit::{
    CommunityStats, CreditLine, CreditLineStatus, CreditScore, CreditTransactionStatus,
    InMemoryMutualCreditStore, MutualCreditStore, MutualCreditTransaction, RepaymentMethod,
    RepaymentRecord,
};
pub use reputation_tokens::{grant_reputation_tokens, use_reputation_tokens, REPUTATION_CLASS};
pub use time_banking::{
    InMemoryTimeBankingStore, TimeBankingStore, TimeRecord, TimeRecordStatus, WorkStatistics,
};

#[cfg(test)]
mod token_tests;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LedgerEvent {
    Credit { did: Did, amount: u64 },
    Debit { did: Did, amount: u64 },
    SetBalance { did: Did, amount: u64 },
}

pub fn balances_from_events(events: &[LedgerEvent]) -> std::collections::HashMap<Did, u64> {
    use std::collections::HashMap;
    let mut bal = HashMap::new();
    for e in events {
        match e {
            LedgerEvent::Credit { did, amount } => {
                *bal.entry(did.clone()).or_insert(0) += *amount;
            }
            LedgerEvent::Debit { did, amount } => {
                let entry = bal.entry(did.clone()).or_insert(0);
                *entry = entry.saturating_sub(*amount);
            }
            LedgerEvent::SetBalance { did, amount } => {
                bal.insert(did.clone(), *amount);
            }
        }
    }
    bal
}

/// Abstraction over the persistence layer storing account balances.
pub trait ManaLedger: Send + Sync {
    /// Retrieve the mana balance for a DID.
    fn get_balance(&self, did: &Did) -> u64;
    /// Persist a new balance for a DID.
    fn set_balance(&self, did: &Did, amount: u64) -> Result<(), CommonError>;
    /// Spend mana from the account.
    fn spend(&self, did: &Did, amount: u64) -> Result<(), CommonError>;
    /// Credit mana to the account.
    fn credit(&self, did: &Did, amount: u64) -> Result<(), CommonError>;
    /// Credit every known account with additional mana.
    ///
    /// The default implementation returns [`CommonError::NotImplementedError`]
    /// if the ledger backend does not support iterating over accounts.
    fn credit_all(&self, amount: u64) -> Result<(), CommonError> {
        let _ = amount;
        Err(CommonError::NotImplementedError(
            "credit_all not implemented for this ledger".into(),
        ))
    }

    /// Returns a list of all known account DIDs.
    ///
    /// The default implementation returns an empty vector if the
    /// underlying ledger does not support account iteration.
    fn all_accounts(&self) -> Vec<Did> {
        Vec::new()
    }
}

impl<T: ManaLedger + ?Sized> ManaLedger for &T {
    fn get_balance(&self, did: &Did) -> u64 {
        (**self).get_balance(did)
    }
    fn set_balance(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
        (**self).set_balance(did, amount)
    }
    fn spend(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
        (**self).spend(did, amount)
    }
    fn credit(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
        (**self).credit(did, amount)
    }
    fn credit_all(&self, amount: u64) -> Result<(), CommonError> {
        (**self).credit_all(amount)
    }
    fn all_accounts(&self) -> Vec<Did> {
        (**self).all_accounts()
    }
}

/// Thin wrapper exposing convenience methods over a [`ManaLedger`].
pub struct ManaRepositoryAdapter<L: ManaLedger> {
    ledger: L,
    #[allow(clippy::type_complexity)]
    event_store: Option<std::sync::Mutex<Box<dyn icn_eventstore::EventStore<LedgerEvent>>>>,
}

impl<L: ManaLedger> ManaRepositoryAdapter<L> {
    /// Construct a new adapter around the provided ledger implementation.
    pub fn new(ledger: L) -> Self {
        ManaRepositoryAdapter {
            ledger,
            event_store: None,
        }
    }

    pub fn with_event_store(
        ledger: L,
        store: Box<dyn icn_eventstore::EventStore<LedgerEvent>>,
    ) -> Self {
        ManaRepositoryAdapter {
            ledger,
            event_store: Some(std::sync::Mutex::new(store)),
        }
    }

    /// Deduct mana from an account via the underlying ledger.
    pub fn spend_mana(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
        metrics::SPEND_MANA_CALLS.inc();
        let res = self.ledger.spend(did, amount);
        if res.is_ok() {
            if let Some(store) = &self.event_store {
                let _ = store.lock().unwrap().append(&LedgerEvent::Debit {
                    did: did.clone(),
                    amount,
                });
            }
        }
        res
    }

    /// Retrieve the account balance.
    pub fn get_balance(&self, did: &Did) -> u64 {
        metrics::GET_BALANCE_CALLS.inc();
        self.ledger.get_balance(did)
    }

    /// Credits the specified account with additional mana.
    pub fn credit_mana(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
        metrics::CREDIT_MANA_CALLS.inc();
        let res = self.ledger.credit(did, amount);
        if res.is_ok() {
            if let Some(store) = &self.event_store {
                let _ = store.lock().unwrap().append(&LedgerEvent::Credit {
                    did: did.clone(),
                    amount,
                });
            }
        }
        res
    }

    pub fn set_balance(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
        let res = self.ledger.set_balance(did, amount);
        if res.is_ok() {
            if let Some(store) = &self.event_store {
                let _ = store.lock().unwrap().append(&LedgerEvent::SetBalance {
                    did: did.clone(),
                    amount,
                });
            }
        }
        res
    }

    pub fn event_store(
        &self,
    ) -> Option<&std::sync::Mutex<Box<dyn icn_eventstore::EventStore<LedgerEvent>>>> {
        self.event_store.as_ref()
    }
}

/// Enforces spending limits and forwards to a [`ManaRepositoryAdapter`].
pub struct ResourcePolicyEnforcer<L: ManaLedger> {
    adapter: ManaRepositoryAdapter<L>,
}

impl<L: ManaLedger> ResourcePolicyEnforcer<L> {
    /// Maximum mana spend allowed per single operation.
    pub const MAX_SPEND_LIMIT: u64 = 1000;

    /// Create a new enforcer using the supplied adapter.
    pub fn new(adapter: ManaRepositoryAdapter<L>) -> Self {
        ResourcePolicyEnforcer { adapter }
    }

    /// Spend mana after applying basic policy checks.
    pub fn spend_mana(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
        debug!("[ResourcePolicyEnforcer] Enforcing spend_mana for DID {did:?}, amount {amount}");

        if amount == 0 {
            return Err(CommonError::PolicyDenied(
                "Spend amount must be greater than zero".into(),
            ));
        }

        let available = self.adapter.get_balance(did);
        if available < amount {
            return Err(CommonError::PolicyDenied(format!(
                "Insufficient mana for DID {did}"
            )));
        }

        if amount > Self::MAX_SPEND_LIMIT {
            return Err(CommonError::PolicyDenied(format!(
                "Spend amount {amount} exceeds limit {limit}",
                limit = Self::MAX_SPEND_LIMIT
            )));
        }

        self.adapter.spend_mana(did, amount)
    }
}

/// Exposes a public function to charge mana, wrapping ResourcePolicyEnforcer with enhanced validation.
pub fn charge_mana<L: ManaLedger>(ledger: L, did: &Did, amount: u64) -> Result<(), CommonError> {
    let start_time = std::time::Instant::now();
    
    // Enhanced validation before charging
    let validation_context = transaction_validation::ValidationContext {
        operation_type: "charge_mana".to_string(),
        amount,
        account: did.clone(),
        resource_type: None,
        cross_cooperative: false,
        reputation_required: false,
    };

    let validation_result = transaction_validation::validate_mana_spend(
        &ledger,
        did,
        amount,
        &validation_context,
    );

    if !validation_result.is_valid {
        // Record failed validation
        if let Ok(mut registry) = metrics::METRICS_REGISTRY.write() {
            registry.record_validation(false, validation_result.validation_time_ms);
        }
        return Err(CommonError::PolicyDenied(
            validation_result.error_message.unwrap_or_else(|| "Mana charge validation failed".to_string())
        ));
    }

    // Log validation warnings
    for warning in &validation_result.warnings {
        log::warn!("Mana charge warning for {}: {}", did, warning);
    }

    let mana_adapter = ManaRepositoryAdapter::new(ledger);
    let policy_enforcer = ResourcePolicyEnforcer::new(mana_adapter);

    info!("[icn-economics] charge_mana called for DID {did:?}, amount {amount}");
    let result = policy_enforcer.spend_mana(did, amount);
    
    // Record performance metrics
    let execution_time = start_time.elapsed().as_millis() as f64;
    if let Ok(mut registry) = metrics::METRICS_REGISTRY.write() {
        registry.record_mana_operation(execution_time);
        registry.record_validation(result.is_ok(), validation_result.validation_time_ms);
        
        if result.is_err() {
            registry.record_insufficient_mana();
        }
    }

    result
}

/// Credits mana to the given DID using the provided ledger.
pub fn credit_mana<L: ManaLedger>(ledger: L, did: &Did, amount: u64) -> Result<(), CommonError> {
    let mana_adapter = ManaRepositoryAdapter::new(ledger);
    info!("[icn-economics] credit_mana called for DID {did:?}, amount {amount}");
    mana_adapter.credit_mana(did, amount)
}

/// Events emitted when resource token balances change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokenEvent {
    Mint {
        class_id: String,
        amount: u64,
        issuer: Did,
        recipient: Did,
        scope: Option<NodeScope>,
    },
    Burn {
        class_id: String,
        amount: u64,
        issuer: Did,
        owner: Did,
        scope: Option<NodeScope>,
    },
    Transfer {
        class_id: String,
        amount: u64,
        issuer: Did,
        from: Did,
        to: Did,
        scope: Option<NodeScope>,
    },
}

/// Adapter over a [`ResourceLedger`] with optional DAG event recording.
pub struct ResourceRepositoryAdapter<L: ResourceLedger> {
    ledger: L,
    issuers: std::collections::HashMap<NodeScope, std::collections::HashSet<Did>>,
    dag_store: Option<std::sync::Mutex<Box<dyn StorageService<DagBlock>>>>,
}

impl<L: ResourceLedger> ResourceRepositoryAdapter<L> {
    pub fn new(ledger: L) -> Self {
        Self {
            ledger,
            issuers: std::collections::HashMap::new(),
            dag_store: None,
        }
    }

    pub fn with_dag_store(ledger: L, dag: Box<dyn StorageService<DagBlock>>) -> Self {
        Self {
            ledger,
            issuers: std::collections::HashMap::new(),
            dag_store: Some(std::sync::Mutex::new(dag)),
        }
    }

    pub fn add_issuer(&mut self, scope: NodeScope, issuer: Did) {
        self.issuers.entry(scope).or_default().insert(issuer);
    }

    pub fn ledger(&self) -> &L {
        &self.ledger
    }

    fn is_authorized(&self, issuer: &Did, scope: &NodeScope) -> bool {
        self.issuers
            .get(scope)
            .map(|s| s.contains(issuer))
            .unwrap_or(false)
    }

    fn record_event(&self, event: &TokenEvent) {
        if let Some(store) = &self.dag_store {
            let mut store = store.lock().unwrap();
            let data = serde_json::to_vec(event).unwrap_or_default();
            let author = match event {
                TokenEvent::Mint { issuer, .. } => issuer.clone(),
                TokenEvent::Burn { issuer, .. } => issuer.clone(),
                TokenEvent::Transfer { issuer, .. } => issuer.clone(),
            };
            let scope = match event {
                TokenEvent::Mint { scope, .. }
                | TokenEvent::Burn { scope, .. }
                | TokenEvent::Transfer { scope, .. } => scope.clone(),
            };
            let ts = SystemTimeProvider.unix_seconds();
            let cid = compute_merkle_cid(0x71, &data, &[], ts, &author, &None, &scope);
            let block = DagBlock {
                cid,
                data,
                links: vec![],
                timestamp: ts,
                author_did: author,
                signature: None,
                scope,
            };
            let _ = store.put(&block);
        }
    }

    pub fn mint(
        &self,
        issuer: &Did,
        class_id: &str,
        amount: u64,
        recipient: &Did,
        scope: Option<NodeScope>,
    ) -> Result<(), CommonError> {
        if let Some(sc) = &scope {
            if !self.is_authorized(issuer, sc) {
                return Err(CommonError::PolicyDenied("issuer not authorized".into()));
            }
        }
        self.ledger.mint(&class_id.to_string(), recipient, amount)?;
        self.record_event(&TokenEvent::Mint {
            class_id: class_id.to_string(),
            amount,
            issuer: issuer.clone(),
            recipient: recipient.clone(),
            scope,
        });
        Ok(())
    }

    pub fn burn(
        &self,
        issuer: &Did,
        class_id: &str,
        amount: u64,
        owner: &Did,
        scope: Option<NodeScope>,
    ) -> Result<(), CommonError> {
        if let Some(sc) = &scope {
            if !self.is_authorized(issuer, sc) {
                return Err(CommonError::PolicyDenied("issuer not authorized".into()));
            }
        }
        self.ledger.burn(&class_id.to_string(), owner, amount)?;
        self.record_event(&TokenEvent::Burn {
            class_id: class_id.to_string(),
            amount,
            issuer: issuer.clone(),
            owner: owner.clone(),
            scope,
        });
        Ok(())
    }

    pub fn transfer(
        &self,
        issuer: &Did,
        class_id: &str,
        amount: u64,
        from: &Did,
        to: &Did,
        scope: Option<NodeScope>,
    ) -> Result<(), CommonError> {
        if let Some(sc) = &scope {
            if !self.is_authorized(issuer, sc) {
                return Err(CommonError::PolicyDenied("issuer not authorized".into()));
            }
        }
        self.ledger
            .transfer(&class_id.to_string(), from, to, amount)?;
        self.record_event(&TokenEvent::Transfer {
            class_id: class_id.to_string(),
            amount,
            issuer: issuer.clone(),
            from: from.clone(),
            to: to.clone(),
            scope,
        });
        Ok(())
    }
}

const TOKEN_FEE: u64 = 1;

/// Execute a marketplace transaction by transferring tokens and updating records with enhanced validation.
pub fn execute_marketplace_transaction<
    L: ResourceLedger,
    M: ManaLedger,
    S: marketplace::MarketplaceStore,
>(
    resource_repo: &ResourceRepositoryAdapter<L>,
    mana_ledger: &M,
    marketplace_store: &S,
    offer_id: &str,
    bid_id: &str,
    executor: &Did, // Could be seller, buyer, or marketplace operator
) -> Result<marketplace::MarketplaceTransaction, CommonError> {
    let start_time = std::time::Instant::now();
    
    // Get the offer and bid
    let offer = marketplace_store
        .get_offer(offer_id)
        .ok_or_else(|| CommonError::InvalidInputError(format!("Offer {offer_id} not found")))?;

    let bid = marketplace_store
        .get_bid(bid_id)
        .ok_or_else(|| CommonError::InvalidInputError(format!("Bid {bid_id} not found")))?;

    // Enhanced validation before execution
    let validation_result = transaction_validation::validate_marketplace_transaction(
        &offer,
        &bid,
        resource_repo.ledger(),
        mana_ledger,
    );

    if !validation_result.is_valid {
        metrics::POLICY_VIOLATIONS.inc();
        return Err(CommonError::PolicyDenied(
            validation_result.error_message.unwrap_or_else(|| "Marketplace transaction validation failed".to_string())
        ));
    }

    // Log validation warnings if any
    for warning in &validation_result.warnings {
        log::warn!("Marketplace transaction warning: {}", warning);
    }

    // Calculate total price
    let total_price = bid.price_per_unit * bid.quantity;

    // Transfer tokens from buyer to seller with validation
    let transfer_context = transaction_validation::ValidationContext {
        operation_type: "marketplace_transfer".to_string(),
        amount: total_price,
        account: bid.buyer.clone(),
        resource_type: Some(bid.payment_token_class.clone()),
        cross_cooperative: true, // Marketplace transactions are cross-cooperative by nature
        reputation_required: false,
    };

    let transfer_validation = transaction_validation::validate_token_transfer(
        resource_repo.ledger(),
        &bid.payment_token_class,
        &bid.buyer,
        &offer.seller,
        total_price,
        &transfer_context,
    );

    if !transfer_validation.is_valid {
        return Err(CommonError::PolicyDenied(
            transfer_validation.error_message.unwrap_or_else(|| "Token transfer validation failed".to_string())
        ));
    }

    resource_repo.transfer(
        executor,
        &bid.payment_token_class,
        total_price,
        &bid.buyer,
        &offer.seller,
        None, // marketplace transactions can be cross-scope
    )?;

    // Charge mana fee for marketplace transaction with validation
    let mana_context = transaction_validation::ValidationContext {
        operation_type: "marketplace_fee".to_string(),
        amount: TOKEN_FEE,
        account: executor.clone(),
        resource_type: None,
        cross_cooperative: true,
        reputation_required: false,
    };

    let mana_validation = transaction_validation::validate_mana_spend(
        mana_ledger,
        executor,
        TOKEN_FEE,
        &mana_context,
    );

    if !mana_validation.is_valid {
        return Err(CommonError::PolicyDenied(
            mana_validation.error_message.unwrap_or_else(|| "Mana fee validation failed".to_string())
        ));
    }

    charge_mana(mana_ledger, executor, TOKEN_FEE)?;

    // Create transaction record
    let transaction = marketplace::MarketplaceTransaction {
        transaction_id: format!("tx_{offer_id}_{bid_id}"),
        offer_id: offer.offer_id.clone(),
        bid_id: bid.bid_id.clone(),
        seller: offer.seller.clone(),
        buyer: bid.buyer.clone(),
        item_type: offer.item_type.clone(),
        quantity: bid.quantity,
        price_per_unit: bid.price_per_unit,
        total_price,
        payment_token_class: bid.payment_token_class.clone(),
        completed_at: SystemTimeProvider.unix_seconds(),
        status: marketplace::TransactionStatus::Pending,
        fulfillment: marketplace::FulfillmentDetails {
            method: marketplace::FulfillmentMethod::Remote, // Default, should be updated
            expected_date: None,
            actual_date: None,
            tracking_info: None,
        },
    };

    // Record the transaction
    marketplace_store.record_transaction(transaction.clone())?;

    // Update offer quantity
    let mut updated_offer = offer.clone();
    updated_offer.quantity -= bid.quantity;
    if updated_offer.quantity == 0 {
        updated_offer.status = marketplace::OfferStatus::Fulfilled;
    }
    marketplace_store.update_offer(updated_offer)?;

    // Update bid status
    let mut updated_bid = bid.clone();
    updated_bid.status = marketplace::BidStatus::Accepted;
    marketplace_store.update_bid(updated_bid)?;

    // Record performance metrics
    let execution_time = start_time.elapsed().as_millis() as f64;
    if let Ok(mut registry) = metrics::METRICS_REGISTRY.write() {
        registry.record_cross_cooperative_share(total_price, 1.0);
        registry.record_validation(true, validation_result.validation_time_ms);
    }
    
    // Update marketplace metrics
    metrics::helpers::record_marketplace_transaction(total_price);
    metrics::TRANSACTION_DURATION.observe(execution_time / 1000.0);

    Ok(transaction)
}

/// Create a marketplace offer with token validation.
pub fn create_marketplace_offer<
    L: ResourceLedger,
    M: ManaLedger,
    S: marketplace::MarketplaceStore,
>(
    resource_ledger: &L,
    mana_ledger: &M,
    marketplace_store: &S,
    offer: marketplace::MarketplaceOffer,
) -> Result<(), CommonError> {
    // Validate that the payment token class exists
    if resource_ledger
        .get_class(&offer.payment_token_class)
        .is_none()
    {
        return Err(CommonError::InvalidInputError(format!(
            "Payment token class {} does not exist",
            offer.payment_token_class
        )));
    }

    // Charge mana fee for creating offer
    charge_mana(mana_ledger, &offer.seller, TOKEN_FEE)?;

    // Create the offer
    marketplace_store.create_offer(offer)?;

    Ok(())
}

/// Create a marketplace bid with balance validation.
pub fn create_marketplace_bid<
    L: ResourceLedger,
    M: ManaLedger,
    S: marketplace::MarketplaceStore,
>(
    resource_ledger: &L,
    mana_ledger: &M,
    marketplace_store: &S,
    bid: marketplace::MarketplaceBid,
) -> Result<(), CommonError> {
    // Validate that the offer exists
    let _offer = marketplace_store.get_offer(&bid.offer_id).ok_or_else(|| {
        CommonError::InvalidInputError(format!("Offer {} not found", bid.offer_id))
    })?;

    // Validate that buyer has sufficient token balance
    let required_tokens = bid.price_per_unit * bid.quantity;
    let buyer_balance = resource_ledger.get_balance(&bid.payment_token_class, &bid.buyer);

    if buyer_balance < required_tokens {
        return Err(CommonError::PolicyDenied(
            "Insufficient token balance for bid".into(),
        ));
    }

    // Charge mana fee for creating bid
    charge_mana(mana_ledger, &bid.buyer, TOKEN_FEE)?;

    // Create the bid
    marketplace_store.create_bid(bid)?;

    Ok(())
}

/// Enhanced token minting with comprehensive validation and metrics
pub fn mint_tokens<L: ResourceLedger, M: ManaLedger>(
    repo: &ResourceRepositoryAdapter<L>,
    mana_ledger: &M,
    issuer: &Did,
    class_id: &str,
    amount: u64,
    recipient: &Did,
    scope: Option<NodeScope>,
) -> Result<(), CommonError> {
    let start_time = std::time::Instant::now();
    
    // Validate mana charge first
    let mana_context = transaction_validation::ValidationContext {
        operation_type: "mint_tokens_fee".to_string(),
        amount: TOKEN_FEE,
        account: issuer.clone(),
        resource_type: None,
        cross_cooperative: scope.is_some(),
        reputation_required: true,
    };

    let mana_validation = transaction_validation::validate_mana_spend(
        mana_ledger,
        issuer,
        TOKEN_FEE,
        &mana_context,
    );

    if !mana_validation.is_valid {
        return Err(CommonError::PolicyDenied(
            mana_validation.error_message.unwrap_or_else(|| "Mana validation failed for token minting".to_string())
        ));
    }

    // Charge mana fee
    charge_mana(mana_ledger, issuer, TOKEN_FEE)?;
    
    // Execute the mint operation
    let result = repo.mint(issuer, class_id, amount, recipient, scope);
    
    // Record metrics
    let execution_time = start_time.elapsed().as_millis() as f64;
    if result.is_ok() {
        metrics::helpers::record_token_operation("mint", amount);
        if let Ok(mut registry) = metrics::METRICS_REGISTRY.write() {
            registry.record_allocation_performance(true, execution_time, 1.0);
        }
    } else {
        if let Ok(mut registry) = metrics::METRICS_REGISTRY.write() {
            registry.record_allocation_performance(false, execution_time, 0.0);
        }
    }
    
    result
}

/// Enhanced token burning with comprehensive validation and metrics
pub fn burn_tokens<L: ResourceLedger, M: ManaLedger>(
    repo: &ResourceRepositoryAdapter<L>,
    mana_ledger: &M,
    issuer: &Did,
    class_id: &str,
    amount: u64,
    owner: &Did,
    scope: Option<NodeScope>,
) -> Result<(), CommonError> {
    let start_time = std::time::Instant::now();
    
    // Validate token burn operation
    let burn_context = transaction_validation::ValidationContext {
        operation_type: "burn_tokens".to_string(),
        amount,
        account: owner.clone(),
        resource_type: Some(class_id.to_string()),
        cross_cooperative: scope.is_some(),
        reputation_required: true,
    };

    // Check if owner has sufficient tokens (basic validation)
    let owner_balance = repo.ledger().get_balance(&class_id.to_string(), owner);
    if owner_balance < amount {
        return Err(CommonError::PolicyDenied(format!(
            "Insufficient tokens for burn: available={}, required={}",
            owner_balance, amount
        )));
    }

    // Validate mana charge
    let mana_context = transaction_validation::ValidationContext {
        operation_type: "burn_tokens_fee".to_string(),
        amount: TOKEN_FEE,
        account: issuer.clone(),
        resource_type: None,
        cross_cooperative: scope.is_some(),
        reputation_required: true,
    };

    let mana_validation = transaction_validation::validate_mana_spend(
        mana_ledger,
        issuer,
        TOKEN_FEE,
        &mana_context,
    );

    if !mana_validation.is_valid {
        return Err(CommonError::PolicyDenied(
            mana_validation.error_message.unwrap_or_else(|| "Mana validation failed for token burning".to_string())
        ));
    }

    // Charge mana fee
    charge_mana(mana_ledger, issuer, TOKEN_FEE)?;
    
    // Execute the burn operation
    let result = repo.burn(issuer, class_id, amount, owner, scope);
    
    // Record metrics
    let execution_time = start_time.elapsed().as_millis() as f64;
    if result.is_ok() {
        metrics::helpers::record_token_operation("burn", amount);
        if let Ok(mut registry) = metrics::METRICS_REGISTRY.write() {
            registry.record_allocation_performance(true, execution_time, 1.0);
        }
    } else {
        if let Ok(mut registry) = metrics::METRICS_REGISTRY.write() {
            registry.record_allocation_performance(false, execution_time, 0.0);
        }
    }
    
    result
}
/// Mint tokens with reputation considerations (currently just wraps mint_tokens)
pub fn mint_tokens_with_reputation<L: ResourceLedger, M: ManaLedger>(
    repo: &ResourceRepositoryAdapter<L>,
    mana_ledger: &M,
    _reputation_store: &dyn icn_reputation::ReputationStore,
    issuer: &Did,
    class_id: &str,
    amount: u64,
    recipient: &Did,
    scope: Option<NodeScope>,
) -> Result<(), CommonError> {
    // For now, this is just a wrapper around mint_tokens
    // In a full implementation, this would check reputation requirements
    mint_tokens(repo, mana_ledger, issuer, class_id, amount, recipient, scope)
}

/// Enhanced token transfer with comprehensive validation and metrics
#[allow(clippy::too_many_arguments)]
pub fn transfer_tokens<L: ResourceLedger, M: ManaLedger>(
    repo: &ResourceRepositoryAdapter<L>,
    mana_ledger: &M,
    issuer: &Did,
    class_id: &str,
    amount: u64,
    from: &Did,
    to: &Did,
    scope: Option<NodeScope>,
) -> Result<(), CommonError> {
    let start_time = std::time::Instant::now();
    
    // Validate token transfer
    let transfer_context = transaction_validation::ValidationContext {
        operation_type: "transfer_tokens".to_string(),
        amount,
        account: from.clone(),
        resource_type: Some(class_id.to_string()),
        cross_cooperative: scope.is_some() || from != to,
        reputation_required: false,
    };

    let transfer_validation = transaction_validation::validate_token_transfer(
        repo.ledger(),
        class_id,
        from,
        to,
        amount,
        &transfer_context,
    );

    if !transfer_validation.is_valid {
        return Err(CommonError::PolicyDenied(
            transfer_validation.error_message.unwrap_or_else(|| "Token transfer validation failed".to_string())
        ));
    }

    // Log transfer warnings
    for warning in &transfer_validation.warnings {
        log::warn!("Token transfer warning: {}", warning);
    }

    // Validate mana charge
    let mana_context = transaction_validation::ValidationContext {
        operation_type: "transfer_tokens_fee".to_string(),
        amount: TOKEN_FEE,
        account: issuer.clone(),
        resource_type: None,
        cross_cooperative: scope.is_some(),
        reputation_required: false,
    };

    let mana_validation = transaction_validation::validate_mana_spend(
        mana_ledger,
        issuer,
        TOKEN_FEE,
        &mana_context,
    );

    if !mana_validation.is_valid {
        return Err(CommonError::PolicyDenied(
            mana_validation.error_message.unwrap_or_else(|| "Mana validation failed for token transfer".to_string())
        ));
    }

    // Charge mana fee
    charge_mana(mana_ledger, issuer, TOKEN_FEE)?;
    
    // Execute the transfer operation
    let result = repo.transfer(issuer, class_id, amount, from, to, scope);
    
    // Record metrics
    let execution_time = start_time.elapsed().as_millis() as f64;
    if result.is_ok() {
        metrics::helpers::record_token_operation("transfer", amount);
        let efficiency = if transfer_context.cross_cooperative { 0.9 } else { 1.0 };
        if let Ok(mut registry) = metrics::METRICS_REGISTRY.write() {
            registry.record_allocation_performance(true, execution_time, efficiency);
            if transfer_context.cross_cooperative {
                registry.record_cross_cooperative_share(amount, efficiency);
            }
        }
    } else {
        if let Ok(mut registry) = metrics::METRICS_REGISTRY.write() {
            registry.record_allocation_performance(false, execution_time, 0.0);
            if transfer_context.cross_cooperative {
                registry.record_cross_cooperative_failure();
            }
        }
    }
    
    result
}

/// Credits mana to all known accounts using their reputation scores.
///
/// Each account receives `base_amount * reputation_score` mana.
pub fn credit_by_reputation(
    ledger: &dyn ManaLedger,
    reputation_store: &dyn icn_reputation::ReputationStore,
    base_amount: u64,
) -> Result<(), CommonError> {
    for did in ledger.all_accounts() {
        let rep = reputation_store.get_reputation(&did);
        let credit_amount = rep.saturating_mul(base_amount);
        ledger.credit(&did, credit_amount)?;
    }
    Ok(())
}

/// Calculates the final mana price for a resource based on reputation.
///
/// Higher reputation results in a lower price. The formula is:
/// `price = base_price * 100 / (100 + reputation)`.
///
/// This provides diminishing returns so reputation never drops the cost below
/// zero and ensures a fair discount curve.
pub fn price_by_reputation(base_price: u64, reputation: u64) -> u64 {
    let denom = 100u128 + reputation as u128;
    let num = (base_price as u128) * 100u128;
    (num / denom) as u64
}

/// Resource specification for cross-cooperative allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSpec {
    pub resource_type: String,
    pub amount: u64,
    pub duration: Option<u64>, // Duration in seconds
    pub priority: ResourcePriority,
    pub requirements: HashMap<String, String>,
}

/// Resource priority levels for allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourcePriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Enhanced transaction validation for economic operations
pub mod transaction_validation {
    use super::*;
    use crate::metrics::{METRICS_REGISTRY, POLICY_VIOLATIONS};
    use std::time::Instant;

    /// Comprehensive validation result
    #[derive(Debug, Clone)]
    pub struct ValidationResult {
        pub is_valid: bool,
        pub error_message: Option<String>,
        pub warnings: Vec<String>,
        pub validation_time_ms: f64,
        pub checks_performed: Vec<String>,
    }

    /// Validation context for economic operations
    #[derive(Debug, Clone)]
    pub struct ValidationContext {
        pub operation_type: String,
        pub amount: u64,
        pub account: Did,
        pub resource_type: Option<String>,
        pub cross_cooperative: bool,
        pub reputation_required: bool,
    }

    // In-memory ledger type for validation
    #[derive(Default)]
    struct InMemoryLedger {
        balances: std::sync::Mutex<std::collections::HashMap<Did, u64>>,
    }

    impl ManaLedger for InMemoryLedger {
        fn get_balance(&self, did: &Did) -> u64 {
            *self.balances.lock().unwrap().get(did).unwrap_or(&0)
        }
        fn set_balance(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
            self.balances.lock().unwrap().insert(did.clone(), amount);
            Ok(())
        }
        fn spend(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
            let mut map = self.balances.lock().unwrap();
            let bal = map.entry(did.clone()).or_insert(0);
            if *bal < amount {
                return Err(CommonError::PolicyDenied("insufficient".into()));
            }
            *bal -= amount;
            Ok(())
        }
        fn credit(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
            let mut map = self.balances.lock().unwrap();
            let entry = map.entry(did.clone()).or_insert(0);
            *entry += amount;
            Ok(())
        }
    }

    /// Enhanced mana spend validation with comprehensive checks
    pub fn validate_mana_spend(
        ledger: &dyn ManaLedger,
        did: &Did,
        amount: u64,
        context: &ValidationContext,
    ) -> ValidationResult {
        let start = Instant::now();
        let mut warnings = Vec::new();
        let mut checks = Vec::new();

        // Check 1: Basic amount validation
        checks.push("amount_validation".to_string());
        if amount == 0 {
            return ValidationResult {
                is_valid: false,
                error_message: Some("Amount must be greater than zero".to_string()),
                warnings,
                validation_time_ms: start.elapsed().as_millis() as f64,
                checks_performed: checks,
            };
        }

        // Check 2: Balance sufficiency with buffer for fees
        checks.push("balance_sufficiency".to_string());
        let current_balance = ledger.get_balance(did);
        let required_amount = if context.cross_cooperative {
            // Cross-cooperative operations may incur additional fees
            amount + (amount / 10).max(1) // 10% fee minimum 1 mana
        } else {
            amount
        };

        if current_balance < required_amount {
            return ValidationResult {
                is_valid: false,
                error_message: Some(format!(
                    "Insufficient mana: available={}, required={} (including fees)",
                    current_balance, required_amount
                )),
                warnings,
                validation_time_ms: start.elapsed().as_millis() as f64,
                checks_performed: checks,
            };
        }

        // Check 3: Spending rate limits
        checks.push("rate_limits".to_string());
        if amount > ResourcePolicyEnforcer::<InMemoryLedger>::MAX_SPEND_LIMIT {
            return ValidationResult {
                is_valid: false,
                error_message: Some(format!(
                    "Amount {} exceeds maximum spend limit {}",
                    amount, ResourcePolicyEnforcer::<InMemoryLedger>::MAX_SPEND_LIMIT
                )),
                warnings,
                validation_time_ms: start.elapsed().as_millis() as f64,
                checks_performed: checks,
            };
        }

        // Check 4: Low balance warning
        checks.push("balance_warning".to_string());
        let remaining_after = current_balance - required_amount;
        if remaining_after < 100 { // Low balance threshold
            warnings.push(format!(
                "Low balance warning: {} mana remaining after operation",
                remaining_after
            ));
        }

        // Check 5: Large spend warning
        checks.push("large_spend_warning".to_string());
        if amount > current_balance / 2 {
            warnings.push("Large spend detected: spending more than 50% of balance".to_string());
        }

        let validation_time = start.elapsed().as_millis() as f64;

        // Record validation metrics
        if let Ok(mut registry) = METRICS_REGISTRY.write() {
            registry.record_validation(true, validation_time);
        }

        ValidationResult {
            is_valid: true,
            error_message: None,
            warnings,
            validation_time_ms: validation_time,
            checks_performed: checks,
        }
    }

    /// Enhanced token transfer validation
    pub fn validate_token_transfer(
        ledger: &dyn ResourceLedger,
        class_id: &str,
        from: &Did,
        to: &Did,
        amount: u64,
        context: &ValidationContext,
    ) -> ValidationResult {
        let start = Instant::now();
        let mut warnings = Vec::new();
        let mut checks = Vec::new();

        // Check 1: Basic parameters
        checks.push("parameter_validation".to_string());
        if amount == 0 {
            return ValidationResult {
                is_valid: false,
                error_message: Some("Transfer amount must be greater than zero".to_string()),
                warnings,
                validation_time_ms: start.elapsed().as_millis() as f64,
                checks_performed: checks,
            };
        }

        if from == to {
            return ValidationResult {
                is_valid: false,
                error_message: Some("Cannot transfer to self".to_string()),
                warnings,
                validation_time_ms: start.elapsed().as_millis() as f64,
                checks_performed: checks,
            };
        }

        // Check 2: Token class exists
        checks.push("token_class_exists".to_string());
        let token_class = match ledger.get_class(&class_id.to_string()) {
            Some(class) => class,
            None => {
                return ValidationResult {
                    is_valid: false,
                    error_message: Some(format!("Token class '{}' not found", class_id)),
                    warnings,
                    validation_time_ms: start.elapsed().as_millis() as f64,
                    checks_performed: checks,
                };
            }
        };

        // Check 3: Transfer allowed by token rules
        checks.push("transferability_rules".to_string());
        match ledger.can_transfer(&class_id.to_string(), from, to, amount) {
            Ok(true) => {},
            Ok(false) => {
                return ValidationResult {
                    is_valid: false,
                    error_message: Some("Transfer not allowed by token rules".to_string()),
                    warnings,
                    validation_time_ms: start.elapsed().as_millis() as f64,
                    checks_performed: checks,
                };
            },
            Err(e) => {
                return ValidationResult {
                    is_valid: false,
                    error_message: Some(format!("Transfer validation failed: {}", e)),
                    warnings,
                    validation_time_ms: start.elapsed().as_millis() as f64,
                    checks_performed: checks,
                };
            }
        }

        // Check 4: Sufficient balance
        checks.push("balance_check".to_string());
        let from_balance = ledger.get_balance(&class_id.to_string(), from);
        if from_balance < amount {
            return ValidationResult {
                is_valid: false,
                error_message: Some(format!(
                    "Insufficient token balance: available={}, required={}",
                    from_balance, amount
                )),
                warnings,
                validation_time_ms: start.elapsed().as_millis() as f64,
                checks_performed: checks,
            };
        }

        // Check 5: Token validity period
        checks.push("validity_period".to_string());
        if let Some((start_time, end_time)) = token_class.scoping_rules.validity_period {
            let current_time = icn_common::SystemTimeProvider.unix_seconds();
            if current_time < start_time || current_time > end_time {
                return ValidationResult {
                    is_valid: false,
                    error_message: Some("Token is outside its validity period".to_string()),
                    warnings,
                    validation_time_ms: start.elapsed().as_millis() as f64,
                    checks_performed: checks,
                };
            }
        }

        // Check 6: Supply limits
        checks.push("supply_limits".to_string());
        if let Some(max_supply) = token_class.scoping_rules.max_supply {
            // This would need a way to track total supply - for now just warn
            warnings.push(format!("Token class has max supply limit: {}", max_supply));
        }

        // Check 7: Large transfer warning
        checks.push("large_transfer_warning".to_string());
        if amount > from_balance / 2 {
            warnings.push("Large transfer: more than 50% of sender's balance".to_string());
        }

        // Check 8: Cross-cooperative transfer
        checks.push("cross_cooperative_check".to_string());
        if context.cross_cooperative {
            warnings.push("Cross-cooperative transfer detected".to_string());
        }

        let validation_time = start.elapsed().as_millis() as f64;

        // Record validation metrics
        if let Ok(mut registry) = METRICS_REGISTRY.write() {
            registry.record_validation(true, validation_time);
        }

        ValidationResult {
            is_valid: true,
            error_message: None,
            warnings,
            validation_time_ms: validation_time,
            checks_performed: checks,
        }
    }

    /// Validate marketplace transaction before execution
    pub fn validate_marketplace_transaction(
        offer: &marketplace::MarketplaceOffer,
        bid: &marketplace::MarketplaceBid,
        resource_ledger: &dyn ResourceLedger,
        mana_ledger: &dyn ManaLedger,
    ) -> ValidationResult {
        let start = Instant::now();
        let mut warnings = Vec::new();
        let mut checks = Vec::new();

        // Check 1: Offer and bid compatibility
        checks.push("offer_bid_compatibility".to_string());
        if bid.offer_id != offer.offer_id {
            return ValidationResult {
                is_valid: false,
                error_message: Some("Bid does not match offer".to_string()),
                warnings,
                validation_time_ms: start.elapsed().as_millis() as f64,
                checks_performed: checks,
            };
        }

        // Check 2: Quantity availability
        checks.push("quantity_availability".to_string());
        if bid.quantity > offer.quantity {
            return ValidationResult {
                is_valid: false,
                error_message: Some("Bid quantity exceeds available quantity".to_string()),
                warnings,
                validation_time_ms: start.elapsed().as_millis() as f64,
                checks_performed: checks,
            };
        }

        // Check 3: Buyer has sufficient payment tokens
        checks.push("buyer_balance".to_string());
        let total_cost = bid.price_per_unit * bid.quantity;
        let buyer_balance = resource_ledger.get_balance(&bid.payment_token_class, &bid.buyer);
        if buyer_balance < total_cost {
            return ValidationResult {
                is_valid: false,
                error_message: Some(format!(
                    "Buyer has insufficient tokens: available={}, required={}",
                    buyer_balance, total_cost
                )),
                warnings,
                validation_time_ms: start.elapsed().as_millis() as f64,
                checks_performed: checks,
            };
        }

        // Check 4: Buyer has sufficient mana for transaction fee
        checks.push("buyer_mana".to_string());
        let buyer_mana = mana_ledger.get_balance(&bid.buyer);
        if buyer_mana < TOKEN_FEE {
            return ValidationResult {
                is_valid: false,
                error_message: Some(format!(
                    "Buyer has insufficient mana for transaction fee: available={}, required={}",
                    buyer_mana, TOKEN_FEE
                )),
                warnings,
                validation_time_ms: start.elapsed().as_millis() as f64,
                checks_performed: checks,
            };
        }

        // Check 5: Offer and bid status
        checks.push("status_check".to_string());
        if offer.status != marketplace::OfferStatus::Active {
            return ValidationResult {
                is_valid: false,
                error_message: Some("Offer is not active".to_string()),
                warnings,
                validation_time_ms: start.elapsed().as_millis() as f64,
                checks_performed: checks,
            };
        }

        if bid.status != marketplace::BidStatus::Active {
            return ValidationResult {
                is_valid: false,
                error_message: Some("Bid is not active".to_string()),
                warnings,
                validation_time_ms: start.elapsed().as_millis() as f64,
                checks_performed: checks,
            };
        }

        // Check 6: Price reasonableness (warning only)
        checks.push("price_reasonableness".to_string());
        let price_difference = if bid.price_per_unit > offer.price_per_unit {
            (bid.price_per_unit - offer.price_per_unit) as f64 / offer.price_per_unit as f64
        } else {
            0.0
        };

        if price_difference > 0.2 { // 20% above asking price
            warnings.push(format!(
                "Bid price is {:.1}% above asking price",
                price_difference * 100.0
            ));
        }

        // Check 7: Large transaction warning
        checks.push("large_transaction_warning".to_string());
        if total_cost > 10000 { // Large transaction threshold
            warnings.push(format!("Large transaction detected: {} tokens", total_cost));
        }

        let validation_time = start.elapsed().as_millis() as f64;

        // Record validation metrics
        if let Ok(mut registry) = METRICS_REGISTRY.write() {
            registry.record_validation(true, validation_time);
        }

        ValidationResult {
            is_valid: true,
            error_message: None,
            warnings,
            validation_time_ms: validation_time,
            checks_performed: checks,
        }
    }

    /// Validate cross-cooperative resource allocation
    pub fn validate_cross_cooperative_allocation(
        requester: &Did,
        resource_spec: &ResourceSpec,
        available_resources: &HashMap<String, u64>,
        reputation_store: &dyn icn_reputation::ReputationStore,
    ) -> ValidationResult {
        let start = Instant::now();
        let mut warnings = Vec::new();
        let mut checks = Vec::new();

        // Check 1: Resource availability
        checks.push("resource_availability".to_string());
        let available = available_resources.get(&resource_spec.resource_type).unwrap_or(&0);
        if *available < resource_spec.amount {
            return ValidationResult {
                is_valid: false,
                error_message: Some(format!(
                    "Insufficient resources: available={}, requested={}",
                    available, resource_spec.amount
                )),
                warnings,
                validation_time_ms: start.elapsed().as_millis() as f64,
                checks_performed: checks,
            };
        }

        // Check 2: Requester reputation
        checks.push("reputation_check".to_string());
        let reputation = reputation_store.get_reputation(requester);
        match resource_spec.priority {
            ResourcePriority::Critical => {
                if reputation < 80 {
                    return ValidationResult {
                        is_valid: false,
                        error_message: Some(format!(
                            "Insufficient reputation for critical priority: required=80, actual={}",
                            reputation
                        )),
                        warnings,
                        validation_time_ms: start.elapsed().as_millis() as f64,
                        checks_performed: checks,
                    };
                }
            },
            ResourcePriority::High => {
                if reputation < 60 {
                    warnings.push(format!(
                        "Low reputation for high priority request: {}",
                        reputation
                    ));
                }
            },
            _ => {}
        }

        // Check 3: Duration limits
        checks.push("duration_limits".to_string());
        if let Some(duration) = resource_spec.duration {
            if duration > 86400 { // 24 hours
                warnings.push("Long duration allocation requested (>24 hours)".to_string());
            }
        }

        // Check 4: Resource utilization efficiency
        checks.push("utilization_efficiency".to_string());
        let utilization_ratio = resource_spec.amount as f64 / *available as f64;
        if utilization_ratio > 0.5 {
            warnings.push(format!(
                "High resource utilization: {:.1}% of available resources",
                utilization_ratio * 100.0
            ));
        }

        let validation_time = start.elapsed().as_millis() as f64;

        // Record validation metrics
        if let Ok(mut registry) = METRICS_REGISTRY.write() {
            registry.record_validation(true, validation_time);
        }

        ValidationResult {
            is_valid: true,
            error_message: None,
            warnings,
            validation_time_ms: validation_time,
            checks_performed: checks,
        }
    }
}

// Import the validation module traits
use crate::transaction_validation::*;

// Add these to the crate-level exports
pub use transaction_validation::{ValidationContext, ValidationResult, validate_mana_spend, validate_token_transfer, validate_marketplace_transaction, validate_cross_cooperative_allocation};

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::ICN_CORE_VERSION;
    use std::collections::HashMap;
    use std::str::FromStr;
    use std::sync::Mutex;
    use tempfile::tempdir;

    #[derive(Default)]
    struct InMemoryLedger {
        balances: Mutex<HashMap<Did, u64>>,
    }

    impl InMemoryLedger {
        fn new() -> Self {
            Self {
                balances: Mutex::new(HashMap::new()),
            }
        }
    }

    impl ManaLedger for InMemoryLedger {
        fn get_balance(&self, did: &Did) -> u64 {
            *self.balances.lock().unwrap().get(did).unwrap_or(&0)
        }

        fn set_balance(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
            self.balances.lock().unwrap().insert(did.clone(), amount);
            Ok(())
        }

        fn spend(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
            let mut map = self.balances.lock().unwrap();
            let bal = map
                .get_mut(did)
                .ok_or_else(|| CommonError::DatabaseError("account".into()))?;
            if *bal < amount {
                return Err(CommonError::PolicyDenied("insufficient".into()));
            }
            *bal -= amount;
            Ok(())
        }

        fn credit(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
            let mut map = self.balances.lock().unwrap();
            let entry = map.entry(did.clone()).or_insert(0);
            *entry = entry.saturating_add(amount);
            Ok(())
        }

        fn credit_all(&self, amount: u64) -> Result<(), CommonError> {
            let mut map = self.balances.lock().unwrap();
            for bal in map.values_mut() {
                *bal = bal.saturating_add(amount);
            }
            Ok(())
        }

        fn all_accounts(&self) -> Vec<Did> {
            self.balances.lock().unwrap().keys().cloned().collect()
        }
    }

    #[test]
    #[ignore] // TODO: Implement process_economic_event function
    fn test_process_economic_event() {
        let node_info = NodeInfo {
            name: "EcoNode".to_string(),
            version: ICN_CORE_VERSION.to_string(),
            status_message: "Economics active".to_string(),
        };
        // let result = process_economic_event(&node_info, "test_transaction");
        // assert!(result.is_ok());
        // assert!(result.unwrap().contains("test_transaction"));
    }

    #[cfg(feature = "persist-sled")]
    #[test]
    fn test_sled_mana_ledger_persistence() {
        use crate::ledger::SledManaLedger;
        let dir = tempdir().unwrap();
        let ledger_path = dir.path().join("mana.sled");
        let ledger = SledManaLedger::new(ledger_path.clone()).unwrap();
        let did = Did::from_str("did:example:alice").unwrap();
        ledger.set_balance(&did, 50).unwrap();
        ledger.credit(&did, 20).unwrap();
        ledger.spend(&did, 30).unwrap();
        assert_eq!(ledger.get_balance(&did), 40);
        drop(ledger);

        let ledger2 = SledManaLedger::new(ledger_path).unwrap();
        assert_eq!(ledger2.get_balance(&did), 40);
    }

    #[cfg(feature = "persist-rocksdb")]
    #[test]
    fn test_rocksdb_mana_ledger_basic_ops() {
        let dir = tempdir().unwrap();
        let ledger_path = dir.path().join("mana.rocks");
        let ledger = RocksdbManaLedger::new(ledger_path.clone()).unwrap();
        let did = Did::from_str("did:example:dan").unwrap();

        ledger.set_balance(&did, 50).unwrap();
        ledger.credit(&did, 20).unwrap();
        ledger.spend(&did, 30).unwrap();
        assert_eq!(ledger.get_balance(&did), 40);

        drop(ledger);
        let ledger2 = RocksdbManaLedger::new(ledger_path).unwrap();
        assert_eq!(ledger2.get_balance(&did), 40);
    }

    #[test]
    fn test_resource_policy_enforcer_spend_success() {
        use crate::ledger::FileManaLedger;
        let dir = tempdir().unwrap();
        let ledger_path = dir.path().join("mana.json");
        let ledger = FileManaLedger::new(ledger_path.clone()).unwrap();
        let did = Did::from_str("did:example:alice").unwrap();
        ledger.set_balance(&did, 150).unwrap();

        let adapter = ManaRepositoryAdapter::new(ledger);
        let enforcer = ResourcePolicyEnforcer::new(adapter);
        let result = enforcer.spend_mana(&did, 100);
        assert!(result.is_ok());

        let ledger_check = FileManaLedger::new(ledger_path).unwrap();
        assert_eq!(ledger_check.get_balance(&did), 50);
    }

    #[test]
    fn test_resource_policy_enforcer_insufficient_balance() {
        use crate::ledger::FileManaLedger;
        let dir = tempdir().unwrap();
        let ledger_path = dir.path().join("mana.json");
        let ledger = FileManaLedger::new(ledger_path).unwrap();
        let did = Did::from_str("did:example:bob").unwrap();
        ledger.set_balance(&did, 20).unwrap();

        let adapter = ManaRepositoryAdapter::new(ledger);
        let enforcer = ResourcePolicyEnforcer::new(adapter);
        let result = enforcer.spend_mana(&did, 30);
        assert!(matches!(result, Err(CommonError::PolicyDenied(_))));
    }

    #[test]
    fn test_resource_policy_enforcer_exceeds_limit() {
        use crate::ledger::FileManaLedger;
        let dir = tempdir().unwrap();
        let ledger_path = dir.path().join("mana.json");
        let ledger = FileManaLedger::new(ledger_path).unwrap();
        let did = Did::from_str("did:example:carol").unwrap();
        ledger.set_balance(&did, 5000).unwrap();

        let adapter = ManaRepositoryAdapter::new(ledger);
        let enforcer = ResourcePolicyEnforcer::new(adapter);
        let over_limit = ResourcePolicyEnforcer::<FileManaLedger>::MAX_SPEND_LIMIT + 1;
        let result = enforcer.spend_mana(&did, over_limit);
        assert!(matches!(result, Err(CommonError::PolicyDenied(_))));
    }

    #[test]
    fn test_credit_by_reputation_basic() {
        use icn_reputation::InMemoryReputationStore;

        let ledger = InMemoryLedger::new();
        let rep_store = InMemoryReputationStore::new();

        let alice = Did::from_str("did:example:alice").unwrap();
        let bob = Did::from_str("did:example:bob").unwrap();

        ledger.set_balance(&alice, 0).unwrap();
        ledger.set_balance(&bob, 0).unwrap();

        rep_store.set_score(alice.clone(), 3);
        rep_store.set_score(bob.clone(), 0);

        credit_by_reputation(&ledger, &rep_store, 10).unwrap();

        assert_eq!(ledger.get_balance(&alice), 30);
        assert_eq!(ledger.get_balance(&bob), 0);
    }

    #[test]
    fn test_credit_by_reputation_overflow() {
        use icn_reputation::InMemoryReputationStore;

        let ledger = InMemoryLedger::new();
        let rep_store = InMemoryReputationStore::new();

        let over = Did::from_str("did:example:overflow").unwrap();
        ledger.set_balance(&over, 0).unwrap();
        rep_store.set_score(over.clone(), 2);

        credit_by_reputation(&ledger, &rep_store, u64::MAX).unwrap();

        assert_eq!(ledger.get_balance(&over), u64::MAX);
    }

    #[test]
    fn test_inmemory_ledger_credit_all() {
        let ledger = InMemoryLedger::new();
        let alice = Did::from_str("did:example:alice").unwrap();
        let bob = Did::from_str("did:example:bob").unwrap();
        ledger.set_balance(&alice, 1).unwrap();
        ledger.set_balance(&bob, 2).unwrap();
        ledger.credit_all(5).unwrap();
        assert_eq!(ledger.get_balance(&alice), 6);
        assert_eq!(ledger.get_balance(&bob), 7);
    }

    #[test]
    fn test_price_by_reputation_reduces_cost() {
        // base price 100 with reputation 0 stays 100
        assert_eq!(price_by_reputation(100, 0), 100);

        // reputation should lower the price but never below zero
        assert!(price_by_reputation(100, 50) < 100);
        assert!(price_by_reputation(100, 200) < price_by_reputation(100, 50));

        // very high reputation approaches zero but never exceeds base
        assert!(price_by_reputation(100, 1000) < 100);
    }
}
