# Proof-of-Solvency for Tokenized Private Credit
### A design-partner brief · working prototype

*(product name TBD)*

---

## The problem

Private credit is the **largest tokenized real-world-asset category** — roughly **$20.5B on-chain**
today (rwa.xyz), on top of a **$1.8–3.14T** off-chain private credit market (IMF / JPMorgan). But the
thing that backs the token — the loan book — lives **off-chain**, in servicer systems and spreadsheets.

So the hardest question an investor, counterparty, or regulator can ask is also the one nobody can
answer well: *"prove this token is actually over-collateralised right now, without showing me the
private loan book."* Today the answer is a periodic manual attestation or a trust-me dashboard. Smart-
contract audits don't help — they verify the **code**, not the **asset state**.

## What we built

A way for a fund to prove — **cryptographically, over a book it cannot fake, revealing nothing** —
that it is solvent and compliant, *before* it is allowed to mint.

The zero-knowledge proof attests, over a **private** loan book, that:

1. **Solvent** — performing collateral ≥ the required over-collateralisation of the token supply
2. **Compliant** — every borrower passed KYC
3. **Authentic** — the book hashes to a commitment the **servicer signed**

…while revealing only the threshold and the commitment. Individual loans, balances, and borrower data
never leave the fund.

The mint is gated on **both** the ZK proof *and* the servicer's signature over the commitment. That is
the answer to the "garbage-in" objection every proof-of-reserves scheme hits: **you cannot prove a
different book than the one the servicer attested** — a swapped book has a different commitment, which
the servicer never signed, and the gate blocks it.

## What runs today (measured, on our hardware)

| Property | Result |
|---|---|
| Proof size | **128 bytes**, constant |
| Book privacy | individual loans never revealed |
| Verification | on **Solana** natively (Groth16 / BN254 via `alt_bn128`) |
| **On-chain cost** | **constant regardless of book size** — a 10-loan and a 10,000-loan fund cost the same to verify |
| Scale | a **10,000-loan** book proves in **~60s**; 1,000 loans in ~3.5s |
| Binding attack (swap book, reuse signature) | **blocked** |

This is a working prototype, verified end-to-end on our own machines — not yet a production, audited
system.

## What we honestly do NOT solve (and where you come in)

A proof certifies that the **attested** data is solvent. It does not certify the data is **true**. The
security therefore rests on the **data-trust anchor**: the servicer/custodian signing the loan tape (or
a bank-API attestation) that feeds the proof. That integration — a signed, authenticated loan-tape feed
from your servicing stack — is exactly what a design partner helps us define against a real book.

## The ask

We're looking for **one design partner**: a tokenized-credit protocol or fund willing to feed a loan
tape (test or anonymised is fine) and co-define the integration. In return:

- **Free integration** and hands-on engineering through the pilot
- Be the **reference deployment** and help shape what becomes the standard for verifiable
  tokenized-credit solvency
- A continuous, privacy-preserving proof your LPs, counterparties, and regulators can check — instead
  of a quarterly PDF

If you tokenize private credit and "prove you're solvent without showing the book" is a real problem
for you, that's the conversation.

---

*Contact: [name / email / handle]*
*Sources: on-chain figures rwa.xyz; private-credit market size IMF (Apr 2024) & JPMorgan (2024).*
