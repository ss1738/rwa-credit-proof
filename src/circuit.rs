//! Zero-knowledge solvency statement, bound to a specific book.
//!
//! Proves, over a PRIVATE loan book:
//!   1. Σ (performing collateral) ≥ public `threshold`
//!   2. every borrower passed KYC
//!   3. the book hashes (Poseidon) to the public `commitment`
//! revealing only `threshold` and `commitment`.
//!
//! (3) is the binding: the servicer signs `commitment` (checked on-chain with Solana's cheap ed25519
//! syscall), and the ZK proof proves the solvent+KYC'd book is exactly the one that hashes to it. So a
//! prover cannot swap in a different book — the whole answer to the garbage-in problem, end to end.
//!
//! Groth16 over BN254 so the pairing verifies via Solana's alt_bn128 syscalls.

use ark_bn254::{Bn254, Fr};
use ark_crypto_primitives::sponge::constraints::CryptographicSpongeVar;
use ark_crypto_primitives::sponge::poseidon::constraints::PoseidonSpongeVar;
use ark_crypto_primitives::sponge::poseidon::{find_poseidon_ark_and_mds, PoseidonConfig, PoseidonSponge};
use ark_crypto_primitives::sponge::{CryptographicSponge, FieldBasedCryptographicSponge};
use ark_ff::PrimeField;
use ark_groth16::{Groth16, Proof, VerifyingKey};
use ark_r1cs_std::fields::fp::FpVar;
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_snark::SNARK;
use core::cmp::Ordering;
use rand::{rngs::StdRng, SeedableRng};

/// Poseidon params over BN254 Fr (rate 2, capacity 1, alpha 5, 8 full + 57 partial rounds). The SAME
/// config is used natively (`commit_book`) and in-circuit, so the commitments match.
pub fn poseidon_config() -> PoseidonConfig<Fr> {
    let (full_rounds, partial_rounds, alpha, rate, capacity) = (8usize, 57usize, 5u64, 2usize, 1usize);
    let (ark, mds) = find_poseidon_ark_and_mds::<Fr>(
        Fr::MODULUS_BIT_SIZE as u64,
        rate,
        full_rounds as u64,
        partial_rounds as u64,
        0,
    );
    PoseidonConfig::new(full_rounds, partial_rounds, alpha, mds, ark, rate, capacity)
}

/// Native Poseidon commitment to a loan book: absorb (collateral, performing, kyc) per loan, squeeze
/// one field element. This is the value the servicer signs.
pub fn commit_book(collateral: &[u128], performing: &[bool], kyc: &[bool]) -> Fr {
    let mut sponge = PoseidonSponge::<Fr>::new(&poseidon_config());
    let mut inputs = Vec::with_capacity(collateral.len() * 3);
    for i in 0..collateral.len() {
        inputs.push(Fr::from(collateral[i]));
        inputs.push(Fr::from(performing[i] as u64));
        inputs.push(Fr::from(kyc[i] as u64));
    }
    sponge.absorb(&inputs);
    sponge.squeeze_native_field_elements(1)[0]
}

#[derive(Clone)]
pub struct SolvencyCircuit {
    pub collateral: Vec<Option<u128>>, // private
    pub performing: Vec<Option<bool>>, // private
    pub kyc: Vec<Option<bool>>,        // private
    pub threshold: Option<u128>,       // PUBLIC input 0
    pub commitment: Option<Fr>,        // PUBLIC input 1: Poseidon commitment to the book
    pub n: usize,
}

impl ConstraintSynthesizer<Fr> for SolvencyCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        let threshold = FpVar::<Fr>::new_input(cs.clone(), || {
            self.threshold.map(Fr::from).ok_or(SynthesisError::AssignmentMissing)
        })?;
        let commitment = FpVar::<Fr>::new_input(cs.clone(), || {
            self.commitment.ok_or(SynthesisError::AssignmentMissing)
        })?;

        let mut sum = FpVar::<Fr>::zero();
        let mut hash_inputs: Vec<FpVar<Fr>> = Vec::with_capacity(self.n * 3);
        for i in 0..self.n {
            let c = FpVar::<Fr>::new_witness(cs.clone(), || {
                self.collateral[i].map(Fr::from).ok_or(SynthesisError::AssignmentMissing)
            })?;
            let p = Boolean::<Fr>::new_witness(cs.clone(), || {
                self.performing[i].ok_or(SynthesisError::AssignmentMissing)
            })?;
            let k = Boolean::<Fr>::new_witness(cs.clone(), || {
                self.kyc[i].ok_or(SynthesisError::AssignmentMissing)
            })?;
            // every borrower must have passed KYC (book is unprovable otherwise)
            k.enforce_equal(&Boolean::constant(true))?;

            let p_f = FpVar::<Fr>::from(p);
            let k_f = FpVar::<Fr>::from(k);
            sum = sum + c.clone() * p_f.clone();

            // absorb order must match commit_book: collateral, performing, kyc
            hash_inputs.push(c);
            hash_inputs.push(p_f);
            hash_inputs.push(k_f);
        }

        // (3) the private book hashes to the public commitment
        let mut sponge = PoseidonSpongeVar::<Fr>::new(cs.clone(), &poseidon_config());
        sponge.absorb(&hash_inputs)?;
        let squeezed = sponge.squeeze_field_elements(1)?;
        squeezed[0].enforce_equal(&commitment)?;

        // (1) solvency
        sum.enforce_cmp(&threshold, Ordering::Greater, true)?;
        Ok(())
    }
}

/// Setup + prove. Returns the verifying key, the proof, and the public inputs `[threshold, commitment]`
/// so both the native and the Solana alt_bn128 verifier consume the same proof.
pub fn prove_solvency(
    collateral: &[u128],
    performing: &[bool],
    kyc: &[bool],
    threshold: u128,
) -> (VerifyingKey<Bn254>, Proof<Bn254>, Vec<Fr>) {
    let n = collateral.len();
    let commitment = commit_book(collateral, performing, kyc);
    let mut rng = StdRng::seed_from_u64(7);
    let (pk, vk) = Groth16::<Bn254>::circuit_specific_setup(
        SolvencyCircuit {
            collateral: vec![None; n],
            performing: vec![None; n],
            kyc: vec![None; n],
            threshold: None,
            commitment: None,
            n,
        },
        &mut rng,
    )
    .expect("setup");
    let circuit = SolvencyCircuit {
        collateral: collateral.iter().map(|&x| Some(x)).collect(),
        performing: performing.iter().map(|&x| Some(x)).collect(),
        kyc: kyc.iter().map(|&x| Some(x)).collect(),
        threshold: Some(threshold),
        commitment: Some(commitment),
        n,
    };
    let proof = Groth16::<Bn254>::prove(&pk, circuit, &mut rng).expect("prove");
    (vk, proof, vec![Fr::from(threshold), commitment])
}
