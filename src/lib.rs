//! credit-solvency-core
//!
//! THE STATEMENT at the heart of the RWA private-credit bet: before a tokenized private-credit fund
//! mints tokens, it must prove that a loan tape SIGNED BY THE SERVICER is solvent and fully KYC'd,
//! WITHOUT revealing the individual loans.
//!
//! Why the signature is inside the statement: a ZK proof only certifies that *the data you were given*
//! satisfies the predicate. It cannot certify the data is true. Binding the servicer's signature into
//! the statement is the answer to that "garbage-in" problem: the proof asserts
//!   "the servicer attested this exact book AND it is solvent",
//! not "someone typed numbers that happen to be solvent". Solve the data-trust anchor or it is theatre.
//!
//! `evaluate()` is written as a pure function so it lifts into an SP1 / RISC Zero guest almost
//! unchanged in weeks 2-3 of the spike (see PRIVATE_CREDIT_SPIKE.md). Plain Rust now => testable today.

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoanStatus {
    Performing,
    Defaulted,
}

/// One loan in the book. Amounts are in minor units (e.g. cents) to stay integer-exact.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Loan {
    pub id: u64,
    pub principal: u128,
    pub collateral_value: u128,
    pub status: LoanStatus,
    pub kyc_ok: bool,
}

/// A loan book plus the claim it backs: `claimed_supply` of tokens at `overcollat_ratio_bps`
/// over-collateralisation (12000 bps = 120%).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoanTape {
    pub loans: Vec<Loan>,
    pub claimed_supply: u128,
    pub overcollat_ratio_bps: u64,
}

impl LoanTape {
    /// The deterministic digest the servicer signs. Any change to a loan or the claim breaks the
    /// signature, so the proof is bound to the exact attested book.
    pub fn signing_digest(&self) -> [u8; 32] {
        let json = serde_json::to_vec(self).expect("serialize loan tape");
        Sha256::digest(json).into()
    }

    /// Servicer signs the book (in production this key belongs to the loan servicer / custodian).
    pub fn sign(&self, key: &SigningKey) -> Signature {
        key.sign(&self.signing_digest())
    }
}

/// What the ZK proof will attest to. `verified()` is the single bit an on-chain mint-gate checks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attestation {
    pub signature_valid: bool,
    pub all_kyc_ok: bool,
    pub performing_collateral: u128,
    pub required_collateral: u128,
    pub solvent: bool,
}

impl Attestation {
    /// Mint is permitted only if the servicer signed the book, every borrower passed KYC, and the
    /// performing collateral covers the required over-collateralisation.
    pub fn verified(&self) -> bool {
        self.signature_valid && self.all_kyc_ok && self.solvent
    }
}

/// THE STATEMENT, evaluated. This exact logic becomes the zkVM guest program: prove that a tape signed
/// by `servicer` is solvent + fully KYC'd, revealing only the aggregate result, never the loans.
pub fn evaluate(tape: &LoanTape, servicer: &VerifyingKey, sig: &Signature) -> Attestation {
    let signature_valid = servicer.verify(&tape.signing_digest(), sig).is_ok();

    let all_kyc_ok = tape.loans.iter().all(|l| l.kyc_ok);

    // Only performing loans count toward collateral coverage.
    let performing_collateral: u128 = tape
        .loans
        .iter()
        .filter(|l| l.status == LoanStatus::Performing)
        .map(|l| l.collateral_value)
        .sum();

    // required = claimed_supply * ratio_bps / 10_000
    let required_collateral =
        tape.claimed_supply.saturating_mul(tape.overcollat_ratio_bps as u128) / 10_000;

    let solvent = performing_collateral >= required_collateral;

    Attestation {
        signature_valid,
        all_kyc_ok,
        performing_collateral,
        required_collateral,
        solvent,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::OsRng;

    fn tape(collat_multiple_pct: u128, all_kyc: bool) -> LoanTape {
        // 3 performing loans; each collateral = principal * multiple%. supply = sum(principal).
        let loans = (0..3)
            .map(|i| Loan {
                id: i,
                principal: 100_000_00, // 100k in cents
                collateral_value: 100_000_00 * collat_multiple_pct / 100,
                status: LoanStatus::Performing,
                kyc_ok: all_kyc || i != 1, // loan #1 fails KYC when all_kyc = false
            })
            .collect::<Vec<_>>();
        LoanTape { loans, claimed_supply: 300_000_00, overcollat_ratio_bps: 12000 }
    }

    #[test]
    fn solvent_signed_and_kyc_passes() {
        let sk = SigningKey::generate(&mut OsRng);
        let t = tape(150, true); // 150% collateral vs 120% required => solvent
        let sig = t.sign(&sk);
        let att = evaluate(&t, &sk.verifying_key(), &sig);
        assert!(att.signature_valid && att.all_kyc_ok && att.solvent);
        assert!(att.verified());
    }

    #[test]
    fn insolvent_book_is_caught_even_when_validly_signed() {
        let sk = SigningKey::generate(&mut OsRng);
        let t = tape(100, true); // 100% collateral < 120% required => insolvent
        let sig = t.sign(&sk); // honestly signed, but the book does not cover the claim
        let att = evaluate(&t, &sk.verifying_key(), &sig);
        assert!(att.signature_valid && !att.solvent);
        assert!(!att.verified());
    }

    #[test]
    fn tampered_book_breaks_the_signature() {
        let sk = SigningKey::generate(&mut OsRng);
        let mut t = tape(150, true);
        let sig = t.sign(&sk);
        t.loans[0].collateral_value *= 3; // inflate collateral AFTER signing
        let att = evaluate(&t, &sk.verifying_key(), &sig);
        assert!(!att.signature_valid); // the digest no longer matches the signature
        assert!(!att.verified());
    }

    #[test]
    fn kyc_failure_blocks_mint() {
        let sk = SigningKey::generate(&mut OsRng);
        let t = tape(150, false); // solvent, but one borrower fails KYC
        let sig = t.sign(&sk);
        let att = evaluate(&t, &sk.verifying_key(), &sig);
        assert!(att.solvent && !att.all_kyc_ok);
        assert!(!att.verified());
    }

    #[test]
    fn wrong_servicer_key_is_rejected() {
        let sk = SigningKey::generate(&mut OsRng);
        let impostor = SigningKey::generate(&mut OsRng);
        let t = tape(150, true);
        let sig = t.sign(&sk);
        let att = evaluate(&t, &impostor.verifying_key(), &sig); // verify against the wrong key
        assert!(!att.signature_valid);
        assert!(!att.verified());
    }
}
