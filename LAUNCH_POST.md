# Launch posts (ready to fire once the repo is public)

Fill `https://github.com/ss1738/rwa-credit-proof`. Honest framing on purpose: a prototype with a working demo beats a hyped claim,
especially on Hacker News and among ZK people who will check.

**Evidence update (2026-09-17):** all six demo stages and the separate audit passed. The latest
local-validator and SBF-harness runs used 83,354 CU; 83,352 CU is retained as the earlier local result.
Solana devnet is pending free test SOL. The Solana program is a pairing verifier; signature binding
and the mint decision are currently demonstrated natively. It does not enforce an approved key or
mint tokens on-chain. See [DEPLOYMENTS.md](DEPLOYMENTS.md) and [KNOWN_LIMITATIONS.md](KNOWN_LIMITATIONS.md).

---

## Show HN

**Title:** Show HN: Zero-knowledge proof-of-solvency for tokenized private credit (Rust)

**Body:**

I built a prototype that lets a tokenized private-credit fund prove, in zero knowledge, that its loan
book is solvent and fully KYC'd, without revealing a single loan. The native model allows a mint only when the proof and
the servicer's signature both check out; the SBF program currently verifies the proof only.

Why: private credit is the largest tokenized real-world-asset category (~$20B on-chain), but the loan
book that backs the token is off-chain and unverifiable. Smart-contract audits check the code, not the
asset state. This checks the asset state.

How it works: a Groth16/BN254 circuit proves (1) performing collateral >= a threshold, (2) every
borrower passed KYC, (3) the book hashes (Poseidon, with a blinding nonce) to a servicer-signed
commitment. 128-byte proof, loans stay private. It verifies on Solana via the alt_bn128 syscalls
(~83k compute units, measured on a live validator) and on any EVM chain via the ecPairing precompile.

It is a prototype, and I have tried to be honest about the limits: the trusted setup is single-party
(needs a ceremony or relying-party setup for production), it is not third-party security-audited, and
the whole thing rests on a trusted signed loan-tape feed from the servicer (a proof certifies the
attested data is solvent, not that the data is true). All of that is written down in
KNOWN_LIMITATIONS.md and AUDIT.md.

`./demo.sh` reproduces the core tests, SBF verification and 100/1,000-loan benchmarks. Run
`cargo run --release --bin bench -- 10000` and `cargo run --release --bin audit` separately for
the larger benchmark and adversarial audit. Two independent code reviews are in the repo.

Repo: https://github.com/ss1738/rwa-credit-proof

I would love feedback, especially from people in RWA / private credit or ZK. And if you run a
tokenized-credit protocol, I am looking for one design partner to pilot against a real (even test)
loan tape.

---

## X / Twitter thread

**1/**
I built a way for a tokenized private-credit fund to prove it is solvent, without showing its loan book.
Zero-knowledge. 128-byte proof. Verifies on Solana. Open-source. [demo GIF]

**2/**
The problem: private credit is the biggest tokenized RWA (~$20B on-chain), but the loan book backing the
token is off-chain. "Prove you are solvent without showing the books" had no good answer. Now it does.

**3/**
How: a Groth16 circuit proves performing collateral >= threshold, every borrower KYC'd, and the book
hashes to a servicer-signed commitment. Loans stay private. Mint is gated on the proof + the signature.
Swap in a different book and reuse the signature -> blocked.

**4/**
Honest limits (written down, not hidden): it is a prototype, not security-audited, the trusted setup is
single-party, and it rests on a trusted signed loan-tape feed. One-command demo + two independent code
reviews in the repo.

**5/**
Open-source: https://github.com/ss1738/rwa-credit-proof
I am looking for one tokenized-credit protocol to pilot with. If that is you, DM me.

---

## Where to post
- Hacker News (Show HN)
- X / Twitter (the thread), tag a couple of RWA / Solana accounts
- Solana developer Discord + r/solana
- Then send the three DMs in OUTREACH_DRAFTS.md
