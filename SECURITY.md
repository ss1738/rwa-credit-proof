# Security

**This is an experimental research prototype, not a production system.** Do not use it to secure real
funds, gate a real mint, or make a solvency claim to investors or regulators without a third-party
security audit and the fixes noted below. The cryptography is genuine and independently reviewed (see
[AUDIT.md](AUDIT.md)), but the trust model is not yet production-grade.

## What is verified
- The ZK circuit genuinely enforces solvency + all-KYC + a servicer-signed hiding commitment over
  private witnesses (empirical audit: `cargo run --bin audit`; two independent code reviews in AUDIT.md).
- The Solana `alt_bn128` verifier performs real Groth16 pairing checks (measured 83k CU on a validator).
- The phase-2 (delta) multi-party setup contribution is implemented correctly.

## What is NOT production-safe (read before relying on it)
See [KNOWN_LIMITATIONS.md](KNOWN_LIMITATIONS.md) for detail. In short:
1. **Trusted setup is not fully multi-party.** Only `delta` (phase 2) is re-randomised; `alpha/beta/tau`
   (phase 1) come from a single party who could, in principle, forge proofs. A complete ceremony needs a
   multi-party Powers-of-Tau (phase 1) and per-contribution consistency proofs. The secret in the
   ceremony is also not zeroized.
2. **No third-party security audit.**
3. **The data-trust anchor is external.** A proof certifies that the *servicer-attested* loan tape is
   solvent, not that the data is *true*. Security depends on a real, authenticated signed loan-tape feed
   that does not exist yet.

## Reporting
Found an issue? Please open a GitHub issue or contact `[ your email ]`. Responsible disclosure
appreciated, this is a prototype and feedback is welcome.
