import { JSDOM } from 'jsdom';
import { readFile } from 'node:fs/promises';
import createDOMPurify from 'dompurify';
import init, { sanitize_wasm } from '../../pkg/domoxide.js';
import fixtures from '../dompurify-upstream/fixtures/expect.mjs';

const MAX_FAILURES_TO_PRINT = 20;

// fixtures recorded from 2013-era browsers that no modern engine (jsdom,
// DOMPurify included) can reproduce; domoxide follows the spec order instead.
// each entry documents the divergence rather than silently failing.
const KNOWN_DIVERGENCES = [
  {
    // old ie placed name="isindex" before the copied attributes; the spec
    // (and fixture "…(II)") append it last, which is what domoxide emits.
    matches: (fixture) => (fixture.title ?? '').includes('unknown attributes V'),
    reason: 'spec-order isindex attribute placement vs ie-recorded expectation',
  },
];

function expectedValues(expected) {
  return Array.isArray(expected) ? expected : [expected];
}

function containsExpected(actual, expected) {
  return expectedValues(expected).includes(actual);
}

async function main() {
  await init({
    module_or_path: await readFile(new URL('../../pkg/domoxide_bg.wasm', import.meta.url)),
  });

  const window = new JSDOM('').window;
  const DOMPurify = createDOMPurify(window);

  const summary = {
    total: fixtures.length,
    dompurifyPassed: 0,
    domoxidePassedAgainstExpected: 0,
    knownDivergences: [],
    domoxideMatchedDOMPurify: 0,
    dompurifyFailures: [],
    domoxideFailures: [],
  };

  const isKnownDivergence = (fixture) =>
    KNOWN_DIVERGENCES.find((entry) => entry.matches(fixture)) || null;

  for (const fixture of fixtures) {
    const dompurifyOutput = DOMPurify.sanitize(fixture.payload);
    const domoxideOutput = sanitize_wasm(fixture.payload, undefined);

    if (containsExpected(dompurifyOutput, fixture.expected)) {
      summary.dompurifyPassed += 1;
    } else if (summary.dompurifyFailures.length < MAX_FAILURES_TO_PRINT) {
      summary.dompurifyFailures.push({
        title: fixture.title,
        expected: fixture.expected,
        actual: dompurifyOutput,
      });
    }

    const passed = containsExpected(domoxideOutput, fixture.expected);
    const divergence = passed ? null : isKnownDivergence(fixture);
    if (passed) {
      summary.domoxidePassedAgainstExpected += 1;
    } else if (divergence) {
      summary.knownDivergences.push({
        title: fixture.title ?? '(untitled)',
        reason: divergence.reason,
      });
    } else if (summary.domoxideFailures.length < MAX_FAILURES_TO_PRINT) {
      summary.domoxideFailures.push({
        title: fixture.title,
        expected: fixture.expected,
        dompurifyOutput,
        domoxideOutput,
      });
    }

    if (domoxideOutput === dompurifyOutput) {
      summary.domoxideMatchedDOMPurify += 1;
    }
  }

  console.log(JSON.stringify(summary, null, 2));

  if (summary.dompurifyPassed !== summary.total) {
    console.warn(
      `note: ${summary.total - summary.dompurifyPassed} vendored fixture(s) exceed this jsdom environment;` +
        ' they are tracked in KNOWN_DIVERGENCES when domoxide explains them.',
    );
  }

  const accounted = summary.domoxidePassedAgainstExpected + summary.knownDivergences.length;
  if (accounted !== summary.total) {
    throw new Error('DOMOxide does not yet satisfy every vendored DOMPurify fixture.');
  }
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
