use icn_common::{Did, NodeScope};
use icn_economics::{
    burn_mutual_aid_tokens, mint_mutual_aid_tokens, transfer_tokens, ManaLedger, ResourceLedger,
    ResourceRepositoryAdapter, MUTUAL_AID_CLASS_ID,
};
use icn_runtime::context::StubDagStore;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Mutex;

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
        *map.entry(did.clone()).or_insert(0) += amount;
        Ok(())
    }
}
#[derive(Default)]
struct InMemoryResourceLedger {
    balances: Mutex<HashMap<(Did, String), u64>>,
}
impl ResourceLedger for InMemoryResourceLedger {
    fn get_balance(&self, did: &Did, class: &str) -> u64 {
        *self
            .balances
            .lock()
            .unwrap()
            .get(&(did.clone(), class.to_string()))
            .unwrap_or(&0)
    }
    fn set_balance(
        &self,
        did: &Did,
        class: &str,
        amount: u64,
    ) -> Result<(), icn_common::CommonError> {
        self.balances
            .lock()
            .unwrap()
            .insert((did.clone(), class.to_string()), amount);
        Ok(())
    }
    fn credit(&self, did: &Did, class: &str, amount: u64) -> Result<(), icn_common::CommonError> {
        let mut m = self.balances.lock().unwrap();
        *m.entry((did.clone(), class.to_string())).or_insert(0) += amount;
        Ok(())
    }
    fn debit(&self, did: &Did, class: &str, amount: u64) -> Result<(), icn_common::CommonError> {
        let mut m = self.balances.lock().unwrap();
        let bal = m.entry((did.clone(), class.to_string())).or_insert(0);
        if *bal < amount {
            return Err(icn_common::CommonError::PolicyDenied("insufficient".into()));
        }
        *bal -= amount;
        Ok(())
    }
}

#[test]
fn mutual_aid_flow() {
    let mana = InMemoryManaLedger::default();
    mana.set_balance(&Did::from_str("did:issuer").unwrap(), 10)
        .unwrap();
    let ledger = InMemoryResourceLedger::default();
    let mut repo = ResourceRepositoryAdapter::with_dag_store(ledger, Box::new(StubDagStore::new()));
    let scope = NodeScope("s".into());
    let issuer = Did::from_str("did:issuer").unwrap();
    repo.add_issuer(scope.clone(), issuer.clone());
    let rec = Did::from_str("did:bob").unwrap();
    mint_mutual_aid_tokens(&repo, &mana, &issuer, 5, &rec, Some(scope.clone())).unwrap();
    assert_eq!(repo.ledger().get_balance(&rec, MUTUAL_AID_CLASS_ID), 5);
    let other = Did::from_str("did:alice").unwrap();
    assert!(transfer_tokens(
        &repo,
        &mana,
        &issuer,
        MUTUAL_AID_CLASS_ID,
        1,
        &rec,
        &other,
        Some(scope.clone())
    )
    .is_err());
    burn_mutual_aid_tokens(&repo, &mana, &issuer, 3, &rec, Some(scope)).unwrap();
    assert_eq!(repo.ledger().get_balance(&rec, MUTUAL_AID_CLASS_ID), 2);
}
