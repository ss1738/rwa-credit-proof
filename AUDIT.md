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
  (live validator).

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
| 3 | No in-circuit range check on collateral | **Accepted with rationale**: the signed commitment pins the values (honest sums ~2^141 are far below the ~2^253 safety bound), so full-system soundness holds. A bit-range check is deferred as prohibitive at scale. |
| 4 | Pipeline BLOCK path was a cleartext check | **Fixed**: the block decision now runs the real circuit (`ConstraintSystem::is_satisfied`). |

## Honest scope

This is a **research prototype**, verified on our own hardware and a live Solana validator. It is not a
production, third-party-security-audited system. The cryptography is genuine; the remaining gap to a
deployable mint-gate is a proper trusted-setup ceremony (or relying-party setup) and an external audit,
both of which belong with a real design partner, not before one. See
[KNOWN_LIMITATIONS.md](KNOWN_LIMITATIONS.md).
