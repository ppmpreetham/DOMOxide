# DOMOxide

DOMOxide is a Rust HTML sanitizer that reproduces DOMPurify's output byte for
byte while keeping the engine native to Rust and WebAssembly.

The engine parses untrusted markup with html5ever into a full document, walks
`body` children using DOMPurify's exact decision rules (allow-lists, URI
policy, namespace validation, mXSS probes) and serializes the survivors,
no browser DOM required.

## Quick Start

```rust
use domoxide::{sanitize, Config};

let clean = sanitize(r#"<img src=x onerror=alert(1)><a href="javascript:evil()">x</a>"#);
assert_eq!(clean, r#"<img src="x"><a>x</a>"#);

let clean = Config::html_profile()
    .add_tags(["custom-card"])
    .add_attr(["data-id"])
    .clean(r#"<custom-card data-id="1" onclick="x()">ok</custom-card>"#);

assert_eq!(clean, r#"<custom-card data-id="1">ok</custom-card>"#);
```

## Benchmarks

DOMOxide (wasm, release build) vs DOMPurify 3.x on jsdom, Node 22 —
`npm run bench`, five payloads from tiny to 12 KB:

| payload               | input size | DOMPurify        | DOMOxide       | speed-up     |
| --------------------- | ---------- | ---------------- | -------------- | ------------ |
| tiny-xss              | 61 B       | ~1.0–1.5 ms/op   | ~22–35 us/op   | **~45x**     |
| rich-text             | 167 B      | ~2.8–3.5 ms/op   | ~20–22 us/op   | **~150x**    |
| svg-filter            | 199 B      | ~2.0–3.9 ms/op   | ~37–50 us/op   | **~40–105x** |
| upstream-fixture-long | 1.3 KB     | ~4.9–7.2 ms/op   | ~34–85 us/op   | **~85–145x** |
| large-repeated-html   | 12.6 KB    | ~17.6–21.6 ms/op | ~0.8–1.0 ms/op | **~21x**     |

Numbers are indicative wall-clock medians from a single desktop machine and
vary between runs; reproduce with `npm run bench`. The gap comes from doing
the whole sanitize pass in one native/wasm call instead of per-node DOM calls
through jsdom.

## JavaScript / Wasm Usage

Build the pkg once — it is plain ESM (`--target web`), no `require()`, no
CommonJS shim, TypeScript types included:

```powershell
wasm-pack build --target web --features wasm
```

That produces a publishable `pkg/` folder:

```
pkg/
├── domoxide.js        esm entry (tree-shakeable)
├── domoxide.d.ts      types for sanitize_wasm + init
├── domoxide_bg.wasm   the engine
└── package.json       name "domoxide", type "module"
```

### Option 1 — drop-in dompurify style (`compat.mjs`)

`compat.mjs` ships the familiar `createDOMPurify()` factory — same call shape
as dompurify, minus the `window` argument. After setup, every call is
synchronous and signature-identical to `DOMPurify.sanitize(dirty, config?)`:

```js
import { createDOMPurify } from "./compat.mjs"; // 'domoxide/compat' once published

// no window needed; pass { module_or_path } only when node needs a hand
const DOMPurify = await createDOMPurify();

DOMPurify.sanitize("<img src=x onerror=alert(1)>"); // '<img src="x">'
DOMPurify.sanitize(dirty, { FORBID_TAGS: ["style"], USE_PROFILES: { html: true } });
```

The factory caches initialization, so calling it repeatedly (or mixing with
`ensureReady()`) never re-instantiates the engine.

### Option 2 — raw esm bindings

```js
import init, { sanitize_wasm } from "./pkg/domoxide.js";

// initialize once (fetches domoxide_bg.wasm next to the module)
await init();

// sanitize is synchronous after init; config uses dompurify option names
const clean = sanitize_wasm(dirty, {
  ALLOWED_TAGS: ["b", "i", "em", "strong", "a"],
  FORBID_ATTR: ["style"],
  USE_PROFILES: { html: true },
});
```

- `init()` returns a promise; call it before first use. Pass nothing to load
  the `.wasm` relative to the js file, or point it somewhere explicit:
  `await init({ module_or_path: '/static/domoxide_bg.wasm' })` — any
  `URL`, `Response`, `BufferSource` or precompiled `WebAssembly.Module` works.
- `sanitize_wasm(dirty, config?)` accepts every DOMPurify-style option:
  `ALLOWED_TAGS`, `ADD_TAGS`, `FORBID_TAGS`, `ALLOWED_ATTR`, `ADD_ATTR`,
  `FORBID_ATTR`, `USE_PROFILES { html, svg, svgFilters, mathMl }`,
  `CUSTOM_ELEMENT_HANDLING`. Omitting the config sanitizes with defaults.
- Invalid config values throw: `sanitize_wasm` returns a plain string on
  success and raises a js error otherwise (see `domoxide.d.ts`).

### Browser (no bundler)

```html
<script type="module">
  import { createDOMPurify } from "./compat.mjs";
  const DOMPurify = await createDOMPurify();
  document.body.innerHTML = DOMPurify.sanitize(userInput);
</script>
```

### Bundler (vite / webpack / rollup)

```js
import init, { sanitize_wasm } from "domoxide"; // point package.json at ./pkg
await init(); // the bundler resolves the .wasm import automatically
export const clean = (dirty) => sanitize_wasm(dirty);
```

### Node (esm)

```js
// node >= 18, "type": "module"
import { readFile } from "node:fs/promises";
import init, { sanitize_wasm } from "./pkg/domoxide.js";

await init({
  module_or_path: await readFile(new URL("./pkg/domoxide_bg.wasm", import.meta.url)),
});

sanitize_wasm("<img src=x onerror=alert(1)>"); // '<img src="x">'
```

This is exactly how the parity runner drives the wasm build against jsdom
(`tests/parity/fixture-parity.mjs`).

## Layout

| module          | role                                                    |
| --------------- | ------------------------------------------------------- |
| `config.rs`     | dompurify-shaped configuration surface                  |
| `policy.rs`     | per-call resolved allow/deny sets (built once)          |
| `lists.rs`      | static tables synced with upstream `tags.ts`/`attrs.ts` |
| `sanitize.rs`   | parse pipeline + tree walker                            |
| `attributes.rs` | `_isValidAttribute` port                                |
| `uri.rs`        | hand-ported URI policy (no regex engine)                |
| `escape.rs`     | browser-faithful serialization escaping                 |
| `preprocess.rs` | `<isindex>` expansion + `xmlns` bookkeeping             |
| `dom.rs`        | markup5ever rc-dom re-export                            |
| `hooks.rs`      | hook engine types                                       |
| `wasm.rs`       | wasm-bindgen exports                                    |

## Parity

Status and gaps live in [docs/parity.md](docs/parity.md): 218 of 219 vendored
upstream fixtures pass exactly; the single divergence is documented in the
fixture runner.
