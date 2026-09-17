#!/usr/bin/env bash
# Reproduce the pilot alpha locally. Uses only synthetic CSV data and ephemeral local signers.
set -euo pipefail
cd "$(dirname "$0")"
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
mkdir -p target
run_dir=$(mktemp -d "$PWD/target/pilot-run.XXXXXX")
rpc_port=${PILOT_RPC_PORT:-18997}
faucet_port=${PILOT_FAUCET_PORT:-18998}
rpc_url="http://127.0.0.1:$rpc_port"
validator_pid=""
cleanup() { if [[ -n "$validator_pid" ]]; then kill "$validator_pid" 2>/dev/null || true; wait "$validator_pid" 2>/dev/null || true; fi; }
trap cleanup EXIT
printf 'Pilot run: %s\n' "$run_dir"
for binary in cargo cargo-build-sbf solana-test-validator solana-keygen curl; do
  command -v "$binary" >/dev/null || { printf 'Missing tool: %s\n' "$binary" >&2; exit 1; }
done
if curl --silent --max-time 1 "$rpc_url/health" >/dev/null; then
  printf 'RPC port already in use; set PILOT_RPC_PORT and PILOT_FAUCET_PORT.\n' >&2; exit 1
fi
printf '\n1/5 SDK and wire-format checks\n'
cargo test --release --test pilot_sdk
cargo test --manifest-path gate-protocol/Cargo.toml
printf '\n2/5 Reusable setup, strict CSV ingestion and private attestation\n'
cargo run --release --bin pilot -- setup 12 "$run_dir/params"
cargo run --release --bin pilot -- prove "$run_dir/params" examples/loan_tape_solvent.csv '120%' "$run_dir/proof"
cargo run --release --bin pilot -- check-attestation examples/loan_tape_solvent.csv "$run_dir/proof/proof-bundle.json" "$run_dir/proof/private-attestation.json"
printf '\n3/5 Compiled Solana gate and adversarial transactions\n'
(cd solana-gate && cargo-build-sbf)
SBF_OUT_DIR="$PWD/solana-gate/target/deploy" CARGO_TARGET_DIR="$PWD/solana-gate/target/host" \
  RUST_LOG=error cargo test --release --manifest-path solana-gate/Cargo.toml --test gate -- --nocapture
printf '\n4/5 Isolated local validator\n'
program_id=$(solana-keygen pubkey solana-gate/target/deploy/caledren_solana_gate-keypair.json)
solana-test-validator --ledger "$run_dir/ledger" --bind-address 127.0.0.1 \
  --rpc-port "$rpc_port" --faucet-port "$faucet_port" \
  --bpf-program "$program_id" solana-gate/target/deploy/caledren_solana_gate.so \
  --quiet > "$run_dir/validator.log" 2>&1 &
validator_pid=$!
ready=false
for _ in $(seq 1 60); do
  if ! kill -0 "$validator_pid" 2>/dev/null; then cat "$run_dir/validator.log" >&2; exit 1; fi
  if [[ $(curl --silent --max-time 1 "$rpc_url/health" || true) == ok ]]; then ready=true;break; fi
  sleep 1
done
if [[ "$ready" != true ]]; then printf 'Local validator did not become ready.\n' >&2; exit 1; fi
printf '\n5/5 Initialize approved policy, co-sign and finalize approval\n'
cargo run --release --manifest-path client/Cargo.toml -- pilot-demo "$rpc_url" "$program_id" \
  "$run_dir/params" examples/loan_tape_solvent.csv "$run_dir/proof" "$run_dir/approval-report.json"
printf '\nPILOT_DEMO_PASSED\nPublic receipt: %s/approval-report.json\n' "$run_dir"
printf 'LOCAL ONLY. Private attestation and ledger stay under ignored target/.\n'
