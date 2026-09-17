import { readFile, stat, readdir } from 'node:fs/promises';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import assert from 'node:assert/strict';
const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const output = resolve(root, 'dist');
const html = await readFile(resolve(output, 'index.html'), 'utf8');
const sourceEvidence = await readFile(resolve(root, 'EVIDENCE.html'));
assert(sourceEvidence.equals(await readFile(resolve(output, 'EVIDENCE.html'))), 'Print evidence sheet must remain unchanged.');
const ids = [...html.matchAll(/\bid="([^"]+)"/g)].map(match => match[1]);
assert.equal(new Set(ids).size, ids.length, 'HTML IDs must be unique.');
for (const path of ['index.html', 'EVIDENCE.html']) {
  const content = await readFile(resolve(output, path), 'utf8');
  assert(!content.includes('\u2014'), `Em dash found in ${path}`);
  assert(!content.includes('{{'), `Unexpanded token in ${path}`);
  for (const [, target] of content.matchAll(/\b(?:href|src)="([^"]+)"/g)) {
    if (/^(https?:|mailto:|data:)/.test(target)) continue;
    if (target.startsWith('#')) { if (path === 'index.html') assert(ids.includes(target.slice(1)), `Missing anchor ${target}`); continue; }
    const pathname = target.split(/[?#]/)[0];
    const local = resolve(output, pathname.replace(/^\//, '') || 'index.html');
    assert(local.startsWith(`${output}/`), `Link escapes public output: ${target}`);
    await stat(local);
  }
}
async function walk(directory) {
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const path = resolve(directory, entry.name);
    assert(!/^(\.env|\.git|\.vercel|CODEX_)/.test(entry.name), `Private/build metadata exposed: ${path}`);
    assert(!/keypair|payer\.json|\.so$/.test(entry.name), `Non-public artifact in output: ${path}`);
    if (entry.isDirectory()) await walk(path);
  }
}
await walk(output);
const provenance = JSON.parse(await readFile(resolve(output, 'provenance.json')));
const evidenceRoot = resolve(output, 'evidence', provenance.evidence_date);
const receipt = JSON.parse(await readFile(resolve(evidenceRoot, 'local-receipt.json')));
const sepolia = JSON.parse(await readFile(resolve(evidenceRoot, 'sepolia-summary.json')));
assert.equal(provenance.measurements.local_compute_units, receipt.meta.computeUnitsConsumed);
assert.equal(provenance.measurements.sepolia_total_transaction_gas, sepolia.total_transaction_gas);
assert(html.includes('no token is minted') && html.includes('verifies the pairing equation only'), 'Scope caveats missing.');
console.log('Site checks passed: local links, anchors, print artifact, evidence, scope and output files.');
