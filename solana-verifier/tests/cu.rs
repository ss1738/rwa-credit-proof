//! Measure the REAL on-chain compute-unit cost of verifying the solvency proof.
//! Loads the compiled solana_verifier.so into the SBF VM (via SBF_OUT_DIR=target/deploy) and reads
//! compute_units_consumed from the transaction metadata.
//!
//! Run on a Mini:
//!   cargo-build-sbf && SBF_OUT_DIR=$PWD/target/deploy cargo test --test cu -- --nocapture

use credit_solvency_core::circuit::prove_solvency;
use credit_solvency_core::onchain_bytes::build_instruction_data;
use solana_program_test::ProgramTest;
use solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Signer, transaction::Transaction,
};

#[tokio::test]
async fn measure_compute_units() {
    let program_id = Pubkey::new_unique();
    // third arg None -> load solana_verifier.so from SBF_OUT_DIR and run it in the SBF VM
    let pt = ProgramTest::new("solana_verifier", program_id, None);
    let ctx = pt.start_with_context().await;
    let mut banks = ctx.banks_client;
    let payer = ctx.payer;
    let recent = ctx.last_blockhash;

    let n = 10usize;
    let collateral: Vec<u128> = (0..n as u128).map(|i| 150_000 + i * 1_000).collect();
    let performing = vec![true; n];
    let kyc = vec![true; n];
    let total: u128 = collateral.iter().sum();
    let threshold = total - 50_000;

    let (vk, proof, public) = prove_solvency(&collateral, &performing, &kyc, threshold);
    let data = build_instruction_data(&vk, &proof, &public);

    let ix = Instruction { program_id, accounts: vec![], data };
    let mut tx = Transaction::new_with_payer(&[ix], Some(&payer.pubkey()));
    tx.sign(&[&payer], recent);

    let meta = banks
        .process_transaction_with_metadata(tx)
        .await
        .expect("banks call");

    println!("RESULT: {:?}", meta.result);
    if let Some(m) = meta.metadata {
        println!("COMPUTE_UNITS_CONSUMED: {}", m.compute_units_consumed);
    }
    assert!(meta.result.is_ok(), "on-chain verify failed: {:?}", meta.result);
}
