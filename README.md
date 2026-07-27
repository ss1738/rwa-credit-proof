# Proof-of-Solvency for Tokenized Private Credit

> A fund proves it is solvent and fully KYC'd, over a loan book it cannot fake, **without revealing a
> single loan**, and a token mints only when that proof and the servicer's signature both check out.

Working prototype. Every claim below is reproducible with `./demo.sh`, and the on-chain verifier has
been deployed to and confirmed on a live Solana validator.

## The problem

Private credit is the largest tokenized real-world-asset category (~$20.5B on-chain, on a $1.8 to $3.14
trillion off-chain market), but the loan book that backs the token lives off-chain. So the question that
matters most, *"prove this token is over-collateralised right now without showing me the private book"*,
has no good answer today. Smart-contract audits verify the code, not the asset state.

## What it proves

A zero-knowledge proof attests, over a **private** loan book, that:

1. **Solvent**: performing collateral covers the required over-collateralisation of the token supply
2. **Compliant**: every borrower passed KYC
3. **Authentic**: the book hashes (Poseidon) to a commitment the **servicer signed**

revealing only the threshold and the commitment. The mint is gated on **both** the ZK proof and the
servicer's ed25519 signature over that commitment, so a prover cannot swap in a different book: a
different book has a different commitment the servicer never signed.

## Results (measured)

| Property | Result |
|---|---|
| Proof size | **128 bytes**, constant regardless of book size |
| Privacy | individual loans never revealed |
| On-chain verification | **83,352 compute units** on a live Solana validator (~6% of the 1.4M cap) |
| On-chain cost vs book size | **constant** (a 10-loan and a 10,000-loan fund cost the same to verify) |
| Scale | 10,000-loan book proves in ~60s; 1,000 in ~3.5s |
| Portability | verifies on Solana (`alt_bn128`) and any EVM chain (`ecPairing`) |
| Attack (swap book, reuse signature) | blocked |

## Run it

```bash
./demo.sh
```

Runs the whole chain: statement soundness, realistic loan-tape decisions, the ZK proof, the on-chain
mint-gate plus the attack test, the scale benchmark, and (if the Solana toolchain is installed) the real
on-chain compute-unit measurement. Needs Rust; stage 6 needs the Solana toolchain.

## How it works

- **The statement** (`src/circuit.rs`): an R1CS circuit: sum of performing collateral `>=` threshold,
  every KYC flag true, and the book's Poseidon commitment equals a public input. Groth16 over BN254.
- **The binding** (`commit_book` + `src/onchain_bytes.rs`): the servicer signs the Poseidon commitment;
  the proof proves the private book hashes to it. This is the answer to the "garbage-in" problem: the
  proof certifies the **attested** book, not arbitrary numbers.
- **The on-chain gate** (`solana-verifier/`): verifies the Groth16 proof via Solana's `alt_bn128`
  pairing syscalls and gates the mint; the servicer signature is checked with the ed25519 syscall.
  Because it is BN254 Groth16, the same proof verifies on any EVM chain via the `ecPairing` precompile.

## Repo layout

```
src/lib.rs            servicer-signed loan tape + native predicate (evaluate)
src/circuit.rs        the ZK statement (Groth16 R1CS) + prove_solvency + Poseidon commitment
src/onchain_bytes.rs  serialize proof/vk/public inputs into the on-chain byte layout
src/prove.rs          ZK proof + soundness checks            (bin: prove)
src/onchain.rs        native alt_bn128 mint-gate + attack     (bin: onchain)
src/pipeline.rs       CSV loan tape -> mint decision          (bin: pipeline)
src/bench.rs          prover cost vs book size                (bin: bench)
solana-verifier/      on-chain Groth16 verifier program (SBF) + compute-unit test
client/               RPC client: sends a real proof tx to a deployed program
examples/             realistic loan-tape CSVs (solvent, insolvent)
DESIGN_PARTNER_BRIEF.md / brief.html   the one-pager
DESIGN_PARTNER_TARGETS.md / OUTREACH_DRAFTS.md   who to pilot with
PRIVATE_CREDIT_SPIKE.md   the 60-day plan
```

## What is proven, and what is not

**Proven:** the cryptography and the on-chain verification, end to end, on real hardware and a live
Solana validator. The proof is sound (insolvent or non-compliant books are unprovable) and private
(loans never revealed).

**Not proven, honestly:** a proof certifies the **attested** data is solvent, not that the data is
**true**. The security rests on the data-trust anchor: the servicer or custodian signing the loan tape
(the CSV in `examples/` is exactly where that feed plugs in). This is a prototype verified on our own
hardware, not a production, audited system. Very large books (>10k loans) will want recursive folding
(Nova); the current monolithic prover is measured to 10k.

*Figures: on-chain market size rwa.xyz; private-credit market size IMF (Apr 2024) and JPMorgan (2024);
performance measured on Apple M4 hardware and the Solana SBF runtime.*
