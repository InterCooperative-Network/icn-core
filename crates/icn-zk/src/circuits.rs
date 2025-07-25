use ark_bn254::Fr;
use ark_r1cs_std::fields::fp::FpVar;
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};

/// Trait for estimating relative circuit complexity.
pub trait CircuitCost {
    /// Returns a complexity score used for mana cost calculation.
    fn complexity() -> u64;
}

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

impl CircuitCost for AgeOver18Circuit {
    fn complexity() -> u64 {
        10
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

impl CircuitCost for MembershipCircuit {
    fn complexity() -> u64 {
        5
    }
}

/// Prove that a private membership flag matches an expected public value.
#[derive(Clone)]
pub struct MembershipProofCircuit {
    /// Membership flag provided by the prover (private).
    pub membership_flag: bool,
    /// Expected membership value published by the verifier (public).
    pub expected: bool,
}

impl ConstraintSynthesizer<Fr> for MembershipProofCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        let flag = Boolean::new_witness(cs.clone(), || Ok(self.membership_flag))?;
        let expected = Boolean::new_input(cs, || Ok(self.expected))?;
        flag.enforce_equal(&expected)?;
        Ok(())
    }
}

impl CircuitCost for MembershipProofCircuit {
    fn complexity() -> u64 {
        5
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

impl CircuitCost for ReputationCircuit {
    fn complexity() -> u64 {
        15
    }
}

/// Prove that `not_before ≤ timestamp ≤ not_after`.
#[derive(Clone)]
pub struct TimestampValidityCircuit {
    /// Timestamp to validate (private).
    pub timestamp: u64,
    /// Earliest acceptable timestamp (public).
    pub not_before: u64,
    /// Latest acceptable timestamp (public).
    pub not_after: u64,
}

impl ConstraintSynthesizer<Fr> for TimestampValidityCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        let ts = FpVar::<Fr>::new_witness(cs.clone(), || Ok(Fr::from(self.timestamp)))?;
        let nb = FpVar::<Fr>::new_input(cs.clone(), || Ok(Fr::from(self.not_before)))?;
        let na = FpVar::<Fr>::new_input(cs.clone(), || Ok(Fr::from(self.not_after)))?;

        // k1 = timestamp - not_before
        let diff1 = self
            .timestamp
            .checked_sub(self.not_before)
            .ok_or(SynthesisError::AssignmentMissing)?;
        let k1 = FpVar::<Fr>::new_witness(cs.clone(), || Ok(Fr::from(diff1)))?;
        (nb + k1.clone()).enforce_equal(&ts)?;

        // k2 = not_after - timestamp
        let diff2 = self
            .not_after
            .checked_sub(self.timestamp)
            .ok_or(SynthesisError::AssignmentMissing)?;
        let k2 = FpVar::<Fr>::new_witness(cs, || Ok(Fr::from(diff2)))?;
        (ts + k2).enforce_equal(&na)?;

        Ok(())
    }
}

impl CircuitCost for TimestampValidityCircuit {
    fn complexity() -> u64 {
        5
    }
}

/// Prove that `min ≤ balance ≤ max`.
#[derive(Clone)]
pub struct BalanceRangeCircuit {
    /// Balance amount to validate (private).
    pub balance: u64,
    /// Minimum acceptable balance (public).
    pub min: u64,
    /// Maximum acceptable balance (public).
    pub max: u64,
}

impl ConstraintSynthesizer<Fr> for BalanceRangeCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        let bal = FpVar::<Fr>::new_witness(cs.clone(), || Ok(Fr::from(self.balance)))?;
        let min = FpVar::<Fr>::new_input(cs.clone(), || Ok(Fr::from(self.min)))?;
        let max = FpVar::<Fr>::new_input(cs.clone(), || Ok(Fr::from(self.max)))?;

        // diff_min = balance - min
        let diff_min_val = self
            .balance
            .checked_sub(self.min)
            .ok_or(SynthesisError::AssignmentMissing)?;
        let diff_min = FpVar::<Fr>::new_witness(cs.clone(), || Ok(Fr::from(diff_min_val)))?;
        (min.clone() + diff_min.clone()).enforce_equal(&bal)?;

        // diff_max = max - balance
        let diff_max_val = self
            .max
            .checked_sub(self.balance)
            .ok_or(SynthesisError::AssignmentMissing)?;
        let diff_max = FpVar::<Fr>::new_witness(cs, || Ok(Fr::from(diff_max_val)))?;
        (bal + diff_max).enforce_equal(&max)?;

        Ok(())
    }
}

impl CircuitCost for BalanceRangeCircuit {
    fn complexity() -> u64 {
        5
    }
}

/// Prove age over 18, membership status, and reputation threshold simultaneously.
#[derive(Clone)]
pub struct AgeRepMembershipCircuit {
    /// Birth year of the subject (private).
    pub birth_year: u64,
    /// Current year (public).
    pub current_year: u64,
    /// Reputation score (public).
    pub reputation: u64,
    /// Required reputation threshold.
    pub threshold: u64,
    /// Membership flag (public).
    pub is_member: bool,
}

impl ConstraintSynthesizer<Fr> for AgeRepMembershipCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        // Age over 18 constraint
        let birth = FpVar::<Fr>::new_witness(cs.clone(), || Ok(Fr::from(self.birth_year)))?;
        let current = FpVar::<Fr>::new_input(cs.clone(), || Ok(Fr::from(self.current_year)))?;
        let diff_age = self
            .current_year
            .checked_sub(self.birth_year + 18)
            .ok_or(SynthesisError::AssignmentMissing)?;
        let k_age = FpVar::<Fr>::new_witness(cs.clone(), || Ok(Fr::from(diff_age)))?;
        let eighteen = FpVar::<Fr>::Constant(Fr::from(18u64));
        (birth + eighteen + k_age).enforce_equal(&current)?;

        // Reputation threshold constraint
        let rep = FpVar::<Fr>::new_input(cs.clone(), || Ok(Fr::from(self.reputation)))?;
        let diff_rep = self
            .reputation
            .checked_sub(self.threshold)
            .ok_or(SynthesisError::AssignmentMissing)?;
        let k_rep = FpVar::<Fr>::new_witness(cs.clone(), || Ok(Fr::from(diff_rep)))?;
        let threshold = FpVar::<Fr>::Constant(Fr::from(self.threshold));
        (threshold + k_rep).enforce_equal(&rep)?;

        // Membership constraint
        let member = Boolean::new_input(cs, || Ok(self.is_member))?;
        member.enforce_equal(&Boolean::TRUE)?;

        Ok(())
    }
}

impl CircuitCost for AgeRepMembershipCircuit {
    fn complexity() -> u64 {
        20
    }
}
