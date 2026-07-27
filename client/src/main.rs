//! Send a real solvency-proof transaction to the deployed on-chain verifier.
//!   verify-client <rpc_url> <program_id> [keypair_path]
//! Prints the on-chain result, compute units, program logs, and the confirmed signature.

use credit_solvency_core::circuit::prove_solvency;
use credit_solvency_core::onchain_bytes::build_instruction_data;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{read_keypair_file, Signer},
    transaction::Transaction,
};
use std::str::FromStr;

fn main() {
    let mut args = std::env::args().skip(1);
    let url = args.next().unwrap_or_else(|| "http://127.0.0.1:8899".into());
    let program_id = Pubkey::from_str(&args.next().expect("program id arg")).expect("valid program id");
    let kp = args
        .next()
        .unwrap_or_else(|| format!("{}/.config/solana/id.json", std::env::var("HOME").unwrap()));

    let client = RpcClient::new_with_commitment(url.clone(), CommitmentConfig::confirmed());
    let payer = read_keypair_file(&kp).expect("read keypair");

    // Build a solvent, fully-KYC'd book and prove it.
    let n = 12usize;
    let collateral: Vec<u128> = (0..n as u128).map(|i| 200_000 + i * 5_000).collect();
    let performing = vec![true; n];
    let kyc = vec![true; n];
    let total: u128 = collateral.iter().sum();
    let threshold = total - 50_000;
    let (vk, proof, public) = prove_solvency(&collateral, &performing, &kyc, threshold);
    let data = build_instruction_data(&vk, &proof, &public);

    let ix = Instruction { program_id, accounts: vec![], data };
    let bh = client.get_latest_blockhash().expect("blockhash");
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[&payer], bh);

    // Simulate first to read compute units + program logs.
    let sim = client.simulate_transaction(&tx).expect("simulate").value;
    println!("=== on-chain verification (live validator: {url}) ===");
    println!("program           : {program_id}");
    println!("simulation error  : {:?}", sim.err);
    println!("compute units used: {:?}", sim.units_consumed);
    for l in sim.logs.unwrap_or_default() {
        println!("  log: {l}");
    }

    // Send and confirm the real transaction.
    let sig = client.send_and_confirm_transaction(&tx).expect("send+confirm");
    println!("CONFIRMED on-chain signature: {sig}");
}
