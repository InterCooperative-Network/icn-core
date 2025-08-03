use icn_common::{Did, NodeScope};
use icn_economics::{
    burn_tokens, grant_mutual_aid, mint_tokens, transfer_tokens, use_mutual_aid, ManaLedger,
    ResourceLedger, ResourceRepositoryAdapter, MUTUAL_AID_CLASS,
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
        let entry = map.entry(did.clone()).or_insert(0);
        *entry += amount;
        Ok(())
    }
}

#[derive(Default)]
struct InMemoryResourceLedger {
    balances: Mutex<HashMap<(String, Did), u64>>,
    classes: Mutex<HashMap<String, icn_economics::TokenClass>>,
}

impl ResourceLedger for InMemoryResourceLedger {
    fn create_class(
        &self,
        class_id: &String,
        class: icn_economics::TokenClass,
    ) -> Result<(), icn_common::CommonError> {
        let mut classes = self.classes.lock().unwrap();
        if classes.contains_key(class_id) {
            return Err(icn_common::CommonError::InvalidInputError(format!(
                "Token class {class_id} already exists"
            )));
        }
        classes.insert(class_id.clone(), class);
        Ok(())
    }
    fn get_class(&self, class_id: &String) -> Option<icn_economics::TokenClass> {
        self.classes.lock().unwrap().get(class_id).cloned()
    }
    fn update_class(
        &self,
        class_id: &String,
        class: icn_economics::TokenClass,
    ) -> Result<(), icn_common::CommonError> {
        let mut classes = self.classes.lock().unwrap();
        if !classes.contains_key(class_id) {
            return Err(icn_common::CommonError::InvalidInputError(format!(
                "Token class {class_id} not found"
            )));
        }
        classes.insert(class_id.clone(), class);
        Ok(())
    }
    fn list_classes(&self) -> Vec<(String, icn_economics::TokenClass)> {
        self.classes
            .lock()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
    fn mint(
        &self,
        class_id: &String,
        owner: &Did,
        amount: u64,
    ) -> Result<(), icn_common::CommonError> {
        let mut balances = self.balances.lock().unwrap();
        let key = (class_id.clone(), owner.clone());
        let entry = balances.entry(key).or_insert(0);
        *entry += amount;
        Ok(())
    }
    fn burn(
        &self,
        class_id: &String,
        owner: &Did,
        amount: u64,
    ) -> Result<(), icn_common::CommonError> {
        let mut balances = self.balances.lock().unwrap();
        let key = (class_id.clone(), owner.clone());
        let entry = balances.entry(key).or_insert(0);
        if *entry < amount {
            return Err(icn_common::CommonError::PolicyDenied("Insufficient balance".into()));
        }
        *entry -= amount;
        Ok(())
    }
    fn transfer(
        &self,
        class_id: &String,
        from: &Did,
        to: &Did,
        amount: u64,
    ) -> Result<(), icn_common::CommonError> {
        let mut balances = self.balances.lock().unwrap();
        let from_key = (class_id.clone(), from.clone());
        let to_key = (class_id.clone(), to.clone());
        
        let from_balance = balances.entry(from_key.clone()).or_insert(0);
        if *from_balance < amount {
            return Err(icn_common::CommonError::PolicyDenied("Insufficient balance".into()));
        }
        *from_balance -= amount;
        
        let to_balance = balances.entry(to_key).or_insert(0);
        *to_balance += amount;
        Ok(())
    }
    fn get_balance(&self, class_id: &String, owner: &Did) -> u64 {
        *self
            .balances
            .lock()
            .unwrap()
            .get(&(class_id.clone(), owner.clone()))
            .unwrap_or(&0)
    }
    fn can_transfer(
        &self,
        _class_id: &String,
        _from: &Did,
        _to: &Did,
        _amount: u64,
    ) -> Result<bool, icn_common::CommonError> {
        Ok(true)
    }
    fn get_transfer_history(
        &self,
        _class_id: &String,
        _did: &Did,
    ) -> Vec<icn_economics::TransferRecord> {
        Vec::new()
    }
}

#[test]
fn mint_transfer_burn_flow() {
    let mana = InMemoryManaLedger::default();
    mana.set_balance(&Did::from_str("did:example:issuer").unwrap(), 10)
        .unwrap();
    let ledger = InMemoryResourceLedger::default();
    let mut repo = ResourceRepositoryAdapter::with_dag_store(ledger, Box::new(StubDagStore::new()));
    let scope = NodeScope("scope".into());
    let issuer = Did::from_str("did:example:issuer").unwrap();
    repo.add_issuer(scope.clone(), issuer.clone());
    let recipient = Did::from_str("did:example:bob").unwrap();

    mint_tokens(
        &repo,
        &mana,
        &issuer,
        "token",
        5,
        &recipient,
        Some(scope.clone()),
    )
    .unwrap();
    assert_eq!(
        repo.ledger().get_balance(&"token".to_string(), &recipient),
        5
    );

    let other = Did::from_str("did:example:alice").unwrap();
    transfer_tokens(
        &repo,
        &mana,
        &issuer,
        "token",
        3,
        &recipient,
        &other,
        Some(scope.clone()),
    )
    .unwrap();
    assert_eq!(
        repo.ledger().get_balance(&"token".to_string(), &recipient),
        2
    );
    assert_eq!(repo.ledger().get_balance(&"token".to_string(), &other), 3);

    burn_tokens(&repo, &mana, &issuer, "token", 2, &other, Some(scope)).unwrap();
    assert_eq!(repo.ledger().get_balance(&"token".to_string(), &other), 1);
}

#[test]
fn mutual_aid_flow() {
    let mana = InMemoryManaLedger::default();
    mana.set_balance(&Did::from_str("did:example:issuer").unwrap(), 10)
        .unwrap();
    let ledger = InMemoryResourceLedger::default();
    let mut repo = ResourceRepositoryAdapter::with_dag_store(ledger, Box::new(StubDagStore::new()));
    let scope = NodeScope("aid".into());
    let issuer = Did::from_str("did:example:issuer").unwrap();
    repo.add_issuer(scope.clone(), issuer.clone());
    let recipient = Did::from_str("did:example:bob").unwrap();

    grant_mutual_aid(&repo, &mana, &issuer, &recipient, 4, Some(scope.clone())).unwrap();
    assert_eq!(
        repo.ledger()
            .get_balance(&MUTUAL_AID_CLASS.to_string(), &recipient),
        4
    );

    use_mutual_aid(&repo, &mana, &issuer, &recipient, 2, Some(scope)).unwrap();
    assert_eq!(
        repo.ledger()
            .get_balance(&MUTUAL_AID_CLASS.to_string(), &recipient),
        2
    );
}
