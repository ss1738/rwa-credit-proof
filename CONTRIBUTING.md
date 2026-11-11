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

## Website build and deployment

The public site is https://caledren.com (also https://www.caledren.com), hosted by the existing
Vercel project `satyawansinghs-projects/rwa-credit-proof`. This repository is linked directly to that
project. Deployments are manual from the repository root; GitHub auto-deploy is not configured.
A push does not publish a reviewed release automatically.

```bash
cd ~/rwa-credit-proof
# Once per fresh checkout, using the already authenticated Vercel CLI:
vercel link --yes --scope satyawansinghs-projects --project rwa-credit-proof
# Build and inspect locally:
npm run build
python3 -m http.server 8765 --bind 127.0.0.1 --directory dist
# Create a preview from this checkout:
vercel deploy --scope satyawansinghs-projects
# Only after checking the preview and receiving explicit production approval:
vercel promote <reviewed-preview-url> --scope satyawansinghs-projects
```

The website source lives in `site/`; `scripts/build-site.mjs` generates `dist/` using the dated
core receipt/log bundle and the latest separately dated devnet-status record linked by
`DEPLOYMENTS.md`. It checks those facts against the deployment and
limitation records and stops if they disagree. `EVIDENCE.html` remains the separate printable
technical sheet and is copied into the built site unchanged. Only the generated `dist/` is served;
the deployment upload is restricted by `.vercelignore` to website source and public evidence inputs.
`.vercel/`, local environment files and generated output are ignored by Git.

The animated proof sphere uses local Canvas and CSS, with no external rendering dependencies.
It illustrates the private-input/public-proof flow; it does not represent live network activity.
Motion can be paused, respects the operating system's reduced-motion setting, and stops rendering
the sphere when the hero is off screen or the tab is hidden. Core content remains available without
JavaScript. Run `npm run check` after building to validate links, source evidence and public output.

The other sections use SVG/CSS motion: evidence traces, a Commit/Prove/Verify path, an illustrated
native gate that responds to the chosen scenario, audit graphics, execution-scope boundaries and
a design-partner invitation. Recorded measurements stay fixed. Each section pauses its decorative
animations off screen, and the global motion control applies to the whole page. The sticky navigation
includes an Explore menu on smaller screens.

Do not deploy from a scratch copy, connect other Vercel projects, or promote production or push to
GitHub without explicit approval. To update a network claim, update `DEPLOYMENTS.md` and its
receipt/status evidence together, then rebuild and check a new preview.
