//! Verify the solvency proof through Solana's `alt_bn128` pairing interface -- the exact arithmetic
//! the on-chain mint-gate runs. This is a NATIVE run (no SBF toolchain, no devnet needed): it proves
//! the arkworks proof serializes correctly into Solana's verifier byte layout (EIP-197), which is the
//! real integration risk. Run on a Mini:  cargo run --release --bin onchain
//!
//! Groth16 check, arranged for one pairing product == 1:
//!   e(-A, B) * e(alpha, beta) * e(vk_x, gamma) * e(C, delta) == 1,  vk_x = IC0 + Σ pub_i · IC_i

use ark_bn254::{Bn254, Fq, Fr, G1Affine, G2Affine};
use ark_ec::{AffineRepr, CurveGroup};
use ark_ff::{BigInteger, PrimeField};
use ark_groth16::{Proof, VerifyingKey};
use credit_solvency_core::circuit::prove_solvency;
use solana_bn254::prelude::alt_bn128_pairing;

fn fq_be(x: &Fq) -> [u8; 32] {
    let v = x.into_bigint().to_bytes_be();
    let mut o = [0u8; 32];
    o[32 - v.len()..].copy_from_slice(&v);
    o
}

fn g1(p: &G1Affine) -> [u8; 64] {
    let mut o = [0u8; 64];
    if p.infinity {
        return o; // point at infinity -> (0, 0)
    }
    o[..32].copy_from_slice(&fq_be(&p.x));
    o[32..].copy_from_slice(&fq_be(&p.y));
    o
}

fn g2(p: &G2Affine) -> [u8; 128] {
    // EIP-197 order: x.c1, x.c0, y.c1, y.c0 (imaginary part first)
    let mut o = [0u8; 128];
    if p.infinity {
        return o;
    }
    o[..32].copy_from_slice(&fq_be(&p.x.c1));
    o[32..64].copy_from_slice(&fq_be(&p.x.c0));
    o[64..96].copy_from_slice(&fq_be(&p.y.c1));
    o[96..].copy_from_slice(&fq_be(&p.y.c0));
    o
}

/// Run the Groth16 pairing check via Solana's alt_bn128 syscall path.
fn on_chain_verify(vk: &VerifyingKey<Bn254>, proof: &Proof<Bn254>, public: &[Fr]) -> bool {
    // vk_x = IC[0] + Σ public_i · IC[i+1]
    let ic = &vk.gamma_abc_g1;
    let mut acc = ic[0].into_group();
    for (i, s) in public.iter().enumerate() {
        acc += ic[i + 1] * *s;
    }
    let vk_x = acc.into_affine();

    let neg_a = -proof.a;
    let pairs = [
        (g1(&neg_a), g2(&proof.b)),
        (g1(&vk.alpha_g1), g2(&vk.beta_g2)),
        (g1(&vk_x), g2(&vk.gamma_g2)),
        (g1(&proof.c), g2(&vk.delta_g2)),
    ];
    let mut input = Vec::with_capacity(4 * 192);
    for (a, b) in pairs {
        input.extend_from_slice(&a);
        input.extend_from_slice(&b);
    }
    match alt_bn128_pairing(&input) {
        Ok(out) => out.last() == Some(&1), // 32-byte big-endian bool
        Err(_) => false,
    }
}

fn main() {
    let n = 10usize;
    let collateral: Vec<u128> = (0..n as u128).map(|i| 150_000 + i * 1_000).collect();
    let performing = vec![true; n];
    let total: u128 = collateral.iter().sum();
    let threshold = total - 50_000;

    let (vk, proof, public) = prove_solvency(&collateral, &performing, threshold);

    let ok = on_chain_verify(&vk, &proof, &public);
    println!("=== Solana alt_bn128 verification of the solvency proof ===");
    println!("book: {n} loans (values hidden)   public threshold: {threshold}");
    println!("alt_bn128_pairing accepts the proof : {ok}");

    // negative control: verify against a threshold the proof was NOT made for -> vk_x differs -> reject
    let wrong = vec![Fr::from(total + 1_000_000)];
    let bad = on_chain_verify(&vk, &proof, &wrong);
    println!("wrong public input rejected         : {}", !bad);

    assert!(ok && !bad, "on-chain verification soundness/completeness failed");
    println!("\nOK: the same proof verifies through Solana's pairing syscall path.");
}
