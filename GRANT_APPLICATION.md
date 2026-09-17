# Solana Foundation grant application: form answers

Authoritative form draft, checked against the live form on 2026-09-17.
Applicant name confirmed by Satyawan Singh. Funding request: **USD 22,000**.

Apply through [Foundation Grants](https://share.hsforms.com/1GE1hYdApQGaDiCgaiWMXHA5lohw),
linked from [solana.org/grants-funding](https://solana.org/grants-funding).
Selecting Developer Tooling reveals an additional required field for a shared Google Doc.
Use `DEVELOPER_TOOLING_PROPOSAL.md` for that document. This replaces the earlier milestone-only
application; the supporting proposal is a separate required attachment, not a competing form draft.

**Status: draft, not submitted.** The shared Google Doc URL and applicant approval of the
proposal's release, maintenance, and adoption commitments are still needed. Devnet deployment is
pending free test SOL. The Foundation's application guide explicitly permits `N/A` for on-chain
accounts when not applicable; a local validator address must not be presented as a devnet deployment.

## 1. Company name

Satyawan Singh

## 2. Website URL

https://caledren.com

## 3. Country

United Kingdom

## 4. First Name

Satyawan

## 5. Last Name

Singh

## 6. Email Address

satyawansinghinuk@gmail.com

## 7. Solana On-Chain Accounts

N/A

*Preparation note, do not paste: replace this with the confirmed devnet program ID and fee payer,
comma-separated, after deployment. No public Solana deployment is claimed yet. See `DEPLOYMENTS.md`.*

## 8. Funding Amount

22000

## 9. Which funding category are you applying for?

Developer Tooling

## 10. Why You?

I am a solo Rust and cryptography developer with experience building a blockchain from scratch,
including consensus, light clients and data-availability sampling, and working with BLS12-381
aggregate signatures, Nova recursive SNARKs, Groth16, KZG/Verkle commitments, on-chain BLS
(EIP-2537), and Coq/TLA+ formal verification.

I have built an MIT-licensed private-credit proof and approval workflow that another developer can
reproduce locally with one command. It ingests a loan CSV, reuses persisted Groth16 parameters,
privately checks the servicer's book commitment and writes an authenticated approval receipt on a
local Solana validator. The compiled program pins the approved key and servicer and enforces
minimum threshold, freshness and sequence. Its runtime suite accepts successive approvals and
rejects 26 invalid requests without changing state. A fresh finalized local approval used 85,529
compute units with two signers (a repeat run measured 85,538 CU); a separate verifier example is
confirmed on public Ethereum Sepolia.

My edge is a working, inspectable reference implementation with explicit trust boundaries. It
complements broader institutional privacy systems such as Zyga. The grant funds the next release:
developer integration and hardening, a pinned setup compatibility gate followed by an externally
contributed ceremony, maintenance and independently evidenced adoption. This remains a pilot alpha
with single-party setup, no live supply or SPL token integration, and no external security audit or
customer traction claimed.

## 11. Developer Tooling Instructions (conditional required field)

[Paste the shared Google Doc URL containing DEVELOPER_TOOLING_PROPOSAL.md after reviewing it.]

*Preparation note, do not paste: the live form asks for a proposal following its linked Developer
Tooling template. Create/share the Google Doc yourself, then insert its accessible URL here. Do not
paste the proposal body into Why You. The form asks that links appear only in fields requesting them.*

## Internal preparation notes (not form answers)

- Proposed budget: $5,000 developer kit; $12,000 setup tooling ($2,000 compatibility and launch gate,
  $10,000 externally contributed ceremony and release); $3,000 maintenance ($500/month for six
  months); $2,000 adoption. Total: $22,000. Full acceptance criteria are in the supporting proposal.
- The original $300,000 application was rejected because the ask exceeded the public good delivered.
  This draft funds reusable open-source components and measurable maintenance/adoption. It does not
  fund a bespoke integration for a named commercial protocol.
- Caledren uses caledren.com. The pilot code, evidence and site are pushed to `main` and live at
  that domain (verified 2026-09-17, commit f4cf7e6, byte-identical to the deployed site).
- No application has been submitted.
- [General application guidance](https://docs.google.com/document/d/1eK-WNhQmQFyk06XhwWRZJrXn3z_0NDbChZ_Rm8XRoXw/edit)
  permits N/A in the on-chain accounts field.
- [Developer Tooling template](https://docs.google.com/document/d/1S28sq80-1Nz5FnD2yepEaldrmbRDB-2Q7g0CilNdHOA/edit)
  requires component betas, at least six months of maintenance, quantifiable adoption milestones,
  and a published production version by the end of the agreement. These are proposed commitments
  for the applicant to approve, not statements that the present prototype is production-safe.
