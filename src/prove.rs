//! First real ZK proof: prove a loan book is solvent while revealing only the public threshold.
//! Run on a Mac Mini:  cargo run --release --bin prove

use ark_bn254::{Bn254, Fr};
use ark_groth16::Groth16;
use ark_serialize::CanonicalSerialize;
use ark_snark::SNARK;
use credit_solvency_core::circuit::SolvencyCircuit;
use rand::{rngs::StdRng, SeedableRng};

fn some_u128(v: &[u128]) -> Vec<Option<u128>> {
    v.iter().map(|&x| Some(x)).collect()
}
fn some_bool(v: &[bool]) -> Vec<Option<bool>> {
    v.iter().map(|&x| Some(x)).collect()
}

fn main() {
    let n = 10usize;
    let collateral: Vec<u128> = (0..n as u128).map(|i| 150_000 + i * 1_000).collect();
    let performing = vec![true; n];
    let total: u128 = collateral.iter().sum();
    let threshold = total - 50_000; // solvent: book covers the claim with margin

    let mut rng = StdRng::seed_from_u64(7);

    // trusted setup for this circuit shape (n loans); witnesses are None here
    let (pk, vk) = Groth16::<Bn254>::circuit_specific_setup(
        SolvencyCircuit { collateral: vec![None; n], performing: vec![None; n], threshold: None, n },
        &mut rng,
    )
    .expect("setup");

    // prove with the real (private) book
    let circuit = SolvencyCircuit {
        collateral: some_u128(&collateral),
        performing: some_bool(&performing),
        threshold: Some(threshold),
        n,
    };
    let proof = Groth16::<Bn254>::prove(&pk, circuit, &mut rng).expect("prove");
    let mut bytes = Vec::new();
    proof.serialize_compressed(&mut bytes).unwrap();

    let ok = Groth16::<Bn254>::verify(&vk, &[Fr::from(threshold)], &proof).expect("verify");

    println!("=== ZK solvency-sum (Groth16 / BN254 = Solana alt_bn128) ===");
    println!("loans in book (PRIVATE)      : {n}");
    println!("proof size                   : {} bytes", bytes.len());
    println!("public threshold             : {threshold}");
    println!("actual collateral sum        : {total}   <- never revealed to the verifier");
    println!("VERIFIED solvent (loans hidden): {ok}");

    // negative control: demand more collateral than the book holds -> must NOT verify
    let bad_threshold = total + 1_000_000;
    let bad = SolvencyCircuit {
        collateral: some_u128(&collateral),
        performing: some_bool(&performing),
        threshold: Some(bad_threshold),
        n,
    };
    let rejected = match Groth16::<Bn254>::prove(&pk, bad, &mut rng) {
        Ok(p) => !Groth16::<Bn254>::verify(&vk, &[Fr::from(bad_threshold)], &p).unwrap_or(false),
        Err(_) => true, // unsatisfiable witness -> no proof at all
    };
    println!("insolvent book correctly rejected: {rejected}");
    assert!(ok && rejected, "solvency proof soundness/completeness check failed");
    println!("\nOK: a fund can prove solvency without revealing a single loan.");
}
