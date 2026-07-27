//! End-to-end demo of the solvency statement on a mock loan book.
//!
//!   servicer signs a book -> evaluate() -> VERIFIED / BLOCKED
//!
//! Run: cargo run --bin demo   (on a Mac Mini, per repo policy)

use credit_solvency_core::{evaluate, Loan, LoanStatus, LoanTape};
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use rand::Rng;

/// Build a mock book of `n` loans. `solvent` picks whether collateral covers the claim.
fn mock_tape(n: usize, solvent: bool, rng: &mut impl Rng) -> LoanTape {
    let loans: Vec<Loan> = (0..n)
        .map(|i| {
            let principal = (rng.gen_range(50_000u128..500_000)) * 100; // cents
            let mult = if solvent { rng.gen_range(130..180) } else { rng.gen_range(60..90) };
            mock_loan(i as u64, principal, mult, rng)
        })
        .collect();
    let claimed_supply: u128 = loans.iter().map(|l| l.principal).sum();
    LoanTape { loans, claimed_supply, overcollat_ratio_bps: 12000 } // require 120%
}

fn mock_loan(id: u64, principal: u128, mult: u128, rng: &mut impl Rng) -> Loan {
    Loan {
        id,
        principal,
        collateral_value: principal * mult / 100,
        status: if rng.gen_bool(0.95) { LoanStatus::Performing } else { LoanStatus::Defaulted },
        kyc_ok: true,
    }
}

fn report(label: &str, tape: &LoanTape, servicer: &SigningKey) {
    let sig = tape.sign(servicer);
    let att = evaluate(tape, &servicer.verifying_key(), &sig);
    println!("--- {label} ({} loans) ---", tape.loans.len());
    println!("  signature valid       : {}", att.signature_valid);
    println!("  all KYC ok            : {}", att.all_kyc_ok);
    println!("  performing collateral : {}", att.performing_collateral);
    println!("  required collateral   : {}", att.required_collateral);
    println!("  solvent               : {}", att.solvent);
    println!("  => MINT {}", if att.verified() { "ALLOWED  ✅" } else { "BLOCKED  ⛔" });
    println!();
}

fn main() {
    let mut rng = OsRng;
    let servicer = SigningKey::generate(&mut rng);

    // 1. A healthy, over-collateralised, servicer-signed book -> mint allowed.
    let good = mock_tape(10, true, &mut rng);
    report("solvent book", &good, &servicer);

    // 2. Same shape, but the book does not cover the claim -> caught, mint blocked.
    let bad = mock_tape(10, false, &mut rng);
    report("insolvent book", &bad, &servicer);

    // 3. Tamper a signed good book (inflate collateral after signing) -> signature breaks.
    let mut tampered = good.clone();
    let sig = tampered.sign(&servicer);
    tampered.loans[0].collateral_value *= 5;
    let att = evaluate(&tampered, &servicer.verifying_key(), &sig);
    println!("--- tampered book (collateral inflated after signing) ---");
    println!("  signature valid : {}  => MINT {}", att.signature_valid,
             if att.verified() { "ALLOWED" } else { "BLOCKED  ⛔" });
}
