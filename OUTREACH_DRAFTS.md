# Outreach drafts — Tier 1 design partners

Send as a technical DM / short email to a founder or eng lead (find them via each project's public
X + Discord). Lead with the working demo, not a deck. Honest: prototype, seeking one design partner.
Fill `[your name]`. Do not send until you've confirmed the person and the project's current chain.

---

## Credix (Solana · emerging-market SME / receivables credit)

> Hi [name] — I built a zero-knowledge proof-of-solvency for tokenized private credit, and Credix is
> the sharpest fit I've found for it.
>
> It lets a pool prove to LPs that it's over-collateralised and fully KYC'd — **continuously, without
> revealing a single loan or borrower** — and it verifies natively on Solana (Groth16 via `alt_bn128`).
> Minting can be gated so tokens only issue when the servicer-signed book actually proves solvent.
>
> For Credix specifically, where LPs fund borrowers in markets they can't see, this turns "trust our
> reporting" into a cryptographic proof. Working demo today: an attested solvent book mints; a swapped
> or insolvent book is blocked. Constant-size 128-byte proof, tested to 10k-loan books.
>
> I'd do the integration **free** to make Credix the reference deployment. Worth a 20-min technical
> call? — [your name]

## Huma Finance (Solana / Stellar · PayFi, receivables / income-backed)

> Hi [name] — I've built a way for a PayFi pool to prove it's solvent and fully KYC'd to its LPs
> **without exposing the underlying receivables** — a zero-knowledge proof, verified on-chain, that the
> servicer-signed book covers the tokens issued.
>
> Huma is a natural fit: real-world receivables sit off-chain, and LPs want continuous assurance rather
> than a periodic report. The proof is 128 bytes, verifies on Solana (`alt_bn128`), and the mint can be
> gated on it — a valid book mints, a tampered one is blocked (working demo).
>
> Happy to integrate it free as a pilot to make Huma the reference. Open to a short technical call?
> — [your name]

## Centrifuge (EVM + appchain · RWA / private-credit pools)

> Hi [name] — Centrifuge already leads on pool transparency; I've built something that makes that
> transparency **cryptographic**.
>
> It's a zero-knowledge proof that a pool is over-collateralised and fully KYC'd, over a servicer-signed
> book, revealing no individual asset. Because it's Groth16 over BN254, it verifies on EVM via the
> `ecPairing` precompile — so it drops into your stack directly (also runs on Solana). Constant 128-byte
> proof, tested to 10k assets, mint-gated: attested book issues, swapped book is blocked.
>
> I'd build the integration free to make a Centrifuge pool the reference deployment. Worth 20 minutes?
> — [your name]

---

## After a reply
- Send the one-pager (`DESIGN_PARTNER_BRIEF.md`).
- On the call, be upfront about the one real dependency: the **signed loan-tape feed** from their
  servicer is what anchors the proof to true data — co-defining that integration is the point of the
  pilot.
- Ask for a **test / anonymised loan tape** to run the first real proof against their data.
