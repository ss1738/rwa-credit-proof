# Deployments and reproducible evidence

Checked 2026-09-24. Network names below are part of each claim. A local validator address is not
proof of a deployment on Solana devnet or mainnet.

## Solana devnet: pending funding

**Not deployed.** On 2026-09-24, the devnet RPC returned `value: null` for the intended program
account and a zero balance for the test payer. A request for 1 free devnet SOL returned HTTP 200
with JSON-RPC error `-32603` (`Internal error`) and `x-ratelimit-airdrop-remaining: 0`. No devnet
transaction signature exists yet.

- Intended program ID, **not yet deployed**: `6vXpHGCL9j94LXtXmNsTvKfAL6DfuWpevpdwSj9487bR`
- Devnet fee payer / intended upgrade authority: `8BurHw6PF4n7otsa7b4R6EZo3km6r39XDKbFv159c6AM`
- RPC: `https://api.devnet.solana.com`
- Current status: [devnet-status.json](evidence/2026-09-24/devnet-status.json)
- Compiled binary hash from the original deployment attempt: [devnet-status.json](evidence/2026-09-17/devnet-status.json)

Request 1 free **devnet** SOL for the payer using https://faucet.solana.com. The fee payer was
created in `target/devnet/payer.json`; the existing program keypair is in
`solana-verifier/target/deploy/solana_verifier-keypair.json`. Both directories are ignored by Git.
Keep a private backup before cleaning build directories, particularly once the payer controls an
upgradeable deployment. The addresses above are public; keypair files are not grant attachments.

The built RPC client already accepts an explicit cluster URL, program ID and keypair path.
No global Solana configuration or Git identity was changed. After funding, run from the repo:

```bash
(cd solana-verifier && cargo-build-sbf)
solana balance --url devnet --keypair target/devnet/payer.json
solana program deploy --url devnet --keypair target/devnet/payer.json \
  --program-id solana-verifier/target/deploy/solana_verifier-keypair.json \
  solana-verifier/target/deploy/solana_verifier.so
solana program show --url devnet 6vXpHGCL9j94LXtXmNsTvKfAL6DfuWpevpdwSj9487bR
cargo run --release --locked --manifest-path client/Cargo.toml -- \
  https://api.devnet.solana.com \
  6vXpHGCL9j94LXtXmNsTvKfAL6DfuWpevpdwSj9487bR target/devnet/payer.json
```

Only after a successful public transaction, record its signature and finalized receipt here and
update the form's comma-separated account list, README, audit, evidence page and pitch documents.
Do not use these prospective addresses as evidence of a deployed program. The grant guide permits
`N/A` for on-chain accounts when not applicable.

## Solana pilot gate: local approval reproduced

The new stateful `solana-gate/` is a separate program from the legacy verifier below. It pins an
approved key and servicer, checks threshold/freshness/sequence, and records an approval. It does
not mint tokens or derive threshold from live supply. `./pilot-demo.sh` passed from an isolated
native build directory and fresh validator. The devnet payer was rechecked after this work and
still had 0 test SOL; no public gate deployment is claimed.

- Local program: `kWitTf3ZRSVJyKpf6gwzG17JEvLsUKf6XCRJSahURtW`.
- Local configuration: `FiT5imbsndpCNDTFdRLpwiYydZPH6AB2Cu4qULxAtco`.
- Finalized LOCAL approval: `4Mt2uB5K99gTzfocq5Qs2tgek6x7mk5iKDc7pQqJm2oKTCkXcuCXJgWZvXNPJNunNGTFnTMbyVAVZuNLnDAbvTyo`.
- 12-loan approval: **85,529 CU**, **672-byte legacy transaction**, separate payer and servicer signatures.
- Account creation plus policy initialization: **1,057 transaction bytes**.
- Compiled SBF suite: 2 accepted updates and **26 rejected cases**, with unchanged state on rejection.
- Evidence and source hashes: [pilot-alpha/README.md](evidence/2026-09-17/pilot-alpha/README.md).
- Finalized receipt: [approval-report.json](evidence/2026-09-17/pilot-alpha/approval-report.json).

The demo operates both signer roles and uses synthetic data. Its local-only client does not sign
on behalf of an external servicer. Consumers must pin both program and configuration address,
check receipt freshness and implement their own action policy. The earlier devnet commands above
target the legacy pairing verifier; they do not deploy this pilot gate.

## Solana local validator: reproduced successfully

Fresh `solana-test-validator` 4.1.1 instance on `http://127.0.0.1:18999`, with the rebuilt SBF
binary loaded at genesis. The existing RPC client generated a fresh 12-loan example proof, simulated
it and sent a real transaction. This local transaction finalized with no error.

- Program ID: `6vXpHGCL9j94LXtXmNsTvKfAL6DfuWpevpdwSj9487bR` (**local cluster only**).
- Signature: `3h42tZ68Y8Xfw3H4tGKzo4bbxE7DdPc3qTUnLeoXaQWmw78qB9XaosduunATgTEndD99uaverZ2MrEAciwRCKPx1`.
- Receipt: [local-receipt.json](evidence/2026-09-17/local-receipt.json).
- Client log: [local-verification.log](evidence/2026-09-17/local-verification.log).
- Simulation and finalized transaction: **83,354 CU**, proof verification only.
- Earlier documented local-validator run: **83,352 CU**. Retained as a historical measurement;
  the current run is 83,354 CU. These are both approximately 6% of the 1.4M maximum transaction
  budget, and roughly 42% of the 200,000-CU budget shown in this particular receipt.

The SBF program checks the pairing equation using a caller-supplied verifying key. It neither
pins an authorized solvency circuit, authenticates the servicer signature nor mints tokens.
See [KNOWN_LIMITATIONS.md](KNOWN_LIMITATIONS.md).

## Ethereum Sepolia: historical public example checked

- Contract: [0x8c3DD5E6b660D6aFdCdCa2FB757b2E777d8511Df](https://sepolia.etherscan.io/address/0x8c3DD5E6b660D6aFdCdCa2FB757b2E777d8511Df).
- Transaction: [0x91196bd0a9b6d192733bdc7df7126520c142f2bbda2c146190ebddd75aaae7c7](https://sepolia.etherscan.io/tx/0x91196bd0a9b6d192733bdc7df7126520c142f2bbda2c146190ebddd75aaae7c7).
- Etherscan reports success, dated 2026-07-28 13:08:24 UTC.
- Event data decodes to `true, 197605`: **197,605 verifier-only gas**.
- The complete transaction used **224,234 gas**, including overhead.
- Extracted evidence: [sepolia-summary.json](evidence/2026-09-17/sepolia-summary.json).

`evm/src/SolvencyVerifierDemo.sol` verifies a baked-in example proof with a baked-in key. It is not
an arbitrary-book mint gate. No new Sepolia transaction was sent in this session. The Solana client
generates a new proof and setup per run, so the new local transaction and this historical Sepolia
transaction must not be described as the same byte-identical proof.

## Reproduction run: 2026-09-17

| Check | Result | Evidence |
|---|---|---|
| `./demo.sh`, all six stages | Passed, including 5 unit tests and SBF stage 6 | [demo.log](evidence/2026-09-17/demo.log) |
| SBF harness, 10-loan fixture | 83,354 CU | demo.log |
| Separate adversarial audit | Passed; 5,911 constraints for its 10-loan circuit | [audit.log](evidence/2026-09-17/audit.log) |
| 100-loan benchmark | 0.43s setup, 0.42s proving, 128-byte compressed proof | demo.log |
| 1,000-loan benchmark | 3.88s setup, 3.82s proving, 128-byte compressed proof | demo.log |
| Separate 10,000-loan benchmark | 64.61s setup, 72.17s proving, verified true, 128-byte compressed proof | [bench-10000.log](evidence/2026-09-17/bench-10000.log) |

Times are observations from one run on this machine; setup is separate from proving. The fixed
verification structure assumes two public inputs; the constraint count and prover work grow with
loan count. A 128-byte compressed proof is not the instruction size: the current instruction carries
961 bytes including uncompressed points, the verifying key and the public scalars.

```bash
./demo.sh
cargo run --release --bin audit
cargo run --release --bin bench -- 10000
```

The six-stage demo does not run the separate audit or 10,000-loan benchmark, and does not deploy
or check public chains. The logs above record those separate checks explicitly.
