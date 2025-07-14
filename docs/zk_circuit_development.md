# Custom ZK Circuit Development

This guide walks through implementing your own zero-knowledge circuit using the `icn-zk` crate. It also covers how to inspect the generated constraints, profile proof generation with Criterion, and register the circuit in the dynamic registry so that nodes can verify proofs.

## 1. Implement a `ConstraintSynthesizer`

Circuits in `icn-zk` implement the `ConstraintSynthesizer<Fr>` trait from `ark-relations`. The example below proves that a private value is less than `10`:

```rust
use ark_bn254::Fr;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_r1cs_std::prelude::*;
use core::cmp::Ordering;

#[derive(Clone)]
pub struct LessThan10Circuit {
    pub value: u64, // public input
}

impl ConstraintSynthesizer<Fr> for LessThan10Circuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        let val = FpVar::<Fr>::new_input(cs, || Ok(Fr::from(self.value)))?;
        let threshold = FpVar::<Fr>::Constant(Fr::from(10u64));
        val.enforce_cmp(&threshold, Ordering::Less, true)?;
        Ok(())
    }
}
```

Implementing the trait defines how the circuit's constraints are produced. Complex circuits may also implement the optional `CircuitCost` trait to report a relative complexity score.

## 2. Inspect Constraints with `icn-zk` Devtools

The `icn-zk` crate provides helper functions behind the `devtools` feature flag. Enable it in your `Cargo.toml`:

```toml
[dependencies]
icn-zk = { path = "../crates/icn-zk", features = ["devtools"] }
```

You can then create a `ConstraintSystem`, synthesize the circuit, and print statistics:

```rust
use ark_relations::r1cs::ConstraintSystem;
use icn_zk::devtools::print_cs_stats;

let cs = ConstraintSystem::<Fr>::new_ref();
LessThan10Circuit { value: 7 }.clone().generate_constraints(cs.clone())?;
print_cs_stats(&cs)?; // prints number of constraints and variables
```

This is useful during development to ensure the circuit is as small as possible.

## 3. Profile Proof Generation with Criterion

Benchmark proof creation by adding a Criterion benchmark under `benches/`:

```rust
use criterion::{criterion_group, criterion_main, Criterion};
use icn_zk::{prove, setup, prepare_vk};
use rand::thread_rng;

fn bench_prove(c: &mut Criterion) {
    let mut rng = thread_rng();
    let pk = setup(LessThan10Circuit { value: 7 }, &mut rng).unwrap();
    c.bench_function("prove_less_than_10", |b| {
        b.iter(|| prove(&pk, LessThan10Circuit { value: 7 }, &mut rng).unwrap());
    });
}

criterion_group!(benches, bench_prove);
criterion_main!(benches);
```

Run `cargo bench` to measure proof times and track regressions.

## 4. Register the Circuit

Once a proving key is generated, register the circuit with the dynamic registry so nodes can verify proofs. Upload the parameters using the `/circuits/register` endpoint:

```bash
curl -X POST http://localhost:7845/circuits/register \
     -H "Content-Type: application/json" \
     --data @docs/examples/custom_circuit_register.json
```

After registration you can reference the circuit by slug and version when creating proofs or verification requests.

## 5. Run the Example

The repository provides [`examples/zk_custom_circuit.rs`](../examples/zk_custom_circuit.rs)
demonstrating this circuit in action. Build and run with the `devtools` feature
to see constraint statistics:

```bash
cargo run --example zk_custom_circuit --features icn-zk/devtools
```

The proof payload generated matches [`docs/examples/custom_circuit_proof.json`](examples/custom_circuit_proof.json).

---

Continue with [zk_disclosure.md](zk_disclosure.md) for general zero-knowledge usage guidelines and [dynamic_circuits.md](dynamic_circuits.md) for registry details.
