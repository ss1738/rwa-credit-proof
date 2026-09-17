# Developer Tooling Grant Proposal

Solana Foundation: Proof-of-Solvency for Tokenized Private Credit

**Draft for applicant review, 2026-09-17.** Prepared in the order of the Foundation's Developer
Tooling template. Confirm the commitments in section 5 and add a Telegram/X handle if desired
before copying this into a shared Google Doc. Deployment status is in `DEPLOYMENTS.md`.

## 1. Applicant Information

**Project / Tool Name:** Proof-of-Solvency for Tokenized Private Credit (rwa-credit-proof)

**Applicant / Organization:** Satyawan Singh, solo developer

**Primary Contact (name, email, Telegram/X):** Satyawan Singh; satyawansinghinuk@gmail.com;
Telegram/X not supplied, contact by email.

**Total Amount Requested (USD):** $22,000

**Relevant Experience & Track Record**

My background includes a from-scratch Rust blockchain with consensus, light clients and
data-availability sampling; BLS12-381 aggregate signatures; Nova recursive SNARKs; Groth16;
KZG/Verkle commitments; on-chain BLS (EIP-2537); and Coq/TLA+ formal verification.
Public work: https://github.com/ss1738.

This project's existing implementation includes a Rust loan-book circuit, a Solana SBF pairing
verifier and RPC client, adversarial checks, documentation of limitations, and a public Sepolia
verifier example. The 2026-09-17 reproduction passed all six demo stages and the separate
adversarial audit. The 10-loan audit circuit has 5,911 constraints; the SBF program-test harness
uses 83,354 CU. The native mint-gate model verifies both a proof and a servicer signature and
rejects a swapped book. Those combined checks are not yet an on-chain token-mint integration.

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
The existing Solana instruction is larger (961 bytes for two public inputs), because it carries
uncompressed proof points and the verifying key as well as public inputs.

The SBF verifier checks a BN254 pairing equation through Solana's `alt_bn128` syscalls. Its current
caller-supplied verifying key is suitable for a demonstration, but does not identify an authorized
solvency circuit. The developer-kit component will pin approved keys/circuit versions and validate
instruction structure. A consuming mint program must separately enforce threshold policy, freshness,
servicer authority/signature binding, and token mint authorization. The existing native example
models the proof/signature conjunction; it is not a deployed token program.

The setup component will select a maintained complete ceremony implementation compatible with this
circuit, document the phase-1/phase-2 boundary, and publish independently checkable contributions,
transcripts, and key hashes. The existing delta-only experiment is insufficient by itself.
Completion requires both a complete verified setup path and circuit/key compatibility tests.
A feasibility checkpoint precedes implementation; any necessary scope change goes back to the
Foundation rather than relabeling a partial ceremony as complete.

**Key features**

- Typed Rust proof/public-input APIs and versioned circuit/key artifacts.
- Solana proof-verifier example, devnet reproduction commands, signature-binding integration guide.
- Tests for invalid proofs, changed public inputs, wrong keys, malformed instructions, and swapped books.
- Public setup transcript verification, reproducible releases and a maintained security limitations record.

**Integration into existing developer workflows**

Developers clone the MIT repository, run `./demo.sh` and the separate audit, then run the RPC client
against their local validator or the recorded devnet deployment when available. The funded release
adds a documented crate interface and example consuming-program integration. Developers retain
control of their signed feeds, keys, threshold policy and deployment. Documentation will explicitly
separate a successful proof-verifier call from an authorized mint.

**Technology stack**

Rust, arkworks Groth16/BN254/R1CS/Poseidon, ed25519-dalek, Solana SBF and `alt_bn128`, Solana RPC
client and program-test, with a compatible maintained ceremony implementation selected at the
feasibility checkpoint. The existing Solidity/EVM example demonstrates portability; this grant's
deliverables focus on Solana tooling.

**Proof-of-Concept**

- Source and demo: https://github.com/ss1738/rwa-credit-proof (`./demo.sh`).
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

**Component 1: Solana developer kit, $5,000.** Target: weeks 1-4. Deliver a versioned library,
documented APIs/encodings, pinned circuit/key configuration, instruction validation, local/devnet
examples and signature/mint integration guidance. Acceptance: a clean checkout reproduces valid
verification, rejects invalid proofs/inputs/keys/malformed data, and passes documented integration
and compute-budget tests. Publish a beta release and reproducible build instructions.

**Component 2: Complete setup workflow, $12,000.** Target: weeks 1-12, including an early
compatibility checkpoint. Deliver a complete phase-1/phase-2 workflow using a suitable maintained
implementation, at least three separately operated contributions, verified public transcripts,
key hashes, circuit integration and a runbook. Test rejection of corrupted contributions and
mismatched circuit/key artifacts, then prove and verify with the final parameters. Publish the
production tooling release by the end of the grant agreement. Participant independence is a
recruitment dependency; ceremony soundness depends on at least one honest contributor and the
chosen protocol's assumptions. An external security audit and a production fund/token deployment
are not included in this budget or implied by this deliverable.

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
| 1 | Developer-kit beta | Versioned APIs, key pinning, validation, examples, positive/negative tests | 5,000 |
| 2 | Setup workflow and tooling release | Complete verified ceremony, 3 separate contributors, reproducible artifacts, production tooling release | 12,000 |
| 3 | Maintenance month 1 | Accepted fixes, compatibility checks and report | 500 |
| 4 | Maintenance month 2 | Accepted fixes, compatibility checks and report | 500 |
| 5 | Maintenance month 3 | Accepted fixes, compatibility checks and report | 500 |
| 6 | Maintenance month 4 | Accepted fixes, compatibility checks and report | 500 |
| 7 | Maintenance month 5 | Accepted fixes, compatibility checks and report | 500 |
| 8 | Maintenance month 6 | Accepted fixes, compatibility checks and report | 500 |
| 9 | Adoption | 4 independently evidenced integrations; $500 per accepted team | 2,000 |
| | **Total** | | **22,000** |

## 5. Acknowledgements

**Applicant confirmation required before submission.** These boxes are intentionally unconfirmed;
they are commitments for Satyawan Singh to review, not permissions granted by this draft.

- [ ] Publish a production version of the developer tooling by the end of the grant agreement.
- [ ] Keep the project completely public and open-source.
- [ ] Provide at least six months of maintenance.
- [ ] Meet the quantifiable user-adoption milestones above.

The current prototype is not production-safe. A tooling release does not certify any particular
loan book, feed provider, token policy or third-party deployment. A proper setup reduces one
soundness risk; it does not replace key pinning, authenticated feeds, integration testing or an
external security audit.
