# Event Sourcing in ICN Core

`icn-eventstore` introduces a simple append-only event log. Modules like `icn-governance` and `icn-economics` emit structured events for every state change. Node configuration changes are captured as `ParameterUpdate` events. The event store can be backed by in-memory storage for tests or persistent files for production.

State for governance proposals and mana balances is rebuilt by replaying events. This replaces ad-hoc storage of object snapshots and allows tamper-evident history anchored in the DAG.

To migrate existing data, export the current proposal and ledger records, generate equivalent events, and append them to a fresh event store. On startup modules rebuild their in-memory state by querying all events.

`ParameterUpdate` events ensure that runtime parameters can be reconstructed even if configuration files are lost. These events record the parameter name and its new value whenever the node applies a change.
