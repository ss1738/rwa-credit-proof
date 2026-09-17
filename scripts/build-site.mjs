import { readFile, writeFile, mkdir, rm, cp } from 'node:fs/promises';
import { resolve, dirname, relative } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';
import assert from 'node:assert/strict';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const output = resolve(root, 'dist');
const read = path => readFile(resolve(root, path), 'utf8');
const deployments = await read('DEPLOYMENTS.md');
const limitations = await read('KNOWN_LIMITATIONS.md');
const auditRecord = await read('AUDIT.md');
const evidenceDate = deployments.match(/^Checked (\d{4}-\d{2}-\d{2})\./m)?.[1];
assert(evidenceDate, 'DEPLOYMENTS.md must declare its evidence date.');
const evidenceDir = `evidence/${evidenceDate}`;
assert(deployments.includes(`${evidenceDir}/local-receipt.json`), 'Deployment record must link the dated receipt.');
const inputs = new Map();
async function source(path) {
  const value = await read(path);
  inputs.set(path, createHash('sha256').update(value).digest('hex'));
  return value;
}
const local = JSON.parse(await source(`${evidenceDir}/local-receipt.json`));
const sepolia = JSON.parse(await source(`${evidenceDir}/sepolia-summary.json`));
const devnet = JSON.parse(await source(`${evidenceDir}/devnet-status.json`));
const audit = await source(`${evidenceDir}/audit.log`);
const benchmark = await source(`${evidenceDir}/bench-10000.log`);
const demo = await source(`${evidenceDir}/demo.log`);
const match = (text, regex, label) => {
  const value = text.match(regex);
  assert(value, `Missing ${label} in source evidence.`);
  return value;
};
const format = value => Number(value).toLocaleString('en-US');
const constraints = match(audit, /constraints in the circuit\s*:\s*(\d+)/, 'constraint count')[1];
const proofBytes = match(audit, /proof is (\d+) bytes/, 'compressed proof size')[1];
const instructionBytes = match(limitations, /(\d+) bytes including uncompressed/, 'instruction size')[1];
const historicalCu = match(deployments, /Earlier documented local-validator run:\s*\*\*([\d,]+) CU/, 'historical local CU')[1];
const bench = match(benchmark, /n=\s*(\d+)\s+setup=\s*([\d.]+)s\s+prove=\s*([\d.]+)s.*proof=(\d+)B\s+verified=true/, 'successful benchmark');
assert(local.confirmationStatus === 'finalized' && local.meta.err === null, 'Local transaction must be finalized and successful.');
assert(sepolia.status === 'Success' && sepolia.event_ok === true, 'Sepolia example must be successful.');
assert(demo.includes('All stages passed.') && audit.includes('AUDIT PASSED'), 'Recorded demo and audit must pass.');
assert.equal(Number(match(demo, /COMPUTE_UNITS_CONSUMED: (\d+)/, 'SBF compute units')[1]), local.meta.computeUnitsConsumed, 'SBF and local CU evidence differ; update the site wording.');
assert.equal(proofBytes, bench[4], 'Audit and benchmark compressed proof sizes differ.');
assert(bench[1] === '10000', 'Update the benchmark description when its fixture changes.');
assert(sepolia.checked === evidenceDate && devnet.checked === evidenceDate, 'Evidence dates differ; refresh the deployment record.');
for (const value of [format(local.meta.computeUnitsConsumed), format(constraints), bench[2], bench[3], format(sepolia.verifier_gas_from_event), format(sepolia.total_transaction_gas), sepolia.contract]) {
  assert(deployments.includes(value), `DEPLOYMENTS.md disagrees with evidence value ${value}.`);
}
assert(auditRecord.includes(format(constraints)) && auditRecord.includes(format(local.meta.computeUnitsConsumed)), 'AUDIT.md disagrees with current measurements.');
assert(limitations.includes('key') && limitations.includes('does not verify a servicer signature') && limitations.includes('or invoke an SPL token mint'), 'Revisit website scope copy when the on-chain implementation changes.');

let devnetLabel, devnetDescription, devnetClass, devnetSource, devnetSourceLabel;
if (devnet.status === 'not_deployed') {
  assert(deployments.includes('## Solana devnet: pending funding') && devnet.program_account_exists === false, 'Devnet prose/status disagree.');
  devnetLabel = 'Pending funding';
  devnetDescription = 'Not deployed. Public faucet requests were rate-limited; no devnet verification transaction is claimed.';
  devnetClass = 'pending';
  devnetSource = `/${evidenceDir}/devnet-status.json`;
  devnetSourceLabel = 'Status record';
} else if (devnet.status === 'deployed') {
  assert(deployments.includes('## Solana devnet: deployed'), 'Update the devnet deployment heading before publishing.');
  assert(devnet.program_account_exists === true && devnet.program_id && devnet.verification_signature && devnet.receipt_path, 'Deployed status requires actual program and transaction evidence.');
  assert(devnet.receipt_path.startsWith(`${evidenceDir}/`) && !devnet.receipt_path.includes('..'), 'Devnet receipt must be in the dated evidence directory.');
  const receipt = JSON.parse(await source(devnet.receipt_path));
  assert(receipt.confirmationStatus === 'finalized' && receipt.meta.err === null, 'Devnet receipt must be finalized and successful.');
  assert(receipt.transaction.signatures.includes(devnet.verification_signature), 'Devnet receipt signature mismatch.');
  assert(receipt.transaction.message.accountKeys.some(key => (typeof key === 'string' ? key : key.pubkey) === devnet.program_id), 'Devnet receipt does not invoke the recorded program.');
  assert(deployments.includes(devnet.program_id) && deployments.includes(devnet.verification_signature), 'Deployment record lacks actual devnet addresses.');
  devnetLabel = 'Confirmed on devnet';
  devnetDescription = 'A public proof-verification transaction is finalized. The program checks the pairing equation only.';
  devnetClass = '';
  devnetSource = `https://explorer.solana.com/tx/${devnet.verification_signature}?cluster=devnet`;
  devnetSourceLabel = 'Transaction';
} else {
  throw new Error(`Unsupported devnet state ${devnet.status}; review its evidence and website wording.`);
}

const escapeHtml = value => String(value).replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;').replaceAll('"', '&quot;');
function inlineMarkdown(text) {
  return escapeHtml(text)
    .replace(/\[([^\]]+)\]\(([^)]+)\)/g, (_, label, target) => {
      const href = target.startsWith('https://') ? target : `https://github.com/ss1738/rwa-credit-proof/blob/main/${target}`;
      return `<a href="${href}">${label}</a>`;
    })
    .replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
    .replace(/`([^`]+)`/g, '<code>$1</code>');
}
const limitDefinitions = [[1, 'Trusted setup'], [3, 'Collateral bounds'], [5, 'On-chain enforcement'], [6, 'Instruction validation']];
const limitHtml = limitDefinitions.map(([number, label]) => {
  const section = match(limitations, new RegExp(`^## ${number}\\. [^\\n]+\\n([\\s\\S]*?)(?=^## |$(?![\\s\\S]))`, 'm'), `limitation section ${number}`)[1];
  const paragraphs = section.trim().split(/\n\s*\n/).map(text => `<p>${inlineMarkdown(text.replaceAll('\n', ' '))}</p>`).join('');
  return `<details><summary>${label}</summary><div class="limit-body">${paragraphs}</div></details>`;
}).join('\n');
const tokens = {
  PROOF_BYTES: proofBytes, INSTRUCTION_BYTES: instructionBytes,
  EVIDENCE_DATE: evidenceDate,
  EVIDENCE_DATE_LONG: new Date(`${evidenceDate}T12:00:00Z`).toLocaleDateString('en-GB', { day: 'numeric', month: 'long', year: 'numeric', timeZone: 'UTC' }),
  LOCAL_CU: format(local.meta.computeUnitsConsumed), CONSTRAINTS: format(constraints),
  BENCH_N: format(bench[1]), PROVE_SECONDS: bench[3], SETUP_SECONDS: bench[2],
  HISTORICAL_CU: historicalCu,
  EVM_GAS: format(sepolia.verifier_gas_from_event), EVM_TOTAL_GAS: format(sepolia.total_transaction_gas),
  SEPOLIA_TX_URL: sepolia.source,
  DEVNET_CLASS: devnetClass, DEVNET_LABEL: devnetLabel, DEVNET_DESCRIPTION: devnetDescription,
  DEVNET_SOURCE: devnetSource, DEVNET_SOURCE_LABEL: devnetSourceLabel,
};
let html = await source('site/index.html');
html = html.replace(/\{\{([A-Z_]+)\}\}/g, (_, token) => {
  if (token === 'LIMITATIONS_HTML') return limitHtml;
  assert(Object.hasOwn(tokens, token), `Unknown template token: ${token}`);
  return escapeHtml(tokens[token]);
});
assert(!html.includes('{{'), 'Unresolved template tokens.');
assert(!html.includes('\u2014'), 'Public copy must not contain em dashes.');
await rm(output, { recursive: true, force: true });
await mkdir(output, { recursive: true });
await writeFile(resolve(output, 'index.html'), html);
for (const name of ['styles.css', 'site.js', 'favicon.svg']) {
  await source(`site/${name}`);
  await cp(resolve(root, 'site', name), resolve(output, name));
}
for (const name of ['EVIDENCE.html', 'DEPLOYMENTS.md', 'KNOWN_LIMITATIONS.md', 'AUDIT.md', 'README.md', 'DESIGN_PARTNER_BRIEF.md', 'PILOT_GUIDE.md', 'PILOT_EVALUATION.md', 'LICENSE']) {
  await source(name);
  await cp(resolve(root, name), resolve(output, name));
}
await cp(resolve(root, evidenceDir), resolve(output, evidenceDir), { recursive: true });
const provenance = {
  evidence_date: evidenceDate,
  source_repository: 'https://github.com/ss1738/rwa-credit-proof',
  source_files_sha256: Object.fromEntries(inputs),
  measurements: { local_compute_units: local.meta.computeUnitsConsumed, constraints: Number(constraints), compressed_proof_bytes: Number(proofBytes), instruction_bytes: Number(instructionBytes), loans: Number(bench[1]), setup_seconds: Number(bench[2]), proving_seconds: Number(bench[3]), sepolia_verifier_gas: sepolia.verifier_gas_from_event, sepolia_total_transaction_gas: sepolia.total_transaction_gas },
  devnet_status: devnet.status,
};
await writeFile(resolve(output, 'provenance.json'), JSON.stringify(provenance, null, 2) + '\n');
await writeFile(resolve(output, 'robots.txt'), 'User-agent: *\nAllow: /\nSitemap: https://caledren.com/sitemap.xml\n');
await writeFile(resolve(output, 'sitemap.xml'), '<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"><url><loc>https://caledren.com/</loc></url><url><loc>https://caledren.com/EVIDENCE.html</loc></url></urlset>\n');
console.log(`Built ${relative(root, output)}/ from verified ${evidenceDate} evidence. Devnet: ${devnet.status}.`);
