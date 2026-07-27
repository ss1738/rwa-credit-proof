//! Zero-knowledge solvency-sum.
//!
//! Prove `Σ (performing collateral) ≥ threshold` where the collateral values and performing flags are
//! PRIVATE and only the public `threshold` is revealed. This is the on-chain-relevant half of the
//! statement in `evaluate()`: a fund can prove its book covers the claim without exposing the book.
//!
//! Groth16 over BN254 on purpose: this is the SAME proof system Solana verifies natively via its
//! `alt_bn128` pairing syscalls, so the first proof is already on the path to the Solana mint-gate.
//! (Signature-in-circuit and Nova folding for 10k loans are later weeks; this is the core predicate.)

use ark_bn254::Fr;
use ark_r1cs_std::fields::fp::FpVar;
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use core::cmp::Ordering;

#[derive(Clone)]
pub struct SolvencyCircuit {
    pub collateral: Vec<Option<u128>>, // private witness: per-loan collateral (minor units)
    pub performing: Vec<Option<bool>>, // private witness: 1 if the loan is performing
    pub threshold: Option<u128>,       // PUBLIC input: required collateral to back the claim
    pub n: usize,                      // book size (fixes the circuit shape)
}

impl ConstraintSynthesizer<Fr> for SolvencyCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        // public: the required collateral threshold
        let threshold = FpVar::<Fr>::new_input(cs.clone(), || {
            self.threshold.map(Fr::from).ok_or(SynthesisError::AssignmentMissing)
        })?;

        // sum of (collateral_i * performing_i) over the private book
        let mut sum = FpVar::<Fr>::zero();
        for i in 0..self.n {
            let c = FpVar::<Fr>::new_witness(cs.clone(), || {
                self.collateral[i].map(Fr::from).ok_or(SynthesisError::AssignmentMissing)
            })?;
            let p = Boolean::<Fr>::new_witness(cs.clone(), || {
                self.performing[i].ok_or(SynthesisError::AssignmentMissing)
            })?;
            // performing flag selects the collateral into the sum (0 or 1)
            sum += c * FpVar::<Fr>::from(p);
        }

        // enforce: sum >= threshold  (Greater with equality allowed)
        sum.enforce_cmp(&threshold, Ordering::Greater, true)?;
        Ok(())
    }
}
