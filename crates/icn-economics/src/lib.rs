#![doc = include_str!("../README.md")]

//! # ICN Economics Crate
//! This crate handles the economic protocols of the InterCooperative Network (ICN).
//! It manages token models, ledger interactions, transaction logic, and incentive mechanisms,
//! aiming for security, accuracy, and interoperability.

use icn_common::{Cid, CommonError, DagBlock, Did, NodeInfo, NodeScope, TimeProvider};
use icn_identity::{MembershipPolicyEnforcer, MembershipResolver};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use serde_json;
pub mod ledger;
pub mod metrics;
pub use ledger::FileManaLedger;
#[cfg(feature = "persist-rocksdb")]
pub use ledger::RocksdbManaLedger;
#[cfg(feature = "persist-sled")]
pub use ledger::SledManaLedger;
#[cfg(feature = "persist-sqlite")]
pub use ledger::SqliteManaLedger;

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

/// Ledger abstraction for scoped resource tokens.
pub trait ResourceLedger: Send + Sync {
    /// Retrieve the token balance for a DID within a scope.
    fn get_balance(&self, did: &Did, token: &str) -> u64;
    /// Persist a new balance for a DID.
    fn set_balance(&self, did: &Did, token: &str, amount: u64) -> Result<(), CommonError>;
    /// Credit tokens to the account.
    fn credit(&self, did: &Did, token: &str, amount: u64) -> Result<(), CommonError>;
    /// Debit tokens from the account.
    fn debit(&self, did: &Did, token: &str, amount: u64) -> Result<(), CommonError>;
}

/// Adapter exposing convenience helpers around a [`ResourceLedger`].
#[derive(Debug)]
pub struct ResourceRepositoryAdapter<L: ResourceLedger> {
    ledger: L,
}

impl<L: ResourceLedger> ResourceRepositoryAdapter<L> {
    /// Create a new adapter.
    pub fn new(ledger: L) -> Self {
        Self { ledger }
    }

    /// Credit tokens to an account.
    pub fn credit_tokens(&self, did: &Did, token: &str, amount: u64) -> Result<(), CommonError> {
        self.ledger.credit(did, token, amount)
    }

    /// Debit tokens from an account.
    pub fn debit_tokens(&self, did: &Did, token: &str, amount: u64) -> Result<(), CommonError> {
        self.ledger.debit(did, token, amount)
    }

    /// Get the balance for a token.
    pub fn get_balance(&self, did: &Did, token: &str) -> u64 {
        self.ledger.get_balance(did, token)
    }
}

/// Thin wrapper exposing convenience methods over a [`ManaLedger`].
#[derive(Debug)]
pub struct ManaRepositoryAdapter<L: ManaLedger> {
    ledger: L,
}

impl<L: ManaLedger> ManaRepositoryAdapter<L> {
    /// Construct a new adapter around the provided ledger implementation.
    pub fn new(ledger: L) -> Self {
        ManaRepositoryAdapter { ledger }
    }

    /// Deduct mana from an account via the underlying ledger.
    pub fn spend_mana(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
        metrics::SPEND_MANA_CALLS.inc();
        self.ledger.spend(did, amount)
    }

    /// Retrieve the account balance.
    pub fn get_balance(&self, did: &Did) -> u64 {
        metrics::GET_BALANCE_CALLS.inc();
        self.ledger.get_balance(did)
    }

    /// Credits the specified account with additional mana.
    pub fn credit_mana(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
        metrics::CREDIT_MANA_CALLS.inc();
        self.ledger.credit(did, amount)
    }
}

/// Enforces spending limits and forwards to a [`ManaRepositoryAdapter`].
#[derive(Debug)]
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

// --- Scoped Token Operations ---

#[derive(Debug, Serialize, Deserialize)]
struct TokenEvent {
    event_type: String,
    token: String,
    issuer: Did,
    recipient: Option<Did>,
    amount: u64,
    scope: NodeScope,
}

fn validate_token_scope(token: &str, scope: &NodeScope) -> Result<(), CommonError> {
    if token.starts_with(&scope.0) {
        Ok(())
    } else {
        Err(CommonError::PolicyDenied(format!(
            "token {token} not in scope {}",
            scope.0
        )))
    }
}

fn record_event<S: icn_dag::StorageService<DagBlock>, T: icn_common::TimeProvider>(
    dag: &mut S,
    author: &Did,
    scope: &NodeScope,
    event: &TokenEvent,
    time: &T,
) -> Result<Cid, CommonError> {
    let data =
        serde_json::to_vec(event).map_err(|e| CommonError::SerializationError(format!("{e}")))?;
    let ts = time.unix_seconds();
    let cid =
        icn_common::compute_merkle_cid(0x71, &data, &[], ts, author, &None, &Some(scope.clone()));
    let block = DagBlock {
        cid: cid.clone(),
        data,
        links: vec![],
        timestamp: ts,
        author_did: author.clone(),
        signature: None,
        scope: Some(scope.clone()),
    };
    dag.put(&block)?;
    Ok(cid)
}

/// Mint new scoped tokens for `recipient`.
pub fn mint_tokens<RL, ML, R, S, T>(
    resource_ledger: &RL,
    mana_ledger: ML,
    membership: &MembershipPolicyEnforcer<R>,
    dag: &mut S,
    issuer: &Did,
    recipient: &Did,
    token: &str,
    scope: &NodeScope,
    amount: u64,
    mana_fee: u64,
    time: &T,
) -> Result<Cid, CommonError>
where
    RL: ResourceLedger,
    ML: ManaLedger,
    R: MembershipResolver,
    S: icn_dag::StorageService<DagBlock>,
    T: icn_common::TimeProvider,
{
    membership.check_permission(issuer, scope)?;
    validate_token_scope(token, scope)?;
    charge_mana(mana_ledger, issuer, mana_fee)?;
    resource_ledger.credit(recipient, token, amount)?;

    let event = TokenEvent {
        event_type: "mint".into(),
        token: token.to_string(),
        issuer: issuer.clone(),
        recipient: Some(recipient.clone()),
        amount,
        scope: scope.clone(),
    };
    record_event(dag, issuer, scope, &event, time)
}

/// Burn scoped tokens from `holder`.
pub fn burn_tokens<RL, ML, R, S, T>(
    resource_ledger: &RL,
    mana_ledger: ML,
    membership: &MembershipPolicyEnforcer<R>,
    dag: &mut S,
    issuer: &Did,
    holder: &Did,
    token: &str,
    scope: &NodeScope,
    amount: u64,
    mana_fee: u64,
    time: &T,
) -> Result<Cid, CommonError>
where
    RL: ResourceLedger,
    ML: ManaLedger,
    R: MembershipResolver,
    S: icn_dag::StorageService<DagBlock>,
    T: icn_common::TimeProvider,
{
    membership.check_permission(issuer, scope)?;
    validate_token_scope(token, scope)?;
    charge_mana(mana_ledger, issuer, mana_fee)?;
    resource_ledger.debit(holder, token, amount)?;

    let event = TokenEvent {
        event_type: "burn".into(),
        token: token.to_string(),
        issuer: issuer.clone(),
        recipient: Some(holder.clone()),
        amount,
        scope: scope.clone(),
    };
    record_event(dag, issuer, scope, &event, time)
}

/// Transfer scoped tokens between accounts.
pub fn transfer_tokens<RL, ML, R, S, T>(
    resource_ledger: &RL,
    mana_ledger: ML,
    membership: &MembershipPolicyEnforcer<R>,
    dag: &mut S,
    issuer: &Did,
    from: &Did,
    to: &Did,
    token: &str,
    scope: &NodeScope,
    amount: u64,
    mana_fee: u64,
    time: &T,
) -> Result<Cid, CommonError>
where
    RL: ResourceLedger,
    ML: ManaLedger,
    R: MembershipResolver,
    S: icn_dag::StorageService<DagBlock>,
    T: icn_common::TimeProvider,
{
    membership.check_permission(issuer, scope)?;
    validate_token_scope(token, scope)?;
    charge_mana(mana_ledger, issuer, mana_fee)?;
    resource_ledger.debit(from, token, amount)?;
    resource_ledger.credit(to, token, amount)?;

    let event = TokenEvent {
        event_type: "transfer".into(),
        token: token.to_string(),
        issuer: issuer.clone(),
        recipient: Some(to.clone()),
        amount,
        scope: scope.clone(),
    };
    record_event(dag, issuer, scope, &event, time)
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
