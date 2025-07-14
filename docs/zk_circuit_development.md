# Zero-Knowledge Circuit Development

This guide walks through adding custom circuits to the `icn-zk` crate and registering them with a running node.

## 1. Implementing `ConstraintSynthesizer`

Circuits are implemented using the Arkworks R1CS traits. A new circuit simply implements `ConstraintSynthesizer<Fr>` and defines witnesses and public inputs.

```rust
use ark_bn254::Fr;
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};

#[derive(Clone)]
pub struct SumCheck {
    pub x: u64,
    pub y: u64,
}

impl ConstraintSynthesizer<Fr> for SumCheck {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        let x = FpVar::<Fr>::new_witness(cs.clone(), || Ok(Fr::from(self.x)))?;
        let y = FpVar::<Fr>::new_witness(cs.clone(), || Ok(Fr::from(self.y)))?;
        let sum = FpVar::<Fr>::new_input(cs, || Ok(Fr::from(self.x + self.y)))?;
        (x + y).enforce_equal(&sum)?;
        Ok(())
    }
}
```

Compile the circuit together with the existing ones by adding it to `crates/icn-zk/src/circuits.rs` and exporting it from the crate root.

## 2. Inspecting Constraints

`icn-zk` provides development helpers under the optional `dev-tools` feature. Using
`ConstraintSystem::<Fr>::new_ref()` you can run the circuit and inspect how many
constraints or variables it created.

```rust
use ark_relations::r1cs::ConstraintSystem;
use icn_zk::AgeOver18Circuit;

let cs = ConstraintSystem::<Fr>::new_ref();
let circuit = AgeOver18Circuit { birth_year: 2000, current_year: 2025 };
circuit.clone().generate_constraints(cs.clone()).unwrap();
println!("{} constraints", cs.num_constraints());
```

This information helps tune mana costs for new circuits.

## 3. Profiling with Criterion

Benchmark proving and verification performance with the `criterion` crate. Create
`benches/proof_bench.rs`:

```rust
use criterion::{criterion_group, criterion_main, Criterion};
use icn_zk::{prove, setup, prepare_vk, verify, AgeOver18Circuit};
use ark_std::rand::{rngs::StdRng, SeedableRng};

fn prove_bench(c: &mut Criterion) {
    let circuit = AgeOver18Circuit { birth_year: 2000, current_year: 2024 };
    let mut rng = StdRng::seed_from_u64(42);
    let pk = setup(circuit.clone(), &mut rng).unwrap();
    let vk = prepare_vk(&pk);
    c.bench_function("prove", |b| b.iter(|| prove(&pk, circuit.clone(), &mut rng)));
    let proof = prove(&pk, circuit, &mut rng).unwrap();
    c.bench_function("verify", |b| b.iter(|| verify(&vk, &proof, &[Fr::from(2024u64)])));
}

criterion_group!(benches, prove_bench);
criterion_main!(benches);
```

Running `cargo bench -p icn-zk --features dev-tools` prints detailed timings for
proving and verification.

## 4. Registering the Circuit

Nodes expose `/circuits/register` for dynamic circuit updates. After generating a
proving key and verifying key, POST them to the registry:

```bash
curl -X POST http://localhost:7845/circuits/register \
     -H "Content-Type: application/json" \
     --data @docs/examples/custom_circuit_register.json
```

The response contains a CID for the stored parameters. Proof payloads can then
reference the circuit by slug and version when calling `/identity/verify` or
other endpoints.

See `docs/examples/zk_custom_circuit.json` for an example proof request.
