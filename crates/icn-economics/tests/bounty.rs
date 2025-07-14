use icn_common::{Did, NodeScope};
use icn_economics::bounty::BountyManager;
use icn_economics::{ManaLedger, ResourceLedger, ResourceRepositoryAdapter};
use icn_reputation::InMemoryReputationStore;
use icn_runtime::context::StubDagStore;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Mutex;

#[derive(Default)]
struct InMemResLedger {
    balances: Mutex<HashMap<(Did, String), u64>>,
}
impl ResourceLedger for InMemResLedger {
    fn get_balance(&self, d: &Did, c: &str) -> u64 {
        *self
            .balances
            .lock()
            .unwrap()
            .get(&(d.clone(), c.to_string()))
            .unwrap_or(&0)
    }
    fn set_balance(&self, d: &Did, c: &str, a: u64) -> Result<(), icn_common::CommonError> {
        self.balances
            .lock()
            .unwrap()
            .insert((d.clone(), c.to_string()), a);
        Ok(())
    }
    fn credit(&self, d: &Did, c: &str, a: u64) -> Result<(), icn_common::CommonError> {
        let mut m = self.balances.lock().unwrap();
        *m.entry((d.clone(), c.to_string())).or_insert(0) += a;
        Ok(())
    }
    fn debit(&self, d: &Did, c: &str, a: u64) -> Result<(), icn_common::CommonError> {
        let mut m = self.balances.lock().unwrap();
        let bal = m.entry((d.clone(), c.to_string())).or_insert(0);
        if *bal < a {
            return Err(icn_common::CommonError::PolicyDenied("insufficient".into()));
        }
        *bal -= a;
        Ok(())
    }
}

#[derive(Default)]
struct InMemoryManaLedger {
    balances: Mutex<HashMap<Did, u64>>,
}
impl ManaLedger for InMemoryManaLedger {
    fn get_balance(&self, d: &Did) -> u64 {
        *self.balances.lock().unwrap().get(d).unwrap_or(&0)
    }
    fn set_balance(&self, d: &Did, a: u64) -> Result<(), icn_common::CommonError> {
        self.balances.lock().unwrap().insert(d.clone(), a);
        Ok(())
    }
    fn spend(&self, d: &Did, a: u64) -> Result<(), icn_common::CommonError> {
        let mut m = self.balances.lock().unwrap();
        let bal = m.entry(d.clone()).or_insert(0);
        if *bal < a {
            return Err(icn_common::CommonError::PolicyDenied("insufficient".into()));
        }
        *bal -= a;
        Ok(())
    }
    fn credit(&self, d: &Did, a: u64) -> Result<(), icn_common::CommonError> {
        let mut m = self.balances.lock().unwrap();
        *m.entry(d.clone()).or_insert(0) += a;
        Ok(())
    }
}

#[test]
fn bounty_flow() {
    let mana = InMemoryManaLedger::default();
    let ledger = InMemResLedger::default();
    let mut repo = ResourceRepositoryAdapter::with_dag_store(ledger, Box::new(StubDagStore::new()));
    let issuer = Did::from_str("did:example:issuer").unwrap();
    let scope = NodeScope("coop".into());
    repo.add_issuer(scope.clone(), issuer.clone());
    let mut manager = BountyManager::new(mana, repo);
    manager.mana.set_balance(&issuer, 10).unwrap();
    let id = manager.create_bounty(issuer.clone(), "token", 5, "fix bug");
    manager
        .claim_bounty(id, Did::from_str("did:example:bob").unwrap())
        .unwrap();
    let rep = InMemoryReputationStore::new();
    manager
        .payout_bounty(id, &rep, Some(NodeScope("coop".into())))
        .unwrap();
    assert!(manager.bounties.get(&id).unwrap().paid);
}
