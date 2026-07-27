# Known limitations (honest)

This is a research prototype. The zero-knowledge cryptography is **genuine and verified** (an
independent code audit plus `cargo run --bin audit` confirm the circuit is real, soundness fails for
the right reasons on honest proofs, and the Solana verifier does real pairing checks). But it is **not a
deployable, adversarially-sound mint-gate.** The independent audit flagged the following. All must be
addressed before any external security claim.

## 1. Fixed-seed trusted setup (soundness-critical)
`prove_solvency` runs the Groth16 setup with a fixed, public RNG seed (`StdRng::seed_from_u64(7)`). The
setup randomness ("toxic waste") is therefore reproducible, so **anyone who knows the seed can forge a
valid proof for any statement, including an insolvent book.** The soundness checks in `audit.rs` use
honestly-generated proofs and do **not** cover this. Soundness currently holds against an **honest**
prover only, not a malicious one. A real deployment needs a proper multi-party trusted-setup ceremony,
or a transparent proof system (no trusted setup). **This is the one issue to fix before claiming
soundness externally.**

## 2. Commitment is binding but not hiding
The Poseidon commitment is a deterministic hash of the book with no blinding nonce. It binds the book
but does not hide it: low-entropy loan values could be brute-forced from the commitment. Add a random
nonce to make it a hiding commitment.

## 3. No in-circuit range constraint on collateral witnesses
Collateral witnesses are not constrained `< 2^128` in-circuit, and `enforce_cmp` is only sound for
operands `< (p-1)/2`. Soundness is rescued in the full system by the signed commitment (which pins the
values to the honest book), but the circuit alone does not enforce it.

## 4. Pipeline BLOCKED path is a cleartext check
In `pipeline.rs` the MINT-BLOCKED decision is a cleartext native check, not a crypto path (only the
ALLOW path runs the real proof + verify). Logically correct (an honest prover cannot prove a false
statement), but the block decision itself exercises no cryptography.

## Confirmed genuine, not fabricated (independent audit)
The circuit really enforces solvency + all-KYC + commitment binding over private witnesses; the same
witnesses tie the collateral sum to the commitment (you cannot commit to one book and prove solvency on
another); the Solana verifier performs real `alt_bn128` pairing checks; individual loans stay private;
the swap-book attack is genuinely blocked by the signature-over-commitment. The gap from the marketing
is **production-readiness, not fabrication.**
