import { performance } from 'node:perf_hooks';
import { readFile } from 'node:fs/promises';
import { JSDOM } from 'jsdom';
import createDOMPurify from 'dompurify';
import init, { sanitize_wasm } from '../pkg/domoxide.js';
import fixtures from '../tests/dompurify-upstream/fixtures/expect.mjs';

const window = new JSDOM('').window;
const DOMPurify = createDOMPurify(window);

const cases = [
  {
    name: 'tiny-xss',
    iterations: 1_000,
    payload: '<img src=x onerror=alert(1)><a href="javascript:evil()">x</a>',
  },
  {
    name: 'rich-text',
    iterations: 300,
    payload:
      '<article><h1>Hello</h1><p onclick="x()">A <strong>small</strong> document with <a href="https://example.test">links</a>.</p><ul><li>one</li><li>two</li></ul></article>',
  },
  {
    name: 'svg-filter',
    iterations: 200,
    payload:
      '<svg><defs><filter id="f1"><feGaussianBlur in="SourceGraphic" stdDeviation="15" /></filter></defs><rect width="90" height="90" stroke="green" stroke-width="3" fill="yellow" filter="url(#f1)" /></svg>',
  },
  {
    name: 'upstream-fixture-long',
    iterations: 50,
    payload: fixtures[0].payload,
  },
  {
    name: 'large-repeated-html',
    iterations: 10,
    payload: Array.from({ length: 75 }, (_, index) => {
      return `<section data-i="${index}"><h2>Title ${index}</h2><p onclick="evil()">Text <a href="javascript:evil()">bad</a><img src="https://example.test/${index}.png" onerror="evil()"></p></section>`;
    }).join(''),
  },
];

const MAX_CASE_MS = 7_500;
const WARMUP_ITERATIONS = 5;

function runOne(label, iterations, fn) {
  let bytes = 0;

  for (let index = 0; index < WARMUP_ITERATIONS; index += 1) {
    bytes += fn().length;
  }

  const start = performance.now();
  let completed = 0;
  for (; completed < iterations; completed += 1) {
    bytes += fn().length;
    if (performance.now() - start > MAX_CASE_MS) {
      completed += 1;
      break;
    }
  }
  const elapsedMs = performance.now() - start;

  return {
    label,
    iterations: completed,
    requestedIterations: iterations,
    elapsedMs,
    opsPerSecond: (completed / elapsedMs) * 1000,
    avgMicroseconds: (elapsedMs * 1000) / completed,
    bytes,
    capped: completed < iterations,
  };
}

function formatNumber(value) {
  return new Intl.NumberFormat('en-US', {
    maximumFractionDigits: 2,
  }).format(value);
}

async function main() {
  await init({
    module_or_path: await readFile(new URL('../pkg/domoxide_bg.wasm', import.meta.url)),
  });

  const results = cases.map((benchCase) => {
    const dompurify = runOne('DOMPurify', benchCase.iterations, () =>
      DOMPurify.sanitize(benchCase.payload)
    );
    const domoxide = runOne('DOMOxide', benchCase.iterations, () =>
      sanitize_wasm(benchCase.payload, undefined)
    );

    return {
      name: benchCase.name,
      inputBytes: benchCase.payload.length,
      dompurify,
      domoxide,
      domoxideVsDOMPurify: domoxide.opsPerSecond / dompurify.opsPerSecond,
    };
  });

  for (const result of results) {
    console.log(`\n${result.name} (${formatNumber(result.inputBytes)} input bytes)`);
    for (const entry of [result.dompurify, result.domoxide]) {
      const capped = entry.capped ? ` capped at ${entry.iterations}/${entry.requestedIterations}` : '';
      console.log(
        `${entry.label.padEnd(9)} ${formatNumber(entry.opsPerSecond).padStart(12)} ops/s  ${formatNumber(entry.avgMicroseconds).padStart(10)} us/op${capped}`
      );
    }
    console.log(`DOMOxide speed ratio: ${formatNumber(result.domoxideVsDOMPurify)}x`);
  }
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
