import { readFile } from 'node:fs/promises';
import assert from 'node:assert/strict';

import { createDOMPurify, ensureReady } from '../compat.mjs';

const wasm = await readFile(new URL('../pkg/domoxide_bg.wasm', import.meta.url));
const DOMPurify = await createDOMPurify({ module_or_path: wasm });

assert.equal(DOMPurify.sanitize('<img src=x onerror=alert(1)>'), '<img src="x">');
assert.equal(DOMPurify.sanitize('<b onclick=x>ok</b>', { FORBID_TAGS: ['b'] }), 'ok');
assert.equal(
  DOMPurify.sanitize('<a href="javascript:evil()">x</a>'),
  '<a>x</a>',
);

// repeated factory calls and ensureReady reuse the same initialized engine.
assert.equal((await createDOMPurify()).sanitize('123456'), '123456');
await ensureReady();

console.log('compat shim ok: dompurify-shaped sanitize verified');
