use ark_bn254::Fr;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystem, ConstraintSystemRef, SynthesisError};
use ark_r1cs_std::prelude::*;
use ark_std::rand::{rngs::StdRng, SeedableRng};
use icn_zk::{prepare_vk, prove, setup, verify};

#[derive(Clone)]
struct SumCheck {
    x: u64,
    y: u64,
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let circuit = SumCheck { x: 1, y: 2 };
    let cs = ConstraintSystem::<Fr>::new_ref();
    circuit.clone().generate_constraints(cs.clone())?;
    println!(
        "constraints: {} inputs: {} witnesses: {}",
        cs.num_constraints(),
        cs.num_instance_variables(),
        cs.num_witness_variables()
    );

    let mut rng = StdRng::seed_from_u64(42);
    let pk = setup(circuit.clone(), &mut rng)?;
    let proof = prove(&pk, circuit, &mut rng)?;
    let vk = prepare_vk(&pk);
    let valid = verify(&vk, &proof, &[Fr::from(3u64)])?;
    println!("proof valid: {}", valid);
    Ok(())
}
