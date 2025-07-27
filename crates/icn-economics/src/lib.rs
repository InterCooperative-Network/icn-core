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
pub mod explorer;
pub mod ledger;
pub mod crdt_ledger;
pub mod marketplace;
pub mod metrics;
pub mod mutual_aid;
pub mod mutual_credit;
pub mod reputation_tokens;
pub mod time_banking;
pub mod economic_dispute_resolver;

/// Comprehensive economic automation and policy enforcement
pub mod automation;
pub use automation::{
    EconomicAutomationConfig, EconomicAutomationEngine, EconomicAutomationStats, EconomicEvent,
    EconomicHealthMetrics,
};
pub use explorer::{FlowStats, LedgerExplorer};
pub use crdt_ledger::{CRDTManaLedger, CRDTManaLedgerConfig, CRDTManaLedgerStats};
pub use ledger::FileResourceLedger;
pub use ledger::{
    FileManaLedger, ResourceLedger, ScopingRules, TokenClass, TokenClassId, TokenType,
    TransferRecord, TransferabilityRule,
};
pub use economic_dispute_resolver::{
    AssetFreeze, BalanceAdjustment, Compensation, CompensationType, DisputeSeverity,
    EconomicDispute, EconomicDisputeConfig, EconomicDisputeResolver, EconomicDisputeType,
    EconomicEvidence, EconomicPenalty, EconomicResolution, EconomicResolutionStatus,
    PenaltyType, ReputationProvider as DisputeReputationProvider, ResourceRedistribution,
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

/// Exposes a public function to charge mana, wrapping ResourcePolicyEnforcer.
pub fn charge_mana<L: ManaLedger>(ledger: L, did: &Did, amount: u64) -> Result<(), CommonError> {
    let mana_adapter = ManaRepositoryAdapter::new(ledger);
    let policy_enforcer = ResourcePolicyEnforcer::new(mana_adapter);

    info!("[icn-economics] charge_mana called for DID {did:?}, amount {amount}");
    policy_enforcer.spend_mana(did, amount)
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

/// Execute a marketplace transaction by transferring tokens and updating records.
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
    // Get the offer and bid
    let offer = marketplace_store
        .get_offer(offer_id)
        .ok_or_else(|| CommonError::InvalidInputError(format!("Offer {} not found", offer_id)))?;

    let bid = marketplace_store
        .get_bid(bid_id)
        .ok_or_else(|| CommonError::InvalidInputError(format!("Bid {} not found", bid_id)))?;

    // Validate the transaction
    if bid.offer_id != offer.offer_id {
        return Err(CommonError::InvalidInputError(
            "Bid does not match offer".into(),
        ));
    }

    if bid.status != marketplace::BidStatus::Active {
        return Err(CommonError::PolicyDenied("Bid is not active".into()));
    }

    if offer.status != marketplace::OfferStatus::Active {
        return Err(CommonError::PolicyDenied("Offer is not active".into()));
    }

    if bid.quantity > offer.quantity {
        return Err(CommonError::PolicyDenied(
            "Bid quantity exceeds available quantity".into(),
        ));
    }

    // Calculate total price
    let total_price = bid.price_per_unit * bid.quantity;

    // Transfer tokens from buyer to seller
    resource_repo.transfer(
        executor,
        &bid.payment_token_class,
        total_price,
        &bid.buyer,
        &offer.seller,
        None, // marketplace transactions can be cross-scope
    )?;

    // Charge mana fee for marketplace transaction
    charge_mana(mana_ledger, executor, TOKEN_FEE)?;

    // Create transaction record
    let transaction = marketplace::MarketplaceTransaction {
        transaction_id: format!("tx_{}_{}", offer_id, bid_id),
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

pub fn mint_tokens<L: ResourceLedger, M: ManaLedger>(
    repo: &ResourceRepositoryAdapter<L>,
    mana_ledger: &M,
    issuer: &Did,
    class_id: &str,
    amount: u64,
    recipient: &Did,
    scope: Option<NodeScope>,
) -> Result<(), CommonError> {
    charge_mana(mana_ledger, issuer, TOKEN_FEE)?;
    repo.mint(issuer, class_id, amount, recipient, scope)
}

#[allow(clippy::too_many_arguments)]
pub fn mint_tokens_with_reputation<L: ResourceLedger, M: ManaLedger>(
    repo: &ResourceRepositoryAdapter<L>,
    mana_ledger: &M,
    reputation_store: &dyn icn_reputation::ReputationStore,
    issuer: &Did,
    class_id: &str,
    amount: u64,
    recipient: &Did,
    scope: Option<NodeScope>,
) -> Result<(), CommonError> {
    let rep = reputation_store.get_reputation(issuer);
    let cost = price_by_reputation(TOKEN_FEE, rep);
    charge_mana(mana_ledger, issuer, cost)?;
    repo.mint(issuer, class_id, amount, recipient, scope)
}

pub fn burn_tokens<L: ResourceLedger, M: ManaLedger>(
    repo: &ResourceRepositoryAdapter<L>,
    mana_ledger: &M,
    issuer: &Did,
    class_id: &str,
    amount: u64,
    owner: &Did,
    scope: Option<NodeScope>,
) -> Result<(), CommonError> {
    charge_mana(mana_ledger, issuer, TOKEN_FEE)?;
    repo.burn(issuer, class_id, amount, owner, scope)
}

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
    charge_mana(mana_ledger, issuer, TOKEN_FEE)?;
    repo.transfer(issuer, class_id, amount, from, to, scope)
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

/// Placeholder function demonstrating use of common types for economics.
pub fn process_economic_event(info: &NodeInfo, event_details: &str) -> Result<String, CommonError> {
    Ok(format!(
        "Processed economic event '{} ' for node: {} (v{})",
        event_details, info.name, info.version
    ))
}

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
    fn test_process_economic_event() {
        let node_info = NodeInfo {
            name: "EcoNode".to_string(),
            version: ICN_CORE_VERSION.to_string(),
            status_message: "Economics active".to_string(),
        };
        let result = process_economic_event(&node_info, "test_transaction");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("test_transaction"));
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
