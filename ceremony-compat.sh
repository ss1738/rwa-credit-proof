#!/usr/bin/env bash
# Existing arkworks circuit -> local snarkjs phase 1 + phase 2 -> arkworks -> compiled gate.
# This rehearses tooling compatibility; all new contributions are local, not independent participants.
set -euo pipefail
cd "$(dirname "$0")"
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
ceremony_node=$(command -v "${CEREMONY_NODE_BIN:-node}")
ceremony_node_major=$("$ceremony_node" -p 'process.versions.node.split(".")[0]')
if [[ "$ceremony_node_major" != 24 ]]; then
  printf 'Requires Node 24 LTS; this pinned snarkjs rehearsal does not support Node %s. Set CEREMONY_NODE_BIN to a Node 24 executable.\n' "$ceremony_node_major" >&2; exit 1
fi
ceremony_node_dir=$(dirname "$ceremony_node")
export PATH="$ceremony_node_dir:$PATH"
mkdir -p target
run_dir=$(mktemp -d "$PWD/target/ceremony-compat.XXXXXX")
printf 'Ceremony compatibility run: %s\n' "$run_dir"
build_target_dir="$PWD/target/ceremony-build"
npm --cache "$PWD/target/npm-cache" --prefix ceremony-toolchain ci --ignore-scripts --no-audit --no-fund
CARGO_TARGET_DIR="$build_target_dir" cargo run --release --locked --bin ceremony-bridge -- export examples/loan_tape_solvent.csv "$run_dir/fixture-1"
CARGO_TARGET_DIR="$build_target_dir" cargo run --release --locked --bin ceremony-bridge -- export examples/loan_tape_solvent.csv "$run_dir/fixture-2"
# Different circuit shape for the mismatched-circuit rejection, using public synthetic data only.
head -n 12 examples/loan_tape_solvent.csv > "$run_dir/different-shape.csv"
CARGO_TARGET_DIR="$build_target_dir" cargo run --release --locked --bin ceremony-bridge -- export "$run_dir/different-shape.csv" "$run_dir/fixture-mismatch"
"$ceremony_node" ceremony-toolchain/rehearse.mjs "$run_dir"
for i in 1 2; do
  CARGO_TARGET_DIR="$build_target_dir" cargo run --release --locked --bin ceremony-bridge -- verify "$run_dir/fixture-$i" "$run_dir/snarkjs-$i" "$run_dir/wire-$i"
done
(cd solana-gate && cargo-build-sbf)
CEREMONY_COMPAT_DIR="$run_dir" SBF_OUT_DIR="$PWD/solana-gate/target/deploy" \
  CARGO_TARGET_DIR="$PWD/solana-gate/target/host" RUST_LOG=error \
  cargo test --release --locked --manifest-path solana-gate/Cargo.toml --test ceremony -- --nocapture
printf 'CEREMONY_COMPATIBILITY_PASSED\nReport: %s/ceremony-report.json\n' "$run_dir"
printf 'Local rehearsal only; contributor recruitment and a real public phase-2 ceremony remain future work.\n'
