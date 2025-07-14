use crate::LedgerEvent;
use icn_common::{CommonError, Did};
use icn_eventstore::EventStore;
use std::collections::HashMap;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct FlowStats {
    pub inflow: u64,
    pub outflow: u64,
}

pub struct LedgerExplorer<S: EventStore<LedgerEvent>> {
    store: S,
}

impl<S: EventStore<LedgerEvent>> LedgerExplorer<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub fn aggregated_flows(&self) -> Result<HashMap<Did, FlowStats>, CommonError> {
        let events = self.store.query(None)?;
        let mut map: HashMap<Did, FlowStats> = HashMap::new();
        for e in events {
            match e {
                LedgerEvent::Credit { did, amount } => {
                    map.entry(did).or_default().inflow += amount;
                }
                LedgerEvent::Debit { did, amount } => {
                    map.entry(did).or_default().outflow += amount;
                }
                LedgerEvent::SetBalance { .. } => {}
            }
        }
        Ok(map)
    }

    pub fn stats_for(&self, did: &Did) -> Result<FlowStats, CommonError> {
        Ok(self
            .aggregated_flows()?
            .get(did)
            .cloned()
            .unwrap_or_default())
    }
}
