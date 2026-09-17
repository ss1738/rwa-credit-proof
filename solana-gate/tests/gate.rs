//! Runs the COMPILED SBF program, never a native processor substitute.
use caledren_gate_protocol::{Approval, Config, Policy, CONFIG_LEN, FR_MODULUS_BE};
use caledren_solana_gate::GateError;
use credit_solvency_core::{
    circuit::{prove_with_pk, setup_only},
    onchain_bytes::{fr_be, gate_key, gate_proof},
};
use solana_program_test::{ProgramTest, ProgramTestContext};
use solana_sdk::{
    account::Account,
    clock::Clock,
    compute_budget::ComputeBudgetInstruction,
    instruction::{AccountMeta, Instruction, InstructionError},
    pubkey::Pubkey,
    rent::Rent,
    signature::{Keypair, Signer},
    system_program,
    transaction::{Transaction, TransactionError},
};
const NOW: i64 = 1_780_000_000;

async fn send(
    ctx: &mut ProgramTestContext,
    program: Pubkey,
    accounts: Vec<AccountMeta>,
    data: Vec<u8>,
    signers: &[&Keypair],
    case: u32,
) -> (Result<(), TransactionError>, u64, usize) {
    let ix = Instruction {
        program_id: program,
        accounts,
        data,
    };
    let mut signing: Vec<&dyn Signer> = vec![&ctx.payer];
    for signer in signers {
        signing.push(*signer);
    }
    let tx = Transaction::new_signed_with_payer(
        &[
            ComputeBudgetInstruction::set_compute_unit_limit(180_000 + case),
            ix,
        ],
        Some(&ctx.payer.pubkey()),
        &signing,
        ctx.last_blockhash,
    );
    let size = bincode::serialize(&tx).unwrap().len();
    assert!(size <= 1232, "pilot must fit a legacy transaction: {size}");
    let result = ctx
        .banks_client
        .process_transaction_with_metadata(tx)
        .await
        .unwrap();
    (
        result.result,
        result
            .metadata
            .map(|m| m.compute_units_consumed)
            .unwrap_or(0),
        size,
    )
}
fn approve_accounts(config: Pubkey, servicer: Pubkey, signed: bool) -> Vec<AccountMeta> {
    vec![
        AccountMeta::new(config, false),
        AccountMeta::new_readonly(servicer, signed),
    ]
}
async fn state(ctx: &mut ProgramTestContext, key: Pubkey) -> Vec<u8> {
    ctx.banks_client
        .get_account(key)
        .await
        .unwrap()
        .unwrap()
        .data
}
async fn reject(
    ctx: &mut ProgramTestContext,
    pid: Pubkey,
    config: Pubkey,
    metas: Vec<AccountMeta>,
    data: Vec<u8>,
    signers: &[&Keypair],
    case: u32,
    name: &str,
    error: GateError,
) {
    let before = state(ctx, config).await;
    let (result, _, _) = send(ctx, pid, metas, data, signers, case).await;
    assert_eq!(
        result,
        Err(TransactionError::InstructionError(
            1,
            InstructionError::Custom(error as u32)
        )),
        "{name}"
    );
    assert_eq!(
        state(ctx, config).await,
        before,
        "rejection changed state: {name}"
    );
    println!("GATE_REJECTED {name}");
}
#[tokio::test]
async fn compiled_gate_enforces_policy_and_preserves_failed_state() {
    assert!(
        std::env::var_os("SBF_OUT_DIR").is_some(),
        "run through ./pilot-demo.sh or set SBF_OUT_DIR to compiled program directory"
    );
    let pid = Pubkey::new_unique();
    let config = Keypair::new();
    let uninitialized = Keypair::new();
    let wrong_owner = Keypair::new();
    let servicer = Keypair::new();
    let attacker = Keypair::new();
    let pk = setup_only(3);
    let (proof, public) = prove_with_pk(&pk, &[100, 100, 100], &[true; 3], &[true; 3], 240);
    let policy = Policy {
        servicer: servicer.pubkey().to_bytes(),
        minimum_threshold: 240,
        max_age_seconds: 300,
        verifying_key: gate_key(&pk.vk).unwrap(),
    };
    let valid = Approval {
        sequence: 1,
        observed_at: NOW,
        expires_at: NOW + 120,
        threshold: 240,
        commitment: fr_be(&public[1]),
        proof: gate_proof(&proof),
    };
    let mut pt = ProgramTest::new("caledren_solana_gate", pid, None);
    for key in [config.pubkey(), uninitialized.pubkey()] {
        pt.add_account(
            key,
            Account {
                lamports: Rent::default().minimum_balance(CONFIG_LEN),
                data: vec![0; CONFIG_LEN],
                owner: pid,
                ..Account::default()
            },
        );
    }
    pt.add_account(
        wrong_owner.pubkey(),
        Account {
            lamports: 1_000_000_000,
            data: vec![0; CONFIG_LEN],
            owner: system_program::id(),
            ..Account::default()
        },
    );
    for key in [servicer.pubkey(), attacker.pubkey()] {
        pt.add_account(
            key,
            Account {
                lamports: 1_000_000_000,
                owner: system_program::id(),
                ..Account::default()
            },
        );
    }
    let mut ctx = pt.start_with_context().await;
    ctx.set_sysvar(&Clock {
        unix_timestamp: NOW,
        slot: 42,
        ..Clock::default()
    });
    let payer = ctx.payer.pubkey();
    reject(
        &mut ctx,
        pid,
        config.pubkey(),
        vec![
            AccountMeta::new(config.pubkey(), false),
            AccountMeta::new_readonly(payer, true),
        ],
        policy.instruction().unwrap(),
        &[],
        1,
        "unsigned_config_initialization",
        GateError::Unauthorized,
    )
    .await;
    let init_accounts = vec![
        AccountMeta::new(config.pubkey(), true),
        AccountMeta::new_readonly(payer, true),
    ];
    let (result, _, _) = send(
        &mut ctx,
        pid,
        init_accounts.clone(),
        policy.instruction().unwrap(),
        &[&config],
        2,
    )
    .await;
    assert!(result.is_ok(), "initialize: {result:?}");
    reject(
        &mut ctx,
        pid,
        config.pubkey(),
        init_accounts,
        policy.instruction().unwrap(),
        &[&config],
        3,
        "reinitialization",
        GateError::AlreadyInitialized,
    )
    .await;
    reject(
        &mut ctx,
        pid,
        config.pubkey(),
        approve_accounts(config.pubkey(), attacker.pubkey(), true),
        valid.instruction(),
        &[&attacker],
        4,
        "wrong_servicer",
        GateError::Unauthorized,
    )
    .await;
    reject(
        &mut ctx,
        pid,
        config.pubkey(),
        approve_accounts(config.pubkey(), servicer.pubkey(), false),
        valid.instruction(),
        &[],
        5,
        "missing_servicer_signature",
        GateError::Unauthorized,
    )
    .await;
    reject(
        &mut ctx,
        pid,
        wrong_owner.pubkey(),
        approve_accounts(wrong_owner.pubkey(), servicer.pubkey(), true),
        valid.instruction(),
        &[&servicer],
        6,
        "wrong_account_owner",
        GateError::WrongOwner,
    )
    .await;
    reject(
        &mut ctx,
        pid,
        uninitialized.pubkey(),
        approve_accounts(uninitialized.pubkey(), servicer.pubkey(), true),
        valid.instruction(),
        &[&servicer],
        7,
        "uninitialized_config",
        GateError::Uninitialized,
    )
    .await;
    reject(
        &mut ctx,
        pid,
        config.pubkey(),
        vec![
            AccountMeta::new_readonly(config.pubkey(), false),
            AccountMeta::new_readonly(servicer.pubkey(), true),
        ],
        valid.instruction(),
        &[&servicer],
        8,
        "readonly_config",
        GateError::ReadOnly,
    )
    .await;
    let mut case = 9;
    let mut variants: Vec<(&str, Approval, GateError)> = vec![];
    let mut request = valid.clone();
    request.threshold = 239;
    variants.push(("below_policy_threshold", request, GateError::Threshold));
    let mut request = valid.clone();
    request.observed_at = NOW + 1;
    variants.push(("future_attestation", request, GateError::Freshness));
    let mut request = valid.clone();
    request.observed_at = NOW - 1;
    request.expires_at = NOW;
    variants.push(("expired_attestation", request, GateError::Freshness));
    let mut request = valid.clone();
    request.expires_at = NOW + 301;
    variants.push(("excessive_validity_window", request, GateError::Freshness));
    let mut request = valid.clone();
    request.observed_at = -1;
    variants.push(("negative_timestamp", request, GateError::Freshness));
    let mut request = valid.clone();
    request.sequence = 2;
    variants.push(("skipped_sequence", request, GateError::Replay));
    let mut request = valid.clone();
    request.proof[0] ^= 1;
    variants.push(("tampered_proof", request, GateError::InvalidProof));
    let mut request = valid.clone();
    request.commitment[31] ^= 1;
    variants.push(("altered_commitment", request, GateError::InvalidProof));
    let mut request = valid.clone();
    request.threshold += 1;
    variants.push(("altered_public_threshold", request, GateError::InvalidProof));
    let other = setup_only(3);
    let (other_proof, other_public) =
        prove_with_pk(&other, &[100, 100, 100], &[true; 3], &[true; 3], 240);
    let mut request = valid.clone();
    request.proof = gate_proof(&other_proof);
    request.commitment = fr_be(&other_public[1]);
    variants.push((
        "valid_proof_from_unapproved_key",
        request,
        GateError::InvalidProof,
    ));
    for (name, request, error) in variants {
        reject(
            &mut ctx,
            pid,
            config.pubkey(),
            approve_accounts(config.pubkey(), servicer.pubkey(), true),
            request.instruction(),
            &[&servicer],
            case,
            name,
            error,
        )
        .await;
        case += 1;
    }
    let encoded = valid.instruction();
    let mut malformed = vec![
        ("empty", vec![]),
        ("short_header", vec![1, 1]),
        ("truncated_proof", encoded[..331].to_vec()),
    ];
    let mut trailing = encoded.clone();
    trailing.push(0);
    malformed.push(("trailing_bytes", trailing));
    let mut reserved = encoded.clone();
    reserved[2] = 1;
    malformed.push(("reserved_byte", reserved));
    let mut version = encoded.clone();
    version[0] = 2;
    malformed.push(("unknown_version", version));
    let mut noncanonical = encoded.clone();
    noncanonical[44..76].copy_from_slice(&FR_MODULUS_BE);
    malformed.push(("noncanonical_scalar", noncanonical));
    for (name, data) in malformed {
        reject(
            &mut ctx,
            pid,
            config.pubkey(),
            approve_accounts(config.pubkey(), servicer.pubkey(), true),
            data,
            &[&servicer],
            case,
            name,
            GateError::Encoding,
        )
        .await;
        case += 1;
    }
    let (result, cu, size) = send(
        &mut ctx,
        pid,
        approve_accounts(config.pubkey(), servicer.pubkey(), true),
        valid.instruction(),
        &[&servicer],
        case,
    )
    .await;
    case += 1;
    assert!(result.is_ok(), "valid approval rejected: {result:?}");
    let accepted = Config::decode(&state(&mut ctx, config.pubkey()).await).unwrap();
    assert_eq!(accepted.policy, policy);
    assert_eq!(accepted.sequence, 1);
    assert_eq!(accepted.commitment, valid.commitment);
    assert_eq!(accepted.approved_threshold, 240);
    assert_eq!(accepted.verified_slot, 42);
    println!("GATE_ACCEPTED sequence=1 compute_units={cu} transaction_bytes={size}");
    reject(
        &mut ctx,
        pid,
        config.pubkey(),
        approve_accounts(config.pubkey(), servicer.pubkey(), true),
        valid.instruction(),
        &[&servicer],
        case,
        "replayed_approval",
        GateError::Replay,
    )
    .await;
    case += 1;
    let mut older = valid.clone();
    older.sequence = 2;
    older.observed_at = NOW - 1;
    reject(
        &mut ctx,
        pid,
        config.pubkey(),
        approve_accounts(config.pubkey(), servicer.pubkey(), true),
        older.instruction(),
        &[&servicer],
        case,
        "older_book_after_acceptance",
        GateError::Freshness,
    )
    .await;
    case += 1;
    let (next_proof, next_public) =
        prove_with_pk(&pk, &[110, 100, 100], &[true; 3], &[true; 3], 240);
    let mut next = valid.clone();
    next.sequence = 2;
    next.proof = gate_proof(&next_proof);
    next.commitment = fr_be(&next_public[1]);
    let (result, cu, size) = send(
        &mut ctx,
        pid,
        approve_accounts(config.pubkey(), servicer.pubkey(), true),
        next.instruction(),
        &[&servicer],
        case,
    )
    .await;
    assert!(result.is_ok());
    let accepted = Config::decode(&state(&mut ctx, config.pubkey()).await).unwrap();
    assert_eq!(accepted.sequence, 2);
    assert_eq!(accepted.commitment, next.commitment);
    println!("GATE_ACCEPTED sequence=2 compute_units={cu} transaction_bytes={size}");
    println!("GATE_SUITE_PASSED");
}
