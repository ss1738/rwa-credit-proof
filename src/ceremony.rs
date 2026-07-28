//! Multi-party trusted-setup ceremony (phase 2 / delta), run across separate machines.
//!
//!   machine 1:  ceremony init       p0.bin                 (initial setup)
//!   machine 2:  ceremony contribute p0.bin p1.bin  mini-2  (secret factor, discarded)
//!   machine 3:  ceremony contribute p1.bin p2.bin  mini-3  (secret factor, discarded)
//!   anyone:     ceremony verify      p2.bin                 (final keys still prove+verify)
//!
//! The params files are PUBLIC and transit between machines; each participant's secret never leaves
//! its own process. After all contributions, the final delta is a product of independent secrets and
//! no single machine knows it. Honest scope: this is the phase-2 half of a trusted setup and omits
//! per-contribution consistency proofs, see KNOWN_LIMITATIONS.md #1.

use ark_bn254::Bn254;
use ark_groth16::{Groth16, ProvingKey};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_snark::SNARK;
use credit_solvency_core::circuit::{contribute_delta, prove_with_pk, setup_only};
use sha2::{Digest, Sha256};
use std::fs::File;

const N: usize = 10;

fn load(path: &str) -> ProvingKey<Bn254> {
    ProvingKey::deserialize_uncompressed(File::open(path).expect("open")).expect("deserialize")
}
fn save(pk: &ProvingKey<Bn254>, path: &str) {
    pk.serialize_uncompressed(File::create(path).expect("create")).expect("serialize");
}
/// A short fingerprint of delta (from vk.delta_g2). Changes at every contribution; a public transcript.
fn delta_fp(pk: &ProvingKey<Bn254>) -> String {
    let mut b = Vec::new();
    pk.vk.delta_g2.serialize_uncompressed(&mut b).unwrap();
    Sha256::digest(&b)[..8].iter().map(|x| format!("{x:02x}")).collect()
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    match a.get(1).map(String::as_str) {
        Some("init") => {
            let pk = setup_only(N);
            save(&pk, &a[2]);
            println!("init         -> delta {}", delta_fp(&pk));
        }
        Some("contribute") => {
            let pk = load(&a[2]);
            let who = a.get(4).cloned().unwrap_or_else(|| "participant".into());
            let before = delta_fp(&pk);
            let pk2 = contribute_delta(pk); // secret s generated + dropped inside
            save(&pk2, &a[3]);
            println!("contribute[{who}]  delta {before} -> {}   (secret factor discarded)", delta_fp(&pk2));
        }
        Some("verify") => {
            let pk = load(&a[2]);
            let n = N;
            let collateral: Vec<u128> = (0..n as u128).map(|i| 150_000 + i * 1_000).collect();
            let performing = vec![true; n];
            let kyc = vec![true; n];
            let total: u128 = collateral.iter().sum();
            let (proof, public) = prove_with_pk(&pk, &collateral, &performing, &kyc, total - 50_000);
            let ok = Groth16::<Bn254>::verify(&pk.vk, &public, &proof).expect("verify");
            println!("verify       final delta {}", delta_fp(&pk));
            println!("proof generated + verified under the CEREMONY keys: {ok}");
            assert!(ok, "ceremony output does not produce a verifying proof");
        }
        _ => {
            eprintln!("usage: ceremony init <out> | contribute <in> <out> [name] | verify <in>");
            std::process::exit(2);
        }
    }
}
