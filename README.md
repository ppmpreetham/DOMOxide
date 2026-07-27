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

