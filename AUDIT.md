# Audit trail

This project was checked for fabrication and soundness three independent ways. This file is the honesty
record: what was verified, what was fixed, and what is still open. Reproduce the empirical part with
`cargo run --release --bin audit`.

## 1. Empirical adversarial audit (`src/audit.rs`, run on hardware)

Checks that would FAIL if the zero-knowledge claims were fabricated (an under-constrained circuit, a
vacuous predicate, or a no-op verifier). Latest run:

- Circuit has **5,911 constraints** (a vacuous circuit would be ~0).
- Solvent + all-KYC + correct-commitment book **satisfies**; insolvent, KYC-fail, and wrong-commitment
  books each **fail** the constraint system (the constraints are genuinely enforced).
- Honest proof verifies; a proof with one flipped bit is **rejected**; an altered public input is
  **rejected** (the verifier is not a no-op).
- Public inputs are only 2 field elements (threshold + commitment); the loans are **not** public.
- On-chain verification measured twice and agrees: **83,354 CU** (solana-program-test) and **83,352 CU**
  (earlier local validator). The 2026-09-17 reproduction measured **83,354 CU** in both the SBF
  harness and a finalized transaction on a fresh local validator. These costs cover proof verification
  only. Logs and network scope: [DEPLOYMENTS.md](DEPLOYMENTS.md).

## 2. Independent code audit (first pass)

An independent reviewer read every source file and judged: **"genuine, working ZK cryptography, not
fabricated or hollow."** It confirmed the circuit really enforces solvency + all-KYC + commitment
binding over private witnesses (the same witnesses tie the collateral sum to the commitment), the Solana
verifier does real `alt_bn128` pairing checks, and the swap-book attack is genuinely blocked.

It also caught four issues the author had not flagged (see below).

## 3. Independent code audit (second pass, verifying the fixes)

After the four issues were addressed, a second independent review re-read the source and confirmed the
fixes are real and introduced no new bugs: *"genuinely in a more honest and sounder state than before...
documentation matches what the code enforces."* In particular it verified the critical property held:
the same collateral witnesses still feed both the solvency sum and the commitment after the nonce change.

## Findings and their status

| # | Finding | Status |
|---|---------|--------|
| 1 | Trusted setup used a fixed public seed (forgeable) | **Improved**: now uses secure randomness (no public seed). Full soundness needs a relying-party setup or a multi-party ceremony (a single party cannot self-attest). Documented, not overclaimed. |
| 2 | Commitment was binding but not hiding | **Fixed**: a random blinding nonce is absorbed natively and in-circuit; the commitment is now hiding. |
| 3 | No in-circuit range check on collateral | **Accepted with rationale**: the signed commitment pins the values (honest sums ~2^141 are far below the ~2^253 safety bound), under the intended honest-feed and authenticated-key model. The SBF verifier alone does not enforce that model (see scope corrections below). A bit-range check is deferred. |
| 4 | Pipeline BLOCK path was a cleartext check | **Fixed**: the block decision now runs the real circuit (`ConstraintSystem::is_satisfied`). |

## 2026-09-17 reproduction and scope corrections

All six stages of `./demo.sh` passed. The separate adversarial audit reproduced 5,911 constraints
for the 10-loan circuit and all expected positive/negative outcomes. The 10,000-loan benchmark
verified successfully: 64.61s setup, 72.17s proving, 128-byte compressed proof. These are one-machine
measurements, not performance guarantees. See `evidence/2026-09-17/` for logs and the local receipt.
The demo script now propagates unit-test failures instead of suppressing them with `|| true`.

Reading `solana-verifier/src/lib.rs` and `client/src/main.rs` also confirms that the deployed-program
example accepts its verifying key from instruction data. It checks the pairing equation, not whether
that key belongs to an approved solvency circuit. It neither authenticates the servicer nor mints
SPL tokens. At the time of this legacy reproduction, those combined checks existed only in the native demo. Input slices also assume
a valid encoding. These integration and validation gaps are now recorded in `KNOWN_LIMITATIONS.md`.
This documentation review is not a new independent security audit.

Sepolia's historical verification transaction is successful. Its event records 197,605 verifier gas;
the complete transaction used 224,234 gas. The contract verifies a baked-in example proof.
Solana devnet deployment remains pending faucet funding; no local address is represented as a
public deployment. See [DEPLOYMENTS.md](DEPLOYMENTS.md).

## 2026-09-17 pilot alpha regression evidence

The new `solana-gate/` is separate from the legacy benchmark. It stores the approved verification
key and servicer, requires runtime signer authorization, validates the instruction format, and
enforces minimum threshold, sequence and freshness before storing the latest approval receipt.
The SDK adds strict CSV ingestion, persisted parameter/hash checks and private commitment checking.

`./pilot-demo.sh` passed the SDK and protocol tests, the compiled SBF suite and a real finalized
local RPC approval. The SBF suite accepted two successive approvals using one key and rejected
26 invalid requests with the entire configuration account unchanged. These include unsigned
initialization, reinitialization, wrong/missing signer, wrong owner, uninitialized/read-only account,
low threshold, bad timestamps, skipped/replayed sequences, tampered proofs/public inputs,
an otherwise valid proof from another key, and malformed/noncanonical encodings.

The finalized 12-loan local approval used 85,529 CU in a 672-byte legacy transaction, including the
compute-budget instruction. Initialization with account creation fit in 1,057 bytes. These are
pilot measurements, separate from the 83,354-CU legacy pairing benchmark. The private CSV and
nonce did not enter instruction data. The demo operated both signer roles; it did not integrate
an external servicer or execute a token mint.

The separate `./ceremony-compat.sh` rehearsal now pins `iden3/snarkjs` 0.7.6 on Node 24 LTS and
passes the exact 6,648-constraint arkworks fixture through local phase 1 and phase 2 transcript
checks, arkworks proof verification and the compiled Solana gate. It also rejects altered public
inputs, corrupted contribution data and a mismatched circuit. The three rehearsal contributions
are controlled by one operator, so they establish compatibility only; Milestone 2b still requires
at least three independently operated contributors and a public beacon.

Evidence: `evidence/2026-09-17/pilot-alpha/`. This is developer-run regression evidence, not an
independent security audit. Setup, circuit range assumptions, consumer policy, deployment authority,
real data authenticity and external integration still need work. `PILOT_GUIDE.md` defines the
integration contract and `KNOWN_LIMITATIONS.md` records what it does not establish.

## Honest scope

This is a pilot alpha with locally reproduced tests and a public Sepolia verifier example.
It is not a production, third-party-security-audited system. A production mint gate needs a complete
trusted setup or suitable transparent proof system, reviewed parameter provenance and range bounds,
real servicer operations, a consumer/token integration and external review. The pilot's stored key,
signer policy and parser are implemented, but are not an independent assurance of those properties.
See [KNOWN_LIMITATIONS.md](KNOWN_LIMITATIONS.md).
