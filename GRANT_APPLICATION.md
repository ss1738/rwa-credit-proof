# Solana Foundation grant application (rescoped v2)

Apply at solana.org/grants-funding (rolling, decisions ~3 weeks). Open-source public-good track.
Fill in contact details before sending. Rescoped after a first-pass rejection (reason: "the dollar
amount requested exceeds the value of the public good delivered") against a **$300,000 ask**. This
version cuts the ask down by roughly 12-15x, to only the milestones that are unambiguously ecosystem-wide
public good, and drops the paid single-protocol integration that likely compounded that judgment.
NOTE: taking grant money is an earning event. Confirm your visa/entity position first (see end).

---

## Project
**Proof-of-Solvency for Tokenized Private Credit** (open-source ZK verification primitive)

## One line
An open-source, on-chain zero-knowledge verifier that lets a tokenized private-credit fund prove it is
solvent and fully KYC'd, without revealing its loan book, before a token can mint.

## What changed since the last application
The previous version asked the Foundation to fund a paid reference integration with one named Solana RWA
protocol. That milestone primarily benefits a single private business, not the ecosystem, and is the most
likely reason the ask outweighed the public good delivered. This version removes it. The scope below is
narrowed to the two milestones that produce reusable, protocol-agnostic infrastructure any Solana RWA
project can adopt without our involvement. A reference integration is still worth doing: we'll do it
ourselves, unpaid, as a demo, once the developer kit exists, and let it speak for itself rather than ask
the Foundation to fund it.

## The problem it solves for the Solana ecosystem
Private credit is the largest tokenized real-world-asset category (~$20B on-chain, and Solana RWA is
growing fast), but the loan book backing each token lives off-chain and cannot be verified on-chain.
Investors, counterparties, and regulators have no cryptographic way to check solvency. This is a
blocker to institutional RWA capital coming on-chain. A reusable, open-source proof-of-solvency primitive
removes it, for any protocol, not one.

## Why Solana specifically
The verifier uses Solana's `alt_bn128` pairing syscalls to check a Groth16/BN254 proof on-chain in
**~83,000 compute units (measured on a live validator), about 6% of the per-transaction budget, and the
cost is constant regardless of loan-book size**. This makes continuous, per-mint solvency verification
economically viable on Solana in a way it is not on higher-fee chains.

## Public good / open source
Everything funded here (circuit, on-chain verifier, prover, ceremony tooling, developer kit, docs) is
open source under MIT from day one. No milestone in this version depends on, or is scoped around, any
single company or protocol.

## Related work
ZK solvency/compliance proofs for RWA are a recognized need across the ecosystem, not a novel concept:
see the "Related work" section in the repo README for the full comparison. The closest adjacent project
is Zyga (SOL Strategies), a general institutional privacy layer that lists solvency attestation as one
of several use cases; it's commercial and, as far as we've found, has no shipped private-credit-specific
reference implementation. This project is narrower, open source, and already working end to end for one
asset class: complementary rather than redundant.

## What is already built (measured, reproducible)
- ZK proof: performing collateral >= threshold, all-KYC, book hashes to a servicer-signed (hiding)
  commitment. 128-byte proof, loans stay private.
- On-chain verifier deployed to and confirmed on a local Solana test validator (83,352 CU).
- EVM verifier also deployed and confirmed **on public Sepolia testnet** (contract
  `0x8c3DD5E6b660D6aFdCdCa2FB757b2E777d8511Df`, 197,605 gas, publicly inspectable on
  sepolia.etherscan.io), same proof, portable across chains.
- One-command demo (`./demo.sh`) reproduces the proof, on-chain verification, a 10k-loan scale test, and
  an adversarial self-audit. Two independent code reviews are in `AUDIT.md`.
- Repo: https://github.com/ss1738/rwa-credit-proof

## Proposed milestones (this is what the grant funds, narrowed from 4 to 2)
1. **Open-source release + developer kit**: clean up the existing crate into a reusable, documented
   library any Solana RWA protocol can integrate in a day (not tied to any one integration partner);
   publish docs, an integration guide, and the audit trail. Largely complete, this milestone is about
   packaging and documentation quality.
2. **Trustless setup**: replace the current single-party trusted setup with a real multi-party ceremony
   (or evaluate a transparent proof system as an alternative), closing the one remaining soundness gap,
   with a public write-up other Solana ZK projects can reuse as a reference.

No milestone in this version involves a paid integration with a named business.

## Budget
**$20,000-$25,000 total**, milestone-based:
- Milestone 1 (open-source release + developer kit): **$5,000-$7,000**. Mostly packaging and
  documentation work on top of what already exists and is already measured.
- Milestone 2 (trustless setup ceremony + public write-up): **$15,000-$18,000**. The more substantial
  piece: real cryptographic/systems engineering work closing the one remaining soundness gap.

This is calibrated against comparable recent Solana Foundation programs for narrowly-scoped open-source
developer tooling (the Solana Actions/blinks tooling call funded individual grants in the $5k-$25k
range for similar-shaped work), not guessed. It is roughly 12-15x smaller than the $300,000 previously
requested for this project, which is the direct fix for the stated rejection reason.

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
- Decide the recipient: individual vs a UK Ltd / other entity. Note: this rejection was about ask-size
  versus public good, not entity status; incorporating does not by itself change the outcome. Kelvotem
  Inc. (US, Delaware) is chartered for off-world infrastructure and is a brand/purpose mismatch for a
  Solana ZK-credit project. If an entity is needed here, it should be a separate one, decided only if
  this progresses to something (like a Colosseum accelerator slot) that actually requires it.
