use ark_bn254::Fr;
use ark_std::rand::{rngs::StdRng, SeedableRng};
use criterion::{criterion_group, criterion_main, Criterion};
use icn_zk::{
    prove, prepare_vk, setup, verify,
    AgeOver18Circuit, AgeRepMembershipCircuit, BalanceRangeCircuit,
    MembershipCircuit, MembershipProofCircuit, ReputationCircuit,
    TimestampValidityCircuit,
};

fn bench_age_over_18(c: &mut Criterion) {
    let circuit = AgeOver18Circuit { birth_year: 2000, current_year: 2020 };
    let mut rng = StdRng::seed_from_u64(42);
    let pk = setup(circuit.clone(), &mut rng).unwrap();
    let vk = prepare_vk(&pk);
    c.bench_function("prove_age_over_18", |b| {
        b.iter(|| {
            let mut r = StdRng::seed_from_u64(42);
            prove(&pk, circuit.clone(), &mut r).unwrap();
        });
    });
    let mut rng2 = StdRng::seed_from_u64(42);
    let proof = prove(&pk, circuit.clone(), &mut rng2).unwrap();
    let inputs = [Fr::from(2020u64)];
    c.bench_function("verify_age_over_18", |b| {
        b.iter(|| verify(&vk, &proof, &inputs).unwrap());
    });
}

fn bench_membership(c: &mut Criterion) {
    let circuit = MembershipCircuit { is_member: true };
    let mut rng = StdRng::seed_from_u64(42);
    let pk = setup(circuit.clone(), &mut rng).unwrap();
    let vk = prepare_vk(&pk);
    c.bench_function("prove_membership", |b| {
        b.iter(|| {
            let mut r = StdRng::seed_from_u64(42);
            prove(&pk, circuit.clone(), &mut r).unwrap();
        });
    });
    let mut rng2 = StdRng::seed_from_u64(42);
    let proof = prove(&pk, circuit.clone(), &mut rng2).unwrap();
    let inputs = [Fr::from(1u64)];
    c.bench_function("verify_membership", |b| {
        b.iter(|| verify(&vk, &proof, &inputs).unwrap());
    });
}

fn bench_membership_proof(c: &mut Criterion) {
    let circuit = MembershipProofCircuit { membership_flag: true, expected: true };
    let mut rng = StdRng::seed_from_u64(42);
    let pk = setup(circuit.clone(), &mut rng).unwrap();
    let vk = prepare_vk(&pk);
    c.bench_function("prove_membership_proof", |b| {
        b.iter(|| {
            let mut r = StdRng::seed_from_u64(42);
            prove(&pk, circuit.clone(), &mut r).unwrap();
        });
    });
    let mut rng2 = StdRng::seed_from_u64(42);
    let proof = prove(&pk, circuit.clone(), &mut rng2).unwrap();
    let inputs = [Fr::from(1u64)];
    c.bench_function("verify_membership_proof", |b| {
        b.iter(|| verify(&vk, &proof, &inputs).unwrap());
    });
}

fn bench_reputation(c: &mut Criterion) {
    let circuit = ReputationCircuit { reputation: 10, threshold: 5 };
    let mut rng = StdRng::seed_from_u64(42);
    let pk = setup(circuit.clone(), &mut rng).unwrap();
    let vk = prepare_vk(&pk);
    c.bench_function("prove_reputation", |b| {
        b.iter(|| {
            let mut r = StdRng::seed_from_u64(42);
            prove(&pk, circuit.clone(), &mut r).unwrap();
        });
    });
    let mut rng2 = StdRng::seed_from_u64(42);
    let proof = prove(&pk, circuit.clone(), &mut rng2).unwrap();
    let inputs = [Fr::from(10u64)];
    c.bench_function("verify_reputation", |b| {
        b.iter(|| verify(&vk, &proof, &inputs).unwrap());
    });
}

fn bench_timestamp_validity(c: &mut Criterion) {
    let circuit = TimestampValidityCircuit {
        timestamp: 1_650_000_000,
        not_before: 1_600_000_000,
        not_after: 1_700_000_000,
    };
    let mut rng = StdRng::seed_from_u64(42);
    let pk = setup(circuit.clone(), &mut rng).unwrap();
    let vk = prepare_vk(&pk);
    c.bench_function("prove_timestamp_validity", |b| {
        b.iter(|| {
            let mut r = StdRng::seed_from_u64(42);
            prove(&pk, circuit.clone(), &mut r).unwrap();
        });
    });
    let mut rng2 = StdRng::seed_from_u64(42);
    let proof = prove(&pk, circuit.clone(), &mut rng2).unwrap();
    let inputs = [Fr::from(1_600_000_000u64), Fr::from(1_700_000_000u64)];
    c.bench_function("verify_timestamp_validity", |b| {
        b.iter(|| verify(&vk, &proof, &inputs).unwrap());
    });
}

fn bench_balance_range(c: &mut Criterion) {
    let circuit = BalanceRangeCircuit { balance: 75, min: 50, max: 100 };
    let mut rng = StdRng::seed_from_u64(42);
    let pk = setup(circuit.clone(), &mut rng).unwrap();
    let vk = prepare_vk(&pk);
    c.bench_function("prove_balance_range", |b| {
        b.iter(|| {
            let mut r = StdRng::seed_from_u64(42);
            prove(&pk, circuit.clone(), &mut r).unwrap();
        });
    });
    let mut rng2 = StdRng::seed_from_u64(42);
    let proof = prove(&pk, circuit.clone(), &mut rng2).unwrap();
    let inputs = [Fr::from(50u64), Fr::from(100u64)];
    c.bench_function("verify_balance_range", |b| {
        b.iter(|| verify(&vk, &proof, &inputs).unwrap());
    });
}

fn bench_age_rep_membership(c: &mut Criterion) {
    let circuit = AgeRepMembershipCircuit {
        birth_year: 2000,
        current_year: 2020,
        reputation: 10,
        threshold: 5,
        is_member: true,
    };
    let mut rng = StdRng::seed_from_u64(42);
    let pk = setup(circuit.clone(), &mut rng).unwrap();
    let vk = prepare_vk(&pk);
    c.bench_function("prove_age_rep_membership", |b| {
        b.iter(|| {
            let mut r = StdRng::seed_from_u64(42);
            prove(&pk, circuit.clone(), &mut r).unwrap();
        });
    });
    let mut rng2 = StdRng::seed_from_u64(42);
    let proof = prove(&pk, circuit.clone(), &mut rng2).unwrap();
    let inputs = [Fr::from(2020u64), Fr::from(10u64), Fr::from(1u64)];
    c.bench_function("verify_age_rep_membership", |b| {
        b.iter(|| verify(&vk, &proof, &inputs).unwrap());
    });
}

criterion_group!(benches,
    bench_age_over_18,
    bench_membership,
    bench_membership_proof,
    bench_reputation,
    bench_timestamp_validity,
    bench_balance_range,
    bench_age_rep_membership,
);
criterion_main!(benches);

