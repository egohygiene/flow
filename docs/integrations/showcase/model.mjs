// Read-only documentation projection. Never an execution/receipt validator.
export const MAX_BYTES = 8 * 1024 * 1024;
const scenarioId = value => typeof value === 'string' && value.length <= 256 && /^scenario:[a-z0-9_.-]+$/.test(value);
export function scenarioLink(kind, id) {
  if (!['acceptance', 'lifecycle'].includes(kind) || !scenarioId(id)) throw new Error('Unsupported canonical scenario selection.');
  return `#scenario=${kind}/${encodeURIComponent(id)}`;
}
export function scenarioSelection(fragment) {
  if (typeof fragment !== 'string' || fragment.length > 1024) throw new Error('Scenario link exceeds its bound.');
  if (fragment.startsWith('#scenario=')) {
    const match = /^#scenario=(acceptance|lifecycle)\/(.+)$/.exec(fragment);
    if (!match) throw new Error('Unsupported canonical scenario selection.');
    let id; try { id = decodeURIComponent(match[2]); } catch { throw new Error('Malformed scenario link encoding.'); }
    if (!scenarioId(id)) throw new Error('Unsupported canonical scenario identity.');
    return {kind: match[1], id};
  }
  // Preserve existing ID-only links without choosing the first catalog match.
  if (/^#scenario(?::|%3a)/i.test(fragment)) {
    let id; try { id = decodeURIComponent(fragment.slice(1)); } catch { throw new Error('Malformed scenario link encoding.'); }
    if (!scenarioId(id)) throw new Error('Unsupported canonical scenario identity.');
    return {kind: null, id};
  }
  return null;
}
export function resolveScenario(rows, selection) {
  if (!selection) return {state: 'none'};
  const matches = rows.filter(row => row.id === selection.id && (!selection.kind || row.kind === selection.kind));
  if (!matches.length) return {state: 'missing'};
  if (matches.length > 1) return {state: 'ambiguous'};
  return {state: 'selected', row: matches[0]};
}
const schemas = new Map([
  ['flow.acceptance-scenario-catalog/v1', 'acceptance'],
  ['flow.lifecycle-scenario-catalog/v1', 'lifecycle'],
]);
const reportSchemas = new Map([
  ['flow.acceptance-scenario-report/v1', 'acceptance'],
  ['flow.lifecycle-scenario-report/v1', 'lifecycle'],
]);
export function text(value, maximum = 16384) {
  if (typeof value !== 'string' || value.length > maximum) throw new Error('Unsupported or oversized text field.');
  return value;
}
export function parse(bytes) {
  if (!(bytes instanceof Uint8Array) || bytes.byteLength > MAX_BYTES) throw new Error('Each selected JSON file must be at most 8 MiB.');
  return JSON.parse(new TextDecoder('utf-8', {fatal: true}).decode(bytes));
}
export function catalog(value, rawSha256, locator) {
  const kind = schemas.get(value?.schema_version);
  if (!kind || !Array.isArray(value.scenarios) || !value.scenarios.length || value.scenarios.length > 1000) throw new Error('Unsupported catalog schema or scenario count.');
  if (!/^[a-f0-9]{64}$/.test(rawSha256)) throw new Error('Missing raw catalog identity.');
  const ids = new Set();
  const rows = value.scenarios.map(case_ => {
    const id = text(case_.scenario_id, 256);
    if (!/^scenario:[a-z0-9_.-]+$/.test(id) || ids.has(id)) throw new Error('Invalid or repeated canonical scenario ID.');
    ids.add(id);
    if (!case_.expected || typeof case_.expected !== 'object' || Array.isArray(case_.expected)) throw new Error('Missing expected outcome.');
    return {id, kind, family: kind === 'acceptance' ? text(case_.family, 128) : 'durable-lifecycle',
      recipe: text(case_.recipe, 256), code: text(case_.expected.code, 256),
      expected: case_.expected, graph: case_.graph === true, mode: case_.mode ?? null,
      recovery: case_.recovery ?? 'Inspect the declared outcome; no recovery is executed by this page.'};
  });
  const gaps = value.known_gaps;
  if (!Array.isArray(gaps) || gaps.length > 64) throw new Error('Missing bounded coverage gaps.');
  gaps.forEach(gap => text(gap));
  return {kind, rawSha256, locator: text(locator, 512), fixtureVersion: text(value.fixture_version, 128),
    tier: text(value.tier, 128), budget: value.budget, gaps, rows};
}
export function report(value, rawSha256) {
  const kind = reportSchemas.get(value?.schema_version);
  if (!kind || !/^[a-f0-9]{64}$/.test(rawSha256) || !/^[a-f0-9]{64}$/.test(value.catalog_digest ?? '')
      || value.source_identity?.algorithm !== 'sha256' || !/^[a-f0-9]{64}$/.test(value.source_identity?.digest ?? '') || !Array.isArray(value.receipts)
      || value.receipts.length > 1000) throw new Error('Unsupported report or missing recorded catalog/source identity.');
  text(value.toolchain, 512);
  const ids = new Set();
  for (const receipt of value.receipts) {
    const id = text(receipt.scenario_id, 256);
    if (ids.has(id)) throw new Error('Repeated report scenario ID.');
    ids.add(id);
  }
  return {kind, rawSha256, value};
}
export function evidence(current, supplied) {
  if (!supplied) return {state: 'unavailable', description: 'No observed execution report supplied. These are authored expectations.'};
  const report_ = supplied.value;
  if (supplied.kind !== current.kind || report_.catalog_digest !== current.rawSha256)
    return {state: 'catalog-mismatch', description: 'Report catalog bytes differ or belong to another family. No current verification claim.'};
  const expected = new Map(current.rows.map(row => [row.id, row.expected]));
  if (report_.status !== 'passed' || report_.scenario_count !== current.rows.length || report_.receipts.length !== current.rows.length
      || report_.receipts.some(r => !expected.has(r.scenario_id) || !sameJson(r.outcome, expected.get(r.scenario_id))))
    return {state: 'incomplete-or-conflicting', description: 'Report claims, membership or outcomes differ from the catalog. Native validation is still required.'};
  return {state: 'source-unconfirmed', description: 'Supplied report declares a pass for these catalog bytes. Present source, freshness, authenticity and native receipt validity are unconfirmed.',
    testedSourceDigest: report_.source_identity.digest, reportDigest: supplied.rawSha256, toolchain: text(report_.toolchain, 512)};
}
function sameJson(left, right, depth = 0) {
  if (depth > 64) return false;
  if (left === right) return true;
  if (left === null || right === null || typeof left !== 'object' || typeof right !== 'object') return false;
  if (Array.isArray(left) !== Array.isArray(right)) return false;
  const a = Object.keys(left).sort(), b = Object.keys(right).sort();
  return a.length <= 10000 && a.length === b.length && a.every((key, index) => key === b[index] && sameJson(left[key], right[key], depth + 1));
}
export function manifest(value, rawSha256, locator) {
  if (value?.schema_version !== 'flow.scenario-manifest/v1') throw new Error('Unsupported manifest.');
  return {id: text(value.scenario_id, 256), title: text(value.title, 512), description: text(value.description),
    expected: value.expectation, stages: value.stages, coverage: value.coverage, rawSha256, locator};
}
export function fixturePaths(value) {
  if (value?.schema_version !== 'flow.scenario-digests/v1' || !Array.isArray(value.entries) || value.entries.length > 64) throw new Error('Unsupported fixture index.');
  return value.entries.map(entry => {
    const path = text(entry.path, 256);
    if (!/^(examples|fixtures\/scenarios)\/[a-z0-9.-]+\.json$/.test(path) || path.split('/').includes('..')) throw new Error('Unsafe fixture locator.');
    return `contracts/${path}`;
  });
}
