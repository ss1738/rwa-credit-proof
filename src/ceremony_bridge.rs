//! Compatibility rehearsal only: export the existing arkworks circuit and verify snarkjs artifacts.
//! The witness file is private. This does not turn a locally operated rehearsal into an MPC ceremony.
use ark_bn254::{Bn254, Fq, Fq2, Fr, G1Affine, G2Affine};
use ark_ff::{BigInteger, PrimeField};
use ark_groth16::{Groth16, Proof, VerifyingKey};
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystem, OptimizationGoal};
use ark_snark::SNARK;
use credit_solvency_core::{
    circuit::{commit_book, random_nonce, SolvencyCircuit},
    onchain_bytes::{fr_be, gate_key, gate_proof},
    pilot::{self, Book, ProofBundle},
};
use serde_json::{json, Value};
use std::{env, fs, io::Write, path::Path};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn u32_bytes(out: &mut Vec<u8>, value: usize) -> Result<()> {
    out.extend_from_slice(&u32::try_from(value)?.to_le_bytes());
    Ok(())
}
fn field_bytes(value: Fr) -> Vec<u8> {
    value.into_bigint().to_bytes_le()
}
fn section(out: &mut Vec<u8>, id: u32, data: &[u8]) {
    out.extend_from_slice(&id.to_le_bytes());
    out.extend_from_slice(&(data.len() as u64).to_le_bytes());
    out.extend_from_slice(data);
}
fn binary(magic: &[u8; 4], version: u32, sections: &[(u32, Vec<u8>)]) -> Vec<u8> {
    let mut out = magic.to_vec();
    out.extend_from_slice(&version.to_le_bytes());
    out.extend_from_slice(&(sections.len() as u32).to_le_bytes());
    for (id, bytes) in sections {
        section(&mut out, *id, bytes);
    }
    out
}
fn write_new(path: &Path, data: &[u8], private: bool) -> Result<()> {
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(if private { 0o600 } else { 0o644 });
    }
    options.open(path)?.write_all(data)?;
    Ok(())
}
fn export(csv: &Path, output: &Path) -> Result<()> {
    let book = Book::parse_csv(&fs::read_to_string(csv)?)?;
    let n = book.collateral.len();
    let threshold = book.threshold_bps(12_000)?;
    let nonce = random_nonce();
    let commitment = commit_book(&book.collateral, &book.performing, &book.kyc, nonce);
    let cs = ConstraintSystem::<Fr>::new_ref();
    cs.set_optimization_goal(OptimizationGoal::Constraints);
    SolvencyCircuit {
        collateral: book.collateral.iter().copied().map(Some).collect(),
        performing: book.performing.iter().copied().map(Some).collect(),
        kyc: book.kyc.iter().copied().map(Some).collect(),
        nonce: Some(nonce),
        threshold: Some(threshold),
        commitment: Some(commitment),
        n,
    }
    .generate_constraints(cs.clone())?;
    if !cs.is_satisfied()? {
        return Err("input fails the actual circuit".into());
    }
    cs.finalize();
    let matrices = cs.to_matrices().ok_or("matrices unavailable")?;
    let state = cs.borrow().ok_or("constraint system unavailable")?;
    if state.instance_assignment != vec![Fr::from(1u64), Fr::from(threshold), commitment] {
        return Err("unexpected public-input order".into());
    }
    let witness: Vec<Fr> = state
        .instance_assignment
        .iter()
        .chain(&state.witness_assignment)
        .copied()
        .collect();
    let mut header = vec![];
    u32_bytes(&mut header, 32)?;
    header.extend_from_slice(&Fr::MODULUS.to_bytes_le());
    u32_bytes(&mut header, witness.len())?;
    u32_bytes(&mut header, 0)?; // No public output wires.
    u32_bytes(&mut header, 2)?; // threshold, commitment, in that order.
                                // Every nonpublic wire is privately assigned by arkworks, including intermediate variables.
    u32_bytes(&mut header, state.witness_assignment.len())?;
    header.extend_from_slice(&(witness.len() as u64).to_le_bytes());
    u32_bytes(&mut header, matrices.num_constraints)?;
    let mut constraints = vec![];
    for i in 0..matrices.num_constraints {
        for matrix in [&matrices.a, &matrices.b, &matrices.c] {
            let mut row = matrix[i].clone();
            row.sort_by_key(|(_, index)| *index);
            u32_bytes(&mut constraints, row.len())?;
            for (coefficient, index) in row {
                u32_bytes(&mut constraints, index)?;
                constraints.extend_from_slice(&field_bytes(coefficient));
            }
        }
    }
    let labels = (0..witness.len() as u64)
        .flat_map(u64::to_le_bytes)
        .collect();
    let r1cs = binary(b"r1cs", 1, &[(1, header), (2, constraints), (3, labels)]);
    let mut witness_header = vec![];
    u32_bytes(&mut witness_header, 32)?;
    witness_header.extend_from_slice(&Fr::MODULUS.to_bytes_le());
    u32_bytes(&mut witness_header, witness.len())?;
    let witness_data = witness.into_iter().flat_map(field_bytes).collect();
    let wtns = binary(b"wtns", 2, &[(1, witness_header), (2, witness_data)]);
    pilot::create_private_directory(output)?;
    write_new(&output.join("circuit.r1cs"), &r1cs, false)?;
    write_new(&output.join("private.wtns"), &wtns, true)?;
    pilot::write_json(
        &output.join("expected-public.json"),
        &vec![threshold.to_string(), commitment.to_string()],
        false,
    )?;
    pilot::write_json(
        &output.join("metadata.json"),
        &json!({"schema_version":1,"loan_count":n,
        "constraints":matrices.num_constraints,"wires":state.instance_assignment.len()+state.witness_assignment.len(),
        "public_input_order":["threshold","commitment"],"r1cs_sha256":pilot::hash_hex(&r1cs),
        "scope":"existing arkworks circuit export; compatibility rehearsal, not a completed ceremony"}),
        false,
    )?;
    println!(
        "R1CS_EXPORTED loans={n} constraints={} hash={}",
        matrices.num_constraints,
        pilot::hash_hex(&r1cs)
    );
    Ok(())
}
fn field<F: PrimeField>(value: &Value) -> Result<F> {
    let text = value.as_str().ok_or("expected decimal field string")?;
    let parsed = F::from_str(text).map_err(|_| "invalid field element")?;
    if parsed.to_string() != text {
        return Err("noncanonical field element".into());
    }
    Ok(parsed)
}
fn g1(value: &Value) -> Result<G1Affine> {
    let p = value.as_array().ok_or("expected G1 array")?;
    if p.len() != 3 || p[2] != "1" {
        return Err("expected affine G1 point".into());
    }
    let point = G1Affine::new_unchecked(field::<Fq>(&p[0])?, field::<Fq>(&p[1])?);
    if !point.is_on_curve() || !point.is_in_correct_subgroup_assuming_on_curve() {
        return Err("invalid G1 point".into());
    }
    Ok(point)
}
fn fq2(value: &Value) -> Result<Fq2> {
    let p = value.as_array().ok_or("expected Fq2 array")?;
    if p.len() != 2 {
        return Err("invalid Fq2 length".into());
    }
    Ok(Fq2::new(field::<Fq>(&p[0])?, field::<Fq>(&p[1])?))
}
fn g2(value: &Value) -> Result<G2Affine> {
    let p = value.as_array().ok_or("expected G2 array")?;
    if p.len() != 3 || p[2] != json!(["1", "0"]) {
        return Err("expected affine G2 point".into());
    }
    let point = G2Affine::new_unchecked(fq2(&p[0])?, fq2(&p[1])?);
    if !point.is_on_curve() || !point.is_in_correct_subgroup_assuming_on_curve() {
        return Err("invalid G2 point".into());
    }
    Ok(point)
}
fn verify(fixture: &Path, artifacts: &Path, output: &Path) -> Result<()> {
    let vk_json: Value = pilot::read_json(&artifacts.join("verification-key.json"))?;
    let proof_json: Value = pilot::read_json(&artifacts.join("proof.json"))?;
    for value in [&vk_json, &proof_json] {
        if value["protocol"] != "groth16" || value["curve"] != "bn128" {
            return Err("unsupported proof protocol or curve".into());
        }
    }
    if vk_json["nPublic"] != 2 {
        return Err("expected exactly two public inputs".into());
    }
    let ic = vk_json["IC"].as_array().ok_or("missing IC")?;
    if ic.len() != 3 {
        return Err("expected three IC points".into());
    }
    let vk = VerifyingKey::<Bn254> {
        alpha_g1: g1(&vk_json["vk_alpha_1"])?,
        beta_g2: g2(&vk_json["vk_beta_2"])?,
        gamma_g2: g2(&vk_json["vk_gamma_2"])?,
        delta_g2: g2(&vk_json["vk_delta_2"])?,
        gamma_abc_g1: ic.iter().map(g1).collect::<Result<_>>()?,
    };
    let proof = Proof::<Bn254> {
        a: g1(&proof_json["pi_a"])?,
        b: g2(&proof_json["pi_b"])?,
        c: g1(&proof_json["pi_c"])?,
    };
    let strings: Vec<String> = pilot::read_json(&artifacts.join("public.json"))?;
    let expected: Vec<String> = pilot::read_json(&fixture.join("expected-public.json"))?;
    if strings.len() != 2 || strings != expected {
        return Err("public inputs differ from exported fixture".into());
    }
    let public: Vec<Fr> = strings
        .iter()
        .map(|s| field(&Value::String(s.clone())))
        .collect::<Result<_>>()?;
    if !Groth16::<Bn254>::verify(&vk, &public, &proof)? {
        return Err("snarkjs proof failed arkworks verification".into());
    }
    for i in 0..2 {
        let mut altered = public.clone();
        altered[i] += Fr::from(1u64);
        if Groth16::<Bn254>::verify(&vk, &altered, &proof)? {
            return Err("altered public input unexpectedly accepted".into());
        }
    }
    let meta: Value = pilot::read_json(&fixture.join("metadata.json"))?;
    let key = gate_key(&vk)?;
    let bundle = ProofBundle {
        version: 1,
        loan_count: meta["loan_count"].as_u64().ok_or("missing loan count")? as usize,
        verifying_key_sha256: pilot::hash_hex(&key),
        threshold: strings[0].clone(),
        commitment_be: pilot::hex(&fr_be(&public[1])),
        proof_uncompressed_be: pilot::hex(&gate_proof(&proof)),
    };
    // Ensures the threshold fits the gate's u128 and commitment is canonical before writing.
    bundle.approval(1, 1, 2)?;
    pilot::create_private_directory(output)?;
    write_new(&output.join("verifying-key.bin"), &key, false)?;
    pilot::write_json(&output.join("proof-bundle.json"), &bundle, false)?;
    println!(
        "CEREMONY_ARKWORKS_VERIFIED public_inputs=2 changed_inputs_rejected=2 vk_sha256={}",
        bundle.verifying_key_sha256
    );
    Ok(())
}
fn run() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        ["export",csv,output]=>export(Path::new(csv),Path::new(output)),
        ["verify",fixture,artifacts,output]=>verify(Path::new(fixture),Path::new(artifacts),Path::new(output)),
        _=>Err("usage: ceremony-bridge export <csv> <new-dir> | verify <fixture-dir> <snarkjs-artifacts> <new-wire-dir>".into()),
    }
}
fn main() {
    if let Err(error) = run() {
        eprintln!("ceremony-bridge: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn imported_fields_and_points_must_be_canonical() {
        assert!(field::<Fr>(&json!(Fr::MODULUS.to_string())).is_err());
        assert!(field::<Fq>(&json!("01")).is_err());
        assert!(field::<Fq>(&json!("-1")).is_err());
        assert!(g1(&json!(["0", "0", "1"])).is_err());
        assert!(g1(&json!(["1", "2", "1", "extra"])).is_err());
        assert!(g1(&json!(["1", "2", "0"])).is_err());
        assert!(g1(&json!(["1", "2", "1"])).is_ok());
        assert!(g2(&json!([["0", "0"], ["0", "0"], ["1", "0"]])).is_err());
    }
}
