# Colosseum hackathon submission (draft)

Submit at colosseum.com/hackathon (or arena.colosseum.org) for whichever hackathon window is currently
open. Check current tracks before finalizing, since they change per event (recent ones range from open
tracks with a single Grand Champion prize to category tracks like DeFi/Infrastructure/DePIN). Judges
spend 3-5 minutes per project and decide in the first 60 seconds whether they understand what you built;
the pitch below is written to survive that filter. **Colosseum requires disclosing all prior work.** This
submission does that explicitly rather than presenting the existing repo as built during the hackathon
window.

**Evidence update (2026-09-17):** all six demo stages and the separate audit passed. The latest
local-validator and SBF-harness runs used 83,354 CU; 83,352 CU is retained as the earlier local result.
Solana devnet is pending free test SOL. The Solana program is a pairing verifier; signature binding
and the mint decision are currently demonstrated natively. It does not enforce an approved key or
mint tokens on-chain. See [DEPLOYMENTS.md](DEPLOYMENTS.md) and [KNOWN_LIMITATIONS.md](KNOWN_LIMITATIONS.md).

---

## Project name
Proof-of-Solvency for Tokenized Private Credit

## Tagline (fits in one line on a judge's screen)
A fund proves it's solvent and fully KYC'd, without showing a single loan, before a token can mint.

## Track
DeFi (primary); also plausibly Infrastructure, since the verifier is a reusable primitive, not a
single-purpose app. Pick whichever the current hackathon defines more generously; the pitch doesn't
change.

## The 60-second pitch
Private credit is the largest tokenized real-world-asset category (around $20B on-chain against a
$1.8-3.1T off-chain market), but the loan book backing every token lives off-chain, so nobody can verify
solvency on-chain. We built a zero-knowledge proof that a fund's private loan book is solvent and every
borrower has a KYC flag, and demonstrated the proof-plus-servicer-signature mint decision natively. The proof verifier is
deployed and confirmed on a local Solana test validator: 83,352 compute units, 6% of the per-transaction
budget, constant cost no matter how many loans are in the book. The proof system is also live on public
Ethereum Sepolia testnet (contract `0x8c3DD5E6b660D6aFdCdCa2FB757b2E777d8511Df`, 197,605 verifier-only gas (224,234 total transaction gas), publicly
inspectable on sepolia.etherscan.io).

## Prior work disclosure (required, read this before judging the rest)
The core cryptography, the Solana on-chain verifier, and the EVM verifier were built and deployed
**before** this hackathon (see commit history at github.com/ss1738/rwa-credit-proof). What's new for
this hackathon: [fill in the specific increment you build during the event window. The two strongest
options, matched to what the grant applications deliberately left out of their scope, are (a) a real
reference integration against a live or simulated Solana RWA protocol's loan tape, producing a public,
independently-verifiable on-chain proof, or (b) the multi-party trusted-setup ceremony, run live and
published, addressing the trusted-setup gap in `KNOWN_LIMITATIONS.md`]. Judges should evaluate the
hackathon submission on that increment, not on the pre-existing core, which is disclosed above.

## What's already proven (measured, reproducible, not claims)
| Property | Result |
|---|---|
| Proof size | 128 bytes, constant regardless of book size |
| On-chain verification (Solana) | 83,352 compute units on a local test validator |
| On-chain verification (EVM) | 197,605 verifier-only gas (224,234 total transaction gas), live on public Sepolia testnet, Etherscan-verifiable |
| Scale | 10,000-loan book proves in ~72s |
| Attack test (swap book, reuse signature) | blocked |

Run `./demo.sh` for the core demo; `cargo run --release --bin bench -- 10000` for the larger
benchmark. Public network evidence is checked separately in `DEPLOYMENTS.md`.

## Why Solana
Continuous, per-mint solvency verification is only economically viable if on-chain verification is cheap
and constant-cost regardless of book size. Solana's `alt_bn128` pairing syscalls make that true here at
~6% of the compute budget; it isn't true on higher-fee chains at the same frequency.

## Why now
Solana RWA and private-credit protocols (Credix, Huma, and others) are the direct beneficiaries of a
shared, protocol-agnostic solvency primitive. This removes a real blocker (no cryptographic solvency
check exists today) rather than building a marginally better version of something that already works.

## Related work (judges will ask, address it up front)
Zyga (SOL Strategies) is a general institutional ZK privacy layer for Solana that lists solvency
attestation among several use cases (alongside FATF travel-rule proofs and MEV-resistant execution).
It's commercial, general-purpose, and, at time of writing, has no shipped private-credit-specific
integration. This project is narrower, open source, and already deployed and measured end to end for
one asset class. Full comparison in the repo README's "Related work" section.

## Team
Solo (Satyawan Singh, UK). Background: from-scratch Rust L1 blockchain, Nova recursive SNARKs, Groth16,
KZG/Verkle commitments, on-chain BLS (EIP-2537), Coq/TLA+ formal verification. GitHub: github.com/ss1738.

## Links
- Repo: https://github.com/ss1738/rwa-credit-proof
- Demo: `./demo.sh` (one command, full reproduction)
- Evidence sheet: `EVIDENCE.html` (print-to-PDF technical writeup, good base for the submission deck)
- Known limitations (read before judges do): `KNOWN_LIMITATIONS.md`

## Demo video plan (Colosseum submissions lean heavily on this)
60-90 seconds: (1) state the problem in one sentence, (2) run `./demo.sh` on screen showing the proof
generate and verify on a live validator, (3) show the compute-unit measurement on-screen (83,352 CU),
(4) show the attack test failing (swapped book rejected), (5) close on the hackathon-specific increment
from the disclosure section above. Judges decide in 60 seconds, so put the on-chain verification in the
first 20.

## Relationship to the grant applications
This is the same underlying project as the Solana Foundation and Superteam applications, not a separate
one. A win here (prize or accelerator interest) is complementary evidence for those, not a substitute.
Mention the parallel applications in the submission if there's a field for it, since foundations and
accelerators generally want to know a project is also pursuing non-dilutive funding, not competing
demand for the same dollars.
