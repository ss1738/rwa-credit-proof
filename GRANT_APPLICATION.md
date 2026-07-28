# Solana Foundation grant application (draft)

Apply at solana.org/grants-funding (rolling, decisions ~3 weeks). Fits the "convertible grant"
(commercial) or open-source public-good track. Fill `https://github.com/ss1738/rwa-credit-proof` and contact details before sending.
NOTE: taking grant money is an earning event, confirm your visa/entity position first (see end).

---

## Project
**Proof-of-Solvency for Tokenized Private Credit** (open-source ZK verification primitive)

## One line
An open-source, on-chain zero-knowledge verifier that lets a tokenized private-credit fund prove it is
solvent and fully KYC'd, without revealing its loan book, before a token can mint.

## The problem it solves for the Solana ecosystem
Private credit is the largest tokenized real-world-asset category (~$20B on-chain, and Solana RWA is
growing fast), but the loan book backing each token lives off-chain and cannot be verified on-chain.
Investors, counterparties, and regulators have no cryptographic way to check solvency. This is a
blocker to institutional RWA capital coming on-chain. A reusable proof-of-solvency primitive removes it.

## Why Solana specifically
The verifier uses Solana's `alt_bn128` pairing syscalls to check a Groth16/BN254 proof on-chain in
**~83,000 compute units (measured on a live validator), about 6% of the per-transaction budget, and the
cost is constant regardless of loan-book size**. This makes continuous, per-mint solvency verification
economically viable on Solana in a way it is not on higher-fee chains. Solana's growing RWA/private-
credit protocols (Credix, Huma, and others) are the direct beneficiaries.

## Public good / open source
The core (circuit, on-chain verifier, prover, reproducible demo, and the full audit trail) is open
source. Any Solana RWA protocol can integrate it. We commit to open-sourcing the developer kit produced
under this grant so the primitive is available to the whole ecosystem, not a single product.

## What is already built (measured, reproducible)
- ZK proof: performing collateral >= threshold, all-KYC, book hashes to a servicer-signed (hiding)
  commitment. 128-byte proof, loans stay private.
- On-chain verifier deployed to and confirmed on a live Solana validator (83,352 CU).
- One-command demo (`./demo.sh`) reproduces the proof, on-chain verification, a 10k-loan scale test, and
  an adversarial self-audit. Two independent code reviews are in `AUDIT.md`.
- Repo: https://github.com/ss1738/rwa-credit-proof

## Proposed milestones (this is what the grant funds)
1. **Open-source release + docs** (largely complete): public repo, reproducible demo, audit trail,
   integration guide.
2. **Trustless setup**: move from a single-party trusted setup to a relying-party / ceremony (or a
   transparent proof system), closing the one remaining soundness gap, with a public write-up.
3. **Reference integration**: integrate with one Solana RWA credit protocol against a test loan tape;
   devnet deployment; a publicly verifiable on-chain proof.
4. **Developer kit**: a reusable open-source crate + docs so any Solana RWA protocol can add
   proof-of-solvency in a day.

## Budget (estimate, to refine with the Foundation)
Milestone-based, in the tens-of-thousands range typical of this program, weighted toward milestones 2
and 3 (the trustless setup and the first real integration). Exact figures to be set with the Foundation
against the milestones above.

## Team
Solo technical founder. Background: from-scratch Rust L1 blockchain; BLS12-381 aggregate signatures,
Nova recursive SNARKs, Groth16, KZG/Verkle commitments, on-chain BLS (EIP-2537); Coq/TLA+ formal
verification. This project is a direct application of that cryptography stack.

## Honest limitations (stated up front)
Prototype, not third-party security-audited; the current trusted setup is single-party (milestone 2
addresses this); and the system rests on a trusted signed loan-tape feed from the servicer (a proof
certifies the attested data is solvent, not that the data is true). Full detail in
`KNOWN_LIMITATIONS.md`.

---

## Before submitting (your checks, not the Foundation's)
- Confirm your **visa/entity** position: receiving grant money is an earning event and may need the
  Graduate visa and/or a company to receive it. Verify with an immigration adviser / accountant first.
- Decide the recipient: individual vs a UK Ltd / other entity.
