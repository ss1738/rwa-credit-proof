//! Serialize an arkworks proof + verifying key + public inputs into the byte layout the on-chain
//! `solana-verifier` program expects (EIP-197 big-endian points, big-endian scalars).

use ark_bn254::{Bn254, Fq, Fr, G1Affine, G2Affine};
use ark_ff::{BigInteger, PrimeField};
use ark_groth16::{Proof, VerifyingKey};

fn fq_be(x: &Fq) -> [u8; 32] {
    let v = x.into_bigint().to_bytes_be();
    let mut o = [0u8; 32];
    o[32 - v.len()..].copy_from_slice(&v);
    o
}

pub fn fr_be(x: &Fr) -> [u8; 32] {
    let v = x.into_bigint().to_bytes_be();
    let mut o = [0u8; 32];
    o[32 - v.len()..].copy_from_slice(&v);
    o
}

fn g1(p: &G1Affine) -> [u8; 64] {
    let mut o = [0u8; 64];
    if p.infinity {
        return o;
    }
    o[..32].copy_from_slice(&fq_be(&p.x));
    o[32..].copy_from_slice(&fq_be(&p.y));
    o
}

fn g2(p: &G2Affine) -> [u8; 128] {
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

/// Two-input key stored once in an immutable pilot policy account.
pub fn gate_key(vk: &VerifyingKey<Bn254>) -> Result<[u8; 640], &'static str> {
    if vk.gamma_abc_g1.len() != 3 { return Err("pilot requires exactly two public inputs"); }
    let mut bytes = Vec::with_capacity(640);
    bytes.extend_from_slice(&g1(&vk.alpha_g1));
    bytes.extend_from_slice(&g2(&vk.beta_g2));
    bytes.extend_from_slice(&g2(&vk.gamma_g2));
    bytes.extend_from_slice(&g2(&vk.delta_g2));
    for point in &vk.gamma_abc_g1 { bytes.extend_from_slice(&g1(point)); }
    bytes.try_into().map_err(|_| "invalid key length")
}

pub fn gate_proof(proof: &Proof<Bn254>) -> [u8; 256] {
    let mut out = [0; 256];
    out[..64].copy_from_slice(&g1(&(-proof.a)));
    out[64..192].copy_from_slice(&g2(&proof.b));
    out[192..].copy_from_slice(&g1(&proof.c));
    out
}

/// Layout: [neg_a:64][b:128][c:64][alpha:64][beta:128][gamma:128][delta:128][ic0:64][n_pub:1]
/// then n_pub × ([ic_i:64][scalar_i:32]).
pub fn build_instruction_data(vk: &VerifyingKey<Bn254>, proof: &Proof<Bn254>, public: &[Fr]) -> Vec<u8> {
    let mut d = Vec::new();
    d.extend_from_slice(&g1(&(-proof.a)));
    d.extend_from_slice(&g2(&proof.b));
    d.extend_from_slice(&g1(&proof.c));
    d.extend_from_slice(&g1(&vk.alpha_g1));
    d.extend_from_slice(&g2(&vk.beta_g2));
    d.extend_from_slice(&g2(&vk.gamma_g2));
    d.extend_from_slice(&g2(&vk.delta_g2));
    d.extend_from_slice(&g1(&vk.gamma_abc_g1[0]));
    d.push(public.len() as u8);
    for (i, s) in public.iter().enumerate() {
        d.extend_from_slice(&g1(&vk.gamma_abc_g1[i + 1]));
        d.extend_from_slice(&fr_be(s));
    }
    d
}
