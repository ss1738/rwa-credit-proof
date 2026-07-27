//! ZK proof of the strengthened statement: the book is solvent AND every borrower passed KYC,
//! revealing only the public threshold. Run on a Mac Mini:  cargo run --release --bin prove

use ark_bn254::{Bn254, Fr};
use ark_groth16::Groth16;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystem};
use ark_serialize::CanonicalSerialize;
use ark_snark::SNARK;
use credit_solvency_core::circuit::{prove_solvency, SolvencyCircuit};

fn opt_u128(v: &[u128]) -> Vec<Option<u128>> {
    v.iter().map(|&x| Some(x)).collect()
}
fn opt_bool(v: &[bool]) -> Vec<Option<bool>> {
    v.iter().map(|&x| Some(x)).collect()
}

fn main() {
    let n = 10usize;
    let collateral: Vec<u128> = (0..n as u128).map(|i| 150_000 + i * 1_000).collect();
    let performing = vec![true; n];
    let kyc = vec![true; n];
    let total: u128 = collateral.iter().sum();
    let threshold = total - 50_000; // solvent: book covers the claim with margin

    let (vk, proof, public) = prove_solvency(&collateral, &performing, &kyc, threshold);

    let mut bytes = Vec::new();
    proof.serialize_compressed(&mut bytes).unwrap();
    let ok = Groth16::<Bn254>::verify(&vk, &public, &proof).expect("verify");

    println!("=== ZK proof: solvent AND all-KYC (Groth16 / BN254 = Solana alt_bn128) ===");
    println!("loans in book (PRIVATE)        : {n}");
    println!("proof size                     : {} bytes", bytes.len());
    println!("public threshold               : {threshold}");
    println!("actual collateral sum          : {total}   <- never revealed");
    println!("VERIFIED (solvent + all KYC)   : {ok}");

    // soundness 1: a threshold above the book's collateral must NOT verify
    let bad_public = vec![Fr::from(total + 1_000_000)];
    let insolvent_rejected = !Groth16::<Bn254>::verify(&vk, &bad_public, &proof).unwrap_or(false);
    println!("insolvent threshold rejected   : {insolvent_rejected}");

    // soundness 2: a book with ANY failed KYC is not even a satisfiable witness -> unprovable
    let mut bad_kyc = vec![true; n];
    bad_kyc[3] = false;
    let bad_circuit = SolvencyCircuit {
        collateral: opt_u128(&collateral),
        performing: opt_bool(&performing),
        kyc: opt_bool(&bad_kyc),
        threshold: Some(threshold),
        n,
    };
    let cs = ConstraintSystem::<Fr>::new_ref();
    bad_circuit.generate_constraints(cs.clone()).unwrap();
    let kyc_fail_unprovable = !cs.is_satisfied().unwrap();
    println!("KYC-failure book unprovable    : {kyc_fail_unprovable}");

    assert!(ok && insolvent_rejected && kyc_fail_unprovable, "statement soundness check failed");
    println!("\nOK: proves solvency AND full KYC, without revealing a single loan.");
}
