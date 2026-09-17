# Proof-of-Solvency for Tokenized Private Credit

> A fund proves it is solvent and fully KYC'd, over a loan book it cannot fake, **without revealing a
> single loan**, and the native mint-gate model allows minting only when that proof and the servicer's signature both check out.

Working research prototype. `./demo.sh` reproduces the core tests and Solana SBF measurement.
The verifier is confirmed on a local Solana test validator, and an EVM example is live on public
Ethereum Sepolia. **Solana devnet deployment is pending faucet funding.** See
[DEPLOYMENTS.md](DEPLOYMENTS.md) for network-specific evidence and reproduction commands.

## The problem

Private credit is the largest tokenized real-world-asset category (~$20.5B on-chain, on a $1.8 to $3.14
trillion off-chain market), but the loan book that backs the token lives off-chain. So the question that
matters most, *"prove this token is over-collateralised right now without showing me the private book"*,
has no good answer today. Smart-contract audits verify the code, not the asset state.

## What it proves

A zero-knowledge proof attests, over a **private** loan book, that:

1. **Solvent**: performing collateral covers the required over-collateralisation of the token supply
2. **Compliant**: every borrower passed KYC
3. **Authentic**: the book hashes (Poseidon) to a commitment the **servicer signed**

revealing only the threshold and the commitment. The native mint-gate model checks **both** the ZK proof and the
servicer's ed25519 signature over that commitment, so a prover cannot swap in a different book: a
different book has a different commitment the servicer never signed.

## Results (measured)

| Property | Result |
|---|---|
| Proof size | **128 bytes**, constant regardless of book size |
| Privacy | individual loans never revealed |
| On-chain verification | **83,354 CU** in both the SBF harness and a fresh local validator run (2026-09-17); earlier local-validator result: **83,352 CU**. About 6% of the 1.4M maximum, proof verification only. |
| On-chain cost vs book size | **constant** (a 10-loan and a 10,000-loan fund cost the same to verify) |
| Scale | 10,000 loans: **72.17s** proving plus **64.61s** setup; 1,000 loans: **3.82s** proving (2026-09-17 run) |
| Portability | verifies on Solana (`alt_bn128`) and any EVM chain (`ecPairing`) |
| Attack (swap book, reuse signature) | blocked |

## Related work

ZK-based solvency and compliance proofs for RWA are an active, recognized need in the ecosystem, not a
novel idea invented here: see Chainlink's and zk.me's write-ups on ZK compliance for institutional
finance, and zkVerify/zkOrigo's compliance-scoring work. The contribution here is not the concept; it's
a concrete, working, narrowly-scoped implementation of it for one specific asset class.

The closest adjacent project is **Zyga** (formerly Darklake, now developed under SOL Strategies since
its April 2026 acquisition), a dynamic zero-knowledge proof system for Solana with a published
construction (IACR ePrint 2025/1802) that lists solvency attestation among several use cases (alongside
FATF travel-rule proofs and MEV-resistant private execution). It's a different bet than this project:

| | This project | Zyga |
|---|---|---|
| Scope | One asset class: private-credit loan-book solvency | General institutional privacy layer |
| License | Open source (MIT), integrate today | Commercial, institution-facing |
| Status | Concrete Groth16 circuit, deployed and measured on a local Solana test validator and live on public Ethereum Sepolia | Published research construction; no shipped private-credit reference integration found at time of writing |
| Who it's for | Any RWA/private-credit protocol wanting a drop-in primitive | Large institutions adopting Solana broadly |

These aren't mutually exclusive: a protocol could use Zyga's general privacy rails for MEV protection or
travel-rule compliance while using this narrow, open-source primitive specifically for per-mint solvency
verification, or use this as the lightweight, inspectable alternative if a general commercial platform
isn't the right fit yet. If that competitive picture changes, this section should be updated rather than
left to go stale.

## Run it

```bash
./demo.sh
```

Runs the whole chain: statement soundness, realistic loan-tape decisions, the ZK proof, the
native mint-gate model plus the attack test, the 100/1,000-loan benchmark, and (if the Solana toolchain is installed) the real
on-chain compute-unit measurement. Needs Rust; stage 6 needs the Solana toolchain. Run
`cargo run --release --bin audit` for the separate adversarial audit and
`cargo run --release --bin bench -- 10000` for the larger benchmark. Public deployment receipts
are checked separately; `./demo.sh` does not redeploy to either public network.

## How it works

- **The statement** (`src/circuit.rs`): an R1CS circuit: sum of performing collateral `>=` threshold,
  every KYC flag true, and the book's Poseidon commitment equals a public input. Groth16 over BN254.
- **The binding** (`commit_book` + `src/onchain_bytes.rs`): the servicer signs the Poseidon commitment;
  the proof proves the private book hashes to it. This is the answer to the "garbage-in" problem: the
  proof certifies the **attested** book, not arbitrary numbers.
- **The on-chain verifier** (`solana-verifier/`): verifies a Groth16 pairing equation via Solana's
  `alt_bn128` syscalls. It accepts a caller-supplied verifying key and does not check the servicer
  signature or invoke a token mint. `src/onchain.rs` models the combined gate natively with
  ed25519-dalek. Production integration needs an approved circuit/key, authenticated signature
  binding, threshold/freshness policy and token authorization. BN254 also permits EVM verification
  through `ecPairing`; the Sepolia example contains its own baked-in proof.

## Repo layout

```
src/lib.rs            servicer-signed loan tape + native predicate (evaluate)
src/circuit.rs        the ZK statement (Groth16 R1CS) + prove_solvency + Poseidon commitment
src/onchain_bytes.rs  serialize proof/vk/public inputs into the on-chain byte layout
src/prove.rs          ZK proof + soundness checks            (bin: prove)
src/onchain.rs        native alt_bn128 mint-gate + attack     (bin: onchain)
src/pipeline.rs       CSV loan tape -> mint decision          (bin: pipeline)
src/bench.rs          prover cost vs book size                (bin: bench)
solana-verifier/      on-chain Groth16 verifier program (SBF) + compute-unit test
client/               RPC client: sends a real proof tx to a deployed program
examples/             realistic loan-tape CSVs (solvent, insolvent)
DESIGN_PARTNER_BRIEF.md / brief.html   the one-pager
DESIGN_PARTNER_TARGETS.md / OUTREACH_DRAFTS.md   who to pilot with
PRIVATE_CREDIT_SPIKE.md   the 60-day plan
```

## What is proven, and what is not

**Reproduced:** honest solvent books satisfy the circuit; insolvent, KYC-failed and wrong-commitment
books fail the audit checks. Honest proofs verify, while a flipped proof bit or changed public input
is rejected. The native signature-binding model rejects a swapped book. These checks do not establish
soundness against an adversary controlling setup parameters or the verifying key.

**Limits:** a proof certifies a predicate over supplied data, not whether the off-chain data is true.
The circuit enforces KYC flags, not an independent real-world KYC assessment. The current setup uses
secure randomness, but is single-party. The blinded commitment and circuit-based BLOCKED path are
implemented; older descriptions saying otherwise were stale. The delta-only ceremony experiment is
not a complete ceremony. The Solana verifier's key is supplied by the caller and it has no on-chain
servicer-signature or token-mint enforcement. See [KNOWN_LIMITATIONS.md](KNOWN_LIMITATIONS.md) and
[AUDIT.md](AUDIT.md). This is not a production or third-party-security-audited system.

*Figures: on-chain market size rwa.xyz; private-credit market size IMF (Apr 2024) and JPMorgan (2024);
performance measured on Apple M4 hardware and the Solana SBF runtime.*
