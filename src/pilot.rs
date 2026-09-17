//! Reusable pilot SDK: strict loan-tape ingestion, persisted parameters, public proof bundles,
//! and a private attestation witness for the servicer's independent commitment check.
//! A parameter hash is an identifier, not proof of a sound setup ceremony.
use crate::{
    circuit::{commit_book, random_nonce, setup_only, SolvencyCircuit},
    onchain_bytes::{fr_be, gate_key, gate_proof},
};
use ark_bn254::{Bn254, Fr};
use ark_ff::PrimeField;
use ark_groth16::{Groth16, ProvingKey};
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystem};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_snark::SNARK;
use caledren_gate_protocol::{Approval, FR_MODULUS_BE};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::HashSet, fs, io::Write, path::Path};

type Result<T> = std::result::Result<T, String>;
pub const MAX_LOANS: usize = 10_000;

#[derive(Clone, Debug)]
pub struct Book {
    pub collateral: Vec<u128>,
    pub performing: Vec<bool>,
    pub kyc: Vec<bool>,
    pub principal_total: u128,
    pub source_sha256: String,
}
impl Book {
    fn validate_shape(&self) -> Result<()> {
        let n = self.collateral.len();
        if !(1..=MAX_LOANS).contains(&n) || self.performing.len() != n || self.kyc.len() != n {
            return Err("invalid book witness lengths".into());
        }
        self.collateral
            .iter()
            .try_fold(0u128, |sum, x| sum.checked_add(*x))
            .ok_or("collateral sum overflow")?;
        Ok(())
    }
    /// Canonical unquoted CSV contract. Ambiguous flags and unknown statuses are errors.
    pub fn parse_csv(text: &str) -> Result<Self> {
        let mut lines = text.lines();
        if lines.next().map(str::trim) != Some("loan_id,principal,collateral_value,status,kyc_ok") {
            return Err("expected canonical five-column loan tape header".into());
        }
        let mut book = Self {
            collateral: vec![],
            performing: vec![],
            kyc: vec![],
            principal_total: 0,
            source_sha256: hash_hex(text.as_bytes()),
        };
        let mut ids = HashSet::new();
        let mut total_collateral = 0u128;
        for (index, line) in lines.enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let invalid = |field: &str| format!("row {}: invalid {field}", index + 2);
            let fields: Vec<_> = line.split(',').map(str::trim).collect();
            if fields.len() != 5 || line.contains('"') {
                return Err(invalid("CSV column count or quoting"));
            }
            if fields[0].is_empty() || fields[0].len() > 128 || !ids.insert(fields[0].to_owned()) {
                return Err(invalid("empty, long or duplicate loan id"));
            }
            if book.collateral.len() >= MAX_LOANS {
                return Err("loan count exceeds pilot limit".into());
            }
            let principal = fields[1]
                .parse::<u128>()
                .map_err(|_| invalid("principal"))?;
            let collateral = fields[2]
                .parse::<u128>()
                .map_err(|_| invalid("collateral"))?;
            let performing = match fields[3] {
                "performing" => true,
                "defaulted" => false,
                _ => return Err(invalid("status")),
            };
            let kyc = match fields[4] {
                "true" => true,
                "false" => false,
                _ => return Err(invalid("KYC flag")),
            };
            book.principal_total = book
                .principal_total
                .checked_add(principal)
                .ok_or("principal sum overflow")?;
            total_collateral = total_collateral
                .checked_add(collateral)
                .ok_or("collateral sum overflow")?;
            book.collateral.push(collateral);
            book.performing.push(performing);
            book.kyc.push(kyc);
        }
        if book.collateral.is_empty() {
            return Err("loan tape is empty".into());
        }
        Ok(book)
    }
    pub fn threshold_bps(&self, ratio_bps: u64) -> Result<u128> {
        if ratio_bps < 10_000 {
            return Err("ratio must be at least 10000 basis points".into());
        }
        let numerator = self
            .principal_total
            .checked_mul(ratio_bps as u128)
            .ok_or("threshold multiplication overflow")?;
        // Round UP to avoid accepting a fractionally undercollateralized claim.
        Ok(numerator / 10_000 + u128::from(numerator % 10_000 != 0))
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    version: u8,
    loan_count: usize,
    proving_key_sha256: String,
    verifying_key_sha256: String,
}
pub struct Parameters {
    pub loan_count: usize,
    pub proving_key: ProvingKey<Bn254>,
}
impl Parameters {
    pub fn create(loan_count: usize, directory: &Path) -> Result<Self> {
        if !(1..=MAX_LOANS).contains(&loan_count) {
            return Err("unsupported loan count".into());
        }
        if directory.exists() {
            return Err("parameter directory already exists; reuse it instead of replacing the approved setup".into());
        }
        let params = Self {
            loan_count,
            proving_key: setup_only(loan_count),
        };
        let mut pk_bytes = vec![];
        params
            .proving_key
            .serialize_compressed(&mut pk_bytes)
            .map_err(|e| e.to_string())?;
        let vk = params.key_bytes()?;
        let manifest = Manifest {
            version: 1,
            loan_count,
            proving_key_sha256: hash_hex(&pk_bytes),
            verifying_key_sha256: hash_hex(&vk),
        };
        create_private_directory(directory)?;
        write_new(&directory.join("proving-key.bin"), &pk_bytes, false)?;
        write_new(&directory.join("verifying-key.bin"), &vk, false)?;
        write_json(&directory.join("manifest.json"), &manifest, false)?;
        Ok(params)
    }
    pub fn load(directory: &Path) -> Result<Self> {
        let manifest: Manifest = read_json(&directory.join("manifest.json"))?;
        if manifest.version != 1 || !(1..=MAX_LOANS).contains(&manifest.loan_count) {
            return Err("unsupported parameter manifest".into());
        }
        let bytes = fs::read(directory.join("proving-key.bin")).map_err(|e| e.to_string())?;
        if hash_hex(&bytes) != manifest.proving_key_sha256 {
            return Err("proving-key hash mismatch".into());
        }
        let mut reader = bytes.as_slice();
        let pk =
            ProvingKey::<Bn254>::deserialize_compressed(&mut reader).map_err(|e| e.to_string())?;
        if !reader.is_empty() {
            return Err("trailing proving-key bytes".into());
        }
        let params = Self {
            loan_count: manifest.loan_count,
            proving_key: pk,
        };
        let key = params.key_bytes()?;
        if hash_hex(&key) != manifest.verifying_key_sha256
            || fs::read(directory.join("verifying-key.bin")).map_err(|e| e.to_string())? != key
        {
            return Err("verifying key and parameter manifest disagree".into());
        }
        Ok(params)
    }
    pub fn key_bytes(&self) -> Result<[u8; 640]> {
        gate_key(&self.proving_key.vk).map_err(str::to_owned)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProofBundle {
    pub version: u8,
    pub loan_count: usize,
    pub verifying_key_sha256: String,
    pub threshold: String,
    pub commitment_be: String,
    pub proof_uncompressed_be: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrivateAttestation {
    pub version: u8,
    pub nonce_be: String,
    pub source_sha256: String,
}
impl ProofBundle {
    pub fn approval(&self, sequence: u64, observed_at: i64, expires_at: i64) -> Result<Approval> {
        if self.version != 1 || !(1..=MAX_LOANS).contains(&self.loan_count) {
            return Err("unsupported proof bundle".into());
        }
        let _: [u8; 32] = decode_hex(&self.verifying_key_sha256)?;
        let request = Approval {
            sequence,
            observed_at,
            expires_at,
            threshold: self
                .threshold
                .parse::<u128>()
                .map_err(|_| "invalid threshold")?,
            commitment: decode_hex(&self.commitment_be)?,
            proof: decode_hex(&self.proof_uncompressed_be)?,
        };
        Approval::decode_instruction(&request.instruction()).map_err(str::to_owned)
    }
    /// Run by the servicer before co-signing. Private CSV and nonce never enter transaction data.
    pub fn check_attestation(&self, book: &Book, private: &PrivateAttestation) -> Result<()> {
        book.validate_shape()?;
        if private.version != 1
            || book.collateral.len() != self.loan_count
            || book.source_sha256 != private.source_sha256
        {
            return Err("private attestation does not match this loan tape".into());
        }
        let nonce: [u8; 32] = decode_hex(&private.nonce_be)?;
        if nonce >= FR_MODULUS_BE {
            return Err("non-canonical nonce".into());
        }
        let request = self.approval(1, 1, 2)?;
        let commitment = commit_book(
            &book.collateral,
            &book.performing,
            &book.kyc,
            Fr::from_be_bytes_mod_order(&nonce),
        );
        if fr_be(&commitment) != request.commitment {
            return Err("book commitment mismatch; do not co-sign".into());
        }
        Ok(())
    }
}
pub fn prove(
    params: &Parameters,
    book: &Book,
    threshold: u128,
) -> Result<(ProofBundle, PrivateAttestation)> {
    book.validate_shape()?;
    if book.collateral.len() != params.loan_count || threshold == 0 {
        return Err("book size or threshold does not match pilot parameters".into());
    }
    let nonce = random_nonce();
    let commitment = commit_book(&book.collateral, &book.performing, &book.kyc, nonce);
    let circuit = SolvencyCircuit {
        collateral: book.collateral.iter().map(|v| Some(*v)).collect(),
        performing: book.performing.iter().map(|v| Some(*v)).collect(),
        kyc: book.kyc.iter().map(|v| Some(*v)).collect(),
        nonce: Some(nonce),
        threshold: Some(threshold),
        commitment: Some(commitment),
        n: params.loan_count,
    };
    let cs = ConstraintSystem::<Fr>::new_ref();
    circuit
        .clone()
        .generate_constraints(cs.clone())
        .map_err(|e| e.to_string())?;
    let variables = cs.num_instance_variables() + cs.num_witness_variables();
    if params.proving_key.a_query.len() != variables
        || params.proving_key.b_g1_query.len() != variables
        || params.proving_key.b_g2_query.len() != variables
        || params.proving_key.l_query.len() != cs.num_witness_variables()
    {
        return Err("parameter circuit shape does not match the loan-count manifest".into());
    }
    if !cs.is_satisfied().map_err(|e| e.to_string())? {
        return Err("book fails the actual solvency/KYC circuit; no proof produced".into());
    }
    let proof = Groth16::<Bn254>::prove(&params.proving_key, circuit, &mut OsRng)
        .map_err(|e| e.to_string())?;
    let public = [Fr::from(threshold), commitment];
    if !Groth16::<Bn254>::verify(&params.proving_key.vk, &public, &proof)
        .map_err(|e| e.to_string())?
    {
        return Err("generated proof did not verify under persisted key".into());
    }
    Ok((
        ProofBundle {
            version: 1,
            loan_count: params.loan_count,
            verifying_key_sha256: hash_hex(&params.key_bytes()?),
            threshold: threshold.to_string(),
            commitment_be: hex(&fr_be(&commitment)),
            proof_uncompressed_be: hex(&gate_proof(&proof)),
        },
        PrivateAttestation {
            version: 1,
            nonce_be: hex(&fr_be(&nonce)),
            source_sha256: book.source_sha256.clone(),
        },
    ))
}
pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
pub fn hash_hex(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}
pub fn decode_hex<const N: usize>(text: &str) -> Result<[u8; N]> {
    if text.len() != N * 2 || !text.is_ascii() {
        return Err("invalid hex length or encoding".into());
    }
    let mut out = [0; N];
    for (i, byte) in out.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[i * 2..i * 2 + 2], 16)
            .map_err(|_| "invalid hexadecimal byte")?;
    }
    Ok(out)
}
pub fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}
pub fn write_json(path: &Path, value: &impl Serialize, private: bool) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    write_new(path, &bytes, private)
}
pub fn create_private_directory(path: &Path) -> Result<()> {
    let mut builder = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(path).map_err(|e| e.to_string())
}
fn write_new(path: &Path, bytes: &[u8], private: bool) -> Result<()> {
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(if private { 0o600 } else { 0o644 });
    }
    options
        .open(path)
        .and_then(|mut file| file.write_all(bytes))
        .map_err(|e| e.to_string())
}
