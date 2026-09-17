use credit_solvency_core::pilot::{self, Book, Parameters};
use std::{fs, path::PathBuf};
fn csv(row: &str) -> String {
    format!("loan_id,principal,collateral_value,status,kyc_ok\n{row}\n")
}
fn directory() -> PathBuf {
    std::env::temp_dir().join(format!(
        "caledren-sdk-{}-{}",
        std::process::id(),
        rand::random::<u64>()
    ))
}

#[test]
fn rejects_ambiguous_or_overflowing_feeds() {
    for text in [
        String::new(),
        csv(""),
        csv("A,1,2,unknown,true"),
        csv("A,1,2,performing,yes"),
        csv("A,1,2,performing,true,extra"),
        csv("A,1,2,performing,true\nA,1,2,performing,true"),
        csv("A,-1,2,performing,true"),
        csv("A,1,340282366920938463463374607431768211455,performing,true\nB,1,1,performing,true"),
        "wrong,header\nA,1,2,performing,true".into(),
    ] {
        assert!(Book::parse_csv(&text).is_err(), "invalid feed accepted");
    }
    let book = Book::parse_csv(&csv("A,1,2,performing,true")).unwrap();
    assert_eq!(book.threshold_bps(12_000).unwrap(), 2, "coverage rounds up");
}

#[test]
fn parameters_are_reused_and_private_attestation_is_checked() {
    let dir = directory();
    let first = Parameters::create(2, &dir).unwrap();
    let approved_key = first.key_bytes().unwrap();
    assert!(
        Parameters::create(2, &dir).is_err(),
        "must not overwrite approved parameters"
    );
    let loaded = Parameters::load(&dir).unwrap();
    let text = csv("A,10,30,performing,true\nB,10,30,performing,true");
    let book = Book::parse_csv(&text).unwrap();
    let (one, private) = pilot::prove(&loaded, &book, 24).unwrap();
    one.check_attestation(&book, &private).unwrap();
    let (two, _) = pilot::prove(&loaded, &book, 24).unwrap();
    assert_eq!(one.verifying_key_sha256, two.verifying_key_sha256);
    assert_eq!(one.verifying_key_sha256, pilot::hash_hex(&approved_key));
    assert_ne!(
        one.commitment_be, two.commitment_be,
        "fresh private blinding per proof"
    );
    let changed = Book::parse_csv(&text.replace("A,10,30", "A,10,31")).unwrap();
    let mut attacker_private = private.clone();
    attacker_private.source_sha256 = changed.source_sha256.clone();
    assert!(
        one.check_attestation(&changed, &attacker_private).is_err(),
        "new CSV digest cannot bypass commitment binding"
    );
    let insolvent = Book::parse_csv(&text.replace(",30,", ",1,")).unwrap();
    assert!(pilot::prove(&loaded, &insolvent, 24).is_err());
    let failed_kyc = Book::parse_csv(&text.replace("true", "false")).unwrap();
    assert!(pilot::prove(&loaded, &failed_kyc, 24).is_err());
    let mut malformed = book.clone();
    malformed.kyc.pop();
    assert!(pilot::prove(&loaded, &malformed, 24).is_err());
    assert!(one.check_attestation(&malformed, &private).is_err());
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(dir.join("manifest.json")).unwrap()).unwrap();
    manifest["loan_count"] = serde_json::json!(1);
    fs::write(
        dir.join("manifest.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    let mismatched = Parameters::load(&dir).unwrap();
    let smaller = Book::parse_csv(&csv("A,10,30,performing,true")).unwrap();
    assert!(pilot::prove(&mismatched, &smaller, 24)
        .unwrap_err()
        .contains("circuit shape"));
    let json = serde_json::to_string(&one).unwrap();
    assert!(!json.contains("nonce") && !json.contains("collateral") && !json.contains("principal"));
    let mut bytes = fs::read(dir.join("proving-key.bin")).unwrap();
    bytes[0] ^= 1;
    fs::write(dir.join("proving-key.bin"), bytes).unwrap();
    assert!(Parameters::load(&dir).is_err());
    fs::remove_dir_all(dir).unwrap();
}
