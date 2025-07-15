# ICN Event Store (`icn-eventstore`)

This crate provides event sourcing utilities for the InterCooperative Network (ICN). It offers append-only event storage with serialization support for building auditable, replayable systems.

See [CONTEXT.md](../../CONTEXT.md) for ICN Core design philosophy and crate roles.
See [docs/ASYNC_OVERVIEW.md](../../docs/ASYNC_OVERVIEW.md) for async API guidelines.

## Purpose

The `icn-eventstore` crate is responsible for:

* **Event Sourcing Pattern**: Providing the foundation for event-driven architecture within ICN
* **Audit Trails**: Maintaining immutable logs of all system events for transparency and accountability
* **State Recovery**: Enabling system state reconstruction from event history
* **Serialization**: Managing JSON serialization of events for storage and transmission
* **Multiple Storage Backends**: Supporting both memory and file-based event storage

## Key Features

### Event Store Trait
The core `EventStore` trait defines the interface for event storage systems:

```rust
pub trait EventStore<E>: Send + Sync
where
    E: Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync,
{
    fn append(&mut self, event: &E) -> Result<(), CommonError>;
    fn query(&self, since: Option<usize>) -> Result<Vec<E>, CommonError>;
}
```

### Memory Event Store
In-memory storage for testing and development:

```rust
use icn_eventstore::MemoryEventStore;

let mut store = MemoryEventStore::new();
store.append(&MyEvent { data: "example" })?;
let events = store.query(None)?; // Get all events
```

### File Event Store
Persistent file-based storage for production use:

```rust
use icn_eventstore::FileEventStore;
use std::path::PathBuf;

let mut store = FileEventStore::new(PathBuf::from("events.jsonl"));
store.append(&MyEvent { data: "example" })?;
let events = store.query(Some(10))?; // Get events since index 10
```

## Usage Examples

### Basic Event Storage
```rust
use icn_eventstore::{EventStore, MemoryEventStore};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TransactionEvent {
    timestamp: u64,
    from: String,
    to: String,
    amount: u64,
}

let mut store = MemoryEventStore::new();
let event = TransactionEvent {
    timestamp: 1234567890,
    from: "alice".to_string(),
    to: "bob".to_string(),
    amount: 100,
};

store.append(&event)?;
let events = store.query(None)?;
assert_eq!(events.len(), 1);
```

### Persistent Event Storage
```rust
use icn_eventstore::{EventStore, FileEventStore};
use std::path::PathBuf;

let mut store = FileEventStore::new(PathBuf::from("ledger.jsonl"));
store.append(&event)?;

// Events are now persisted to disk
let events = store.query(None)?;
```

### Incremental Event Querying
```rust
// Get all events initially
let initial_events = store.query(None)?;
let checkpoint = initial_events.len();

// Add more events
store.append(&new_event)?;

// Get only new events since checkpoint
let new_events = store.query(Some(checkpoint))?;
```

## Storage Formats

### File Format
The `FileEventStore` uses JSON Lines format where each line contains a serialized event:

```jsonl
{"timestamp":1234567890,"from":"alice","to":"bob","amount":100}
{"timestamp":1234567891,"from":"bob","to":"charlie","amount":50}
```

### Memory Format
The `MemoryEventStore` maintains events in a Vec for fast access during testing and development.

## Error Handling

All operations return `Result<T, CommonError>` for consistent error handling:

```rust
match store.append(&event) {
    Ok(()) => println!("Event stored successfully"),
    Err(e) => eprintln!("Storage error: {}", e),
}
```

## Thread Safety

Both implementations are `Send + Sync` and can be safely shared across threads:

```rust
use std::sync::{Arc, Mutex};

let store = Arc::new(Mutex::new(MemoryEventStore::new()));
// Store can now be shared across threads
```

## Integration with ICN

The event store is used throughout ICN for:

* **Governance Events**: Tracking proposals, votes, and policy changes
* **Economic Events**: Recording mana transactions and token movements
* **Mesh Events**: Logging job submissions, executions, and completions
* **Identity Events**: Storing credential issuance and revocation events
* **Network Events**: Recording peer connections and federation changes

## Testing

```bash
# Run all tests
cargo test -p icn-eventstore

# Run specific test
cargo test -p icn-eventstore memory_store_round_trip
```

## Performance Considerations

* **Memory Store**: Fast for development and testing, but limited by available RAM
* **File Store**: Persistent but slower due to disk I/O
* **Querying**: Linear scan for file storage; consider external indexing for large datasets
* **Serialization**: JSON format is human-readable but may be slower than binary formats

## Contributing

Contributions are welcome! Please see the main [CONTRIBUTING.md](../../CONTRIBUTING.md) in the root of the `icn-core` repository for guidelines.

**Areas for contribution:**
- Additional storage backends (database, distributed stores)
- Performance optimizations
- Event indexing and query optimization
- Compression and archival support

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](../../LICENSE). 