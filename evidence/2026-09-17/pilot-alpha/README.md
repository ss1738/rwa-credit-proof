# Pilot alpha evidence, 2026-09-17

This is a local developer reproduction using synthetic loan data. It is not a public Solana
deployment, an external customer integration, an independent audit or an SPL token mint.

- `pilot-demo.log`: complete successful five-stage `./pilot-demo.sh` run, including SDK/protocol tests,
  compiled SBF execution, local initialization and finalized approval.
- `approval-report.json`: finalized transaction, public account state, key hash, addresses, timings,
  sizes, costs and two signed negative simulations. The client requested finalized RPC commitment.
- `rejection-cases.json`: all 26 executed SBF rejection scenarios. Each checks the precise error and
  byte-for-byte unchanged configuration state. Two valid updates use the same approved setup key.
- `build-manifest.json`: source hashes, base commit, tool versions and compiled binary hash.
  The changes are uncommitted; the base commit alone does not identify this implementation.
- `legacy-demo.log`: all six legacy demo stages still pass, including the 83,354-CU pairing benchmark.
- `circuit-audit.log`: separate developer-run adversarial circuit checks pass.
- `ceremony-report.json` and `ceremony-compat.log`: pinned `iden3/snarkjs` 0.7.6 / Node 24
  compatibility rehearsal for the exact arkworks fixture, including cross-verification and negative
  artifact checks. All three contributions were local and controlled by one operator; this is not a
  production multi-party ceremony.

The pilot approval used 85,529 total transaction CU in 672 serialized legacy transaction bytes,
including the compute-budget instruction. The gate itself used 85,379 CU. Account creation plus
initialization fit in 1,057 bytes. Public loan count: 12; threshold: 3396000 integer units.
Only the commitment and aggregate threshold enter the circuit's public inputs.

Local program: `kWitTf3ZRSVJyKpf6gwzG17JEvLsUKf6XCRJSahURtW`.
Local approved configuration: `FiT5imbsndpCNDTFdRLpwiYydZPH6AB2Cu4qULxAtco`.
Local finalized signature: `4Mt2uB5K99gTzfocq5Qs2tgek6x7mk5iKDc7pQqJm2oKTCkXcuCXJgWZvXNPJNunNGTFnTMbyVAVZuNLnDAbvTyo`.
These addresses/signatures cannot be used as public devnet explorer evidence. The isolated local
validator was stopped after the run; the saved receipt remains an inspectable historical artifact.

The public account bytes confirm the stored key hash and receipt fields. Private loan tapes,
blinding nonces, signer keypairs, proving parameters, ledgers and binaries are excluded from this
folder. The parameter hash is an identifier, not a completed ceremony attestation. See
`../../../PILOT_GUIDE.md` and `../../../KNOWN_LIMITATIONS.md` for integration assumptions.
