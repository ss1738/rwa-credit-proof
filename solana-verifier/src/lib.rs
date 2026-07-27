//! On-chain Groth16 verifier for the private-credit solvency proof.
//!
//! Runs the full BN254 pairing check using Solana's `alt_bn128` syscalls:
//!   vk_x = IC0 + Σ scalar_i · IC_i            (alt_bn128 multiplication + addition)
//!   e(-A, B) · e(alpha, beta) · e(vk_x, gamma) · e(C, delta) == 1   (alt_bn128 pairing)
//!
//! Instruction data (all points EIP-197 big-endian; scalars 32-byte big-endian), laid out by the
//! host harness that also builds the proof:
//!   [neg_a:64][b:128][c:64][alpha:64][beta:128][gamma:128][delta:128][ic0:64][n_pub:1]
//!   then n_pub × ([ic_i:64][scalar_i:32])
//!
//! This is what a real mint-gate program calls; the point of the crate is to MEASURE its actual
//! on-chain compute-unit cost under the SBF VM.

use solana_bn254::prelude::{alt_bn128_addition, alt_bn128_multiplication, alt_bn128_pairing};
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey,
};

entrypoint!(process_instruction);

pub fn process_instruction(_program_id: &Pubkey, _accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let neg_a = &data[0..64];
    let b = &data[64..192];
    let c = &data[192..256];
    let alpha = &data[256..320];
    let beta = &data[320..448];
    let gamma = &data[448..576];
    let delta = &data[576..704];
    let ic0 = &data[704..768];
    let n_pub = data[768] as usize;

    // vk_x = IC0 + Σ scalar_i · IC_i
    let mut vk_x = ic0.to_vec();
    let mut off = 769;
    for _ in 0..n_pub {
        let ic_i = &data[off..off + 64];
        let scalar = &data[off + 64..off + 96];
        off += 96;

        let mut mul_in = Vec::with_capacity(96);
        mul_in.extend_from_slice(ic_i);
        mul_in.extend_from_slice(scalar);
        let term = alt_bn128_multiplication(&mul_in).map_err(|_| ProgramError::InvalidInstructionData)?;

        let mut add_in = Vec::with_capacity(128);
        add_in.extend_from_slice(&vk_x);
        add_in.extend_from_slice(&term);
        vk_x = alt_bn128_addition(&add_in).map_err(|_| ProgramError::InvalidInstructionData)?;
    }

    // e(-A,B)·e(alpha,beta)·e(vk_x,gamma)·e(C,delta) == 1
    let mut pairing = Vec::with_capacity(4 * 192);
    for (g1, g2) in [(neg_a, b), (alpha, beta), (vk_x.as_slice(), gamma), (c, delta)] {
        pairing.extend_from_slice(g1);
        pairing.extend_from_slice(g2);
    }
    let res = alt_bn128_pairing(&pairing).map_err(|_| ProgramError::InvalidInstructionData)?;
    let valid = res.last() == Some(&1);

    msg!("groth16 solvency proof valid: {}", valid);
    if valid {
        Ok(())
    } else {
        Err(ProgramError::InvalidInstructionData)
    }
}
