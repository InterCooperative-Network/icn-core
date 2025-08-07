use icn_common::{Did, SystemTimeProvider};
use icn_economics::automation::{
    DynamicPricingModel, EconomicAutomationConfig, EconomicAutomationEngine, EconomicHealthMetrics,
    EconomicPolicy, MarketMakingState, PolicyStatus, PolicyType,
};
use icn_economics::{ManaLedger, ResourceLedger};
use std::collections::{HashMap, VecDeque};
use std::str::FromStr;
use std::sync::{Arc, Mutex, RwLock};
use tokio::sync::mpsc;

#[derive(Default)]
struct InMemoryManaLedger {
    balances: Mutex<HashMap<Did, u64>>,
}

impl ManaLedger for InMemoryManaLedger {
    fn get_balance(&self, did: &Did) -> u64 {
        *self.balances.lock().unwrap().get(did).unwrap_or(&0)
    }
    fn set_balance(&self, did: &Did, amount: u64) -> Result<(), icn_common::CommonError> {
        self.balances.lock().unwrap().insert(did.clone(), amount);
        Ok(())
    }
    fn spend(&self, did: &Did, amount: u64) -> Result<(), icn_common::CommonError> {
        let mut map = self.balances.lock().unwrap();
        let bal = map.entry(did.clone()).or_insert(0);
        if *bal < amount {
            return Err(icn_common::CommonError::PolicyDenied("insufficient".into()));
        }
        *bal -= amount;
        Ok(())
    }
    fn credit(&self, did: &Did, amount: u64) -> Result<(), icn_common::CommonError> {
        let mut map = self.balances.lock().unwrap();
        let entry = map.entry(did.clone()).or_insert(0);
        *entry += amount;
        Ok(())
    }
}

#[derive(Default)]
struct InMemoryResourceLedger;

impl ResourceLedger for InMemoryResourceLedger {
    fn create_class(
        &self,
        _class_id: &icn_economics::TokenClassId,
        _class: icn_economics::TokenClass,
    ) -> Result<(), icn_common::CommonError> {
        Ok(())
    }
    fn get_class(
        &self,
        _class_id: &icn_economics::TokenClassId,
    ) -> Option<icn_economics::TokenClass> {
        None
    }
    fn update_class(
        &self,
        _class_id: &icn_economics::TokenClassId,
        _class: icn_economics::TokenClass,
    ) -> Result<(), icn_common::CommonError> {
        Ok(())
    }
    fn list_classes(&self) -> Vec<(icn_economics::TokenClassId, icn_economics::TokenClass)> {
        Vec::new()
    }
    fn mint(
        &self,
        _class_id: &icn_economics::TokenClassId,
        _owner: &Did,
        _amount: u64,
    ) -> Result<(), icn_common::CommonError> {
        Ok(())
    }
    fn burn(
        &self,
        _class_id: &icn_economics::TokenClassId,
        _owner: &Did,
        _amount: u64,
    ) -> Result<(), icn_common::CommonError> {
        Ok(())
    }
    fn transfer(
        &self,
        _class_id: &icn_economics::TokenClassId,
        _from: &Did,
        _to: &Did,
        _amount: u64,
    ) -> Result<(), icn_common::CommonError> {
        Ok(())
    }
    fn get_balance(&self, _class_id: &icn_economics::TokenClassId, _owner: &Did) -> u64 {
        0
    }
    fn can_transfer(
        &self,
        _class_id: &icn_economics::TokenClassId,
        _from: &Did,
        _to: &Did,
        _amount: u64,
    ) -> Result<bool, icn_common::CommonError> {
        Ok(true)
    }
    fn get_transfer_history(
        &self,
        _class_id: &icn_economics::TokenClassId,
        _did: &Did,
    ) -> Vec<icn_economics::TransferRecord> {
        Vec::new()
    }

    /// Apply demurrage to all accounts in a token class with demurrage rules
    fn apply_demurrage(&self, _class_id: &icn_economics::TokenClassId, _current_time: u64) -> Result<u64, icn_common::CommonError> {
        Ok(0)
    }

    /// Check if a transfer violates velocity limits
    fn check_velocity_limits(
        &self,
        _class_id: &icn_economics::TokenClassId,
        _from: &Did,
        _amount: u64,
        _current_time: u64,
    ) -> Result<bool, icn_common::CommonError> {
        Ok(true)
    }

    /// Verify if token redemption is allowed for specified purpose
    fn check_purpose_lock(
        &self,
        _class_id: &icn_economics::TokenClassId,
        _purpose: &str,
    ) -> Result<bool, icn_common::CommonError> {
        Ok(true)
    }

    /// Get transfer tracker for velocity limit enforcement
    fn get_transfer_tracker(&self, _class_id: &icn_economics::TokenClassId, _did: &Did) -> Option<icn_economics::TransferTracker> {
        None
    }

    /// Update transfer tracker after a successful transfer
    fn update_transfer_tracker(
        &self,
        _class_id: &icn_economics::TokenClassId,
        _did: &Did,
        _amount: u64,
        _current_time: u64,
    ) -> Result<(), icn_common::CommonError> {
        Ok(())
    }
}

#[tokio::test]
async fn policy_enforcement_min_balance() {
    let ledger = Arc::new(InMemoryManaLedger::default());
    let policies = Arc::new(RwLock::new(HashMap::new()));
    let alice = Did::from_str("did:icn:alice").unwrap();
    ledger.set_balance(&alice, 10).unwrap();

    policies.write().unwrap().insert(
        "min".into(),
        EconomicPolicy {
            policy_id: "min".into(),
            policy_type: PolicyType::ManaRegeneration,
            parameters: {
                let mut m = HashMap::new();
                m.insert("min_balance".into(), 50.0);
                m
            },
            enforcement_level: 1.0,
            last_updated: std::time::Instant::now(),
            status: PolicyStatus::Active,
        },
    );

    let (tx, _rx) = mpsc::unbounded_channel();
    let tp = Arc::new(SystemTimeProvider);

    let ledger: Arc<dyn ManaLedger> = ledger;
    let tp: Arc<dyn icn_common::TimeProvider> = tp;
    EconomicAutomationEngine::enforce_economic_policies(
        &policies,
        &ledger,
        &EconomicAutomationConfig::default(),
        &tx,
        &tp,
    )
    .await
    .unwrap();

    assert_eq!(ledger.get_balance(&alice), 50);
}

#[tokio::test]
async fn health_monitoring_updates_metrics() {
    let ledger: Arc<dyn ManaLedger> = Arc::new(InMemoryManaLedger::default());
    let resource: Arc<dyn ResourceLedger> = Arc::new(InMemoryResourceLedger);
    let metrics = Arc::new(RwLock::new(EconomicHealthMetrics {
        overall_health: 1.0,
        mana_inequality: 0.0,
        resource_efficiency: 0.0,
        market_liquidity: 0.0,
        price_stability: 0.0,
        activity_level: 0.0,
        last_updated: 0,
    }));
    let (tx, _rx) = mpsc::unbounded_channel();
    let tp = Arc::new(SystemTimeProvider);
    let alice = Did::from_str("did:icn:alice").unwrap();
    let bob = Did::from_str("did:icn:bob").unwrap();
    ledger.set_balance(&alice, 10).unwrap();
    ledger.set_balance(&bob, 40).unwrap();

    let tp: Arc<dyn icn_common::TimeProvider> = tp;
    EconomicAutomationEngine::monitor_economic_health(
        &metrics,
        &ledger,
        &resource,
        &EconomicAutomationConfig::default(),
        &tx,
        &tp,
    )
    .await
    .unwrap();

    let m = metrics.read().unwrap();
    assert!(m.overall_health < 1.0);
    assert!(m.mana_inequality > 0.0);
}

#[tokio::test]
async fn market_making_and_prediction() {
    let state = Arc::new(RwLock::new(MarketMakingState {
        positions: std::collections::HashMap::new(),
        inventory: std::collections::HashMap::new(),
        performance: icn_economics::automation::MarketMakingPerformance::default(),
        risk_metrics: icn_economics::automation::RiskMetrics::default(),
    }));
    let model = DynamicPricingModel {
        base_price: 10.0,
        current_price: 10.0,
        price_history: VecDeque::from(vec![
            (std::time::Instant::now(), 9.0),
            (std::time::Instant::now(), 10.0),
            (std::time::Instant::now(), 11.0),
        ]),
        supply_demand_ratio: 1.0,
        quality_factor: 1.0,
        competition_factor: 1.0,
        last_updated: std::time::Instant::now(),
    };
    let models = Arc::new(RwLock::new(HashMap::from([(
        "cpu".to_string(),
        model.clone(),
    )])));
    let (tx, _rx) = mpsc::unbounded_channel();
    let cfg = EconomicAutomationConfig::default();

    let tp: Arc<dyn icn_common::TimeProvider> = Arc::new(SystemTimeProvider);
    EconomicAutomationEngine::execute_market_making(&state, &models, &cfg, &tx, &tp)
        .await
        .unwrap();
    {
        let s = state.read().unwrap();
        assert!(s.performance.total_trades > 0);
    }

    let accounts = Arc::new(RwLock::new(HashMap::new()));
    let metrics = Arc::new(RwLock::new(EconomicHealthMetrics {
        overall_health: 1.0,
        mana_inequality: 0.0,
        resource_efficiency: 0.0,
        market_liquidity: 0.0,
        price_stability: 0.0,
        activity_level: 0.0,
        last_updated: 0,
    }));

    EconomicAutomationEngine::run_predictive_models(&metrics, &models, &accounts)
        .await
        .unwrap();
    let new_price = models.read().unwrap().get("cpu").unwrap().current_price;
    assert_ne!(new_price, model.current_price);
}
