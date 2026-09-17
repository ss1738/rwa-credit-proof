# Caledren: private-credit proof and approval tooling

A local pilot alpha for a protocol engineer and its servicing/data partner.

A protocol needs to check coverage of its private-credit book without publishing individual loans.
Caledren proves that performing collateral covers an approved threshold, that supplied KYC flags
are true, and that the witnesses match a blinded book commitment. It records an approval on Solana
only when the configured servicer authorizes the exact transaction and the stored key, threshold,
freshness and sequence checks pass.

The working flow is CSV ingestion, proof generation with reusable parameters, a private servicer
commitment check, and an on-chain approval receipt. A developer can reproduce it with
`./pilot-demo.sh`. The compiled gate suite accepts two successive approvals and rejects 26 invalid
requests without changing its state. A finalized local 12-loan approval used 85,529 compute units
in a 672-byte transaction with separate payer and servicer signatures. The public proof bundle
contains no individual loan records or blinding nonce.

We are seeking one external developer evaluation using synthetic or appropriately approved test
inputs. The goal is to reproduce the workflow independently, read the receipt from a consuming
application, and identify the servicing schema, threshold source and authorization requirements
for a useful integration. The protocol retains control of its feed, keys and policy.

This is not a deployed fund or token mint. The proof certifies a predicate over supplied collateral
and flags, not their real-world truth. Loan IDs and principal are outside the circuit commitment;
threshold-to-supply policy remains a consumer responsibility. The current setup is single-party,
range assumptions remain, no external security audit is claimed, and no customer or third-party
integration has been completed. Solana devnet is pending test funding. The separate historical
pairing benchmark and public Sepolia example are described in `DEPLOYMENTS.md`.

Start with [PILOT_GUIDE.md](PILOT_GUIDE.md). Record independent results and integration feedback
using [PILOT_EVALUATION.md](PILOT_EVALUATION.md). Read [KNOWN_LIMITATIONS.md](KNOWN_LIMITATIONS.md)
before deciding whether the trust model suits your use case.

Contact: Satyawan Singh, satyawansinghinuk@gmail.com.
