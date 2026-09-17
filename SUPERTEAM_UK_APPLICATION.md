# Superteam UK grant application (draft)

Program: **Solana Foundation UK Grants**, run through Superteam UK — https://superteam.fun/earn/grants/solana-foundation-uk-grants
Eligibility: restricted to applicants based in the United Kingdom. You qualify (Leicester, UK).

**Status check (as of this draft): applications are currently marked "Applications Paused."** Before
submitting, email **uk@superteam.fun** to ask whether the program is reopening and whether they're
holding a waitlist — don't wait for the portal to silently reopen. Keep this draft ready to submit the
moment it does. Average response time when open is ~1 week.

Program stats for calibration: up to $10,000 USD per grant, average award **$4,707** across 87 funded
recipients (~$409.5k total approved to date). Categories: Frontend, Blockchain, Backend, Content. This
application sits squarely in **Blockchain** (primary) / **Backend** (secondary) — it is not a frontend or
content submission, so don't let the form's category list push you toward a mismatched one.

---

## Project name
Proof-of-Solvency for Tokenized Private Credit (rwa-credit-proof)

## One-liner
An open-source zero-knowledge verifier — already deployed and confirmed on a local Solana test validator
and live on public Ethereum Sepolia — that lets a tokenized private-credit fund prove solvency and KYC
compliance without revealing its loan book.

## Links
- Repo: https://github.com/ss1738/rwa-credit-proof
- Demo: `./demo.sh` (one command, reproduces every claim below)
- Evidence sheet: `EVIDENCE.html` in the repo (print-to-PDF technical writeup)
- Live Sepolia deployment: contract `0x8c3DD5E6b660D6aFdCdCa2FB757b2E777d8511Df`, verify tx
  `0x91196bd0a9b6d192733bdc7df7126520c142f2bbda2c146190ebddd75aaae7c7` — publicly inspectable on
  sepolia.etherscan.io
- GitHub profile: https://github.com/ss1738

## Location / eligibility
Based in Leicester, United Kingdom — meets the UK-only eligibility requirement for this program.

## The problem
Private credit is the largest tokenized real-world-asset category, but the loan book backing each token
lives off-chain with no cryptographic way to verify solvency on-chain. This blocks institutional RWA
capital and leaves Solana RWA protocols (Credix, Huma, and others) with no shared, trustworthy primitive
for this.

## What's already working (not a plan — measured and reproducible today)
- 128-byte ZK proof (Groth16/BN254) that a private loan book is solvent and fully KYC'd.
- On-chain verifier deployed to and confirmed on a local Solana test validator: 83,352 compute units
  (~6% of the per-transaction budget), constant cost regardless of book size.
- Same proof is also live on public Ethereum Sepolia testnet (see links above) — cross-chain portable.
- Two independent code audits documented in `AUDIT.md`; honest limitations documented in
  `KNOWN_LIMITATIONS.md` rather than glossed over.

## What this grant funds
One concrete, scoped piece of work: **replacing the current single-party trusted setup with a real
multi-party ceremony** (or evaluating a transparent proof system as an alternative), which is the one
soundness gap standing between this being a prototype and being safe for a real integration. Output:
working ceremony tooling, a public write-up, and an updated audit note — all open source, reusable by
any Solana ZK project doing a Groth16 setup, not just this one.

## Why this is a good fit for this program specifically
It's UK-eligible, it's a Blockchain-category submission with working code rather than a pitch deck, and
the ask sits comfortably within this program's typical range (average award $4,707, cap $10k) for a
well-defined milestone with a clear finish line — not an open-ended roadmap item.

## Ask
$5,000-$6,000 (equity-free), scoped to the ceremony milestone above — in line with this program's
average award size rather than pushing toward the $10k cap.

## Team
Solo technical founder (Satyawan Singh, Leicester, UK). Background: from-scratch Rust L1 blockchain
(consensus, light clients, data-availability sampling), BLS12-381 aggregate signatures, Nova recursive
SNARKs, Groth16, KZG/Verkle commitments, on-chain BLS (EIP-2537), Coq/TLA+ formal verification. GitHub:
github.com/ss1738 (37 public repos, most in formal verification and ZK).

## Related work
The closest adjacent project is Zyga (SOL Strategies), a general institutional ZK privacy layer for
Solana that lists solvency attestation as one of several use cases. It's commercial and general-purpose;
this project is open source, narrow, and already shipped end to end for private-credit specifically —
see the repo README's "Related work" section for the full comparison.

## Note
Previously submitted a larger-scope application to the Solana Foundation's main grants program (open-
source track); it was not funded because the ask exceeded the public good delivered at that scope. This
application is the same underlying project cut down to a single, self-contained, low-cost milestone —
not a resubmission of the same ask, and not exclusive with the rescoped Foundation application or a
Colosseum hackathon submission running in parallel.
