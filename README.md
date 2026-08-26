# DOMOxide

DOMOxide is a HTML sanitizer that's 70x faster than DOMPurify.

## Benchmarks

<!--image-->

[![Benchmark results](readme/time.png)
](https://github.com/ppmpreetham/DOMOxide/blob/main/readme/Time.png)
[![Benchmark results](readme/time2.png)
](https://github.com/ppmpreetham/DOMOxide/blob/main/readme/Time2.png)

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

### Raw ESM bindings

If you prefer raw esm bindings, you can use `sanitize_wasm` directly.

```js
import init, { sanitize_wasm } from "domoxide";
await init();

const clean = sanitize_wasm(dirty, {
  ALLOWED_TAGS: ["b", "i", "em", "strong", "a"],
  FORBID_ATTR: ["style"],
  USE_PROFILES: { html: true },
});
```

### Browser (no bundler)

```html
<script type="module">
  import { createDOMPurify } from "domoxide/compat.mjs";
  const DOMPurify = await createDOMPurify();
  document.body.innerHTML = DOMPurify.sanitize(userInput);
</script>
```

### Bundler (vite / webpack / rollup)

```js
import init, { sanitize_wasm } from "domoxide";
await init();
export const clean = (dirty) => sanitize_wasm(dirty);
```

### Node (esm)

```js
// node >= 18, "type": "module"
import { readFile } from "node:fs/promises";
import init, { sanitize_wasm } from "domoxide.js";

await init({
  module_or_path: await readFile(new URL("domoxide_bg.wasm", import.meta.url)),
});

sanitize_wasm("<img src=x onerror=alert(1)>"); // '<img src="x">'
```

## Parity

218 of 219 test cases from DOMPurify

> [!NOTE]  
> (the 219th is a documented IE era `isindex` ordering that no modern engine, including jsdom DOMPurify can pass)
