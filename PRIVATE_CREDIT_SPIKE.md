# Private-credit proof-of-solvency — 60-day spike

**The bet:** the trust/verification layer for tokenized private credit. Private credit is the largest
tokenized RWA category ($20.5B on-chain, rwa.xyz) on top of a $1.8–3.14T off-chain market (IMF/JPM).
A fund should have to prove solvency *cryptographically* before it mints — and that proof layer is the
toll booth every tokenized credit asset passes through.

**One-line goal:** servicer-signed loan tape → recursive ZK proof of over-collateralisation + KYC →
verified on Solana, gating the mint → and it scales to a real loan book and fits on-chain.

## The statement (built, verified — `credit-solvency-core`)
Given a loan tape signed by the servicer, prove in ZK:
1. `Σ collateral(performing loans) ≥ claimed_supply · ratio`
2. every borrower `kyc_ok`
3. the tape is signed by the servicer key (verified **inside** the proof)
— revealing only the aggregate result, never the loans.

Point 3 is the answer to garbage-in: the proof asserts *"the servicer attested this book AND it is
solvent"*, not *"someone typed solvent-looking numbers"*. Status: `evaluate()` implemented as pure
Rust, 5 tests green on Mini, demo shows solvent→ALLOW, insolvent→BLOCK, tampered→BLOCK.

## Architecture
- **Fold (the edge):** each loan = one Nova IVC step folding a running `(collateral, liability, kyc_AND)`
  accumulator. N loans → O(1) verifier work + one proof.
- **Compress → on-chain:** Nova → Spartan → final Groth16 (BN254) → verify on Solana via `alt_bn128`
  pairing syscalls.
- **Reality check:** Groth16-verify on Solana is ~200–400k CU (request toward the 1.4M cap), NOT the
  "<10k CU" a panel guessed. "Does it fit in one tx" is a success criterion, not an assumption.

## Tooling (pragmatic sequencing)
1. **Weeks 1–4 MVP:** zkVM (SP1 / RISC Zero). Lift `evaluate()` into a Rust guest, prove, Groth16-wrap,
   verify on Solana. Fastest path to end-to-end; plays to Rust strength.
2. **Weeks 4–6 differentiation:** swap in Nova folding, show it beats the monolithic circuit at 10k loans.

## Week-by-week
| Wk | Focus | Output |
|---|---|---|
| 1 | Schema + predicate + mock data | ✅ `credit-solvency-core` (done): signed tape, solvency predicate, 5 tests, demo |
| 2–3 | Solvency proof (zkVM MVP) | SP1/RISC0 guest = `evaluate()`; proof verifies locally |
| 4 | Scale test | Prove 1k + 10k loans on the Minis; record prover time/memory |
| 4–5 | Nova folding | Accumulator as Nova IVC; folding beats monolithic at 10k |
| 5 | Compression | Groth16/BN254 wrap of the final proof |
| 6 | Solana verifier | Devnet program verifies via `alt_bn128`; measure CU; minimal mint-gate |
| 7 | End-to-end + tamper demo | Sign→prove→verify→gate mint; flip a loan insolvent → mint blocked. Write-up + 90s video |
| 8 | Distribution + grant | Demo to 8–10 private-credit protocols; land 1 design-partner LOI; Solana Foundation grant |

## Falsifiable success criteria
1. **Prover scales:** 10k-loan proof in < ~30 min on the Minis. Hours ⇒ model doesn't scale.
2. **On-chain fits:** Groth16 verify + mint-gate in one Solana tx < 1.4M CU. Else fallback to off-chain
   verify + on-chain commitment (weaker; flag it).
3. **Trust anchor holds:** proof binds to the servicer signature (done at the logic layer; keep it in the circuit).
4. **Demand real:** ≥1 private-credit protocol signs an LOI / written "we'd pilot this". Zero after 10
   convos ⇒ shelve.

## Kill criteria
Prover can't reach 10k in reasonable time and folding doesn't fix it; or on-chain verify can't fit and
the fallback kills the value prop; or zero design-partner interest → shelve.

## Cost
Time (~8 wks part-time) + the Mac Minis (owned) for proving + free devnet SOL. Non-dilutive. Grant-fundable
after the demo.

## The real moat question
Not "can I build the proof" (done-ish) — it's **"who signs the underlying loan data?"** Solve the
servicer/custodian attestation feed and this becomes the standard; don't and it's theatre. The signature
is in the statement from day one for exactly this reason.
