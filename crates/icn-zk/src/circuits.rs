use ark_bn254::Fr;
use ark_r1cs_std::fields::fp::FpVar;
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};

/// Prove that `current_year >= birth_year + 18`.
#[derive(Clone)]
pub struct AgeOver18Circuit {
    /// Birth year of the subject (private).
    pub birth_year: u64,
    /// Current year (public).
    pub current_year: u64,
}

impl ConstraintSynthesizer<Fr> for AgeOver18Circuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        let birth = FpVar::<Fr>::new_witness(cs.clone(), || Ok(Fr::from(self.birth_year)))?;
        let current = FpVar::<Fr>::new_input(cs.clone(), || Ok(Fr::from(self.current_year)))?;

        // k = current_year - birth_year - 18
        let diff = self
            .current_year
            .checked_sub(self.birth_year + 18)
            .ok_or(SynthesisError::AssignmentMissing)?;
        let k = FpVar::<Fr>::new_witness(cs, || Ok(Fr::from(diff)))?;

        let eighteen = FpVar::<Fr>::Constant(Fr::from(18u64));
        (birth + eighteen + k).enforce_equal(&current)?;
        Ok(())
    }
}

/// Prove knowledge of membership boolean (must equal `true`).
#[derive(Clone)]
pub struct MembershipCircuit {
    /// Whether the prover is a member (public).
    pub is_member: bool,
}

impl ConstraintSynthesizer<Fr> for MembershipCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        let member = Boolean::new_input(cs, || Ok(self.is_member))?;
        member.enforce_equal(&Boolean::TRUE)?;
        Ok(())
    }
}

/// Prove that `reputation >= threshold`.
#[derive(Clone)]
pub struct ReputationCircuit {
    /// Reputation score (public).
    pub reputation: u64,
    /// Required threshold.
    pub threshold: u64,
}

impl ConstraintSynthesizer<Fr> for ReputationCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        let rep = FpVar::<Fr>::new_input(cs.clone(), || Ok(Fr::from(self.reputation)))?;
        let diff = self
            .reputation
            .checked_sub(self.threshold)
            .ok_or(SynthesisError::AssignmentMissing)?;
        let k = FpVar::<Fr>::new_witness(cs.clone(), || Ok(Fr::from(diff)))?;
        let threshold = FpVar::<Fr>::Constant(Fr::from(self.threshold));
        (threshold + k).enforce_equal(&rep)?;
        Ok(())
    }
}

/// Prove that `balance` lies within the inclusive range `[min, max]`.
#[derive(Clone)]
pub struct BalanceRangeCircuit {
    /// Balance to prove knowledge of (private).
    pub balance: u64,
    /// Minimum acceptable balance (public).
    pub min: u64,
    /// Maximum acceptable balance (public).
    pub max: u64,
}

impl ConstraintSynthesizer<Fr> for BalanceRangeCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        let balance = FpVar::<Fr>::new_witness(cs.clone(), || Ok(Fr::from(self.balance)))?;
        let min = FpVar::<Fr>::new_input(cs.clone(), || Ok(Fr::from(self.min)))?;
        let max = FpVar::<Fr>::new_input(cs.clone(), || Ok(Fr::from(self.max)))?;

        let diff_min_val = self
            .balance
            .checked_sub(self.min)
            .ok_or(SynthesisError::AssignmentMissing)?;
        let diff_min = FpVar::<Fr>::new_witness(cs.clone(), || Ok(Fr::from(diff_min_val)))?;

        let diff_max_val = self
            .max
            .checked_sub(self.balance)
            .ok_or(SynthesisError::AssignmentMissing)?;
        let diff_max = FpVar::<Fr>::new_witness(cs, || Ok(Fr::from(diff_max_val)))?;

        (min + diff_min.clone()).enforce_equal(&balance)?;
        (balance + diff_max).enforce_equal(&max)?;
        Ok(())
    }
}
