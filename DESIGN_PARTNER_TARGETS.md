# Design-partner targets — tokenized private credit

Goal: land ONE design partner to move the proof-of-solvency prototype from demo to pilot.
Tags: `[grounded]` = verified this session · `[verify]` = from general knowledge, confirm before acting.

---

## Key insight: we are NOT locked to Solana

The proof is **Groth16 over BN254**. That same proof verifies:
- on **Solana** via the `alt_bn128` syscalls (built + verified) `[grounded]`
- on **Ethereum / any EVM chain** via the `ecPairing` precompile (EIP-197) — a trivial port `[verify, but standard]`

So we can meet a partner **on their chain**. This widens the target set from "Solana private credit"
to "all tokenized private credit," and kills the "but we're not on Solana" objection before it lands.

---

## Tier 1 — best fit for a solo technical founder (acute pain + reachable)

Smaller, community/dev-driven, and the pain (prove solvency without exposing the book) is existential
for them because their investors are far from the borrowers.

**Credix** — B2B private credit marketplace `[grounded: "B2B Credit to Grow Your Business"]`, emerging-
market SME / receivables lending, Solana-native `[verify]`.
- *Why the pain is acute:* LPs fund borrowers in markets they can't see; trust/opacity is THE barrier.
  Proving loan-book solvency without exposing borrower data is directly their bottleneck.
- *Fit:* Solana = our built chain. Highest warmth.
- *Reach:* their public Discord / X and docs (linked from credix.finance); technical DM to an eng/BD
  lead with the 90-second mint-gate demo.

**Huma Finance** — "The First PayFi Network" `[grounded]`, income/receivables-backed lending, Solana +
Stellar `[verify]`.
- *Why:* real-world receivables sit off-chain; LPs want continuous assurance, not a quarterly report.
- *Fit:* Solana-aligned. Warm.

**Centrifuge** — RWA / private-credit pools, Ethereum + own appchain `[verify]`.
- *Why:* pool transparency is already a core value prop — they will immediately get the ZK angle.
- *Fit:* EVM, so our proof ports via `ecPairing` (even simpler than Solana). Dev-accessible community.

## Tier 2 — higher value, harder to reach solo (use a Tier-1 win as the reference)

**Maple Finance** — $23.98B originated since 2019 `[grounded]`, institutional lending, Ethereum +
Robinhood Chain `[grounded]`. No public proof-of-reserves mechanism surfaced `[grounded: not shown]`.
- *Why:* institutional lenders increasingly demand provable solvency; big brand = big reference.
- *Caveat:* large org, longer sales cycle; land a smaller partner first.

**Figure** — the single largest tokenized private-credit issuer (~$20.5B HELOC on-chain) `[grounded:
rwa.xyz]`, own chain (Provenance) `[verify]`.
- *Why:* highest value in the category. *Caveat:* large regulated entity, hardest to partner as a solo.

**Apollo / Securitize (ACRED)** — institutional tokenized credit fund via Securitize `[verify]`.
- *Why:* enterprise credibility. *Caveat:* enterprise sale, not a solo-founder first move.

## Tier 3 — infrastructure leverage (one integration → many issuers)

**Securitize** — tokenization platform behind many funds. A platform-level integration reaches many
issuers at once. Longer play, but high leverage `[verify]`.

---

## Outreach approach (warm, not cold)

1. **Lead with the working demo, not a deck.** The 90-second mint-gate (ALLOW for the attested book,
   BLOCK for a swapped book) is the hook. Show it on *their* chain.
2. **Channel: their dev/Discord + a sharp technical DM** to a founder or eng lead on X. Not cold email.
3. **The one-line hook:** "You tokenize private credit. Here's a way to prove to your LPs you're
   solvent and fully KYC'd — continuously, without exposing the loan book. Working demo, verifies on
   [your chain]. Free integration to be the reference deployment."
4. **The offer (from the brief):** free integration + hands-on eng through the pilot.
5. **Credibility warm-up (optional):** open-source the Solana `alt_bn128` verifier as a standalone
   dev tool → visibility in the RWA/Solana dev community → inbound instead of outbound.

## Sequence
Start with **Credix + Huma + Centrifuge** (3 parallel, low-friction, high-pain). Land one → use it as
the named reference to open **Maple / Apollo**. Do NOT start at the top of the market solo.

## Honest notes
- These are organisations and public channels, not fabricated personal contacts — find the actual
  eng/BD lead via each project's public X + Discord before sending.
- Confirm each project's current chain and status (`[verify]` tags) — this space moves fast and web
  search was unavailable this session.
