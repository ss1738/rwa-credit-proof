//! Stateful pilot approval gate, separate from the legacy pairing-only benchmark program.
//! The transaction authorizes the exact proof/public inputs, policy account, sequence and times.
//! Consumers must pin this program AND the approved config address, and inspect receipt freshness.
//! No SPL token mint or live token-supply policy is implemented here.
use caledren_gate_protocol::{Approval, Config, Policy, CONFIG_LEN};
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey, rent::Rent, sysvar::Sysvar,
};
#[cfg(not(feature = "no-entrypoint"))]
solana_program::entrypoint!(process_instruction);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum GateError {
    Encoding = 1,
    WrongOwner = 2,
    ReadOnly = 3,
    Uninitialized = 4,
    AlreadyInitialized = 5,
    Unauthorized = 6,
    Threshold = 7,
    Freshness = 8,
    Replay = 9,
    InvalidProof = 10,
    AccountCount = 11,
    Rent = 12,
}
impl From<GateError> for ProgramError {
    fn from(e: GateError) -> Self {
        Self::Custom(e as u32)
    }
}

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    if accounts.len() != 2 {
        return Err(GateError::AccountCount.into());
    }
    let account = &accounts[0];
    let signer = &accounts[1];
    if account.owner != program_id || account.executable {
        return Err(GateError::WrongOwner.into());
    }
    if !account.is_writable {
        return Err(GateError::ReadOnly.into());
    }
    if account.data_len() != CONFIG_LEN {
        return Err(GateError::Encoding.into());
    }
    match data.get(..4) {
        Some([1, 0, 0, 0]) => initialize(account, signer, data),
        Some([1, 1, 0, 0]) => approve(program_id, account, signer, data),
        _ => Err(GateError::Encoding.into()),
    }
}
fn initialize(account: &AccountInfo, authority: &AccountInfo, data: &[u8]) -> ProgramResult {
    // The new config's own signature prevents initialization front-running after account creation.
    if !account.is_signer || !authority.is_signer {
        return Err(GateError::Unauthorized.into());
    }
    let policy = Policy::decode_instruction(data).map_err(|_| GateError::Encoding)?;
    if !Rent::get()?.is_exempt(account.lamports(), CONFIG_LEN) {
        return Err(GateError::Rent.into());
    }
    let mut bytes = account.try_borrow_mut_data()?;
    if bytes.iter().any(|byte| *byte != 0) {
        return Err(GateError::AlreadyInitialized.into());
    }
    bytes.copy_from_slice(&Config::new(authority.key.to_bytes(), policy).encode());
    msg!("Caledren immutable pilot policy initialized");
    Ok(())
}
fn approve(
    program_id: &Pubkey,
    account: &AccountInfo,
    servicer: &AccountInfo,
    data: &[u8],
) -> ProgramResult {
    let request = Approval::decode_instruction(data).map_err(|_| GateError::Encoding)?;
    let mut config =
        Config::decode(&account.try_borrow_data()?).map_err(|_| GateError::Uninitialized)?;
    if !servicer.is_signer || servicer.key.to_bytes() != config.policy.servicer {
        return Err(GateError::Unauthorized.into());
    }
    if request.threshold < config.policy.minimum_threshold {
        return Err(GateError::Threshold.into());
    }
    if config.sequence.checked_add(1) != Some(request.sequence) {
        return Err(GateError::Replay.into());
    }
    let clock = Clock::get()?;
    let lifetime = request
        .expires_at
        .checked_sub(request.observed_at)
        .ok_or(GateError::Freshness)?;
    if request.observed_at < 0
        || request.observed_at > clock.unix_timestamp
        || request.observed_at < config.observed_at
        || request.expires_at <= clock.unix_timestamp
        || lifetime <= 0
        || lifetime as u64 > config.policy.max_age_seconds
    {
        return Err(GateError::Freshness.into());
    }
    // Fixed public-input count and stored key: caller cannot replace the approved circuit.
    let pairing = request.pairing_instruction(&config.policy.verifying_key);
    solana_verifier::process_instruction(program_id, &[], &pairing)
        .map_err(|_| GateError::InvalidProof)?;
    // No state changes occur until every check passes; transaction failure also rolls writes back.
    config.sequence = request.sequence;
    config.observed_at = request.observed_at;
    config.expires_at = request.expires_at;
    config.verified_slot = clock.slot;
    config.commitment = request.commitment;
    config.approved_threshold = request.threshold;
    account
        .try_borrow_mut_data()?
        .copy_from_slice(&config.encode());
    msg!(
        "Caledren approval accepted: sequence={}, slot={}",
        config.sequence,
        config.verified_slot
    );
    Ok(())
}
