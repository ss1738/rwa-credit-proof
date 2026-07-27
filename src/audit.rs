//! Adversarial self-audit. These checks FAIL loudly if the ZK claims are fabricated:
//! an under-constrained circuit, a vacuous predicate, or a verifier that is a no-op.
//! Run on a Mini:  cargo run --release --bin audit

use ark_bn254::{Bn254, Fr};
use ark_groth16::{Groth16, Proof};
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystem};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_snark::SNARK;
use credit_solvency_core::circuit::{commit_book, prove_solvency, SolvencyCircuit};

fn ou(v: &[u128]) -> Vec<Option<u128>> { v.iter().map(|&x| Some(x)).collect() }
fn ob(v: &[bool]) -> Vec<Option<bool>> { v.iter().map(|&x| Some(x)).collect() }

/// Build the circuit with a full witness and report (satisfied?, constraint_count).
fn check(collateral: &[u128], performing: &[bool], kyc: &[bool], threshold: u128, commitment: Fr) -> (bool, usize) {
    let n = collateral.len();
    let c = SolvencyCircuit {
        collateral: ou(collateral), performing: ob(performing), kyc: ob(kyc),
        threshold: Some(threshold), commitment: Some(commitment), n,
    };
    let cs = ConstraintSystem::<Fr>::new_ref();
    c.generate_constraints(cs.clone()).unwrap();
    (cs.is_satisfied().unwrap(), cs.num_constraints())
}

fn main() {
    let n = 10usize;
    let collateral: Vec<u128> = (0..n as u128).map(|i| 150_000 + i * 1_000).collect();
    let performing = vec![true; n];
    let kyc = vec![true; n];
    let sum: u128 = collateral.iter().sum();
    let commitment = commit_book(&collateral, &performing, &kyc);

    println!("== Is the circuit real, or a vacuous no-op? ==");
    let (ok1, nc) = check(&collateral, &performing, &kyc, sum - 50_000, commitment);
    println!("constraints in the circuit : {nc}   (a vacuous circuit would be ~0)");
    println!("[1] solvent + KYC + correct commitment -> satisfied : {ok1}   expect TRUE");

    let (ok2, _) = check(&collateral, &performing, &kyc, sum + 1, commitment);
    println!("[2] insolvent (threshold = collateral + 1) -> satisfied : {ok2}   expect FALSE");

    let mut bad_kyc = kyc.clone(); bad_kyc[3] = false;
    let (ok3, _) = check(&collateral, &performing, &bad_kyc, sum - 50_000, commit_book(&collateral, &performing, &bad_kyc));
    println!("[3] one borrower fails KYC -> satisfied : {ok3}   expect FALSE");

    let (ok4, _) = check(&collateral, &performing, &kyc, sum - 50_000, commitment + Fr::from(1u64));
    println!("[4] book with a MISMATCHED commitment -> satisfied : {ok4}   expect FALSE");

    let (ok5, _) = check(&collateral, &performing, &kyc, sum, commitment);
    println!("[5] boundary: threshold == collateral -> satisfied : {ok5}   expect TRUE (>= inclusive)");

    println!("\n== Can a forged proof or altered claim pass verification? ==");
    let (vk, proof, public) = prove_solvency(&collateral, &performing, &kyc, sum - 50_000);
    println!("[6] honest proof verifies : {}   expect TRUE", Groth16::<Bn254>::verify(&vk, &public, &proof).unwrap());

    let mut pb = Vec::new(); proof.serialize_compressed(&mut pb).unwrap();
    pb[0] ^= 0x01; // flip one bit
    let forged_ok = match Proof::<Bn254>::deserialize_compressed(&pb[..]) {
        Ok(fp) => Groth16::<Bn254>::verify(&vk, &public, &fp).unwrap_or(false),
        Err(_) => false, // rejected at deserialization is also a rejection
    };
    println!("[7] proof with one flipped bit verifies : {forged_ok}   expect FALSE");

    let altered = vec![public[0] + Fr::from(1u64), public[1]];
    println!("[8] honest proof vs altered public input verifies : {}   expect FALSE",
        Groth16::<Bn254>::verify(&vk, &altered, &proof).unwrap_or(false));

    println!("\n== Privacy ==");
    println!("[9] public inputs = {} field elements (threshold + commitment only); {} loans are NOT public",
        public.len(), n);
    println!("[10] proof is {} bytes, constant; it cannot encode {} loan records", pb.len(), n);

    let pass = ok1 && !ok2 && !ok3 && !ok4 && ok5
        && Groth16::<Bn254>::verify(&vk, &public, &proof).unwrap()
        && !forged_ok
        && !Groth16::<Bn254>::verify(&vk, &altered, &proof).unwrap_or(false)
        && public.len() == 2;
    println!("\n{}", if pass {
        "AUDIT PASSED (honest-prover): constraints are real, verifier is not a no-op, privacy holds.\n\
         CAVEAT: this only exercises HONESTLY-generated proofs. The Groth16 setup uses a fixed public\n\
         seed, so it is NOT sound against a malicious prover who knows it. See KNOWN_LIMITATIONS.md #1."
    } else { "AUDIT FAILED: a claim did not hold. Investigate." });
    assert!(pass);
}
