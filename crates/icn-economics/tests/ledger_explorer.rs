use icn_eventstore::EventStore;
use icn_common::Did;
use icn_economics::{FileManaLedger, LedgerEvent, LedgerExplorer, ManaRepositoryAdapter};
use icn_eventstore::MemoryEventStore;
use std::str::FromStr;
use tempfile::tempdir;

#[test]
fn explorer_aggregates_flows() {
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

    let mut explorer_store = MemoryEventStore::new();
    for e in &events {
        explorer_store.append(e).unwrap();
    }
    let explorer = LedgerExplorer::new(explorer_store);
    let stats = explorer.stats_for(&did).unwrap();
    assert_eq!(stats.inflow, 5);
    assert_eq!(stats.outflow, 3);
}
