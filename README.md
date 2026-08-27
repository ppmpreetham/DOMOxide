# DOMOxide

DOMOxide is a high-performance Rust + WebAssembly HTML sanitizer that is **~70x faster than DOMPurify**.

## Installation

```bash
npm install domoxide
# or
pnpm add domoxide
# or
yarn add domoxide
```

## Benchmarks

[![Benchmark results](https://raw.githubusercontent.com/ppmpreetham/DOMOxide/main/readme/Time.png)](https://github.com/ppmpreetham/DOMOxide/blob/main/readme/Time.png)
[![Benchmark results](https://raw.githubusercontent.com/ppmpreetham/DOMOxide/main/readme/Time2.png)](https://github.com/ppmpreetham/DOMOxide/blob/main/readme/Time2.png)

> [!TIP]
> Try it yourself: `pnpm run bench`

| payload               | input size | DOMPurify        | DOMOxide       | speed up     |
| --------------------- | ---------- | ---------------- | -------------- | ------------ |
| tiny-xss              | 61 B       | ~1.0–1.5 ms/op   | ~22–35 μs/op   | **~45x**     |
| rich-text             | 167 B      | ~2.8–3.5 ms/op   | ~20–22 μs/op   | **~150x**    |
| svg-filter            | 199 B      | ~2.0–3.9 ms/op   | ~37–50 μs/op   | **~40-105x** |
| upstream-fixture-long | 1.3 KB     | ~4.9-7.2 ms/op   | ~34-85 μs/op   | **~85-145x** |
| large-repeated-html   | 12.6 KB    | ~17.6–21.6 ms/op | ~0.8–1.0 ms/op | **~21x**     |

## Usage

There are a few ways:

### Dompurify way

```js
import { createDOMPurify } from "domoxide/compat";

// no window needed lol
const DOMPurify = await createDOMPurify();

DOMPurify.sanitize("<img src=x onerror=alert(1)>"); // '<img src="x">'
DOMPurify.sanitize(dirty, { FORBID_TAGS: ["style"], USE_PROFILES: { html: true } });
```

### 2. Bundlers (Vite, Next.js, Webpack, Rollup)


```js
import { createDOMPurify } from "domoxide/compat";

const DOMPurify = await createDOMPurify();
export const sanitize = (html) => DOMPurify.sanitize(html);
```

### 3. Raw WebAssembly ESM Bindings

If you prefer using the raw WebAssembly exports directly:

```js
import init, { sanitize_wasm } from "domoxide";
await init();

const clean = sanitize_wasm(dirty, {
  ALLOWED_TAGS: ["b", "i", "em", "strong", "a"],
  FORBID_ATTR: ["style"],
  USE_PROFILES: { html: true },
});
```

### 4. Node.js (ESM)


```js
import { readFile } from "node:fs/promises";
import { createDOMPurify } from "domoxide/compat";

const wasm = await readFile(new URL("./node_modules/domoxide/domoxide_bg.wasm", import.meta.url));
const DOMPurify = await createDOMPurify({ module_or_path: wasm });

DOMPurify.sanitize("<img src=x onerror=alert(1)>"); // '<img src="x">'
```

### 5. Browser (No Bundler / ESM CDN)

```html
<script type="module">
  import { createDOMPurify } from "https://esm.sh/domoxide/compat";

  const DOMPurify = await createDOMPurify();
  document.body.innerHTML = DOMPurify.sanitize(userInput);
</script>
```

## Parity

**218 of 219** test cases from DOMPurify's test suite pass identically.

> [!NOTE]  
> The 219th is a documented IE-era `isindex` ordering quirk that no modern engine (including jsdom DOMPurify) reproduces; DOMOxide adheres strictly to HTML5 spec order.
