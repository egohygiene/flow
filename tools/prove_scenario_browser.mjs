// Optional local browser proof. The viewer itself has no Playwright dependency.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import {createReadStream} from 'node:fs';
import {readFile, writeFile, mkdir, realpath, stat} from 'node:fs/promises';
import path from 'node:path';
import {fileURLToPath} from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const options = new Map();
for (let i = 2; i < process.argv.length; i += 2) {
  const key = process.argv[i], value = process.argv[i + 1];
  assert.ok(['--url', '--chromium', '--output', '--playwright-module'].includes(key) && value && !options.has(key), 'Expected one value for each known option.');
  options.set(key, value);
}
for (const key of ['--url', '--chromium', '--output']) assert.ok(options.has(key), `Missing ${key}.`);
const url = new URL(options.get('--url'));
assert.ok(url.protocol === 'http:' && ['127.0.0.1', 'localhost', '[::1]'].includes(url.hostname)
  && url.pathname === '/docs/integrations/showcase/'
  && !url.username && !url.password && !url.search && !url.hash, 'Select the exact local HTTP showcase path.');
const output = path.resolve(options.get('--output'));
assert.ok(output !== root && !output.startsWith(root + path.sep), 'Browser proof output must be outside the checkout.');
assert.equal(await realpath(path.dirname(output)), path.dirname(output), 'Select a canonical existing output parent.');
await mkdir(output, {mode: 0o700}); // Existing output refuses; never replace evidence.
const require = createRequire(import.meta.url);
const moduleName = options.get('--playwright-module') ?? 'playwright';
const {chromium} = require(moduleName);
const browserPath = await realpath(options.get('--chromium'));
assert.ok((await stat(browserPath)).isFile(), 'Select a Chromium executable.');
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
async function executableIdentity() {
  const hash = createHash('sha256');
  for await (const bytes of createReadStream(browserPath)) hash.update(bytes);
  return {bytes: (await stat(browserPath)).size, sha256: hash.digest('hex')};
}
const read = relative => readFile(path.join(root, relative));
const acceptancePath = 'tests/fixtures/acceptance-scenarios.v1.json';
const lifecyclePath = 'tests/fixtures/lifecycle-scenarios.v1.json';
const acceptanceBytes = await read(acceptancePath), lifecycleBytes = await read(lifecyclePath);
const acceptance = JSON.parse(acceptanceBytes), lifecycle = JSON.parse(lifecycleBytes);
const index = JSON.parse(await read('contracts/fixtures/scenarios/canonical-digests.v1.json'));
const guides = JSON.parse(await read('docs/integrations/showcase/workflows.json'));
const expectedCount = acceptance.scenarios.length + lifecycle.scenarios.length;
const sourceFiles = [acceptancePath, lifecyclePath,
  'contracts/fixtures/scenarios/canonical-digests.v1.json',
  ...index.entries.map(entry => 'contracts/' + entry.path),
  ...['index.html', 'style.css', 'view.mjs', 'model.mjs', 'explanations.mjs',
    'workflow-view.mjs', 'workflows.mjs', 'workflows.json', 'workflows.schema.json']
    .map(name => 'docs/integrations/showcase/' + name),
  'tools/prove_scenario_browser.mjs'];
async function sourceIdentity() {
  const files = [];
  for (const relative of [...new Set(sourceFiles)].sort()) {
    const bytes = await read(relative);
    files.push({path: relative, bytes: bytes.length, sha256: sha(bytes)});
  }
  return {algorithm: 'sha256', digest: sha(Buffer.from(JSON.stringify(files))), files};
}
const report = {schema: 'flow.scenario-browser-proof/v1', state: 'failed',
  claim_scope: 'local_browser_presentation_only', product_qualified: false,
  native_provider_qualified: false, publication_performed: false,
  synthetic_report_fixtures: true, source_identity: await sourceIdentity(),
  observed_counts: {acceptance: acceptance.scenarios.length, lifecycle: lifecycle.scenarios.length,
    manifest_examples: index.entries.length, publishing_guides: guides.workflows.length},
  checks: [], screenshots: [], browser: {}, failure: null,
  remaining: ['Other browser engines and assistive technologies', 'Published Pages/deployment behavior',
    'Actual source/native report qualification, provider execution and full Flow #88 acceptance']};
const network = [], blocked = [], pageErrors = [];
const served = new Map(), bodyTasks = [], bodyFailures = [];
const canonical = new Map(report.source_identity.files.map(file => [file.path, file]));
const browserIdentity = await executableIdentity();
const browser = await chromium.launch({executablePath: browserPath, headless: true,
  args: ['--no-sandbox']});
const context = await browser.newContext({viewport: {width: 1280, height: 900}});
context.setDefaultTimeout(10000);
context.setDefaultNavigationTimeout(15000);
await context.route('**/*', async route => {
  const request = route.request(), target = new URL(request.url());
  network.push({method: request.method(), path: target.pathname, same_origin: target.origin === url.origin});
  if (target.origin !== url.origin || request.method() !== 'GET') {
    blocked.push({method: request.method(), path: target.pathname});
    await route.abort();
  } else await route.continue();
});
const page = await context.newPage();
page.on('pageerror', error => pageErrors.push(error.message));
page.on('response', response => {
  bodyTasks.push((async () => {
    const target = new URL(response.url());
    if (target.pathname === '/favicon.ico' && response.status() === 404) return;
    assert.equal(target.origin, url.origin, 'Unexpected response origin.');
    assert.equal(response.status(), 200, 'A selected source response did not succeed.');
    let relative = decodeURIComponent(target.pathname.slice(1));
    if (relative.endsWith('/')) relative += 'index.html';
    const expected = canonical.get(relative);
    assert.ok(expected, 'The browser requested a noncanonical source path.');
    const bytes = await response.body();
    assert.equal(bytes.length, expected.bytes, 'Served source length differs: ' + relative);
    assert.equal(sha(bytes), expected.sha256, 'Served source bytes differ: ' + relative);
    served.set(relative, {path: relative, bytes: bytes.length, sha256: sha(bytes)});
  })().catch(error => bodyFailures.push(String(error.message).slice(0, 500))));
});
async function check(id, operation) {
  await operation(); report.checks.push({id, state: 'passed'});
}
async function loaded() {
  await page.waitForFunction(() => document.getElementById('status').textContent.startsWith('Local catalog files loaded.'));
  assert.equal(await page.locator('#cases article').count(), expectedCount);
}
async function pick(name, value) {
  await page.locator('#files').setInputFiles({name, mimeType: 'application/json', buffer: Buffer.from(JSON.stringify(value))});
}
async function screenshot(name) {
  const bytes = await page.screenshot({path: path.join(output, name), fullPage: false});
  assert.ok(bytes.length <= 2 * 1024 * 1024, 'Viewport screenshot exceeds 2 MiB.');
  report.screenshots.push({path: name, bytes: bytes.length, sha256: sha(bytes)});
}
const first = acceptance.scenarios[0].scenario_id;
const link = id => '#scenario=acceptance/' + encodeURIComponent(id);
const synthetic = () => ({schema_version: 'flow.acceptance-scenario-report/v1',
  catalog_digest: sha(acceptanceBytes), source_identity: {algorithm: 'sha256', digest: 'b'.repeat(64)},
  status: 'passed', scenario_count: acceptance.scenarios.length,
  toolchain: 'Synthetic browser fixture only; no scenario or provider executed.',
  receipts: acceptance.scenarios.map(row => ({scenario_id: row.scenario_id, outcome: row.expected}))});
try {
  report.browser = {engine: 'Chromium', version: browser.version(), headless: true,
    executable: browserIdentity,
    playwright_version: require(path.join(path.dirname(require.resolve(moduleName)), 'package.json')).version};
  await check('initial_unavailable_and_explicit_keyboard_load', async () => {
    await page.goto(url.href);
    assert.match(await page.locator('#status').innerText(), /Execution evidence is unavailable/);
    assert.equal(await page.locator('#cases article').count(), 0);
    await page.keyboard.press('Tab');
    assert.equal(await page.locator(':focus').innerText(), 'Reading guide and source links');
    await page.keyboard.press('Tab');
    assert.equal(await page.locator(':focus').getAttribute('id'), 'repository');
    await page.keyboard.press('Enter'); await loaded();
    assert.equal(await page.locator('#manifests article').count(), index.entries.length);
    assert.equal(await page.locator('#catalogs .evidence').count(), 2);
    for (const text of await page.locator('#catalogs .evidence').allTextContents()) assert.match(text, /^unavailable:/);
  });
  await check('canonical_ids_and_success_refusal_explanations', async () => {
    const actual = await page.locator('#cases article a').allTextContents();
    assert.deepEqual(actual, [...acceptance.scenarios, ...lifecycle.scenarios].map(row => row.scenario_id));
    const refusal = acceptance.scenarios.find(row => row.expected.accepted === false);
    const article = page.locator('#cases article').filter({has: page.getByRole('link', {name: `Link to acceptance source recipe ${refusal.scenario_id}`, exact: true})});
    assert.match(await article.innerText(), /does not accept artifacts/);
    await article.locator('summary').press('Enter');
    assert.ok(await article.locator('details').getAttribute('open') !== null);
    const success = acceptance.scenarios.find(row => row.expected.accepted === true);
    const accepted = page.locator('#cases article').filter({has: page.getByRole('link', {name: `Link to acceptance source recipe ${success.scenario_id}`, exact: true})});
    assert.match(await accepted.innerText(), /authored expectation accepts the observed artifacts/);
    await accepted.locator('summary').press('Enter');
    assert.deepEqual(JSON.parse(await accepted.locator('pre').innerText()), success.expected);
  });
  await check('lifecycle_trace_has_readable_exact_expected_data', async () => {
    const selected = lifecycle.scenarios[0];
    await page.locator('#search').fill(selected.scenario_id);
    assert.equal(await page.locator('#cases article').count(), 1);
    const article = page.locator('#cases article');
    assert.match(await article.innerText(), /Preserve durable execution and recovery decisions/);
    await article.locator('summary').press('Enter');
    assert.deepEqual(JSON.parse(await article.locator('pre').innerText()), selected.expected);
  });
  await check('keyboard_search_family_and_link_navigation', async () => {
    await page.locator('#search').fill(''); await page.locator('#search').focus(); await page.keyboard.type(first);
    assert.equal(await page.locator('#cases article').count(), 1);
    await page.getByRole('link', {name: `Link to acceptance source recipe ${first}`, exact: true}).focus();
    await page.keyboard.press('Enter');
    await page.waitForFunction(() => document.activeElement?.classList.contains('linked-case'));
    await page.locator('#search').fill('no-source-recipe-can-match-this');
    await page.locator('#family').focus();
    await page.keyboard.press('Home'); await page.keyboard.press('ArrowDown'); await page.keyboard.press('Enter');
    assert.equal(await page.locator('#cases article').count(), 1);
    assert.match(await page.locator('.linked-case').innerText(), /Linked source recipe — authored expectation only/);
    await page.locator('#clear-link').focus(); await page.keyboard.press('Enter');
    assert.equal(await page.locator(':focus').getAttribute('id'), 'search');
    assert.equal(new URL(page.url()).hash, '');
    assert.equal(await page.locator('#cases article').count(), 0);
  });
  await check('deep_link_focus_survives_all_catalog_and_manifest_renders', async () => {
    await page.goto(url.href + link(first));
    await page.reload(); // A hash-only navigation otherwise retains prior filters.
    await page.locator('#repository').focus(); await page.keyboard.press('Enter'); await loaded();
    assert.equal(await page.locator(':focus').getAttribute('id'), `scenario-acceptance-${first}`);
    await page.locator('#reset').focus(); await page.keyboard.press('Enter');
    assert.equal(await page.locator('#cases article').count(), 0);
    assert.match(await page.locator('#link-status').innerText(), /not loaded/);
    await page.locator('#repository').focus(); await page.keyboard.press('Enter'); await loaded();
    assert.equal(await page.locator(':focus').getAttribute('id'), `scenario-acceptance-${first}`);
  });
  for (const [id, state, alter] of [
    ['matching_synthetic_report_stays_unconfirmed', 'source-unconfirmed', () => {}],
    ['different_catalog_report_is_mismatched', 'catalog-mismatch', value => { value.catalog_digest = 'd'.repeat(64); }],
    ['incomplete_report_stays_incomplete', 'incomplete-or-conflicting', value => { value.receipts.pop(); }],
    ['conflicting_report_stays_conflicting', 'incomplete-or-conflicting', value => { value.receipts[0].outcome = {code: 'synthetic-conflict', accepted: true}; }],
  ]) await check(id, async () => {
    const value = synthetic(); alter(value); await pick(id + '.json', value);
    await page.waitForFunction(expected => document.querySelector('#catalogs .evidence').textContent.startsWith(expected + ':'), state);
    assert.equal(await page.locator('#cases article').count(), expectedCount);
    assert.match(await page.locator('#count').innerText(), /No current verified count is asserted/);
    if (state === 'source-unconfirmed') {
      assert.ok((await page.locator('#catalogs').innerText()).includes('b'.repeat(64)));
      assert.match(await page.locator('#catalogs').innerText(), /Synthetic browser fixture only/);
      await page.locator('#catalogs').scrollIntoViewIfNeeded();
      await screenshot('desktop-unconfirmed.png');
    }
  });
  await check('invalid_selected_file_refuses_without_network_or_view_loss', async () => {
    const count = network.length;
    await pick('https-example-invalid-private.json', {schema_version: 'foreign/report/v9'});
    await page.waitForFunction(() => document.getElementById('status').textContent.startsWith('Load refused:'));
    assert.equal(await page.locator('#cases article').count(), expectedCount);
    assert.equal(network.length, count);
  });
  await check('publishing_guides_keep_separate_source_counts', async () => {
    await page.locator('#workflow-load').focus(); await page.keyboard.press('Enter');
    await page.waitForFunction(expected => document.querySelectorAll('#workflows article').length === expected, guides.workflows.length);
    assert.equal(await page.locator('#cases article').count(), expectedCount);
    assert.match(await page.locator('#workflow-count').innerText(), /not test scenarios or observed successful runs/);
    await page.locator('#workflow-search').fill(guides.workflows[0].title);
    assert.ok(await page.locator('#workflows article').count() >= 1);
    assert.ok((await page.locator('#workflows').innerText()).includes(guides.workflows[0].title));
  });
  await check('narrow_view_and_keyboard_details_keep_text_and_focus', async () => {
    await page.setViewportSize({width: 360, height: 800});
    await page.locator('#search').fill(first);
    const article = page.locator('.linked-case');
    await article.locator('summary').focus(); await page.keyboard.press('Enter');
    assert.ok(await article.locator('details').getAttribute('open') !== null);
    const layout = await page.evaluate(() => ({width: innerWidth, scroll_width: document.documentElement.scrollWidth,
      overflow: [...document.querySelectorAll('body *')].map(element => ({tag: element.tagName, id: element.id,
        right: element.getBoundingClientRect().right, width: element.getBoundingClientRect().width}))
        .filter(box => box.width > 0 && box.right > innerWidth).slice(0, 12)}));
    assert.ok(layout.scroll_width <= layout.width, JSON.stringify(layout));
    await article.scrollIntoViewIfNeeded(); await screenshot('narrow-linked-expectation.png');
  });
  await check('malformed_hash_never_fetches_or_executes', async () => {
    const count = network.length;
    await page.evaluate(() => { location.hash = '#scenario=acceptance/..%2Fprivate'; });
    await page.waitForFunction(() => document.getElementById('link-status').textContent.includes('malformed'));
    assert.equal(network.length, count);
    assert.equal(await page.locator('.linked-case').count(), 0);
    await page.locator('#clear-link').focus(); await page.keyboard.press('Enter');
    assert.equal(await page.locator(':focus').getAttribute('id'), 'search');
  });
  await check('reset_clears_catalogs_reports_manifests_and_guides', async () => {
    await page.locator('#reset').focus(); await page.keyboard.press('Enter');
    for (const selector of ['#cases article', '#catalogs section', '#manifests article', '#workflows article'])
      assert.equal(await page.locator(selector).count(), 0);
    assert.match(await page.locator('#workflow-count').innerText(), /No workflow guides loaded/);
    assert.equal(await page.locator(':focus').getAttribute('id'), 'reset');
  });
  await check('no_upload_external_requests_or_page_errors', async () => {
    assert.deepEqual(pageErrors, []); assert.deepEqual(blocked, []);
    assert.ok(network.every(request => request.same_origin && request.method === 'GET'));
    assert.deepEqual(await sourceIdentity(), report.source_identity);
    assert.deepEqual(await executableIdentity(), browserIdentity);
  });
  await check('served_viewer_catalog_manifest_and_guide_bytes_match_checkout', async () => {
    await Promise.all(bodyTasks);
    assert.deepEqual(bodyFailures, []);
    const required = [...canonical.keys()].filter(relative => ![
      'tools/prove_scenario_browser.mjs', 'docs/integrations/showcase/workflows.schema.json'].includes(relative));
    assert.deepEqual([...served.keys()].sort(), required.sort());
    report.served_sources = [...served.values()].sort((a, b) => a.path.localeCompare(b.path));
  });
  report.state = 'passed';
} catch (error) {
  report.failure = String(error.message).slice(0,2000);
  try { await screenshot('failed-view.png'); } catch {}
} finally {
  report.network = {requests: network.length, all_same_origin_get: network.every(item => item.same_origin && item.method === 'GET'),
    blocked_requests: blocked.length, page_errors: pageErrors.length};
  await browser.close();
  const bytes = Buffer.from(JSON.stringify(report, null, 2) + '\n');
  assert.ok(bytes.length <= 131072, 'Browser proof exceeds 128 KiB.');
  await writeFile(path.join(output, 'browser-proof.json'), bytes, {flag: 'wx', mode: 0o600});
}
console.log(JSON.stringify({state: report.state, checks: report.checks.length,
  report_sha256: sha(await readFile(path.join(output, 'browser-proof.json'))),
  product_qualified: false, publication_performed: false}));
if (report.state !== 'passed') process.exitCode = 1;
