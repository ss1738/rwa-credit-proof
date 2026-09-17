//! See PILOT_GUIDE.md. Never send private-attestation.json to a public chain or a grant attachment.
use credit_solvency_core::pilot::{self, Book, Parameters, PrivateAttestation, ProofBundle};
use std::{env, fs, path::Path, process};
fn run() -> Result<(), String> {
    let args: Vec<_> = env::args().skip(1).collect();
    match args.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        ["setup",count,directory] => {
            let params=Parameters::create(count.parse().map_err(|_|"invalid loan count")?,Path::new(directory))?;
            println!("PARAMETERS_READY loans={} vk_sha256={}",params.loan_count,pilot::hash_hex(&params.key_bytes()?));
            println!("Single-party setup. Approval of this key does not establish a complete ceremony.");
        }
        ["prove",directory,csv,threshold,output] => {
            let params=Parameters::load(Path::new(directory))?;
            let book=Book::parse_csv(&fs::read_to_string(csv).map_err(|e|e.to_string())?)?;
            let threshold=if *threshold=="120%" { book.threshold_bps(12_000)? } else { threshold.parse().map_err(|_|"invalid threshold")? };
            let (bundle,private)=pilot::prove(&params,&book,threshold)?;
            pilot::create_private_directory(Path::new(output))?;
            pilot::write_json(&Path::new(output).join("proof-bundle.json"),&bundle,false)?;
            pilot::write_json(&Path::new(output).join("private-attestation.json"),&private,true)?;
            println!("PROOF_READY loans={} threshold={} vk_sha256={}",bundle.loan_count,bundle.threshold,bundle.verifying_key_sha256);
            println!("Public: {output}/proof-bundle.json; private servicer witness: {output}/private-attestation.json");
        }
        ["check-attestation",csv,bundle,private] => {
            let book=Book::parse_csv(&fs::read_to_string(csv).map_err(|e|e.to_string())?)?;
            let bundle:ProofBundle=pilot::read_json(Path::new(bundle))?;
            let private:PrivateAttestation=pilot::read_json(Path::new(private))?;
            bundle.check_attestation(&book,&private)?;
            println!("ATTESTATION_MATCHES: independently recomputed the private book commitment");
        }
        _=>return Err("usage: pilot setup <loans> <new-params-dir> | prove <params-dir> <csv> <threshold|120%> <new-output-dir> | check-attestation <csv> <proof-bundle.json> <private-attestation.json>".into()),
    }
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("pilot: {error}");
        process::exit(1);
    }
}
