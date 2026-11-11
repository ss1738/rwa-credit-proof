# Threat model (pilot alpha)

This describes what the code in this repository does today, who must be trusted, and one design
question that is still open. It is not an audit. Statements are derived from `solana-gate/src/lib.rs`,
`src/circuit.rs` and `src/pilot.rs`.

## What the system establishes

- A Groth16 proof under one pinned verifying key shows that, for a private book, the sum of collateral
  on performing loans is at least a public threshold, every supplied KYC flag is true, and the book
  hashes to a public Poseidon commitment.
- The gate records an approval only if: the configured servicer signed the transaction; the threshold is
  at least the configured minimum; the sequence is exactly the next one; the timestamps are fresh and
  within the configured window; and the proof verifies under the key stored in the policy account.
- A rejected request leaves the policy account unchanged (checked by the compiled SBF suite).

## What it does not establish

- That the collateral values, performance status or KYC flags are true.
- That the book is complete, or anything about liabilities or token supply.
- That the setup was honest: it is single-party.
- That a consumer uses the receipt correctly: consumers must pin the program and the configuration
  address, check expiry, and enforce one-time use themselves.

## Parties and trust

| Party | Holds | Can do if malicious or compromised |
|---|---|---|
| Servicer | Full plaintext book, nonce, signing key | Approve any commitment; re-approve an old commitment with fresh timestamps |
| Prover | Book, nonce, proving key | Cannot forge a proof without the setup secret; can prove false data if the servicer signs it |
| Setup operator | Setup secret (toxic waste) if retained | Forge proofs for any statement under that key |
| Upgrade authority | Program upgrade key | Replace the gate program (not yet defined; nothing is deployed) |
| Consumer program | Its own policy | Misuse a receipt: reuse it, accept another configuration account |

## Known weaknesses, in priority order

1. **No in-circuit range check on collateral.** Witnesses are not bounded and the comparison is only
   sound for small operands. The SDK uses `u128` witnesses, which cannot wrap the field, and the servicer
   recomputes the commitment from a bounded book before signing. A prover running its own witness code
   is stopped only by that servicer check.
2. **One loan count per key.** A change in book size needs a new setup and configuration.
3. **Configuration accounts are not program-derived addresses.** Anyone can initialise their own policy
   with their own key and servicer. A consumer that accepts a caller-chosen configuration is unsafe.
4. **The proof is not bound to the observation time.** The servicer chooses `observed_at`.
5. **`authority` is stored at initialisation and never used again.**
6. **Legacy verifier.** It accepts a caller-supplied key and indexes its input without length checks.
   It is a library for the gate and must not be deployed on its own.

## Open design question (the author has to answer this)

The gate requires the servicer to sign each approval, and the servicer must already hold the full book
to check the commitment. What does the zero-knowledge proof add over a servicer-signed statement
"coverage >= threshold"?

Candidate answers, none of which the repository currently demonstrates:

- **Separation of prover and servicer.** The servicer signs only a commitment and a different party
  proves coverage, so the servicer does not run the computation. The current gate does not work this
  way: the servicer signs the full approval transaction including the threshold.
- **Public verifiability without the servicer's computation being trusted.** Anyone can recheck the
  proof against the pinned key, which a bare signature cannot offer.
- **Privacy from the chain, not from the servicer.** This is true today and is a narrower claim.

Until one of these is chosen, documented and tested, describe the project as privacy-preserving
approval recording, not as a replacement for trusting the servicer.
