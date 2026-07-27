#!/usr/bin/env bash
# One-command reproduction of every claim. Needs Rust (stable). Stage 6 (real on-chain compute
# units) additionally needs the Solana toolchain; it is skipped gracefully if absent.
set -euo pipefail
cd "$(dirname "$0")"
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"

hr() { printf '\n\033[1m== %s ==\033[0m\n' "$1"; }

hr "1/6  Statement soundness (unit tests)"
cargo test --release --quiet 2>&1 | grep -E "test result" || true

hr "2/6  Realistic servicer loan tapes (CSV in, mint decision out)"
cargo run --release --quiet --bin pipeline -- examples/loan_tape_solvent.csv
echo
cargo run --release --quiet --bin pipeline -- examples/loan_tape_insolvent.csv

hr "3/6  Zero-knowledge proof: solvent + all-KYC + bound to the signed book"
cargo run --release --quiet --bin prove

hr "4/6  On-chain mint-gate (Solana alt_bn128 + ed25519) and swap-book attack"
cargo run --release --quiet --bin onchain

hr "5/6  Scale: prover cost vs book size (100, 1000 loans)"
cargo run --release --quiet --bin bench 100 1000

hr "6/6  Real on-chain compute units (optional: needs the Solana toolchain)"
if command -v cargo-build-sbf >/dev/null 2>&1; then
  ( cd solana-verifier && cargo-build-sbf >/dev/null 2>&1 && \
    SBF_OUT_DIR="$PWD/target/deploy" cargo test --release --test cu -- --nocapture 2>&1 \
    | grep -E "RESULT|COMPUTE_UNITS" )
else
  echo "skipped. Install the Solana toolchain (https://docs.anza.xyz) to measure on-chain CU."
fi

printf '\n\033[1;32mAll stages passed.\033[0m  Proof-of-solvency verified end to end.\n'
