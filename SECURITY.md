# Security

**This is an experimental research prototype, not a production system.** Do not use it to secure real
funds, gate a real mint, or make a solvency claim to investors or regulators without a third-party
security audit and the fixes noted below. The cryptography is genuine and independently reviewed (see
[AUDIT.md](AUDIT.md)), but the trust model is not yet production-grade.

## What is verified
- The ZK circuit enforces coverage + all-KYC + a blinded Poseidon commitment over private witnesses
  (empirical audit: `cargo run --bin audit`; historical review record in AUDIT.md). The circuit does
  not verify a servicer signature or establish that off-chain inputs are true.
- The Solana `alt_bn128` verifier performs real Groth16 pairing checks (measured 83k CU on a validator).
- The local pilot gate pins a verification key and servicer, applies threshold/freshness/sequence
  policy and records an approval receipt. This is developer-run local evidence, not a public deployment
  or independent audit.
- The pinned setup-tool compatibility rehearsal verifies locally generated phase-1/phase-2 artifacts.
  It is not an externally contributed production ceremony.

## What is NOT production-safe (read before relying on it)
See [KNOWN_LIMITATIONS.md](KNOWN_LIMITATIONS.md) for detail. In short:
1. **Trusted setup is not fully multi-party.** Only `delta` (phase 2) is re-randomised; `alpha/beta/tau`
   (phase 1) come from a single party who could, in principle, forge proofs. A complete ceremony needs a
   multi-party Powers-of-Tau (phase 1) and per-contribution consistency proofs. The secret in the
   ceremony is also not zeroized.
2. **No third-party security audit.**
3. **No production mint authorization integration.** The legacy verifier accepts a caller-supplied
   key and does not enforce signature or mint policy. The separate pilot gate pins the key and servicer
   and validates its instruction format, but it is local-only, does not derive policy from live supply,
   does not mint SPL tokens and does not prevent repeated downstream use of one receipt. See
   `KNOWN_LIMITATIONS.md` sections 5 through 7.
4. **The data-trust anchor is external.** A proof certifies that the *servicer-attested* loan tape is
   solvent, not that the data is *true*. Security depends on a real, authenticated signed loan-tape feed
   that does not exist yet.

## Reporting
Found a security issue? Email `satyawansinghinuk@gmail.com` before public disclosure; do not include
exploitable details in a public GitHub issue. This is a prototype and responsible feedback is welcome.
