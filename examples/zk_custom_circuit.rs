//! Example custom circuit demonstrating constraint analysis and proof verification.
//!
//! Run with `cargo run --example zk_custom_circuit`.

use ark_bn254::Fr;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystem, ConstraintSystemRef, SynthesisError};
use ark_r1cs_std::prelude::*;
use icn_zk::{prepare_vk, prove, setup, verify};
#[cfg(feature = "devtools")]
use icn_zk::devtools::print_cs_stats;
use rand::thread_rng;
use core::cmp::Ordering;

#[derive(Clone)]
struct LessThan10Circuit {
    value: u64,
}

impl ConstraintSynthesizer<Fr> for LessThan10Circuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        let val = FpVar::<Fr>::new_input(cs, || Ok(Fr::from(self.value)))?;
        let threshold = FpVar::<Fr>::Constant(Fr::from(10u64));
        val.enforce_cmp(&threshold, Ordering::Less, true)?;
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Inspect constraint count
    let cs = ConstraintSystem::<Fr>::new_ref();
    LessThan10Circuit { value: 7 }.clone().generate_constraints(cs.clone())?;
    #[cfg(feature = "devtools")]
    print_cs_stats(&cs)?;
    #[cfg(not(feature = "devtools"))]
    println!("constraints: {}", cs.num_constraints());

    // Generate parameters and a proof
    let mut rng = thread_rng();
    let pk = setup(LessThan10Circuit { value: 7 }, &mut rng)?;
    let proof = prove(&pk, LessThan10Circuit { value: 7 }, &mut rng)?;
    let pvk = prepare_vk(&pk);

    // Public input is the value itself
    let verified = verify(&pvk, &proof, &[Fr::from(7u64)])?;
    println!("verified: {}", verified);
    Ok(())
}
