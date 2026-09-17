# First external developer evaluation

Status: ready for a local technical evaluation. No external team, integration or customer is claimed.
This checklist is for a protocol engineer and its servicing/data contact, using synthetic data first.

## Evaluation outcome

Determine whether a private-book proof and authenticated approval receipt are useful inputs to the
protocol's risk checks. Record the developer's own run, integration friction and missing policy
requirements. Do not use real funds or treat a successful demo as a production authorization.

| Step | Acceptance evidence |
|---|---|
| Reproduce | Run `./pilot-demo.sh` from the reviewed source snapshot; retain the public approval report |
| Reuse parameters | Generate two proofs with the same parameter directory and confirm the same key hash |
| Bind the private book | Change collateral after proving and show that the private attestation check fails |
| Enforce policy | Reproduce the compiled runtime suite's wrong signer, wrong key, low threshold, expired claim and replay rejections |
| Consume a receipt | Read an approved configuration, check owner/expiry/threshold, and reject an arbitrary substituted configuration |
| Specify real integration | Identify feed owner, units, valuation timestamps, threshold source, signer custody, approval lifetime and policy rotation |
| Record feedback | Document time to first approval, blockers, required changes and whether the team wants another evaluation |

Record team, engineer, date, source snapshot, runtime versions, hardware, public evidence location
and permission to cite the results. Keep loan tapes, nonces and keypairs private. An anonymized
team name is acceptable if the reviewer can inspect consented evidence. Applicant-run demos,
forks without integration, planned calls and expressions of interest do not count as adoption.

## Evidence register

| Team | Date | Independent run | Consumer integration | Evidence / consent |
|---|---|---|---|---|
| None yet | Pending | Pending | Pending | No external adoption claimed |

## Submission evidence that still needs to be earned

The current code can support a precise claim about a working local pilot alpha. The next stronger
claim requires a finalized public devnet approval and at least one external developer's reproducible
integration with consent to cite it. A complete setup ceremony and independent security review are
separate milestones; neither is implied by this evaluation. The four-team grant adoption target
remains a proposed future commitment, not an achieved result.
