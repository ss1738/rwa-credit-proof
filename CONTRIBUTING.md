# Contributing

Contributions that improve reproducibility, integration safety, documentation or test coverage are
welcome. This repository is a local pilot alpha, not production financial infrastructure.

## Start here

1. Read [README.md](README.md), [PILOT_GUIDE.md](PILOT_GUIDE.md) and
   [KNOWN_LIMITATIONS.md](KNOWN_LIMITATIONS.md).
2. Run `./pilot-demo.sh` for the current local workflow or `./demo.sh` for the legacy verifier example.
3. Keep private loan tapes, commitment nonces, payer files and keypairs outside Git.

The checked pilot used the tool versions recorded in [DEPLOYMENTS.md](DEPLOYMENTS.md). The setup
compatibility workflow additionally requires Node 24 LTS and the lockfile-pinned ceremony packages.

## Change requirements

- Add or update tests for behavior changes.
- Preserve the distinction between local-validator, devnet and public-chain evidence.
- Describe measured observations with their date, environment and evidence path.
- Do not describe a single measurement as a performance guarantee.
- Do not call developer-run tests or code-review passes an external security audit.
- Do not imply token minting, live supply integration, customer adoption or a completed public
  ceremony unless independently inspectable evidence is committed.
- Update `DEPLOYMENTS.md`, dated evidence and public copy together when a network claim changes.

For website changes, run:

```bash
npm run build
npm run check
```

For Rust changes, run the relevant locked tests and record the exact command and environment. The full
pilot script requires the Solana SBF toolchain and a local validator.

## External evaluation evidence

Use [PILOT_EVALUATION.md](PILOT_EVALUATION.md) for independent reproduction or consumer-integration
results. Include the source commit, runtime versions, hardware, public evidence location and permission
to cite the result. Never commit private source data merely to prove an evaluation occurred.

## Security reports

Do not publish exploitable details in a GitHub issue. Follow [SECURITY.md](SECURITY.md) for the private
reporting channel and current security scope.
