use icn_common::Did;
use icn_economics::{
    balances_from_events, ledger::FileManaLedger, ManaRepositoryAdapter,
};
use icn_eventstore::MemoryEventStore;
use std::str::FromStr;
use tempfile::tempdir;

#[test]
fn ledger_event_replay() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("mana.json");
    let ledger = FileManaLedger::new(path).unwrap();
    let store = MemoryEventStore::new();
    let adapter = ManaRepositoryAdapter::with_event_store(ledger, Box::new(store));
    let did = Did::from_str("did:example:alice").unwrap();
    adapter.set_balance(&did, 10).unwrap();
    adapter.credit_mana(&did, 5).unwrap();
    adapter.spend_mana(&did, 3).unwrap();

    let events = adapter
        .event_store()
        .unwrap()
        .lock()
        .unwrap()
        .query(None)
        .unwrap();
    let balances = balances_from_events(&events);
    assert_eq!(balances.get(&did).cloned().unwrap_or(0), 12);
}
