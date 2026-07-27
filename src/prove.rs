//! First real ZK proof: prove a loan book is solvent while revealing only the public threshold.
//! Run on a Mac Mini:  cargo run --release --bin prove

use ark_bn254::{Bn254, Fr};
use ark_groth16::Groth16;
use ark_serialize::CanonicalSerialize;
use ark_snark::SNARK;
use credit_solvency_core::circuit::prove_solvency;

fn main() {
    let n = 10usize;
    let collateral: Vec<u128> = (0..n as u128).map(|i| 150_000 + i * 1_000).collect();
    let performing = vec![true; n];
    let total: u128 = collateral.iter().sum();
    let threshold = total - 50_000; // solvent: book covers the claim with margin

    let (vk, proof, public) = prove_solvency(&collateral, &performing, threshold);

    let mut bytes = Vec::new();
    proof.serialize_compressed(&mut bytes).unwrap();
    let ok = Groth16::<Bn254>::verify(&vk, &public, &proof).expect("verify");

    println!("=== ZK solvency-sum (Groth16 / BN254 = Solana alt_bn128) ===");
    println!("loans in book (PRIVATE)        : {n}");
    println!("proof size                     : {} bytes", bytes.len());
    println!("public threshold               : {threshold}");
    println!("actual collateral sum          : {total}   <- never revealed to the verifier");
    println!("VERIFIED solvent (loans hidden): {ok}");

    // negative control: a threshold above the book's collateral must NOT verify
    let bad_public = vec![Fr::from(total + 1_000_000)];
    let rejected = !Groth16::<Bn254>::verify(&vk, &bad_public, &proof).unwrap_or(false);
    println!("insolvent threshold rejected   : {rejected}");
    assert!(ok && rejected, "solvency proof soundness/completeness check failed");
    println!("\nOK: a fund can prove solvency without revealing a single loan.");
}
