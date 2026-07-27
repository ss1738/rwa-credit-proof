//! Scale test: how does proving cost grow with the loan book size?
//! Answers the architecture question: is monolithic Groth16 enough, or do we need Nova folding?
//! Run on a Mini (capture peak RAM):  /usr/bin/time -l ./target/release/bench 100 1000 10000

use ark_bn254::{Bn254, Fr};
use ark_groth16::Groth16;
use ark_serialize::CanonicalSerialize;
use ark_snark::SNARK;
use credit_solvency_core::circuit::{commit_book, SolvencyCircuit};
use rand::{rngs::StdRng, SeedableRng};
use std::time::Instant;

fn bench(n: usize) {
    // bounded collateral values so the sum stays well inside the field
    let collateral: Vec<u128> = (0..n as u128).map(|i| 150_000 + (i % 100) * 1_000).collect();
    let performing = vec![true; n];
    let kyc = vec![true; n];
    let total: u128 = collateral.iter().sum();
    let threshold = total - 50_000;
    let commitment = commit_book(&collateral, &performing, &kyc);

    let mut rng = StdRng::seed_from_u64(7);
    let shape = SolvencyCircuit {
        collateral: vec![None; n],
        performing: vec![None; n],
        kyc: vec![None; n],
        threshold: None,
        commitment: None,
        n,
    };

    let t = Instant::now();
    let (pk, vk) = Groth16::<Bn254>::circuit_specific_setup(shape, &mut rng).expect("setup");
    let setup_s = t.elapsed().as_secs_f64();

    let circuit = SolvencyCircuit {
        collateral: collateral.iter().map(|&x| Some(x)).collect(),
        performing: performing.iter().map(|&x| Some(x)).collect(),
        kyc: kyc.iter().map(|&x| Some(x)).collect(),
        threshold: Some(threshold),
        commitment: Some(commitment),
        n,
    };
    let t = Instant::now();
    let proof = Groth16::<Bn254>::prove(&pk, circuit, &mut rng).expect("prove");
    let prove_s = t.elapsed().as_secs_f64();

    let t = Instant::now();
    let ok = Groth16::<Bn254>::verify(&vk, &[Fr::from(threshold), commitment], &proof).expect("verify");
    let verify_ms = t.elapsed().as_secs_f64() * 1000.0;

    let mut pb = Vec::new();
    proof.serialize_compressed(&mut pb).unwrap();
    println!(
        "n={:>6}  setup={:>8.2}s  prove={:>8.2}s  verify={:>6.1}ms  proof={}B  verified={}",
        n, setup_s, prove_s, verify_ms, pb.len(), ok
    );
}

fn main() {
    let args: Vec<usize> = std::env::args().skip(1).filter_map(|a| a.parse().ok()).collect();
    let sizes = if args.is_empty() { vec![100, 1000, 10000] } else { args };
    for n in sizes {
        bench(n);
    }
}
