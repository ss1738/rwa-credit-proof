//! Cross-tool compatibility: proof from snarkjs's final contributed zkey, executed by compiled SBF.
use caledren_gate_protocol::{Config, Policy, CONFIG_LEN};
use caledren_solana_gate::GateError;
use credit_solvency_core::pilot::{self, ProofBundle};
use solana_program_test::{ProgramTest, ProgramTestContext};
use solana_sdk::{
    account::Account,
    clock::Clock,
    compute_budget::ComputeBudgetInstruction,
    instruction::{AccountMeta, Instruction, InstructionError},
    pubkey::Pubkey,
    rent::Rent,
    signature::{Keypair, Signer},
    transaction::{Transaction, TransactionError},
};
use std::{fs, path::PathBuf};

async fn send(
    ctx: &mut ProgramTestContext,
    ix: Instruction,
    signer: &Keypair,
    case: u32,
) -> (Result<(), TransactionError>, u64, usize) {
    let tx = Transaction::new_signed_with_payer(
        &[
            ComputeBudgetInstruction::set_compute_unit_limit(180_000 + case),
            ix,
        ],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, signer],
        ctx.last_blockhash,
    );
    let size = bincode::serialize(&tx).unwrap().len();
    assert!(size <= 1232);
    let result = ctx
        .banks_client
        .process_transaction_with_metadata(tx)
        .await
        .unwrap();
    (
        result.result,
        result.metadata.unwrap().compute_units_consumed,
        size,
    )
}
async fn bytes(ctx: &mut ProgramTestContext, key: Pubkey) -> Vec<u8> {
    ctx.banks_client
        .get_account(key)
        .await
        .unwrap()
        .unwrap()
        .data
}
#[tokio::test]
async fn snarkjs_final_key_and_proofs_are_accepted_by_compiled_gate() {
    assert!(
        std::env::var_os("SBF_OUT_DIR").is_some(),
        "requires compiled SBF program"
    );
    let run = PathBuf::from(
        std::env::var_os("CEREMONY_COMPAT_DIR").expect("run ./ceremony-compat.sh first"),
    );
    let key: [u8; 640] = fs::read(run.join("wire-1/verifying-key.bin"))
        .unwrap()
        .try_into()
        .unwrap();
    assert_eq!(
        key.as_slice(),
        fs::read(run.join("wire-2/verifying-key.bin")).unwrap()
    );
    let first: ProofBundle = pilot::read_json(&run.join("wire-1/proof-bundle.json")).unwrap();
    let second: ProofBundle = pilot::read_json(&run.join("wire-2/proof-bundle.json")).unwrap();
    assert_eq!(first.verifying_key_sha256, pilot::hash_hex(&key));
    assert_eq!(first.verifying_key_sha256, second.verifying_key_sha256);
    assert_ne!(first.commitment_be, second.commitment_be);
    let pid = Pubkey::new_unique();
    let config = Keypair::new();
    let servicer = Keypair::new();
    let policy = Policy {
        servicer: servicer.pubkey().to_bytes(),
        minimum_threshold: first.threshold.parse().unwrap(),
        max_age_seconds: 300,
        verifying_key: key,
    };
    let mut program = ProgramTest::new("caledren_solana_gate", pid, None);
    program.add_account(
        config.pubkey(),
        Account {
            lamports: Rent::default().minimum_balance(CONFIG_LEN),
            data: vec![0; CONFIG_LEN],
            owner: pid,
            ..Account::default()
        },
    );
    program.add_account(
        servicer.pubkey(),
        Account {
            lamports: 1_000_000_000,
            ..Account::default()
        },
    );
    let mut ctx = program.start_with_context().await;
    const NOW: i64 = 1_780_000_000;
    ctx.set_sysvar(&Clock {
        unix_timestamp: NOW,
        slot: 42,
        ..Clock::default()
    });
    let initialize = Instruction {
        program_id: pid,
        accounts: vec![
            AccountMeta::new(config.pubkey(), true),
            AccountMeta::new_readonly(ctx.payer.pubkey(), true),
        ],
        data: policy.instruction().unwrap(),
    };
    assert!(send(&mut ctx, initialize, &config, 1).await.0.is_ok());
    let approval_ix = |data| Instruction {
        program_id: pid,
        accounts: vec![
            AccountMeta::new(config.pubkey(), false),
            AccountMeta::new_readonly(servicer.pubkey(), true),
        ],
        data,
    };
    let before = bytes(&mut ctx, config.pubkey()).await;
    let mut bad = first.approval(1, NOW, NOW + 120).unwrap();
    bad.threshold += 1;
    let rejected = send(&mut ctx, approval_ix(bad.instruction()), &servicer, 2)
        .await
        .0;
    assert_eq!(
        rejected,
        Err(TransactionError::InstructionError(
            1,
            InstructionError::Custom(GateError::InvalidProof as u32)
        ))
    );
    assert_eq!(before, bytes(&mut ctx, config.pubkey()).await);
    println!("CEREMONY_SBF_REJECTED altered_threshold state_unchanged=true");
    for (sequence, bundle) in [(1, first), (2, second)] {
        let approval = bundle.approval(sequence, NOW, NOW + 120).unwrap();
        let (result, cu, size) = send(
            &mut ctx,
            approval_ix(approval.instruction()),
            &servicer,
            sequence as u32 + 2,
        )
        .await;
        assert!(result.is_ok(), "ceremony proof rejected: {result:?}");
        let receipt = Config::decode(&bytes(&mut ctx, config.pubkey()).await).unwrap();
        assert_eq!(receipt.policy, policy);
        assert_eq!(receipt.sequence, sequence);
        assert_eq!(receipt.commitment, approval.commitment);
        println!(
            "CEREMONY_SBF_ACCEPTED sequence={sequence} compute_units={cu} transaction_bytes={size}"
        );
    }
    println!("CEREMONY_SBF_COMPATIBILITY_PASSED");
}
