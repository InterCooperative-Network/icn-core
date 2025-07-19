# ICN Common (`icn-common`) - Foundation Crate

> **Foundation layer providing shared types, utilities, and error handling for the entire ICN ecosystem**

## Overview

The `icn-common` crate serves as the foundational layer for the InterCooperative Network, providing essential data structures, error handling, cryptographic operations, and utility functions that are used across all other ICN crates.

**Key Principle**: This crate maintains minimal dependencies and contains no business logic—only fundamental types and utilities that enable higher-level functionality.

## Core Components

### 📊 Data Structures

#### Decentralized Identifiers (DIDs)
```rust
pub struct Did {
    pub method: String,      // e.g., "key", "web"
    pub id_string: String,   // Method-specific identifier
    pub path: Option<String>,
    pub query: Option<String>, 
    pub fragment: Option<String>,
}
```

**Features:**
- **Parsing**: Full DID URL parsing with path/query/fragment support
- **Display**: Canonical string representation
- **Validation**: Format validation and semantic checks
- **Examples**: `did:key:z6Mk...`, `did:web:example.com:user:alice/profile#key-1`

#### Content Identifiers (CIDs)
```rust
pub struct Cid {
    pub version: u64,        // CIDv0 or CIDv1
    pub codec: u64,          // Multicodec (dag-pb, dag-cbor, etc.)
    pub hash_alg: u64,       // Hash algorithm (sha2-256, etc.)
    pub hash_bytes: Vec<u8>, // Raw hash bytes
}
```

**Features:**
- **Creation**: `new_v1_sha256()` for SHA-256 based CIDs
- **Serialization**: Multibase encoding/decoding
- **Validation**: Integrity verification
- **Standards**: IPFS-compatible CID implementation

#### DAG Blocks
```rust
pub struct DagBlock {
    pub cid: Cid,
    pub data: Vec<u8>,
    pub links: Vec<DagLink>,
    pub timestamp: u64,
    pub author_did: Did,
    pub signature: Option<SignatureBytes>,
    pub scope: Option<NodeScope>,
}
```

**Features:**
- **Merkle Integrity**: `compute_merkle_cid()` and `verify_block_integrity()`
- **Cryptographic Signing**: Ed25519 signature support via `Signable` trait
- **Content Addressing**: Deterministic CID computation
- **DAG Links**: References to other blocks

#### Transactions
```rust
pub struct Transaction {
    pub id: String,
    pub timestamp: u64,
    pub sender_did: Did,
    pub recipient_did: Option<Did>,
    pub payload_type: String,
    pub payload: Vec<u8>,
    pub nonce: u64,
    pub mana_limit: u64,
    pub mana_price: u64,
    pub signature: Option<SignatureBytes>,
}
```

**Features:**
- **Economic Integration**: Built-in mana limit and pricing
- **Cryptographic Security**: Ed25519 signing and verification
- **Flexible Payload**: Type-tagged arbitrary data
- **Nonce Protection**: Replay attack prevention

### 🔐 Cryptographic Operations

#### Signable Trait
```rust
pub trait Signable {
    fn to_signable_bytes(&self) -> Result<Vec<u8>, CommonError>;
    fn sign(&self, key: &ed25519_dalek::SigningKey) -> Result<SignatureBytes, CommonError>;
    fn verify(&self, signature: &SignatureBytes, key: &ed25519_dalek::VerifyingKey) -> Result<(), CommonError>;
}
```

**Implementations:**
- `DagBlock`: Signs data, links, timestamp, and author
- `Transaction`: Signs all transaction fields including payload
- `DidDocument`: Signs DID and public key

**Security Features:**
- **Canonical Serialization**: Consistent byte representation for signing
- **Ed25519 Integration**: High-performance signature algorithm
- **Verification**: Built-in signature validation

### 🌐 Zero-Knowledge Support

#### ZK Credential Proofs
```rust
pub struct ZkCredentialProof {
    pub issuer: Did,
    pub holder: Did,
    pub claim_type: String,
    pub proof: Vec<u8>,
    pub schema: Cid,
    pub vk_cid: Option<Cid>,
    pub disclosed_fields: Vec<String>,
    pub challenge: Option<String>,
    pub backend: ZkProofType,
    pub verification_key: Option<Vec<u8>>,
    pub public_inputs: Option<serde_json::Value>,
}
```

**Supported Backends:**
- **Groth16**: zk-SNARK proofs
- **Bulletproofs**: Range proofs and more
- **Extensible**: `Other(String)` for custom systems

#### ZK Revocation Proofs
```rust
pub struct ZkRevocationProof {
    pub issuer: Did,
    pub subject: Did,
    pub proof: Vec<u8>,
    pub backend: ZkProofType,
    // ... verification fields
}
```

### 💰 Resource Token System

#### Token Classes
```rust
pub struct TokenClassMetadata {
    pub scope: Option<NodeScope>,
    pub issuer: Did,
    pub unit: String,
    pub is_transferable: bool,
    pub fungible: bool,
    pub description: Option<String>,
}
```

**Features:**
- **Scoped Tokens**: Geographic or community-limited tokens
- **Fungible/Non-Fungible**: Support for both token types
- **Transfer Controls**: Configurable transferability
- **Rich Metadata**: Human-readable descriptions

### ⚡ Resilience Patterns

#### Circuit Breaker
```rust
pub struct CircuitBreaker<T: TimeProvider> {
    failure_threshold: usize,
    timeout: Duration,
    // ... state management
}
```

**States:**
- **Closed**: Normal operation
- **Open**: Failing fast to prevent cascade failures
- **Half-Open**: Testing if service has recovered

#### Retry with Backoff
```rust
pub async fn retry_with_backoff<F, Fut, T, E>(
    operation: F,
    max_retries: u32,
    initial_delay: Duration,
    max_delay: Duration,
) -> Result<T, E>
```

**Features:**
- **Exponential Backoff**: Increasing delay between attempts
- **Jitter**: Random variation to prevent thundering herd
- **Configurable**: Customizable retry parameters

### 🔧 Provider Abstractions

#### Time Provider
```rust
pub trait TimeProvider: Send + Sync {
    fn unix_seconds(&self) -> u64;
}
```

**Implementations:**
- `SystemTimeProvider`: Uses `std::time::SystemTime`
- `FixedTimeProvider`: Deterministic time for testing

#### System Info Provider
```rust
pub trait SystemInfoProvider: Send + Sync {
    fn cpu_cores(&self) -> u32;
    fn memory_mb(&self) -> u32;
}
```

**Implementations:**
- `SysinfoSystemInfoProvider`: Real system information
- `FixedSystemInfoProvider`: Fixed values for testing

### ❌ Error Handling

#### CommonError Enum
```rust
pub enum CommonError {
    InvalidInputError(String),
    SerializationError(String),
    CryptoError(String),
    NetworkError(String),
    StorageError(String),
    PolicyDenied(String),
    // ... 20+ variants
}
```

**Categories:**
- **Input/Validation**: Invalid data or parameters
- **Serialization**: JSON/binary encoding issues
- **Cryptography**: Signature or key problems
- **Network**: P2P communication failures
- **Storage**: DAG or database issues
- **Policy**: Governance rule violations
- **System**: I/O, timeout, and internal errors

## Node Information Types

### Node Status
```rust
pub struct NodeInfo {
    pub version: String,
    pub name: String,
    pub status_message: String,
}

pub struct NodeStatus {
    pub is_online: bool,
    pub peer_count: u32,
    pub current_block_height: u64,
    pub version: String,
}
```

### DAG Sync Status
```rust
pub struct DagSyncStatus {
    pub current_root: Option<Cid>,
    pub in_sync: bool,
}
```

## Testing Support

### Test Utilities
```rust
// Fixed providers for deterministic testing
let time_provider = FixedTimeProvider::new(1640995200); // Fixed timestamp
let system_info = FixedSystemInfoProvider::new(4, 8192); // 4 cores, 8GB RAM

// DID creation
let did = Did::new("key", "z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuias7ux1jEZ6KATp8");

// CID creation
let cid = Cid::new_v1_sha256(0x71, b"test data");

// Signature testing
let (private_key, public_key) = generate_test_keypair();
let signature = transaction.sign(&private_key)?;
assert!(transaction.verify(&signature, &public_key).is_ok());
```

## Constants and Version

```rust
pub const ICN_CORE_VERSION: &str = "0.2.0-beta";
```

## Dependencies

**Core Dependencies:**
- `serde`: Serialization framework
- `thiserror`: Error handling macros
- `ed25519-dalek`: Cryptographic signatures
- `sha2`: Hash functions
- `multibase`/`multicodec`: IPFS compatibility
- `tokio`: Async time utilities

**Development Dependencies:**
- `rand_core`: Key generation for tests
- `tokio`: Test runtime

## Usage Patterns

### Basic Usage
```rust
use icn_common::{Did, Cid, CommonError, Signable};
use std::str::FromStr;

// Parse a DID
let did = Did::from_str("did:key:z6MkjExample")?;
println!("Method: {}, ID: {}", did.method, did.id_string);

// Create a content identifier
let cid = Cid::new_v1_sha256(0x71, b"hello world");
println!("CID: {}", cid);

// Error handling
fn process_data(data: &[u8]) -> Result<String, CommonError> {
    if data.is_empty() {
        return Err(CommonError::InvalidInputError("Empty data".to_string()));
    }
    Ok(String::from_utf8_lossy(data).to_string())
}
```

### Cryptographic Operations
```rust
use icn_common::{Transaction, Signable, Did};
use ed25519_dalek::SigningKey;
use rand_core::OsRng;

// Create and sign a transaction
let signing_key = SigningKey::generate(&mut OsRng);
let verifying_key = signing_key.verifying_key();

let transaction = Transaction {
    id: "tx123".to_string(),
    sender_did: Did::new("key", "alice"),
    // ... other fields
};

let signature = transaction.sign(&signing_key)?;
assert!(transaction.verify(&signature, &verifying_key).is_ok());
```

### Resilience Patterns
```rust
use icn_common::{retry_with_backoff, CircuitBreaker, SystemTimeProvider};
use std::time::Duration;

// Retry with exponential backoff
let result = retry_with_backoff(
    || async { risky_operation().await },
    3,                                    // max retries
    Duration::from_millis(100),          // initial delay
    Duration::from_secs(5),              // max delay
).await?;

// Circuit breaker
let breaker = CircuitBreaker::new(
    SystemTimeProvider,
    5,                          // failure threshold
    Duration::from_secs(30),    // timeout
);

let result = breaker.call(|| async {
    external_service_call().await
}).await?;
```

## Architecture Guidelines

### Design Principles
1. **Minimal Dependencies**: Keep external dependencies to essential libraries only
2. **No Business Logic**: This crate contains only foundational types and utilities
3. **Deterministic**: All operations should be reproducible given the same inputs
4. **Error-First**: All fallible operations return `Result<T, CommonError>`
5. **Async-Ready**: Support for async operations where needed

### Extension Points
- **New Error Variants**: Add to `CommonError` enum for new error categories
- **Provider Traits**: Implement new providers for different environments
- **Signable Types**: Implement `Signable` for new data structures requiring signatures
- **Token Types**: Extend token system for new economic models

## Future Development

### Planned Enhancements
- **Quantum-Resistant Cryptography**: Post-quantum signature algorithms
- **Enhanced CID Support**: Additional hash algorithms and codecs
- **Advanced Resilience**: More sophisticated failure handling patterns
- **Performance Optimizations**: Zero-copy operations where possible

### Migration Considerations
- **Backward Compatibility**: Major changes will be versioned appropriately
- **Deprecation Process**: Old APIs will be marked deprecated before removal
- **Migration Guides**: Documentation for upgrading between versions

---

**The `icn-common` crate provides the stable, well-tested foundation that enables all higher-level ICN functionality. Every type and function is designed for reusability, performance, and correctness.** 