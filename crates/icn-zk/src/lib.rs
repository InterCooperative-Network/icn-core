//! Reusable zero-knowledge circuits for ICN credential proofs.

use ark_bn254::{Bn254, Fr};
use ark_groth16::{Groth16, PreparedVerifyingKey, Proof, ProvingKey};
use ark_relations::r1cs::{ConstraintSynthesizer, SynthesisError};
use ark_snark::SNARK;
use ark_std::rand::{CryptoRng, RngCore};
use rayon::prelude::*;

mod circuits;
pub mod devtools;
mod params;

pub use circuits::{
    AgeOver18Circuit, AgeRepMembershipCircuit, BalanceRangeCircuit, CircuitCost, MembershipCircuit,
    MembershipProofCircuit, ReputationCircuit, TimestampValidityCircuit,
};
pub use params::{CircuitParameters, CircuitParametersStorage, MemoryParametersStorage};

/// Reputation thresholds required to prove or verify each circuit type.
#[derive(Debug, Clone)]
pub struct ReputationThresholds {
    pub age_over_18: u64,
    pub membership: u64,
    pub membership_proof: u64,
    pub reputation: u64,
    pub timestamp_validity: u64,
    pub balance_range: u64,
    pub age_rep_membership: u64,
}

impl Default for ReputationThresholds {
    fn default() -> Self {
        Self {
            age_over_18: 10,
            membership: 5,
            membership_proof: 5,
            reputation: 15,
            timestamp_validity: 5,
            balance_range: 5,
            age_rep_membership: 20,
        }
    }
}

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

/// Verify multiple Groth16 proofs with a single verifying key.
/// Returns `Ok(true)` if all proofs succeed.
pub fn verify_batch<'a>(
    vk: &PreparedVerifyingKey<Bn254>,
    batch: &[(&'a Proof<Bn254>, &'a [Fr])],
) -> Result<bool, SynthesisError> {
    batch
        .par_iter()
        .map(|(proof, inputs)| Groth16::<Bn254>::verify_with_processed_vk(vk, inputs, proof))
        .collect::<Result<Vec<_>, _>>()
        .map(|results| results.into_iter().all(|r| r))
}

#[cfg(test)]
mod tests;
