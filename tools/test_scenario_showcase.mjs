// Authored/unrun pure documentation-boundary regressions; no provider/browser.
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {explainScenario} from '../docs/integrations/showcase/explanations.mjs';
import {catalog, report, evidence, fixturePaths, parse, MAX_BYTES, scenarioLink, scenarioSelection, resolveScenario} from '../docs/integrations/showcase/model.mjs';
const sha = 'a'.repeat(64), source = 'b'.repeat(64);
test('every current canonical recipe has an explanation keyed by its actual ID', async () => {
  for (const relative of ['tests/fixtures/acceptance-scenarios.v1.json','tests/fixtures/lifecycle-scenarios.v1.json']) {
    const bytes = await readFile(new URL('../' + relative, import.meta.url));
    const value = parse(new Uint8Array(bytes));
    const current = catalog(value, createHash('sha256').update(bytes).digest('hex'), relative);
    const explained = new Map(current.rows.map(row => [row.id, explainScenario(row)]));
    assert.equal(explained.size, value.scenarios.length);
    for (const row of current.rows) {
      const explanation = explained.get(row.id);
      assert.equal(explanation.scenarioId, row.id); assert.equal(explanation.authority, 'authored_expectation_only');
      assert.notEqual(explanation.purpose, 'Inspect an unclassified scenario family');
      assert.ok(explanation.title.length && explanation.why.length && explanation.setup.length && explanation.decision.length);
    }
  }
});
test('a passing refusal remains different from artifact acceptance in the explanation', () => {
  const row = {id:'scenario:synthetic-refusal',kind:'acceptance',recipe:'missing-provider',family:'resolution',code:'no-compatible-provider',expected:{accepted:false}};
  assert.match(explainScenario(row).decision,/does not accept artifacts/);
  assert.equal(evidence(catalog(fixture(),sha,'synthetic')).state,'unavailable');
});
test('lifecycle explanation counts only declared final checkpoints', () => {
  const row={id:'scenario:synthetic-recovery',kind:'lifecycle',recipe:'changed-input',graph:true,code:'stale',recovery:'operator-decision-required',expected:{history:[{steps:[{status:'stale',checkpoint:false},{status:'complete',checkpoint:true}]}]}};
  const value=explainScenario(row);assert.match(value.decision,/2 steps, 1 checkpoints/);assert.match(value.setup,/A → B/);
  assert.equal(value.authority,'authored_expectation_only');
  assert.throws(()=>explainScenario({...row,id:'../not-a-scenario'}));
});
test('catalog-specific links disambiguate identical canonical IDs', () => {
  const rows = [{kind:'acceptance',id:'scenario:same'}, {kind:'lifecycle',id:'scenario:same'}];
  const selected = resolveScenario(rows, scenarioSelection(scenarioLink('lifecycle','scenario:same')));
  assert.equal(selected.state, 'selected'); assert.equal(selected.row.kind, 'lifecycle');
  assert.equal(resolveScenario(rows, scenarioSelection('#scenario%3Asame')).state, 'ambiguous');
});
test('links wait for explicit catalogs without inventing verification or loading paths', () => {
  const selection = scenarioSelection(scenarioLink('acceptance','scenario:not-loaded'));
  assert.equal(resolveScenario([],selection).state,'missing');
  assert.equal(scenarioSelection('#browse-title'),null);
  assert.equal(scenarioSelection('#javascript:alert(1)'),null);
});
test('malformed or traversal-like scenario fragments cannot become locators', () => {
  for (const fragment of ['#scenario=acceptance/%GG','#scenario=acceptance/..%2Fsecret','#scenario=foreign/scenario%3Afoo','#scenario='+ 'a'.repeat(1024)]) assert.throws(() => scenarioSelection(fragment));
  assert.throws(() => scenarioLink('acceptance','https://example.com/private'));
});
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
