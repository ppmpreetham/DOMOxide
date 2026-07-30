# Tests

This folder contains DOMOxide's Rust and WebAssembly tests, plus a vendored copy
of DOMPurify's upstream `test/` directory for parity work.

## Commands

```powershell
cargo test
wasm-pack test --node --features wasm
```

## DOMPurify Upstream Tests

`tests/dompurify-upstream/` contains every file from:

`https://github.com/cure53/DOMPurify/tree/main/test`

Imported from commit:

`f8902545d9ca579a78f746e73f0b9712526c89da`

These tests are vendored as parity source material. The fixture adapter in
`parity/fixture-parity.mjs` runs every recorded expectation through both
DOMPurify (jsdom) and the DOMOxide wasm build. Fixtures that no modern engine
can reproduce (2013-era browser recordings) are listed in the runner's
`KNOWN_DIVERGENCES` with an explanation instead of failing silently.

## Parity Debugging

`parity/debug-failures.mjs [filter]` prints payload, expectation, DOMPurify
output and DOMOxide output for each failing fixture, optionally filtered by a
title/payload substring.
