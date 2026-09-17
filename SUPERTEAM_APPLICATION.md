# Superteam microgrant application (draft)

Apply via a regional listing at earn.superteam.fun/grants/ (pick the chapter that matches you — check
current listings for one open to UK/Europe-based applicants, e.g. Superteam Germany or a general/global
grants track; exact form fields vary by chapter but all ask for the sections below). Equity-free,
typically $200-$10k, fast turnaround. Good fit here because the ask is small, the milestone is concrete,
and — unlike most microgrant applicants — there's already a live, measured, reproducible prototype to
point to instead of a plan.

---

## Project name
Proof-of-Solvency for Tokenized Private Credit (rwa-credit-proof)

## One-liner
An open-source zero-knowledge verifier, already deployed and confirmed on a local Solana test validator
and live on public Ethereum Sepolia, that lets a tokenized private-credit fund prove solvency and KYC
compliance without revealing its loan book.

## Links
- Repo: https://github.com/ss1738/rwa-credit-proof
- Demo: `./demo.sh` (one command, reproduces every claim below)
- Evidence sheet: `EVIDENCE.html` in the repo (print-to-PDF technical writeup)
- GitHub profile: https://github.com/ss1738

## The problem
Private credit is the largest tokenized real-world-asset category, but the loan book backing each token
lives off-chain with no cryptographic way to verify solvency on-chain. This blocks institutional RWA
capital and leaves Solana RWA protocols (Credix, Huma, and others) with no shared, trustworthy primitive
for this.

## What's already working (not a plan — measured and reproducible today)
- 128-byte ZK proof (Groth16/BN254) that a private loan book is solvent and fully KYC'd.
- On-chain verifier deployed to and confirmed on a **local Solana test validator**: 83,352 compute units
  (~6% of the per-transaction budget), constant cost regardless of book size.
- Same proof is also **live on public Ethereum Sepolia testnet** (contract
  `0x8c3DD5E6b660D6aFdCdCa2FB757b2E777d8511Df`, 197,605 gas, publicly inspectable on
  sepolia.etherscan.io) — cross-chain portable.
- Two independent code audits documented in `AUDIT.md`; honest limitations documented in
  `KNOWN_LIMITATIONS.md` rather than glossed over.

## What this grant funds
One concrete, scoped piece of work: **replacing the current single-party trusted setup with a real
multi-party ceremony** (or evaluating a transparent proof system as an alternative), which is the one
soundness gap standing between this being a prototype and being safe for a real integration. Output:
working ceremony tooling, a public write-up, and an updated audit note — all open source, reusable by
any Solana ZK project doing a Groth16 setup, not just this one.

## Why this is a good fit for a microgrant specifically
It's small, well-defined, and has a clear finish line (the ceremony either closes the soundness gap or it
doesn't) — not an open-ended roadmap item. It doesn't depend on landing a business partner first. And
because the cryptography, the circuit, and the on-chain verifier already exist and are measured, the
grant is funding the last mile to production-safety, not R&D risk.

## Ask
In the $5,000-$8,000 range (equity-free), scoped to the ceremony milestone above. Open to adjusting to
whatever the chapter's typical microgrant size is.

## Related work
The closest adjacent project is Zyga (SOL Strategies), a general institutional ZK privacy layer for
Solana that lists solvency attestation as one of several use cases. It's commercial and general-purpose;
this project is open source, narrow, and already shipped end to end for private-credit specifically —
see the repo README's "Related work" section for the full comparison.

## Team
Solo technical founder (Satyawan Singh, UK). Background: from-scratch Rust L1 blockchain (consensus,
light clients, data-availability sampling), BLS12-381 aggregate signatures, Nova recursive SNARKs,
Groth16, KZG/Verkle commitments, on-chain BLS (EIP-2537), Coq/TLA+ formal verification. GitHub:
github.com/ss1738 (37 public repos, most in formal verification and ZK).

## Note
Previously submitted a larger-scope application to the Solana Foundation's main grants program (open-
source track); it was not funded because the ask exceeded the public good delivered at that scope. This
application is the same underlying project cut down to a single, self-contained, low-cost milestone —
not a resubmission of the same ask, and not exclusive with a rescoped Foundation application running in
parallel.
