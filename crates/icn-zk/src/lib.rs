//! Reusable zero-knowledge circuits for ICN credential proofs.

use ark_bn254::{Bn254, Fr};
use ark_groth16::{Groth16, PreparedVerifyingKey, Proof, ProvingKey};
use ark_relations::r1cs::{ConstraintSynthesizer, SynthesisError};
use ark_snark::SNARK;
use ark_std::rand::{CryptoRng, RngCore};

mod circuits;
mod params;

pub use circuits::{
    AgeOver18Circuit, BalanceRangeCircuit, MembershipCircuit, MembershipProofCircuit,
    ReputationCircuit, TimestampValidityCircuit,
};
pub use params::{CircuitParameters, CircuitParametersStorage, MemoryParametersStorage};

/// Generate Groth16 parameters for a given circuit.
pub fn setup<C: ConstraintSynthesizer<Fr>, R: RngCore + CryptoRng>(
    circuit: C,
    rng: &mut R,
) -> Result<ProvingKey<Bn254>, SynthesisError> {
    let (pk, _vk) = Groth16::<Bn254>::circuit_specific_setup(circuit, rng)?;
    Ok(pk)
}

/// Create a Groth16 proof for the provided circuit and proving key.
pub fn prove<C: ConstraintSynthesizer<Fr>, R: RngCore + CryptoRng>(
    pk: &ProvingKey<Bn254>,
    circuit: C,
    rng: &mut R,
) -> Result<Proof<Bn254>, SynthesisError> {
    Groth16::<Bn254>::prove(pk, circuit, rng)
}

/// Prepare the verifying key for use in verification.
pub fn prepare_vk(pk: &ProvingKey<Bn254>) -> PreparedVerifyingKey<Bn254> {
    Groth16::<Bn254>::process_vk(&pk.vk).unwrap()
}

/// Verify a Groth16 proof with the given verifying key and public inputs.
pub fn verify(
    vk: &PreparedVerifyingKey<Bn254>,
    proof: &Proof<Bn254>,
    public_inputs: &[Fr],
) -> Result<bool, SynthesisError> {
    Groth16::<Bn254>::verify_with_processed_vk(vk, public_inputs, proof)
}

#[cfg(test)]
mod tests;
