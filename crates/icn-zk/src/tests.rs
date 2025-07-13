use super::*;
use ark_std::rand::{rngs::StdRng, SeedableRng};

#[test]
fn age_over_18_proof() {
    let circuit = AgeOver18Circuit {
        birth_year: 2000,
        current_year: 2020,
    };
    let mut rng = StdRng::seed_from_u64(42);
    let pk = setup(circuit.clone(), &mut rng).unwrap();
    let proof = prove(&pk, circuit, &mut rng).unwrap();
    let vk = prepare_vk(&pk);
    assert!(verify(&vk, &proof, &[Fr::from(2020u64)]).unwrap());
}

#[test]
fn membership_proof() {
    let circuit = MembershipCircuit { is_member: true };
    let mut rng = StdRng::seed_from_u64(42);
    let pk = setup(circuit.clone(), &mut rng).unwrap();
    let proof = prove(&pk, circuit, &mut rng).unwrap();
    let vk = prepare_vk(&pk);
    assert!(verify(&vk, &proof, &[Fr::from(1u64)]).unwrap());
}

#[test]
fn reputation_threshold_proof() {
    let circuit = ReputationCircuit {
        reputation: 10,
        threshold: 5,
    };
    let mut rng = StdRng::seed_from_u64(42);
    let pk = setup(circuit.clone(), &mut rng).unwrap();
    let proof = prove(&pk, circuit, &mut rng).unwrap();
    let vk = prepare_vk(&pk);
    assert!(verify(&vk, &proof, &[Fr::from(10u64)]).unwrap());
}

#[test]
fn balance_range_proof() {
    let circuit = BalanceRangeCircuit {
        balance: 50,
        min: 10,
        max: 100,
    };
    let mut rng = StdRng::seed_from_u64(42);
    let pk = setup(circuit.clone(), &mut rng).unwrap();
    let proof = prove(&pk, circuit, &mut rng).unwrap();
    let vk = prepare_vk(&pk);
    assert!(verify(&vk, &proof, &[Fr::from(10u64), Fr::from(100u64)]).unwrap());
}
