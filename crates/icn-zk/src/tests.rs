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
fn membership_flag_proof() {
    let circuit = MembershipProofCircuit {
        membership_flag: true,
        expected: true,
    };
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
fn circuit_parameters_roundtrip() {
    let circuit = AgeOver18Circuit {
        birth_year: 2000,
        current_year: 2020,
    };
    let mut rng = StdRng::seed_from_u64(42);
    let pk = setup(circuit.clone(), &mut rng).unwrap();
    let params = CircuitParameters::from_proving_key(&pk).unwrap();
    let mut store = MemoryParametersStorage::default();
    store.put("age_over_18", params.clone());
    let fetched = store.get("age_over_18").unwrap();
    let pk2 = fetched.proving_key().unwrap();
    let proof = prove(&pk2, circuit, &mut rng).unwrap();
    let vk = fetched.prepared_vk().unwrap();
    assert!(verify(&vk, &proof, &[Fr::from(2020u64)]).unwrap());
}

#[test]
fn timestamp_validity_proof() {
    let circuit = TimestampValidityCircuit {
        timestamp: 1_650_000_000,
        not_before: 1_600_000_000,
        not_after: 1_700_000_000,
    };
    let mut rng = StdRng::seed_from_u64(42);
    let pk = setup(circuit.clone(), &mut rng).unwrap();
    let proof = prove(&pk, circuit, &mut rng).unwrap();
    let vk = prepare_vk(&pk);
    assert!(verify(
        &vk,
        &proof,
        &[Fr::from(1_600_000_000u64), Fr::from(1_700_000_000u64)]
    )
    .unwrap());
}

#[test]
fn balance_range_proof() {
    let circuit = BalanceRangeCircuit {
        balance: 75,
        min: 50,
        max: 100,
    };
    let mut rng = StdRng::seed_from_u64(42);
    let pk = setup(circuit.clone(), &mut rng).unwrap();
    let proof = prove(&pk, circuit, &mut rng).unwrap();
    let vk = prepare_vk(&pk);
    assert!(verify(&vk, &proof, &[Fr::from(50u64), Fr::from(100u64)]).unwrap());
}

#[test]
fn age_rep_membership_proof() {
    let circuit = AgeRepMembershipCircuit {
        birth_year: 2000,
        current_year: 2020,
        reputation: 10,
        threshold: 5,
        is_member: true,
    };
    let mut rng = StdRng::seed_from_u64(42);
    let pk = setup(circuit.clone(), &mut rng).unwrap();
    let proof = prove(&pk, circuit, &mut rng).unwrap();
    let vk = prepare_vk(&pk);
    assert!(verify(
        &vk,
        &proof,
        &[Fr::from(2020u64), Fr::from(10u64), Fr::from(1u64)]
    )
    .unwrap());
}

#[test]
fn batch_verify_multiple_proofs() {
    let circuit1 = AgeOver18Circuit {
        birth_year: 1990,
        current_year: 2020,
    };
    let circuit2 = AgeOver18Circuit {
        birth_year: 1995,
        current_year: 2020,
    };
    let mut rng = StdRng::seed_from_u64(42);
    let pk = setup(circuit1.clone(), &mut rng).unwrap();
    let proof1 = prove(&pk, circuit1, &mut rng).unwrap();
    let proof2 = prove(&pk, circuit2, &mut rng).unwrap();
    let vk = prepare_vk(&pk);
    let inputs1 = [Fr::from(2020u64)];
    let inputs2 = [Fr::from(2020u64)];
    let batch = [(&proof1, &inputs1[..]), (&proof2, &inputs2[..])];
    assert!(verify_batch(&vk, &batch).unwrap());
}

#[test]
fn batch_verify_detects_invalid() {
    let circuit = AgeOver18Circuit {
        birth_year: 2000,
        current_year: 2020,
    };
    let mut rng = StdRng::seed_from_u64(42);
    let pk = setup(circuit.clone(), &mut rng).unwrap();
    let valid = prove(&pk, circuit.clone(), &mut rng).unwrap();
    let invalid = prove(&pk, circuit, &mut rng).unwrap();
    let vk = prepare_vk(&pk);
    let inputs1 = [Fr::from(2020u64)];
    let inputs2 = [Fr::from(2019u64)];
    let batch = [(&valid, &inputs1[..]), (&invalid, &inputs2[..])];
    assert!(!verify_batch(&vk, &batch).unwrap());
}
