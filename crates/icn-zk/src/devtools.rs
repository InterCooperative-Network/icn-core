use super::Fr;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystem, SynthesisError};

/// Count the number of constraints for the provided circuit.
pub fn count_constraints<C: ConstraintSynthesizer<Fr>>(
    circuit: C,
) -> Result<usize, SynthesisError> {
    let cs = ConstraintSystem::<Fr>::new_ref();
    circuit.generate_constraints(cs.clone())?;
    cs.finalize();
    Ok(cs.num_constraints())
}

/// Log information about all constraints in the provided circuit.
pub fn log_constraints<C: ConstraintSynthesizer<Fr>>(circuit: C) -> Result<(), SynthesisError> {
    let cs = ConstraintSystem::<Fr>::new_ref();
    circuit.generate_constraints(cs.clone())?;
    cs.finalize();

    println!("instance variables: {}", cs.num_instance_variables());
    println!("witness variables: {}", cs.num_witness_variables());
    println!("constraints: {}", cs.num_constraints());

    if let Some(names) = cs.constraint_names() {
        for (i, name) in names.into_iter().enumerate() {
            println!("{i}: {name}");
        }
    }
    Ok(())
}
