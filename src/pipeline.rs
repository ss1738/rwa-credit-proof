//! End-to-end pipeline on a realistic servicer loan tape (CSV).
//!
//!   loan tape CSV -> servicer signs the book commitment -> ZK proof of solvency+KYC ->
//!   mint-gate (ZK proof AND servicer signature)
//!
//! This is the shape a pilot takes: a servicer exports its book, signs it, and the fund can only mint
//! when the private book provably covers the token supply. Run on a Mini:
//!   cargo run --release --bin pipeline -- examples/loan_tape_solvent.csv
//!
//! CSV columns: loan_id,principal,collateral_value,status(performing|defaulted),kyc_ok(true|false)

use ark_bn254::{Bn254, Fr};
use ark_groth16::Groth16;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystem};
use ark_serialize::CanonicalSerialize;
use ark_snark::SNARK;
use credit_solvency_core::circuit::{commit_book, prove_solvency, SolvencyCircuit};
use ed25519_dalek::{Signer, SigningKey, Verifier};
use rand::rngs::OsRng;
use std::fs;

const RATIO_BPS: u128 = 12_000; // require 120% over-collateralisation

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| "examples/loan_tape_solvent.csv".into());
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"));

    let (mut collateral, mut performing, mut kyc, mut principals) = (vec![], vec![], vec![], vec![]);
    for (i, line) in text.lines().enumerate() {
        if i == 0 || line.trim().is_empty() {
            continue; // header / blank
        }
        let f: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        principals.push(f[1].parse::<u128>().expect("principal"));
        collateral.push(f[2].parse::<u128>().expect("collateral"));
        performing.push(f[3].eq_ignore_ascii_case("performing"));
        kyc.push(matches!(f[4].to_ascii_lowercase().as_str(), "true" | "1" | "yes"));
    }

    let n = collateral.len();
    let supply: u128 = principals.iter().sum();
    let threshold = supply * RATIO_BPS / 10_000;
    let performing_collateral: u128 =
        collateral.iter().zip(&performing).filter(|(_, &p)| p).map(|(&c, _)| c).sum();
    let all_kyc = kyc.iter().all(|&x| x);

    println!("Loan tape: {path}");
    println!("  loans                 : {n}");
    println!("  token supply          : {supply}");
    println!("  required collateral   : {threshold}   (120% of supply)");
    println!("  performing collateral : {performing_collateral}");
    println!("  all borrowers KYC'd   : {all_kyc}");

    // Decide via the ACTUAL circuit constraints, not a cleartext check: build the circuit for this
    // book and test satisfiability. An unsatisfiable circuit means no valid ZK proof can exist, so an
    // insolvent / non-compliant tape genuinely cannot open the gate.
    let commitment = commit_book(&collateral, &performing, &kyc);
    let cs = ConstraintSystem::<Fr>::new_ref();
    SolvencyCircuit {
        collateral: collateral.iter().map(|&x| Some(x)).collect(),
        performing: performing.iter().map(|&x| Some(x)).collect(),
        kyc: kyc.iter().map(|&x| Some(x)).collect(),
        threshold: Some(threshold),
        commitment: Some(commitment),
        n,
    }
    .generate_constraints(cs.clone())
    .unwrap();
    if !cs.is_satisfied().unwrap() {
        println!("  => circuit unsatisfiable for this book: no valid ZK proof can exist");
        println!("  => MINT BLOCKED");
        return;
    }

    // servicer signs the Poseidon commitment to the book
    let mut commit_bytes = Vec::new();
    commitment.serialize_compressed(&mut commit_bytes).unwrap();
    let servicer = SigningKey::generate(&mut OsRng);
    let signature = servicer.sign(&commit_bytes);

    // fund generates the ZK proof over its private book
    let (vk, proof, public) = prove_solvency(&collateral, &performing, &kyc, threshold);
    let mut proof_bytes = Vec::new();
    proof.serialize_compressed(&mut proof_bytes).unwrap();

    // the mint-gate: ZK proof AND servicer signature over the same commitment
    let zk_ok = Groth16::<Bn254>::verify(&vk, &public, &proof).unwrap_or(false);
    let sig_ok = servicer.verifying_key().verify(&commit_bytes, &signature).is_ok();

    println!("  ZK proof ({} bytes)   : {}", proof_bytes.len(), if zk_ok { "valid" } else { "INVALID" });
    println!("  servicer signature    : {}", if sig_ok { "valid" } else { "INVALID" });
    println!("  => MINT {}", if zk_ok && sig_ok { "ALLOWED" } else { "BLOCKED" });
}
