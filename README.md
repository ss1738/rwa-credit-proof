# credit-solvency-core

Cryptographic proof-of-solvency for tokenized private credit. A fund proves — in zero-knowledge, over a
**servicer-signed** loan tape — that its book is solvent and fully KYC'd **before it is allowed to mint**,
without revealing individual loans.

Why it matters: private credit is the largest tokenized RWA category (~$20.5B on-chain, on a $1.8–3.14T
off-chain market), and its core risk is off-chain loan-book opacity — which smart-contract auditors
(who verify *code*) do not touch. This is a different layer: verifying the *asset state*.

## Status
- `src/lib.rs` — the statement (`evaluate()`), pure Rust, lifts into an SP1/RISC Zero guest unchanged.
- `src/demo.rs` — end-to-end demo on a mock book.
- 5 tests green; demo shows solvent→ALLOW, insolvent→BLOCK, tampered→BLOCK.

## Build & test (on a Mac Mini, per workspace policy — never the MacBook)
```bash
cargo test
cargo run --bin demo
```

## Plan
See [PRIVATE_CREDIT_SPIKE.md](PRIVATE_CREDIT_SPIKE.md) for the 60-day experiment: zkVM MVP → Nova folding
→ Groth16 compression → Solana `alt_bn128` verifier + mint-gate → design-partner LOI.

The make-or-break is the data-trust anchor: **who signs the loan tape.** The servicer signature is bound
into the statement from day one so the proof certifies attested data, not arbitrary input.
