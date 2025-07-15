# ICN Zero-Knowledge Circuits (`icn-zk`)

This crate provides reusable zero-knowledge circuits for privacy-preserving credential proofs within the InterCooperative Network (ICN). It implements various circuits using the [arkworks](https://arkworks.rs/) framework with Groth16 proofs on the BN254 curve.

See [CONTEXT.md](../../CONTEXT.md) for ICN Core design philosophy and crate roles.
See [docs/ASYNC_OVERVIEW.md](../../docs/ASYNC_OVERVIEW.md) for async API guidelines.

## Purpose

The `icn-zk` crate is responsible for:

* **Privacy-Preserving Proofs**: Enabling users to prove attributes about themselves without revealing sensitive information
* **Credential Verification**: Providing circuits for verifying credentials while maintaining privacy
* **Reputation Proofs**: Allowing users to prove reputation thresholds without disclosing exact scores
* **Age Verification**: Enabling age-over-18 proofs without revealing birth dates
* **Membership Proofs**: Proving membership in organizations without revealing identity details
* **Range Proofs**: Proving values fall within specified ranges without disclosing exact amounts

## Available Circuits

### Age Verification

#### `AgeOver18Circuit`
Proves that a user is at least 18 years old without revealing their exact birth year.

**Private Inputs:**
- `birth_year`: The user's birth year

**Public Inputs:**
- `current_year`: The current year

**Complexity**: 10 units

```rust
use icn_zk::{AgeOver18Circuit, setup, prove, prepare_vk, verify};

let circuit = AgeOver18Circuit {
    birth_year: 2000,  // Private - not revealed
    current_year: 2024, // Public - part of proof
};
```

### Membership Verification

#### `MembershipCircuit`
Proves knowledge of a membership boolean that equals `true`.

**Public Inputs:**
- `is_member`: Whether the prover is a member

**Complexity**: 5 units

#### `MembershipProofCircuit`
Proves that a private membership flag matches an expected public value.

**Private Inputs:**
- `membership_flag`: The actual membership status

**Public Inputs:**
- `expected`: The expected membership value

**Complexity**: 5 units

### Reputation Verification

#### `ReputationCircuit`
Proves that a user's reputation score meets or exceeds a required threshold.

**Public Inputs:**
- `reputation`: The user's reputation score
- `threshold`: The required minimum reputation

**Complexity**: 15 units

```rust
use icn_zk::{ReputationCircuit, setup, prove, prepare_vk, verify};

let circuit = ReputationCircuit {
    reputation: 85,  // Public - but could be made private
    threshold: 50,   // Public - verification requirement
};
```

### Timestamp Validation

#### `TimestampValidityCircuit`
Proves that a timestamp falls within a valid time range.

**Private Inputs:**
- `timestamp`: The timestamp to validate

**Public Inputs:**
- `not_before`: Earliest acceptable timestamp
- `not_after`: Latest acceptable timestamp

**Complexity**: 5 units

### Balance Range Proofs

#### `BalanceRangeCircuit`
Proves that a balance amount falls within specified minimum and maximum bounds.

**Private Inputs:**
- `balance`: The actual balance amount

**Public Inputs:**
- `min`: Minimum acceptable balance
- `max`: Maximum acceptable balance

**Complexity**: 5 units

### Composite Circuits

#### `AgeRepMembershipCircuit`
A composite circuit that combines age verification, reputation threshold, and membership checks.

**Private Inputs:**
- `birth_year`: The user's birth year

**Public Inputs:**
- `current_year`: The current year
- `reputation`: The user's reputation score
- `threshold`: Required reputation threshold
- `is_member`: Membership status

**Complexity**: 20 units

## Circuit Parameters and Setup

### Parameter Management

The crate provides a parameter storage system for managing circuit proving keys:

```rust
use icn_zk::{CircuitParameters, MemoryParametersStorage, CircuitParametersStorage};

// Create in-memory parameter storage
let mut storage = MemoryParametersStorage::default();

// Store parameters for a circuit
let params = CircuitParameters::from_proving_key(&proving_key)?;
storage.put("age_over_18", params);

// Retrieve parameters
let stored_params = storage.get("age_over_18").unwrap();
let pk = stored_params.proving_key()?;
let prepared_vk = stored_params.prepared_vk()?;
```

### Trusted Setup

Each circuit requires a trusted setup to generate proving and verifying keys:

```rust
use icn_zk::{setup, prove, prepare_vk, verify};
use ark_std::rand::{rngs::StdRng, SeedableRng};

let mut rng = StdRng::seed_from_u64(42);
let circuit = AgeOver18Circuit { birth_year: 2000, current_year: 2024 };

// Trusted setup
let proving_key = setup(circuit.clone(), &mut rng)?;
let prepared_vk = prepare_vk(&proving_key);
```

## Usage Examples

### Basic Proof Generation and Verification

```rust
use icn_zk::{AgeOver18Circuit, setup, prove, prepare_vk, verify};
use ark_bn254::Fr;
use ark_std::rand::{rngs::StdRng, SeedableRng};

// Create circuit
let circuit = AgeOver18Circuit {
    birth_year: 2000,
    current_year: 2024,
};

// Setup (trusted ceremony)
let mut rng = StdRng::seed_from_u64(42);
let proving_key = setup(circuit.clone(), &mut rng)?;
let prepared_vk = prepare_vk(&proving_key);

// Generate proof
let proof = prove(&proving_key, circuit, &mut rng)?;

// Verify proof
let public_inputs = [Fr::from(2024u64)]; // Current year
let is_valid = verify(&prepared_vk, &proof, &public_inputs)?;
assert!(is_valid);
```

### Batch Verification

The crate supports efficient batch verification of multiple proofs:

```rust
use icn_zk::{verify_batch, prepare_vk};

let proofs_and_inputs = vec![
    (&proof1, &public_inputs1[..]),
    (&proof2, &public_inputs2[..]),
    (&proof3, &public_inputs3[..]),
];

let all_valid = verify_batch(&prepared_vk, &proofs_and_inputs)?;
```

### Reputation Thresholds

The crate defines default reputation thresholds required for different circuit types:

```rust
use icn_zk::ReputationThresholds;

let thresholds = ReputationThresholds::default();
println!("Age verification requires {} reputation", thresholds.age_over_18);
println!("Membership proof requires {} reputation", thresholds.membership);
```

## Circuit Complexity and Mana Costs

Each circuit implements the `CircuitCost` trait to provide complexity estimates for mana cost calculation:

```rust
use icn_zk::{AgeOver18Circuit, CircuitCost};

let complexity = AgeOver18Circuit::complexity(); // Returns 10
```

**Complexity Scores:**
- `AgeOver18Circuit`: 10 units
- `MembershipCircuit`: 5 units
- `MembershipProofCircuit`: 5 units
- `ReputationCircuit`: 15 units
- `TimestampValidityCircuit`: 5 units
- `BalanceRangeCircuit`: 5 units
- `AgeRepMembershipCircuit`: 20 units

## Integration with ICN Identity

This crate integrates with the `icn-identity` crate to provide zero-knowledge credential proofs:

```rust
use icn_identity::{ZkProver, Groth16Prover};
use icn_zk::{AgeOver18Circuit, setup};

// Create a Groth16 prover for age verification
let circuit = AgeOver18Circuit { birth_year: 2000, current_year: 2024 };
let proving_key = setup(circuit, &mut rng)?;
let prover = Groth16Prover::new("age_over_18".to_string(), proving_key);

// Use with identity system
let proof = prover.generate_proof(&credential_data)?;
```

## Security Considerations

### Trusted Setup

- Circuit parameters must be generated through a trusted setup ceremony
- Proving keys should be stored securely and distributed through trusted channels
- Parameter tampering can compromise proof security

### Circuit Design

- Private inputs should never be revealed in constraints
- Public inputs are visible to all verifiers
- Ensure sufficient entropy in randomness sources

### Implementation Notes

- Uses BN254 curve for efficiency and compatibility
- Groth16 proofs provide succinct verification
- Batch verification reduces verification costs for multiple proofs

## Testing

The crate includes comprehensive tests for all circuits:

```bash
# Run all tests
cargo test -p icn-zk

# Run specific circuit tests
cargo test -p icn-zk age_over_18_proof
cargo test -p icn-zk reputation_threshold_proof
cargo test -p icn-zk batch_verify_multiple_proofs
```

## Contributing

Contributions are welcome! Please see the main [CONTRIBUTING.md](../../CONTRIBUTING.md) in the root of the `icn-core` repository for guidelines.

**Areas for contribution:**
- New circuit implementations for additional use cases
- Optimization of existing circuits
- Enhanced parameter management systems
- Integration with hardware security modules

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](../../LICENSE). 