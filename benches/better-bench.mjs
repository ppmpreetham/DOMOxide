import { performance } from 'node:perf_hooks';
import { JSDOM } from 'jsdom';
import createDOMPurify from 'dompurify';
import pkg from '../pkg/domoxide.js';

const { sanitize_wasm } = pkg;
import fixtures from '../tests/dompurify-upstream/fixtures/expect.mjs';

const window = new JSDOM('').window;
const DOMPurify = createDOMPurify(window);

const cases = [
  {
    name: 'tiny-xss',
    iterations: 1000,
    payload: '<img src=x onerror=alert(1)><a href="javascript:evil()">x</a>',
  },
  {
    name: 'rich-text',
    iterations: 1000,
    payload:
      '<article><h1>Hello</h1><p onclick="x()">A <strong>small</strong> document with <a href="https://example.test">links</a>.</p><ul><li>one</li><li>two</li></ul></article>',
  },
  {
    name: 'svg-filter',
    iterations: 500,
    payload:
      '<svg><defs><filter id="f1"><feGaussianBlur in="SourceGraphic" stdDeviation="15" /></filter></defs><rect width="90" height="90" stroke="green" stroke-width="3" fill="yellow" filter="url(#f1)" /></svg>',
  },
  {
    name: 'fixture',
    iterations: 200,
    payload: fixtures[0].payload,
  },
  {
    name: 'large-html',
    iterations: 20,
    payload: Array.from({ length: 75 }, (_, i) =>
      `<section data-i="${i}">
         <h2>Title ${i}</h2>
         <p onclick="evil()">
           Text
           <a href="javascript:evil()">bad</a>
           <img src="https://example.test/${i}.png" onerror="evil()">
         </p>
       </section>`
    ).join(''),
  },
];

function bench(fn, iterations) {
  for (let i = 0; i < 100; i++) fn();

  const start = performance.now();

  for (let i = 0; i < iterations; i++) {
    fn();
  }

  const elapsed = performance.now() - start;

  return {
    ms: elapsed,
    opsPerSecond: iterations * 1000 / elapsed,
    usPerOp: elapsed * 1000 / iterations,
  };
}

function fmt(n) {
  return n.toLocaleString('en-US', {
    maximumFractionDigits: 2,
  });
}

console.log('\n=== Correctness Check ===\n');

let exactMatches = 0;
let mismatches = [];

for (const fixture of fixtures.slice(0, 100)) {
  const dompurifyOut = DOMPurify.sanitize(fixture.payload);
  const domoxideOut = sanitize_wasm(fixture.payload);

  if (dompurifyOut === domoxideOut) {
    exactMatches++;
  } else if (mismatches.length < 10) {
    mismatches.push({
      title: fixture.title,
      expected: dompurifyOut,
      actual: domoxideOut,
    });
  }
}

console.log(`First 100 fixtures exact matches: ${exactMatches}/100`);

if (mismatches.length) {
  console.log('\n=== Sample Mismatches ===\n');

  for (const m of mismatches) {
    console.log('Fixture:', m.title);
    console.log('DOMPurify:', m.expected);
    console.log('DOMOxide :', m.actual);
    console.log('---');
  }
}

console.log('\n=== Performance ===');

for (const c of cases) {
  const dompurify = bench(
    () => DOMPurify.sanitize(c.payload),
    c.iterations
  );

  const domoxide = bench(
    () => sanitize_wasm(c.payload),
    c.iterations
  );

  console.log(
    `\n${c.name} (${c.payload.length.toLocaleString()} bytes)`
  );

  console.log(
    `DOMPurify ${fmt(dompurify.opsPerSecond).padStart(12)} ops/s  ${fmt(dompurify.usPerOp).padStart(10)} us/op`
  );

  console.log(
    `DOMOxide  ${fmt(domoxide.opsPerSecond).padStart(12)} ops/s  ${fmt(domoxide.usPerOp).padStart(10)} us/op`
  );

  console.log(
    `Speedup: ${(domoxide.opsPerSecond / dompurify.opsPerSecond).toFixed(2)}x`
  );
}