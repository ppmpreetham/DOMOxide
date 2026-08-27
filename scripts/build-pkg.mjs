import { execSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const rootDir = resolve(__dirname, "..");
const pkgDir = resolve(rootDir, "pkg");

console.log("[build-pkg] Running wasm-pack build...");
execSync("wasm-pack build --target web --features wasm", {
  cwd: rootDir,
  stdio: "inherit",
});

// Read version and metadata from Cargo.toml
const cargoToml = readFileSync(resolve(rootDir, "Cargo.toml"), "utf-8");
const versionMatch = cargoToml.match(/version\s*=\s*"([^"]+)"/);
const version = versionMatch ? versionMatch[1] : "0.1.0";

// 1. Prepare pkg/compat.js
console.log("[build-pkg] Creating pkg/compat.js...");
let compatContent = readFileSync(resolve(rootDir, "compat.mjs"), "utf-8");
// In the distributed package, domoxide.js is in the same directory
compatContent = compatContent.replace(
  /from\s+['"]\.\/pkg\/domoxide\.js['"]/g,
  "from './domoxide.js'",
);
writeFileSync(resolve(pkgDir, "compat.js"), compatContent, "utf-8");

// 2. Copy compat.d.ts
console.log("[build-pkg] Copying pkg/compat.d.ts...");
const compatDts = readFileSync(resolve(rootDir, "compat.d.ts"), "utf-8");
writeFileSync(resolve(pkgDir, "compat.d.ts"), compatDts, "utf-8");

// 3. Copy README.md and LICENSE
console.log("[build-pkg] Copying README.md and LICENSE to pkg/...");
const readmeContent = readFileSync(resolve(rootDir, "README.md"), "utf-8");
writeFileSync(resolve(pkgDir, "README.md"), readmeContent, "utf-8");

const licenseContent = readFileSync(resolve(rootDir, "LICENSE"), "utf-8");
writeFileSync(resolve(pkgDir, "LICENSE"), licenseContent, "utf-8");

// 4. Generate pkg/package.json
console.log("[build-pkg] Generating pkg/package.json...");
const pkgJson = {
  name: "domoxide",
  version: version,
  description:
    "DOMOxide is a high-performance Rust + WebAssembly HTML sanitizer that is ~70x faster than DOMPurify",
  license: "Apache-2.0",
  type: "module",
  main: "./domoxide.js",
  module: "./domoxide.js",
  types: "./domoxide.d.ts",
  exports: {
    ".": {
      types: "./domoxide.d.ts",
      import: "./domoxide.js",
    },
    "./compat": {
      types: "./compat.d.ts",
      import: "./compat.js",
    },
    "./domoxide_bg.wasm": "./domoxide_bg.wasm",
  },
  files: [
    "domoxide_bg.wasm",
    "domoxide_bg.wasm.d.ts",
    "domoxide.js",
    "domoxide.d.ts",
    "compat.js",
    "compat.d.ts",
    "README.md",
    "LICENSE",
  ],
  repository: {
    type: "git",
    url: "git+https://github.com/ppmpreetham/DOMOxide.git",
  },
  homepage: "https://github.com/ppmpreetham/DOMOxide#readme",
  bugs: {
    url: "https://github.com/ppmpreetham/DOMOxide/issues",
  },
  keywords: [
    "html",
    "sanitizer",
    "sanitize",
    "xss",
    "dompurify",
    "wasm",
    "webassembly",
    "security",
  ],
  sideEffects: false,
};

writeFileSync(resolve(pkgDir, "package.json"), JSON.stringify(pkgJson, null, 2) + "\n", "utf-8");

console.log("[build-pkg] Package ready in pkg/ directory!");
