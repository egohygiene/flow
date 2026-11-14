// Presentation over canonical declarations, never execution/receipt validation.
const families = {
  resolution: {
    title: 'Choose a compatible provider',
    why: 'A provider must match the requested capability, contract and policy before work starts.',
  },
  contract: {
    title: 'Check a versioned boundary',
    why: 'Malformed, contradictory or incompatible declarations must not cross a trusted contract boundary.',
  },
  artifact: {
    title: 'Accept only observed artifacts',
    why: 'Reported outputs need independent file, identity and confinement checks before they can be accepted.',
  },
  provider: {
    title: 'Handle a provider result',
    why: 'Process completion, partial output and provider claims must be distinguished from accepted artifacts.',
  },
  privacy: {
    title: 'Keep private process data private',
    why: 'Arguments, environment values and operational captures must not become portable result data.',
  },
};
function words(value) {
  if (typeof value !== 'string' || value.length > 256) throw new Error('Bounded scenario label required.');
  const result = value.replace(/[-_]+/g, ' ');
  return result ? result[0].toUpperCase() + result.slice(1) : 'Unnamed recipe';
}
export function explainScenario(row) {
  if (!row || !['acceptance', 'lifecycle'].includes(row.kind) || typeof row.id !== 'string' || row.id.length > 256 || !/^scenario:[a-z0-9_.-]+$/.test(row.id))
    throw new Error('Canonical scenario identity required for an explanation.');
  const title = words(row.recipe);
  if (row.kind === 'acceptance') {
    const family = families[row.family];
    const accepted = row.expected?.accepted;
    const decision = accepted === true ? 'The authored expectation accepts the observed artifacts.'
      : accepted === false ? 'The authored expectation does not accept artifacts. A correct refusal can still pass the test.'
      : 'Artifact acceptance is unspecified in this declaration; inspect the expected record.';
    return {scenarioId: row.id, title, purpose: family?.title ?? 'Inspect an unclassified scenario family',
      why: family?.why ?? 'This family has no presentation mapping yet. Read its canonical recipe and expected outcome.',
      setup: `Exercise the declared “${title}” recipe at the ${row.family} boundary.`, decision,
      expectedCode: row.code, authority: 'authored_expectation_only'};
  }
  const history = row.expected?.history;
  const final = Array.isArray(history) && history.length ? history.at(-1) : null;
  const steps = Array.isArray(final?.steps) ? final.steps : null;
  const statuses = steps ? [...new Set(steps.map(step => step.status).filter(s => typeof s === 'string'))] : [];
  const checkpoints = steps ? steps.filter(step => step.checkpoint === true).length : null;
  return {scenarioId: row.id, title, purpose: 'Preserve durable execution and recovery decisions',
    why: 'Interruption, stale inputs and changed authority must not silently reuse invalid work or replay uncertain effects.',
    setup: row.graph ? 'Use declared dependent steps A → B and an independent step C.' : 'Use the declared single-step durable lifecycle.',
    decision: steps ? `The final authored snapshot contains ${steps.length} steps, ${checkpoints} checkpoints and states: ${statuses.join(', ') || 'unspecified'}.`
      : 'No final authored snapshot is available; inspect the declared expected record.',
    expectedCode: row.code, recovery: row.recovery, authority: 'authored_expectation_only'};
}
