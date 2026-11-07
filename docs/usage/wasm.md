---
icon: lucide/package
---

# WebAssembly & npm

mlt compiles to WebAssembly and is published to npm as **`mlt-wasm`**, so you
can lint MATLAB code in the browser, in Node.js, or embed the linter in your own
tooling. The same engine powers the [Playground](../playground.md) and the CLI.

## Installing

```bash
npm install mlt-wasm
```

## Usage

```javascript
import init, { Linter } from 'mlt-wasm';

await init();

const linter = new Linter({
  disable: ['NOSEMI'],
});

// Lint MATLAB content — returns a JSON array of warnings.
const warnings = JSON.parse(linter.check('x = 1'));

// Apply all auto-fixes.
const fixed = linter.fix('x = 1');

// Or start from a `.mlt.toml` string (full fidelity: severities,
// categories, rule params, exclude). Throws on invalid TOML.
const configured = Linter.from_toml('[lint.rules]\nNOSEMI = "off"\n');
```

See the package [`README`](https://github.com/watermarkhu/mlt/tree/main/wasm-pkg)
for the full API reference (warning format, browser/Node usage, configuration).

## Building locally

The `mlt-wasm` package is built with `wasm-pack`. Development tools are managed
by [mise](https://mise.jdx.dev/):

```bash
mise install          # install wasm-pack, node, clang
mise run setup-wasi   # download the WASI SDK (compiles the tree-sitter C parser)
mise run build-wasm   # produce crates/mlt_wasm/pkg (the npm package contents)
mise run test-wasm    # run the host-side binding tests
```

> **Why the WASI SDK?** The `tree-sitter-matlab` grammar ships a C parser that
> must be cross-compiled to `wasm32-unknown-unknown`. mlt uses the WASI SDK's
> `clang` and sysroot headers, with `__wasi__` defined to skip the host-only
> `dup` path in tree-sitter's C runtime. The resulting module is a standard
> browser wasm with no WASI imports.

The built package lands in `crates/mlt_wasm/pkg/`. Releases are published
automatically: when a pull request into `main` is merged, `release.yml` stamps the
version, builds the package, and runs `npm publish --provenance`. See
[CONTRIBUTING](https://github.com/watermarkhu/mlt/blob/main/.github/CONTRIBUTING.md).

## Local docs build

The [Playground](../playground.md) page loads `mlt-wasm` from the jsDelivr CDN
by default. To use a locally-built copy instead (e.g. for offline docs or while
developing):

```bash
mise run build-wasm
mise run serve-local-wasm   # copies crates/mlt_wasm/pkg -> docs/public/assets/mlt-wasm
```

The page tries `assets/mlt-wasm/mlt_lib.js` first and falls back to the CDN, so
the same page works with and without a local build.
