import { JSDOM } from 'jsdom';
import { readFile } from 'node:fs/promises';
import createDOMPurify from 'dompurify';
import init, { sanitize_wasm } from '../../pkg/domoxide.js';
import fixtures from '../dompurify-upstream/fixtures/expect.mjs';

const FILTER = process.argv[2] ? process.argv[2].toLowerCase() : null;

await init({ module_or_path: await readFile(new URL('../../pkg/domoxide_bg.wasm', import.meta.url)) });
const DOMPurify = createDOMPurify(new JSDOM('').window);

const expectedValues = (e) => (Array.isArray(e) ? e : [e]);

for (const fixture of fixtures) {
  if (FILTER && !fixture.title.toLowerCase().includes(FILTER) && !fixture.payload.toLowerCase().includes(FILTER)) continue;
  const expected = expectedValues(fixture.expected);
  const dompurifyOutput = DOMPurify.sanitize(fixture.payload);
  const domoxideOutput = sanitize_wasm(fixture.payload, undefined);
  const okExpected = expected.includes(domoxideOutput);
  const okPurify = domoxideOutput === dompurifyOutput;
  if (okExpected || okPurify) continue;
  console.log('TITLE:', fixture.title);
  console.log('PAYLOAD:', JSON.stringify(fixture.payload));
  console.log('EXPECTED:', expected.map((e) => JSON.stringify(e)).join('\n          '));
  console.log('DOMOXIDE:', JSON.stringify(domoxideOutput));
  console.log('PURIFY  :', JSON.stringify(dompurifyOutput));
  console.log('---');
}
