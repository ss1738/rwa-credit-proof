# Caledren pilot alpha

Run a private loan book through a reusable prover, a servicer commitment check and a stateful
Solana approval program. This is an integration starting point for a credit protocol developer.
It creates a verifiable approval receipt; it does not mint tokens or verify real-world asset values.

## One-command evaluation

Requirements: Rust, the Solana SBF build tools, `solana-test-validator`, `solana-keygen`, curl,
and network access for first-time dependency downloads. The checked run used Solana CLI 4.1.1.

```bash
./pilot-demo.sh
```

The script tests the SDK and wire format, creates a 12-loan parameter set, proves the supplied
synthetic CSV at 120% of its principal total, checks the private commitment, builds the actual
SBF binary, runs adversarial runtime transactions, and starts a fresh local validator. The RPC
client creates an immutable policy, submits an approval signed by distinct payer and servicer
keys, waits for finalization and reads back the receipt. It also simulates wrong-signer and replay
rejections with signature verification enabled. The validator shuts down when the script exits.

Artifacts go in a fresh ignored `target/pilot-run.*` directory. Its `approval-report.json` contains
public addresses, signatures, the finalized transaction, sizes, costs and the negative simulation
results. The loan tape is synthetic, and both signer roles are operated by this local demo. This
is not evidence of an external partner or public devnet deployment.

Set `PILOT_RPC_PORT` and `PILOT_FAUCET_PORT` if the default 18997/18998 ports are occupied.
The script never changes the global Solana cluster configuration or uses a personal wallet.

## Reuse the approved parameters

```bash
# Pick a new local directory for the initial setup. It must not already exist.
cargo run --release --bin pilot -- setup 12 target/my-pilot-params

# Reuse that directory for every book of the same circuit size.
cargo run --release --bin pilot -- prove target/my-pilot-params \
  examples/loan_tape_solvent.csv '120%' target/my-first-proof

# Servicer independently checks its own copy of the CSV and the private nonce.
cargo run --release --bin pilot -- check-attestation examples/loan_tape_solvent.csv \
  target/my-first-proof/proof-bundle.json target/my-first-proof/private-attestation.json
```

`prove` accepts a positive integer threshold or the convenience value `120%`. Percentage thresholds
round up. All amounts must use the same integer unit. The circuit proves coverage of this threshold;
it does not prove that the threshold equals 120% of principal or current token supply. The configured
policy sets the independently approved minimum. Changing book size requires another setup and policy.

CSV header: `loan_id,principal,collateral_value,status,kyc_ok`. Use unquoted rows, unique nonempty
IDs, unsigned 128-bit amounts, `performing`/`defaulted`, and `true`/`false`. Unknown flags, duplicate
IDs, wrong column counts, overflow, empty tapes and more than 10,000 loans are rejected. The
servicer should use a reviewed mapping from its own servicing system into this schema.

The manifest hashes the proving key and exact verification-key bytes. Loading checks consistency;
proving checks circuit shape. These checks identify parameters and catch mismatches. A hash does
not prove that the setup was conducted honestly. Setup is still single-party with secure randomness.

## Public and private artifacts

| Artifact | Contents | Handling |
|---|---|---|
| `proof-bundle.json` | Version, loan count, approved-key hash, threshold, commitment, proof | Public; publish only when intended |
| `private-attestation.json` | Commitment blinding nonce and source CSV digest | Private; share only with the authorized servicer through a secure channel |
| Loan CSV | IDs, principal, collateral and flags | Private; stays outside transaction data |
| Parameter files | Proving key, verification key and manifest | Parameter distribution needs reviewed provenance; these files are not ceremony evidence |
| `approval-report.json` | Public receipt, signatures and runtime results | Suitable for the synthetic demo evidence bundle |

New output directories use owner-only permissions; the private attestation file uses mode 0600
on Unix. Permissions do not replace encryption or secure transport. The commitment covers ordered
collateral, performance/KYC flags and the nonce. It does not cover loan IDs or principal amounts.
The local source digest checks that the same CSV was supplied to the workflow; it is not an extra
on-chain commitment. The servicer must separately review IDs, amounts, valuation and threshold policy.

## Program integration contract

`caledren-gate-protocol` is the shared wire-format crate. Use `Policy::instruction()` and
`Approval::instruction()` instead of manually assembling bytes. The full working RPC example is
`client/src/gate.rs`; it intentionally accepts localhost only and uses ephemeral signers.

1. Approve a parameter set and its provenance, a servicer public key, a minimum threshold and a
   maximum validity duration of 1 to 86,400 seconds. The program stores these immutably.
2. Create a rent-exempt 816-byte account owned by the gate. Atomically send the 700-byte initialize
   instruction with `[configuration: writable + signer, authority: signer]`. Both signatures are
   required, including the new account's signature to prevent initialization front-running.
3. Prove with the persisted key. Have the servicer recompute the commitment from its independently
   checked bounded book and private nonce. It must review the exact policy account, threshold,
   next sequence and timestamps before signing the transaction.
4. Send the 332-byte approval instruction with `[configuration: writable, servicer: signer]`.
   The Solana runtime authenticates signer privileges. A configured PDA can also authorize through
   CPI; its controlling program then becomes part of the trust boundary. The direct demo uses
   an ordinary transaction signature from a separate servicer keypair.
5. Check finalization and decode the program-owned account using `Config::decode()`.

The gate requires the configured signer, threshold at least the configured minimum, exactly the
next sequence, a non-future and nondecreasing observation time, an unexpired deadline, and a
validity window within policy. It verifies the proof against the stored key and the threshold and
commitment from the request. It writes the receipt only after all checks pass.

A consuming program must pin **both the gate program ID and the approved configuration address**,
check the account owner, decode the exact format, require sequence greater than zero, check
`expires_at` against its current chain clock, and apply its own threshold and action policy.
If an approval should authorize only one action, the consumer must store a consumed sequence
and update it atomically. The gate's sequence protects approval updates, not repeated downstream
actions. Reading a previously accepted receipt does not guarantee it is still fresh.

Do not accept an arbitrary caller-selected configuration: an attacker can initialize its own
policy and key. Review deployment upgrade authority as well as the program ID. The gate stores
the latest receipt only; prior approvals remain in transaction history. Policy rotation uses a
new configuration and explicit consumer acceptance, not an in-place update.

## What to validate with a first external developer

Use the evaluation checklist in `PILOT_EVALUATION.md`. The immediate integration target is a
protocol's risk check against a synthetic or appropriately approved test book. A successful pilot
means another developer reproduces acceptance and rejection on their own machine and can explain
which data and authorities they trust. It does not establish production financial safety.

Remaining work includes running the externally contributed setup ceremony using the pinned
`iden3/snarkjs` 0.7.6 workflow, explicit circuit range hardening, production servicer/key operations,
live supply and SPL token policy, public deployment, consumer integration, independent security
review and externally evidenced adoption. The local rehearsal proves compatibility only; it is not
ceremony evidence. See `KNOWN_LIMITATIONS.md`.
