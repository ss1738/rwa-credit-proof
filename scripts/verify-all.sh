#!/usr/bin/env bash
# One-command verification. Runs everything that the installed tools allow and says what was skipped.
# Never deploys, never touches a public network, never uses a personal wallet.
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"

out="${VERIFY_OUT:-target/verify-$(date -u +%Y%m%dT%H%M%SZ)}"
mkdir -p "$out"
summary="$out/summary.txt"
: > "$summary"
log() { printf '%s\n' "$*" | tee -a "$summary"; }
have() { command -v "$1" >/dev/null 2>&1; }

log "verify-all started $(date -u +%FT%TZ)"
log "commit: $(git rev-parse HEAD 2>/dev/null || echo unknown)"
log "dirty files: $(git status --porcelain 2>/dev/null | wc -l | tr -d ' ')"
have cargo || { log "FAIL: cargo not found"; exit 1; }
log "cargo: $(cargo --version)"
log "rustc: $(rustc --version)"
have solana && log "solana: $(solana --version)" || log "solana: not installed"
have cargo-build-sbf && log "cargo-build-sbf: present" || log "cargo-build-sbf: not installed"
have node && log "node: $(node --version)" || log "node: not installed"

step() { log ""; log "== $1 =="; }

step "1. Host tests: gate-protocol"
cargo test --release --locked --manifest-path gate-protocol/Cargo.toml 2>&1 | tee "$out/gate-protocol.log" | grep -E '^test |test result' | tee -a "$summary"

step "2. Host tests: root crate (circuit, native model, SDK, ceremony bridge)"
cargo test --release --locked 2>&1 | tee "$out/root.log" | grep -E '^test |test result' | tee -a "$summary"

step "3. Website build and checks"
if have node && have npm; then
  npm run build 2>&1 | tee "$out/site-build.log" | tail -n 2 | tee -a "$summary"
  npm run check 2>&1 | tee "$out/site-check.log" | tail -n 2 | tee -a "$summary"
else
  log "SKIPPED: node/npm not installed"
fi

step "4. Solana gate: compiled SBF suite and local validator"
if have cargo-build-sbf && have solana-test-validator && have solana-keygen && have curl; then
  ./pilot-demo.sh 2>&1 | tee "$out/pilot-demo.log" | grep -E 'GATE_|PILOT_|APPROVAL_|LOCAL_APPROVAL' | tee -a "$summary"
else
  log "SKIPPED: needs cargo-build-sbf, solana-test-validator, solana-keygen and curl."
  log "         The 26-rejection SBF suite and the local-validator approval were NOT run."
fi

log ""
log "verify-all finished $(date -u +%FT%TZ). Logs: $out"
