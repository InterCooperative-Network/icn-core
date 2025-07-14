use icn_common::Did;
use icn_economics::{FileManaLedger, ManaRepositoryAdapter};
use icn_eventstore::MemoryEventStore;
use std::str::FromStr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ledger_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "sim_ledger.json".to_string());
    let iterations: u32 = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "10".to_string())
        .parse()
        .unwrap_or(10);
    let ledger = FileManaLedger::new(ledger_path.into())?;
    let store = MemoryEventStore::new();
    let repo = ManaRepositoryAdapter::with_event_store(ledger, Box::new(store));
    let did = Did::from_str("did:example:sim")?;
    repo.set_balance(&did, 100)?;
    for _ in 0..iterations {
        let _ = repo.spend_mana(&did, 1);
        let _ = repo.credit_mana(&did, 2);
    }
    println!("Final balance: {}", repo.get_balance(&did));
    Ok(())
}
