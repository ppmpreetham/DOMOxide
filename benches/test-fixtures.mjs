import { JSDOM } from 'jsdom';
import createDOMPurify from 'dompurify';
import pkg from '../pkg/domoxide.js';
import fixtures from '../tests/dompurify-fixtures.mjs';

const { sanitize_wasm } = pkg;

const window = new JSDOM('').window;
const DOMPurify = createDOMPurify(window);

function normalize(str) {
  return String(str)
    .replace(/\s+/g, ' ')
    .replace(/>\s+</g, '><')
    .trim();
}

function matchesExpected(actual, expected) {
  if (Array.isArray(expected)) {
    return expected.some(e => normalize(actual) === normalize(e));
  }

  return normalize(actual) === normalize(expected);
}

let passed = 0;
let failed = 0;

const failures = [];

for (const fixture of fixtures) {
  const actual = sanitize_wasm(fixture.payload);

  if (matchesExpected(actual, fixture.expected)) {
    passed++;
  } else {
    failed++;

    failures.push({
      title: fixture.title ?? '(untitled)',
      expected: fixture.expected,
      actual,
    });
  }
}

console.log('\n=== DOMOxide Fixture Results ===\n');

console.log(`Passed: ${passed}`);
console.log(`Failed: ${failed}`);
console.log(
  `Success Rate: ${((passed / (passed + failed)) * 100).toFixed(2)}%`
);

console.log('\n=== First 20 Failures ===\n');

for (const failure of failures.slice(0, 20)) {
  console.log(`Fixture: ${failure.title}`);

  console.log(
    'Expected:',
    Array.isArray(failure.expected)
      ? failure.expected[0]
      : failure.expected
  );

  console.log('Actual  :', failure.actual);

  console.log('---');
}