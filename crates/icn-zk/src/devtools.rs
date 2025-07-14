use super::Fr;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystem, SynthesisError};

/// Count the number of constraints for the given circuit.
pub fn count_constraints<C: ConstraintSynthesizer<Fr>>(
    circuit: C,
) -> Result<usize, SynthesisError> {
    let cs = ConstraintSystem::<Fr>::new_ref();
    circuit.generate_constraints(cs.clone())?;
    Ok(cs.num_constraints())
}
