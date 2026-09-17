//! Local pilot integration: separate payer and servicer signers, persisted key and real receipt.
use caledren_gate_protocol::{Config, Policy, CONFIG_LEN};
use credit_solvency_core::pilot::{self, Book, Parameters, PrivateAttestation, ProofBundle};
use serde_json::json;
use solana_client::{
    rpc_client::RpcClient, rpc_config::RpcSimulateTransactionConfig, rpc_request::RpcRequest,
};
use solana_sdk::{
    clock::Clock,
    commitment_config::CommitmentConfig,
    compute_budget::ComputeBudgetInstruction,
    instruction::{AccountMeta, Instruction, InstructionError},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction, sysvar,
    transaction::{Transaction, TransactionError},
};
use std::{error::Error, fs, path::Path, str::FromStr, thread, time::Duration};
type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn transaction(
    client: &RpcClient,
    payer: &Keypair,
    signers: &[&Keypair],
    instructions: &[Instruction],
) -> Result<(Transaction, usize)> {
    let tx = Transaction::new_signed_with_payer(
        instructions,
        Some(&payer.pubkey()),
        signers,
        client.get_latest_blockhash()?,
    );
    let size = bincode::serialize(&tx)?.len();
    if size > 1232 {
        return Err(format!("legacy transaction exceeds 1232 bytes: {size}").into());
    }
    Ok((tx, size))
}
fn approval_ix(program: Pubkey, config: Pubkey, servicer: Pubkey, data: Vec<u8>) -> Instruction {
    Instruction {
        program_id: program,
        accounts: vec![
            AccountMeta::new(config, false),
            AccountMeta::new_readonly(servicer, true),
        ],
        data,
    }
}
pub fn run(args: Vec<String>) -> Result<()> {
    if args.len() != 6 {
        return Err("usage: verify-client pilot-demo <local-rpc> <program-id> <params-dir> <csv> <proof-dir> <new-report.json>".into());
    }
    let url = &args[0];
    // This helper requests free funds and uses throwaway signers. Restrict it to a local validator.
    let port = url
        .strip_prefix("http://127.0.0.1:")
        .or_else(|| url.strip_prefix("http://localhost:"));
    if port
        .and_then(|s| s.parse::<u16>().ok())
        .filter(|p| *p != 0)
        .is_none()
    {
        return Err("pilot-demo requires an explicit localhost RPC port".into());
    }
    if Path::new(&args[5]).exists() {
        return Err("report already exists; choose a new path".into());
    }
    let program = Pubkey::from_str(&args[1])?;
    let parameters = Parameters::load(Path::new(&args[2]))?;
    let book = Book::parse_csv(&fs::read_to_string(&args[3])?)?;
    let bundle: ProofBundle = pilot::read_json(&Path::new(&args[4]).join("proof-bundle.json"))?;
    let private: PrivateAttestation =
        pilot::read_json(&Path::new(&args[4]).join("private-attestation.json"))?;
    bundle.check_attestation(&book, &private)?;
    let key = parameters.key_bytes()?;
    if bundle.verifying_key_sha256 != pilot::hash_hex(&key)
        || bundle.loan_count != parameters.loan_count
    {
        return Err("bundle uses an unapproved parameter set".into());
    }
    let client = RpcClient::new_with_commitment(url.clone(), CommitmentConfig::finalized());
    let payer = Keypair::new();
    let servicer = Keypair::new();
    let config = Keypair::new();
    let airdrop = client.request_airdrop(&payer.pubkey(), 2_000_000_000)?;
    let mut funded = false;
    for _ in 0..120 {
        if client.confirm_transaction(&airdrop)? {
            funded = true;
            break;
        }
        thread::sleep(Duration::from_millis(500));
    }
    if !funded {
        return Err("local faucet transaction did not finalize".into());
    }
    let policy = Policy {
        servicer: servicer.pubkey().to_bytes(),
        minimum_threshold: bundle.threshold.parse()?,
        max_age_seconds: 300,
        verifying_key: key,
    };
    let initialize = Instruction {
        program_id: program,
        accounts: vec![
            AccountMeta::new(config.pubkey(), true),
            AccountMeta::new_readonly(payer.pubkey(), true),
        ],
        data: policy.instruction()?,
    };
    let (init_tx, init_size) = transaction(
        &client,
        &payer,
        &[&payer, &config],
        &[
            system_instruction::create_account(
                &payer.pubkey(),
                &config.pubkey(),
                client.get_minimum_balance_for_rent_exemption(CONFIG_LEN)?,
                CONFIG_LEN as u64,
                &program,
            ),
            initialize,
        ],
    )?;
    let init_signature = client.send_and_confirm_transaction(&init_tx)?;
    println!(
        "POLICY_FINALIZED config={} transaction_bytes={init_size}",
        config.pubkey()
    );

    let clock: Clock = bincode::deserialize(&client.get_account(&sysvar::clock::id())?.data)?;
    let request = bundle.approval(1, clock.unix_timestamp, clock.unix_timestamp + 120)?;
    let (approve_tx, approve_size) = transaction(
        &client,
        &payer,
        &[&payer, &servicer],
        &[
            ComputeBudgetInstruction::set_compute_unit_limit(180_000),
            approval_ix(
                program,
                config.pubkey(),
                servicer.pubkey(),
                request.instruction(),
            ),
        ],
    )?;
    let sim = client
        .simulate_transaction_with_config(
            &approve_tx,
            RpcSimulateTransactionConfig {
                sig_verify: true,
                commitment: Some(CommitmentConfig::finalized()),
                ..Default::default()
            },
        )?
        .value;
    if let Some(error) = sim.err {
        return Err(format!("valid approval simulation failed: {error}").into());
    }
    let signature = client.send_and_confirm_transaction(&approve_tx)?;
    let account = client.get_account(&config.pubkey())?;
    if account.owner != program {
        return Err("receipt has wrong program owner".into());
    }
    let receipt = Config::decode(&account.data)?;
    if receipt.sequence != 1
        || receipt.commitment != request.commitment
        || receipt.policy != policy
        || receipt.approved_threshold != request.threshold
        || receipt.expires_at != request.expires_at
    {
        return Err("finalized receipt differs from requested policy/approval".into());
    }
    // Negative cases are signed simulations here. The SBF suite also executes rejected transactions.
    let attacker = Keypair::new();
    let mut failures = vec![];
    for (name, signer, code) in [
        ("wrong_servicer", &attacker, 6),
        ("replayed_approval", &servicer, 9),
    ] {
        let (tx, _) = transaction(
            &client,
            &payer,
            &[&payer, signer],
            &[approval_ix(
                program,
                config.pubkey(),
                signer.pubkey(),
                request.instruction(),
            )],
        )?;
        let result = client
            .simulate_transaction_with_config(
                &tx,
                RpcSimulateTransactionConfig {
                    sig_verify: true,
                    commitment: Some(CommitmentConfig::finalized()),
                    ..Default::default()
                },
            )?
            .value;
        let expected = TransactionError::InstructionError(0, InstructionError::Custom(code));
        if result.err.as_ref() != Some(&expected) {
            return Err(format!("unexpected {name} outcome: {:?}", result.err).into());
        }
        failures.push(json!({"case":name,"simulation_only":true,"expected_custom_error":code,"error":result.err}));
    }
    if client.get_account(&config.pubkey())?.data != account.data {
        return Err("negative checks changed receipt".into());
    }
    let finalized:serde_json::Value=client.send(RpcRequest::GetTransaction,json!([signature.to_string(),{"encoding":"json","commitment":"finalized","maxSupportedTransactionVersion":0}]))?;
    if finalized.is_null() || !finalized["meta"]["err"].is_null() {
        return Err("finalized transaction receipt unavailable or failed".into());
    }
    let report = json!({"schema_version":1,"scope":"LOCAL validator pilot alpha; no public deployment, token mint or external servicer",
        "program_id":program.to_string(),"configuration":config.pubkey().to_string(),"servicer":servicer.pubkey().to_string(),"payer":payer.pubkey().to_string(),
        "loan_count":bundle.loan_count,"verifying_key_sha256":bundle.verifying_key_sha256,"threshold":bundle.threshold,
        "commitment_be":bundle.commitment_be,"sequence":receipt.sequence,"observed_at":receipt.observed_at,
        "expires_at":receipt.expires_at,"verified_slot":receipt.verified_slot,
        "configuration_data_hex":pilot::hex(&account.data),
        "initialization_signature":init_signature.to_string(),"approval_signature":signature.to_string(),
        "initialization_transaction_bytes":init_size,"approval_transaction_bytes":approve_size,
        "simulation_compute_units":sim.units_consumed,"negative_simulations":failures,"finalized_transaction":finalized});
    pilot::write_json(Path::new(&args[5]), &report, false)?;
    println!(
        "APPROVAL_FINALIZED sequence=1 compute_units={} transaction_bytes={approve_size}",
        sim.units_consumed.unwrap_or(0)
    );
    println!("LOCAL_APPROVAL_SIGNATURE {signature}");
    println!("PILOT_RPC_PASSED report={}", args[5]);
    Ok(())
}
