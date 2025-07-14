use super::*;
use crate::devtools::{count_constraints, log_constraints};

#[test]
fn constraint_counts() {
    assert_eq!(
        count_constraints(AgeOver18Circuit {
            birth_year: 2000,
            current_year: 2024,
        })
        .unwrap(),
        1
    );
    assert_eq!(
        count_constraints(MembershipCircuit { is_member: true }).unwrap(),
        2
    );
    assert_eq!(
        count_constraints(MembershipProofCircuit {
            membership_flag: true,
            expected: true,
        })
        .unwrap(),
        3
    );
    assert_eq!(
        count_constraints(ReputationCircuit {
            reputation: 10,
            threshold: 5,
        })
        .unwrap(),
        1
    );
    assert_eq!(
        count_constraints(TimestampValidityCircuit {
            timestamp: 0,
            not_before: 0,
            not_after: 1,
        })
        .unwrap(),
        2
    );
    assert_eq!(
        count_constraints(BalanceRangeCircuit {
            balance: 10,
            min: 0,
            max: 100,
        })
        .unwrap(),
        2
    );
    assert_eq!(
        count_constraints(AgeRepMembershipCircuit {
            birth_year: 2000,
            current_year: 2024,
            reputation: 10,
            threshold: 5,
            is_member: true,
        })
        .unwrap(),
        4
    );

    // Ensure log_constraints executes without error
    log_constraints(AgeOver18Circuit {
        birth_year: 2000,
        current_year: 2024,
    })
    .unwrap();
}
