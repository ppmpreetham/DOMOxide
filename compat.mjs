//! drop-in dompurify-shaped wrapper around the raw wasm bindings.
//!
//! ```js
//! import { createDOMPurify } from './compat.mjs';
//! const DOMPurify = await createDOMPurify();
//! DOMPurify.sanitize(dirty, { FORBID_TAGS: ['style'] });
//! ```

import init, { sanitize_wasm } from './pkg/domoxide.js';

/** cached init promise so repeated factory calls never re-initialize. */
let ready = null;

/**
 * mirrors dompurify's `createDOMPurify(window)` minus the window argument.
 * resolves to a synchronous `DOMPurify` object whose `sanitize` matches the
 * upstream signature `(dirty, config?) -> string`.
 *
 * @param {{ module_or_path?: InitInput }} [options] forwarded to the
 * generated initializer; omit it in browsers to load the `.wasm` relative
 * to the module, pass `{ module_or_path }` in node.
 */
export function createDOMPurify(options) {
  ready ??= Promise.resolve(init(options));
  return ready.then(() => ({
    /** identical signature to `DOMPurify.sanitize`; always returns a string. */
    sanitize: (dirty, config) => sanitize_wasm(dirty, config),
  }));
}

/** resolves when the engine is initialized; safe to call any number of times. */
export function ensureReady(options) {
  ready ??= Promise.resolve(init(options));
  return ready;
}
