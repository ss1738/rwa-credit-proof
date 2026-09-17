// Local compatibility rehearsal. All three contributions are operated on this one machine.
// snarkjs retains its upstream GPL-3.0 license; it is an external, pinned ceremony tool.
import { createHash, randomBytes } from 'node:crypto';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import assert from 'node:assert/strict';
import * as snarkjs from 'snarkjs';

const [runArg] = process.argv.slice(2);
assert(runArg, 'usage: node rehearse.mjs <run-dir>');
const run = resolve(runArg), ptau = join(run,'phase1-final.ptau');
const logger = { info() {}, debug() {}, warn() {}, error(message) { console.log(`VERIFIER_DIAGNOSTIC ${message}`); } };
const sha = data => createHash('sha256').update(data).digest('hex');
const check = (condition, name) => { assert(condition, name); console.log(`CEREMONY_CHECK_PASSED ${name}`); };
const json = async (path, value) => writeFile(path, JSON.stringify(value, null, 2) + '\n', {flag:'wx',mode:0o600});
const timings = {}, checks = [];
async function stage(name, action) {
  console.log(`CEREMONY_STAGE ${name}`);
  const start = performance.now(); const result = await action();
  timings[name] = Math.round(performance.now() - start);
  return result;
}
async function rejects(name, action) {
  let rejected = false;
  try { rejected = await action() === false; } catch (error) {
    // These tests start from verified artifacts and mutate only the named component.
    rejected = true; console.log(`EXPECTED_REJECTION ${name}: ${error.message}`);
  }
  check(rejected, name);checks.push(name);
}
function sectionOffset(bytes, target) {
  let offset=12;
  for (let n=0; n<bytes.readUInt32LE(8); n++) {
    const id=bytes.readUInt32LE(offset), length=Number(bytes.readBigUInt64LE(offset+4));
    if (id===target) return offset+12;
    offset += 12+length;
  }
  throw new Error(`Missing section ${target}`);
}

try {
  const curve=await snarkjs.curves.getCurveFromName('bn128');
  let phase1=join(run,'phase1-initial.ptau');
  // The exported 12-loan fixture has 6,648 constraints, so power 13 (8,192 capacity) is sufficient
  // for this compatibility rehearsal and keeps a clean local run practical. The funded ceremony
  // will choose a public phase-1 capacity with documented headroom for the production circuit.
  await stage('phase1_initialize',()=>snarkjs.powersOfTau.newAccumulator(curve,13,phase1,logger));
  await rejects('uncontributed_phase1',()=>snarkjs.powersOfTau.verify(phase1,logger));
  const phase1Contributions=[];
  for (let i=1;i<=3;i++) {
    const next=join(run,`phase1-contribution-${i}.ptau`);
    const contribution=await stage(`phase1_local_contribution_${i}`,()=>snarkjs.powersOfTau.contribute(phase1,next,`LOCAL-REHEARSAL-${i}`,randomBytes(64).toString('hex'),logger));
    check(await snarkjs.powersOfTau.verify(next,logger),`phase1_contribution_${i}_verified`);
    phase1Contributions.push(Buffer.from(contribution).toString('hex'));phase1=next;
  }
  const phase1Beacon=randomBytes(32).toString('hex');
  const phase1BeaconPath=join(run,'phase1-beacon.ptau');
  await stage('phase1_local_beacon',()=>snarkjs.powersOfTau.beacon(phase1,phase1BeaconPath,'LOCAL-REHEARSAL-BEACON',phase1Beacon,10,logger));
  await stage('prepare_phase2',()=>snarkjs.powersOfTau.preparePhase2(phase1BeaconPath,ptau,logger));
  check(await stage('verify_final_phase1',()=>snarkjs.powersOfTau.verify(ptau,logger)), 'final_phase1_verified');
  const first=join(run,'fixture-1'), second=join(run,'fixture-2'), mismatch=join(run,'fixture-mismatch');
  const r1cs=join(first,'circuit.r1cs');
  check((await readFile(r1cs)).equals(await readFile(join(second,'circuit.r1cs'))), 'same_matrix_for_different_private_witnesses');
  for (const fixture of [first,second]) {
    check(await snarkjs.wtns.check(r1cs,join(fixture,'private.wtns'),logger), 'arkworks_witness_satisfies_exported_r1cs');
  }
  const badWitness=Buffer.from(await readFile(join(first,'private.wtns')));
  badWitness[sectionOffset(badWitness,2)+64] ^= 1; // Public commitment at wire 2.
  const badWitnessPath=join(run,'private-corrupt.wtns');await writeFile(badWitnessPath,badWitness,{flag:'wx',mode:0o600});
  await rejects('altered_witness_commitment',()=>snarkjs.wtns.check(r1cs,badWitnessPath,logger));
  let previous=join(run,'initial.zkey');
  await stage('circuit_specific_setup',()=>snarkjs.zKey.newZKey(r1cs,ptau,previous,logger));
  const initVk=await snarkjs.zKey.exportVerificationKey(previous,logger);
  const contributionHashes=[];
  for (let i=1;i<=3;i++) {
    const next=join(run,`contribution-${i}.zkey`);
    // Entropy is local OS randomness, never a fixed public seed or printed command argument.
    const contribution=await stage(`local_contribution_${i}`,()=>snarkjs.zKey.contribute(previous,next,`LOCAL-REHEARSAL-${i}`,randomBytes(64).toString('hex'),logger));
    check(await snarkjs.zKey.verifyFromR1cs(r1cs,ptau,next,logger),`contribution_${i}_verified_against_r1cs_and_phase1`);
    contributionHashes.push(Buffer.from(contribution).toString('hex'));previous=next;
  }
  const final=join(run,'final.zkey');
  const rehearsalBeacon=randomBytes(32).toString('hex');
  await stage('local_beacon_rehearsal',()=>snarkjs.zKey.beacon(previous,final,'LOCAL-REHEARSAL-BEACON',rehearsalBeacon,10,logger));
  check(await snarkjs.zKey.verifyFromR1cs(r1cs,ptau,final,logger),'final_phase2_transcript_verified');
  const vk=await snarkjs.zKey.exportVerificationKey(final,logger);
  for (const [i,fixture] of [[1,first],[2,second]]) {
    const output=join(run,`snarkjs-${i}`);await mkdir(output,{mode:0o700});
    const {proof, publicSignals}=await stage(`prove_witness_${i}`,()=>snarkjs.groth16.prove(final,join(fixture,'private.wtns'),logger));
    check(await snarkjs.groth16.verify(vk,publicSignals,proof,logger),`proof_${i}_verifies_with_final_key`);
    assert.deepEqual(publicSignals,JSON.parse(await readFile(join(fixture,'expected-public.json'),'utf8')));
    await json(join(output,'verification-key.json'),vk);await json(join(output,'proof.json'),proof);await json(join(output,'public.json'),publicSignals);
    if (i===1) {
      for (let j=0;j<2;j++) {
        const altered=[...publicSignals];altered[j]=(BigInt(altered[j])+1n).toString();
        await rejects(`altered_public_input_${j}`,()=>snarkjs.groth16.verify(vk,altered,proof,logger));
      }
      await rejects('final_proof_under_initial_key',()=>snarkjs.groth16.verify(initVk,publicSignals,proof,logger));
    }
  }
  const corrupt=Buffer.from(await readFile(final));
  // MPC section: 64-byte circuit hash, u32 count, then the first contribution's deltaAfter G1.
  corrupt[sectionOffset(corrupt,10)+68] ^= 1;
  const corruptPath=join(run,'corrupt-contribution.zkey');await writeFile(corruptPath,corrupt,{flag:'wx'});
  await rejects('corrupted_contribution_point',()=>snarkjs.zKey.verifyFromR1cs(r1cs,ptau,corruptPath,logger));
  await rejects('different_circuit_same_final_key',()=>snarkjs.zKey.verifyFromR1cs(join(mismatch,'circuit.r1cs'),ptau,final,logger));
  const fixture=JSON.parse(await readFile(join(first,'metadata.json'),'utf8'));
  await json(join(run,'ceremony-report.json'),{
    schema_version:1,scope:'Compatibility rehearsal only. Three local contributions are NOT three independent participants. No production ceremony completed.',
    tool:'iden3/snarkjs',version:'0.7.6',phase1_source:'Locally initialized power-13 accumulator with three local contributions',phase1_sha256:sha(await readFile(ptau)),
    phase1_verified:true,final_transcript_verified:true,local_phase1_contributions:3,local_phase2_contributions:3,independent_participants_recruited:0,
    fixture,phase1_contribution_hashes:phase1Contributions,contribution_hashes:contributionHashes,phase1_rehearsal_beacon:phase1Beacon,rehearsal_beacon:rehearsalBeacon,
    beacon_scope:'Locally generated demo value; not a precommitted future public beacon',
    final_zkey_sha256:sha(await readFile(final)),verification_key_json_sha256:sha(await readFile(join(run,'snarkjs-1/verification-key.json'))),
    negative_checks:checks,timings_ms:timings});
  console.log('CEREMONY_SNARKJS_REHEARSAL_PASSED');
} finally {
  const curve=await snarkjs.curves.getCurveFromName('bn128');await curve.terminate();
}
