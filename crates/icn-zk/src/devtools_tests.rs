#[cfg(feature = "devtools")]
use super::count_constraints;
use super::{
    AgeOver18Circuit, AgeRepMembershipCircuit, BalanceRangeCircuit, MembershipCircuit,
    MembershipProofCircuit, ReputationCircuit, TimestampValidityCircuit,
};

#[cfg(all(test, feature = "devtools"))]
mod tests {
    use super::*;

    #[test]
    fn age_over_18_constraints() {
        let c = AgeOver18Circuit {
            birth_year: 2000,
            current_year: 2020,
        };
        assert_eq!(count_constraints(c).unwrap(), 1);
    }

    #[test]
    fn membership_constraints() {
        let c = MembershipCircuit { is_member: true };
        assert_eq!(count_constraints(c).unwrap(), 2);
    }

    #[test]
    fn membership_proof_constraints() {
        let c = MembershipProofCircuit {
            membership_flag: true,
            expected: true,
        };
        assert_eq!(count_constraints(c).unwrap(), 3);
    }

    #[test]
    fn reputation_constraints() {
        let c = ReputationCircuit {
            reputation: 10,
            threshold: 5,
        };
        assert_eq!(count_constraints(c).unwrap(), 1);
    }

    #[test]
    fn timestamp_validity_constraints() {
        let c = TimestampValidityCircuit {
            timestamp: 1,
            not_before: 0,
            not_after: 2,
        };
        assert_eq!(count_constraints(c).unwrap(), 2);
    }

    #[test]
    fn balance_range_constraints() {
        let c = BalanceRangeCircuit {
            balance: 10,
            min: 5,
            max: 15,
        };
        assert_eq!(count_constraints(c).unwrap(), 2);
    }

    #[test]
    fn age_rep_membership_constraints() {
        let c = AgeRepMembershipCircuit {
            birth_year: 2000,
            current_year: 2020,
            reputation: 10,
            threshold: 5,
            is_member: true,
        };
        assert_eq!(count_constraints(c).unwrap(), 4);
    }
}
