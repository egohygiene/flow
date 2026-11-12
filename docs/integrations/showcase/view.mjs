import {loadWorkflows, clearWorkflows} from './workflow-view.mjs';
import {MAX_BYTES, parse, catalog, report, evidence, manifest, fixturePaths} from './model.mjs';
const catalogs = new Map(), reports = new Map(), manifests = new Map();
const root = new URL('../../../', import.meta.url);
const $ = id => document.getElementById(id);
function node(tag, value, parent) {
  const element = document.createElement(tag);
  if (value !== undefined) element.textContent = value;
  if (parent) parent.append(element);
  return element;
}
function json(value, parent) { node('pre', JSON.stringify(value, null, 2), parent); }
async function digest(bytes) {
  return Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256', bytes)), v => v.toString(16).padStart(2, '0')).join('');
}
async function load(bytes, locator) {
  const value = parse(bytes), hash = await digest(bytes);
  if (value.schema_version === 'flow.workflow-guide-catalog/v1') {loadWorkflows(value, hash, locator);}
  else if (value.schema_version?.endsWith('-catalog/v1')) { const item = catalog(value, hash, locator); catalogs.set(item.kind, item); }
  else if (value.schema_version?.endsWith('-report/v1')) { const item = report(value, hash); reports.set(item.kind, item); }
  else { const item = manifest(value, hash, locator); manifests.set(item.id, item); }
  render();
}
async function repositoryFile(path) {
  const url = new URL(path, root);
  if (url.origin !== root.origin) throw new Error('Only the selected local repository origin is supported.');
  const response = await fetch(url, {credentials: 'omit', cache: 'no-store', redirect: 'error'});
  if (!response.ok || Number(response.headers.get('content-length')) > MAX_BYTES) throw new Error(`Cannot load bounded repository file ${path}.`);
  // Bound streaming input, including responses without Content-Length.
  const reader = response.body.getReader(); const parts = []; let length = 0;
  while (true) { const {value, done} = await reader.read(); if (done) break; length += value.byteLength;
    if (length > MAX_BYTES) {await reader.cancel(); throw new Error('Repository JSON exceeds 8 MiB.');} parts.push(value); }
  const bytes = new Uint8Array(length); let offset = 0;
  for (const part of parts) { bytes.set(part, offset); offset += part.byteLength; }
  return bytes;
}
function recordFailure(error) { $('status').textContent = `Load refused: ${error.message}. Existing loaded views are retained.`; }
$('repository').addEventListener('click', async () => {
  $('status').textContent = 'Reading local source catalogs; no tests, providers or workflows are launched.';
  try {
    for (const path of ['tests/fixtures/acceptance-scenarios.v1.json', 'tests/fixtures/lifecycle-scenarios.v1.json']) await load(await repositoryFile(path), path);
    const index = parse(await repositoryFile('contracts/fixtures/scenarios/canonical-digests.v1.json'));
    for (const path of fixturePaths(index)) await load(await repositoryFile(path), path);
    $('status').textContent = 'Local catalog files loaded. No execution report was fetched. Counts describe source recipes, not passing tests.';
  } catch (error) { recordFailure(error); }
});
$('workflow-load').addEventListener('click', async () => {try {await load(await repositoryFile('docs/integrations/showcase/workflows.json'), 'docs/integrations/showcase/workflows.json');$('status').textContent='Source workflow guides loaded locally. No execution or qualification evidence was fetched.';} catch(error){recordFailure(error);}});
$('files').addEventListener('change', async event => {
  const files = [...event.target.files];
  if (files.length > 8 || files.reduce((sum, file) => sum + file.size, 0) > 32 * 1024 * 1024) return recordFailure(new Error('Select at most eight files totaling 32 MiB'));
  try {
    for (const file of files) { if (file.size > MAX_BYTES) throw new Error('Each JSON file must be at most 8 MiB');
      // File.name is only a display label; it never becomes a network locator.
      await load(new Uint8Array(await file.arrayBuffer()), `Selected file: ${file.name}`); }
    $('status').textContent = 'Selected files loaded locally. Nothing was uploaded or executed.';
  } catch (error) { recordFailure(error); }
});
for (const id of ['search', 'family']) $(id).addEventListener('input', render);
$('reset').addEventListener('click', () => { catalogs.clear(); reports.clear(); manifests.clear(); clearWorkflows(); $('files').value = ''; render(); $('status').textContent = 'Loaded data cleared from this page.'; });
function details(title, value, parent) { const detail = node('details', undefined, parent); node('summary', title, detail); json(value, detail); }
function render() {
  $('catalogs').replaceChildren(); $('cases').replaceChildren(); $('manifests').replaceChildren();
  const rows = [...catalogs.values()].flatMap(c => c.rows);
  $('count').textContent = `${rows.length} source recipes across ${catalogs.size} loaded catalogs; ${manifests.size} manifest examples. No current verified count is asserted.`;
  const families = [...new Set(rows.map(r => r.family))].sort(); const selected = $('family').value;
  $('family').replaceChildren();
  for (const value of ['', ...families]) { const option = node('option', value || 'All families', $('family')); option.value = value; }
  $('family').value = families.includes(selected) ? selected : '';
  for (const current of catalogs.values()) {
    const section = node('section', undefined, $('catalogs')); node('h2', `${current.kind}: ${current.rows.length} recipes`, section);
    node('p', `${current.locator} · fixture ${current.fixtureVersion} · tier ${current.tier}`, section);
    node('p', `Raw catalog SHA-256: ${current.rawSha256}`, section).className = 'identity';
    const observed = evidence(current, reports.get(current.kind)); node('p', `${observed.state}: ${observed.description}`, section).className = 'evidence';
    if (observed.testedSourceDigest) { node('p', `Report-recorded tested-source SHA-256: ${observed.testedSourceDigest}`, section).className = 'identity';
      node('p', `Selected report SHA-256: ${observed.reportDigest}; toolchain declaration: ${observed.toolchain}`, section).className = 'identity'; }
    details('Declared coverage gaps and budgets', {known_gaps: current.gaps, budget: current.budget}, section);
  }
  const query = $('search').value.trim().toLowerCase(); let visible = 0;
  for (const row of rows) {
    if ($('family').value && row.family !== $('family').value) continue;
    if (query && !JSON.stringify(row).toLowerCase().includes(query)) continue;
    visible++; const article = node('article', undefined, $('cases')); article.id = row.id;
    const heading = node('h3', undefined, article); const anchor = node('a', row.id, heading); anchor.href = `#${encodeURIComponent(row.id)}`;
    node('p', `${row.kind} / ${row.family} · ${row.recipe} · expected code: ${row.code}`, article);
    node('p', `Expected ${row.graph ? 'graph A → B, independent C' : 'single boundary/step'}; recovery classification: ${row.recovery}`, article);
    details('Expected typed outcome / ordered trace (test intent)', row.expected, article);
  }
  $('shown').textContent = `${visible} matching source recipes`;
  for (const item of manifests.values()) {
    const article = node('article', undefined, $('manifests')); node('h3', item.title, article); node('p', item.id, article); node('p', item.description, article);
    node('p', `Raw example SHA-256: ${item.rawSha256} · ${item.locator}`, article).className = 'identity';
    details('Declared topology, outcome and coverage (not execution)', {stages: item.stages, expected: item.expected, coverage: item.coverage}, article);
  }
}
render();
