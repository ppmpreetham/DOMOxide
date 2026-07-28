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

