// Authored/unrun pure documentation-boundary regressions; no provider/browser.
import test from 'node:test';
import assert from 'node:assert/strict';
import {catalog, report, evidence, fixturePaths, parse, MAX_BYTES} from '../docs/integrations/showcase/model.mjs';
const sha = 'a'.repeat(64), source = 'b'.repeat(64);
const fixture = () => ({schema_version:'flow.acceptance-scenario-catalog/v1',fixture_version:'1.0.0',tier:'synthetic',budget:{},known_gaps:['No native qualification.'],
  scenarios:[{scenario_id:'scenario:fixture-refusal',family:'artifact',recipe:'fixture',expected:{code:'invalid',accepted:false}}]});
test('expected refusal is catalog intent and cannot create a verified result', () => {
  const current = catalog(fixture(), sha, 'fixture.json');
  assert.equal(evidence(current).state, 'unavailable');
  assert.equal(current.rows[0].expected.accepted, false);
  assert.throws(() => catalog({...fixture(),scenarios:[...fixture().scenarios,...fixture().scenarios]},sha,'fixture'));
});
test('matching report remains source-unconfirmed and changed catalog cannot appear current', () => {
  const current = catalog(fixture(),sha,'fixture');
  const supplied = report({schema_version:'flow.acceptance-scenario-report/v1',catalog_digest:sha,source_identity:{algorithm:'sha256',digest:source},
    status:'passed',scenario_count:1,toolchain:'fixture only',receipts:[{scenario_id:'scenario:fixture-refusal',outcome:{accepted:false,code:'invalid'}}]},source);
  assert.equal(evidence(current,supplied).state,'source-unconfirmed');
  assert.equal(evidence({...current,rawSha256:source},supplied).state,'catalog-mismatch');
  supplied.value.receipts[0].outcome.accepted = true;
  assert.equal(evidence(current,supplied).state,'incomplete-or-conflicting');
});
test('duplicate receipts, unsafe fixture locators and oversized documents refuse', () => {
  assert.throws(() => report({schema_version:'flow.lifecycle-scenario-report/v1',catalog_digest:sha,source_identity:{algorithm:'sha256',digest:source},toolchain:'fixture',receipts:[{scenario_id:'x'},{scenario_id:'x'}]},sha));
  for (const path of ['https://example.com/a.json','fixtures/scenarios/../secret.json','fixtures/scenarios/%2e%2e.json'])
    assert.throws(() => fixturePaths({schema_version:'flow.scenario-digests/v1',entries:[{path}]}));
  assert.throws(() => parse(new Uint8Array(MAX_BYTES+1)));
});
