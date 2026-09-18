# Developer Tooling Grant Proposal

Solana Foundation: Proof-of-Solvency for Tokenized Private Credit

Prepared in the order of the Foundation's Developer Tooling template. Commitments in section 5
are confirmed by Satyawan Singh as of 2026-09-18. Deployment status is in `DEPLOYMENTS.md`.

## 1. Applicant Information

**Project / Tool Name:** Caledren: private-credit proof and approval tooling (rwa-credit-proof)

**Applicant / Organization:** Satyawan Singh, solo developer

**Primary Contact (name, email, Telegram/X):** Satyawan Singh; satyawansinghinuk@gmail.com;
Telegram/X not supplied, contact by email.

**Total Amount Requested (USD):** $22,000

**Relevant Experience & Track Record**

My background includes a from-scratch Rust blockchain with consensus, light clients and
data-availability sampling; BLS12-381 aggregate signatures; Nova recursive SNARKs; Groth16;
KZG/Verkle commitments; on-chain BLS (EIP-2537); and Coq/TLA+ formal verification.
Public work: https://github.com/ss1738.

The current pilot alpha includes strict CSV ingestion, persisted Groth16 parameters, typed proof
bundles, a private servicer commitment check, and a separate stateful Solana approval gate. The gate
stores an approved key and servicer, enforces threshold/freshness/sequence policy, and records the
latest receipt. Its compiled SBF suite accepts two successive approvals and rejects 26 invalid
requests while preserving state. A finalized local 12-loan approval used 85,529 CU in a 672-byte
legacy transaction signed by separate payer and servicer keys (a repeat run measured 85,538 CU).
`./pilot-demo.sh` reproduces the workflow; measured CU may vary slightly.

The existing six-stage demo and separate adversarial audit also pass. Their 10-loan circuit has
5,911 constraints and legacy pairing-only SBF harness uses 83,354 CU. A separate verifier example
is confirmed on public Sepolia. These programs do not mint tokens. Pilot code, evidence and the
site are pushed to `main` and public.

Repository: https://github.com/ss1738/rwa-credit-proof.
Audit record: https://github.com/ss1738/rwa-credit-proof/blob/main/AUDIT.md.
No production customers, third-party integrations or usage metrics are claimed.

## 2. Overview of Ecosystem Impact

**How is this project a public good for the Solana community?**

Solana developers working with private-credit data need a reusable example of proving a property
of a private loan book, serializing its proof, and verifying it within Solana's compute budget.
The grant funds an MIT-licensed developer kit and a reproducible setup workflow, with public
source, tests, documentation and release artifacts. It is available to any protocol and requires
no proprietary service or paid agreement with the applicant.

**Specific benefits to Solana developers**

- Start from a tested Groth16/BN254 reference for solvency, KYC flags and a blinded book commitment.
- Use typed proof encodings, a pinned circuit/verifying-key configuration, and examples that explain
  how to bind the servicer signature and mint policy in a consuming Solana program.
- Reproduce positive and negative verification tests and compute costs before adapting the design.
- Reuse documented ceremony/contribution verification and key-provenance checks.

The project's contribution is this narrow, inspectable private-credit implementation. Zyga is
adjacent broader institutional privacy work; the README preserves the related-work comparison.
This proposal does not claim that ZK solvency is a new concept or that competitors have no solutions.

## 3. Product Design

**Architecture & how it works**

A servicer-attested loan book supplies collateral, performance and KYC flags. A blinded Poseidon
commitment binds the private witnesses to a public commitment. The R1CS circuit enforces that
performing collateral covers a public threshold, every KYC flag is true, and the commitment matches.
Groth16 over BN254 produces a compressed 128-byte proof with two public field elements.
The pilot approval instruction is 332 bytes and its policy account is 816 bytes; proof points in
the instruction are uncompressed. The legacy pairing-only instruction remains 961 bytes because
it carries the key on each call. These are distinct interfaces and size measurements.

The gate validates strict instruction structure and checks the pairing equation through Solana's
`alt_bn128` syscalls using its stored approved key. Solana signer privileges authenticate the
configured servicer on the exact approval transaction, including the policy account, claim,
sequence and timestamps. A consumer must pin both program and policy account, check expiry and
apply its action policy. Live supply, one-time downstream action consumption and SPL mint authority
remain consuming-program work. The servicer must independently validate the bounded off-chain book.
The commitment includes collateral and flags, but not loan IDs or principal; the host's percentage
threshold calculation is not an in-circuit token-supply calculation.

The setup implementation is now pinned to [iden3/snarkjs 0.7.6](https://github.com/iden3/snarkjs/releases/tag/v0.7.6),
which supports BN128 Groth16 Powers of Tau and circuit-specific phase 2 workflows. The lockfile,
`ceremony-compat.sh`, `ceremony-toolchain/rehearse.mjs` and the Rust `ceremony-bridge` export the
existing arkworks R1CS, run the phase-1 and phase-2 transcript checks, verify the resulting proof
and key back in arkworks, and execute the final proof through the compiled Solana gate. The local
rehearsal also checks altered witnesses, public inputs, corrupted contribution data and a mismatched
circuit. This resolves the prior compatibility unknown for this circuit shape.

The rehearsal is deliberately not called a ceremony: all contributions are local and there are no
independent participants. The funded setup milestone begins with an external reproducibility and
launch gate, then runs a public phase-1/phase-2 ceremony with at least three separately operated
contributors and a precommitted beacon. It publishes the contribution transcript, hashes, final
key, circuit hash and verification report. If the launch gate fails, the remaining setup budget is
paused and the scope is returned to the Foundation before any partial transcript is described as a
completed ceremony. The existing delta-only experiment is insufficient by itself.

**Key features**

- Typed Rust proof/public-input APIs and versioned circuit/key artifacts.
- Solana proof-verifier example, devnet reproduction commands, signature-binding integration guide.
- Tests for invalid proofs, changed public inputs, wrong keys, malformed instructions, and swapped books.
- Public setup transcript verification, reproducible releases and a maintained security limitations record.

**Integration into existing developer workflows**

Developers clone the MIT repository and run `./pilot-demo.sh`, then follow `PILOT_GUIDE.md` to reuse
parameters and produce another approval. The legacy `./demo.sh` and separate audit remain available.
The funded release hardens and versions the crate interface, adds an example consuming-program
integration and public devnet reproduction. Developers retain
control of their signed feeds, keys, threshold policy and deployment. Documentation will explicitly
separate a successful proof-verifier call from an authorized mint.

**Technology stack**

Rust, arkworks Groth16/BN254/R1CS/Poseidon, ed25519-dalek, Solana SBF and `alt_bn128`, Solana RPC
client and program-test, plus the pinned iden3/snarkjs 0.7.6 ceremony tooling. The existing
Solidity/EVM example demonstrates portability; this grant's deliverables focus on Solana tooling.

**Proof-of-Concept**

- Source and demos: https://github.com/ss1738/rwa-credit-proof (`./pilot-demo.sh`, `./demo.sh`).
- Adversarial audit: `cargo run --release --bin audit`.
- Larger benchmark: `cargo run --release --bin bench -- 10000`.
- Sepolia example: https://sepolia.etherscan.io/tx/0x91196bd0a9b6d192733bdc7df7126520c142f2bbda2c146190ebddd75aaae7c7.
- Public Solana deployment: pending devnet faucet funding; no deployed devnet address claimed yet.
- Limitations: https://github.com/ss1738/rwa-credit-proof/blob/main/KNOWN_LIMITATIONS.md.

## 4. Budget Breakdown (Milestones)

Amounts below are proposed, payable on acceptance of the stated outcomes. Existing prototype work
is the starting point, not a request for retroactive payment. Proposed implementation window is
12 weeks from agreement, followed by six months of maintenance. No paid bespoke integration with
a named protocol is included. Adoption targets are future outcomes, not existing traction.

### 4a. Completed First Version (Beta), per component

**Component 1: Solana developer-kit release, $5,000.** Target: weeks 1-4. Build on the existing
pilot's persisted parameters, key pinning, signer policy and strict encodings. Deliver a versioned
library and migration policy, documented APIs, a consuming-program example with receipt pinning,
freshness and one-time action enforcement, public devnet examples, and CI/reproducible release
artifacts. Review circuit range hardening and document the final threat model. Acceptance: a clean
checkout reproduces valid verification, rejects invalid proofs/inputs/keys/malformed data and
consumer replay/configuration substitution, and passes documented compute-budget tests. Publish
the beta and integration guide. Existing pilot features are the baseline, not unpaid future scope.

**Component 2: Verified setup workflow, $12,000.** Target: weeks 1-12. The concrete tool is
iden3/snarkjs 0.7.6, pinned in `ceremony-toolchain/package-lock.json`, with the Rust bridge and
Solana compatibility test in this repository. The existing local rehearsal is baseline evidence,
not retroactive billing. The funded work has two acceptance gates:

1. **Compatibility and launch gate, $2,000, weeks 1-3.** A clean checkout reproduces the pinned
   phase-1/phase-2 local rehearsal (power 13, sufficient for the current 6,648-constraint fixture),
   exports the exact arkworks R1CS, verifies two independently generated witness instances and
   proofs in arkworks, rejects altered public inputs, corrupted contributions and a
   mismatched circuit, and runs the final proof through the compiled Solana gate. Publish a runbook,
   dependency/license record, circuit hash and key-conversion test. If this gate fails, pause the
   remaining setup funds and agree a scope change with the Foundation.
2. **Externally contributed ceremony and release, $10,000, weeks 4-12.** Run the same pinned
   workflow with at least three independently operated contributors, a publicly specified beacon,
   contribution identity records, independently verified phase-1 and phase-2 transcripts, final
   proving/verifying-key hashes, circuit compatibility checks and proofs accepted by arkworks and
   the Solana gate. Before launch, publish a participation request with the pinned client, exact
   commands and contribution-record format; accept a contributor only after an independently
   generated contribution and a separate operator record. If three independent operators are not
   confirmed by week 6, pause unreleased setup funds and return a rescope to the Foundation rather
   than calling a partial transcript complete. Publish the transcript, final artifacts, verification
   commands and a production tooling release. Ceremony soundness still depends on at least one
   honest contributor and the chosen protocol's assumptions. An external security audit and a
   production fund/token deployment are not included in this budget or implied by this deliverable.

### 4b. Maintenance, minimum 6 months

**Total: $3,000, paid as six monthly milestones of $500.** Start after the production tooling
release. Each month covers public issue triage, bug fixes, dependency/Solana compatibility checks,
reproduction of the documented test suite, documentation corrections and a public maintenance
report. Security fixes use the repository's disclosure process. Payment requires the month's
maintenance work to the Foundation's satisfaction.

### 4c. User Adoption

**Total: $2,000. Target: four independent external developer teams complete a reproducible local
or devnet integration within the six-month maintenance period.** No integrations are committed
or claimed today. Each team must run its own valid and invalid proof tests and provide an
inspectable integration artifact (public repository/PR or consented evidence accepted by the
Foundation). Forks without working integration and applicant-controlled projects do not count.

Track the count in a public evidence register with project owner, version, integration artifact,
test results, date and any devnet transaction. Each accepted team reaches 25% of the target and
unlocks $500 at the reporting period's end. Provide common documentation and issue support to all
participants rather than funding a private integration. This measures adoption of tooling, not
financial TVL or a claim of production safety.

### Milestone Summary Table

| # | Milestone / Deliverable | Success Criteria | Amount (USD) |
|---|---|---|---:|
| 1 | Developer-kit beta | Consumer integration, receipt pinning, one-time action checks, range review, devnet example, CI and release artifacts | 5,000 |
| 2a | Setup compatibility and launch gate | Clean-checkout snarkjs 0.7.6 rehearsal, arkworks/Solana cross-verification, negative artifact checks and go/no-go report | 2,000 |
| 2b | Externally contributed setup and release | Public phase 1 and phase 2, 3 independent contributors, beacon, verified transcripts, final hashes and production tooling release | 10,000 |
| 3 | Maintenance month 1 | Accepted fixes, compatibility checks and report | 500 |
| 4 | Maintenance month 2 | Accepted fixes, compatibility checks and report | 500 |
| 5 | Maintenance month 3 | Accepted fixes, compatibility checks and report | 500 |
| 6 | Maintenance month 4 | Accepted fixes, compatibility checks and report | 500 |
| 7 | Maintenance month 5 | Accepted fixes, compatibility checks and report | 500 |
| 8 | Maintenance month 6 | Accepted fixes, compatibility checks and report | 500 |
| 9 | Adoption | 4 independently evidenced integrations; $500 per accepted team | 2,000 |
| | **Total** | | **22,000** |

## 5. Acknowledgements

**Confirmed by Satyawan Singh, 2026-09-18.**

- [x] Publish a production version of the developer tooling by the end of the grant agreement.
- [x] Keep the project completely public and open-source.
- [x] Provide at least six months of maintenance.
- [x] Meet the quantifiable user-adoption milestones above.

The current prototype is not production-safe. A tooling release does not certify any particular
loan book, feed provider, token policy or third-party deployment. A proper setup reduces one
soundness risk; it does not replace key pinning, authenticated feeds, integration testing or an
external security audit.
