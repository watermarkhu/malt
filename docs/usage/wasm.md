---
icon: lucide/package
---

# WebAssembly & npm

malt compiles to WebAssembly and is published to npm as **`malt-wasm`**, so you
can lint MATLAB code in the browser, in Node.js, or embed the linter in your own
tooling. The same engine powers the [Playground](../playground.md) and the CLI.

## Installing

```bash
npm install malt-wasm
```

## Usage

```javascript
import init, { Linter } from 'malt-wasm';

await init();

const linter = new Linter({
  disable: ['NOSEMI'],
});

// Lint MATLAB content — returns a JSON array of warnings.
const warnings = JSON.parse(linter.check('x = 1'));

// Apply all auto-fixes.
const fixed = linter.fix('x = 1');

// Or start from a `.malt.toml` string (full fidelity: severities,
// categories, rule params, exclude). Throws on invalid TOML.
const configured = Linter.from_toml('[lint.rules]\nNOSEMI = "off"\n');
```

See the package [`README`](https://github.com/watermarkhu/malt/tree/main/wasm-pkg)
for the full API reference (warning format, browser/Node usage, configuration).

## Building locally

The `malt-wasm` package is built with `wasm-pack`. Development tools are managed
by [mise](https://mise.jdx.dev/):

```bash
mise install          # install wasm-pack, node, clang
mise run setup-wasi   # download the WASI SDK (compiles the tree-sitter C parser)
mise run build-wasm   # produce crates/malt_wasm/pkg (the npm package contents)
mise run test-wasm    # run the host-side binding tests
```

> **Why the WASI SDK?** The `tree-sitter-matlab` grammar ships a C parser that
> must be cross-compiled to `wasm32-unknown-unknown`. malt uses the WASI SDK's
> `clang` and sysroot headers, with `__wasi__` defined to skip the host-only
> `dup` path in tree-sitter's C runtime. The resulting module is a standard
> browser wasm with no WASI imports.

The built package lands in `crates/malt_wasm/pkg/`. Releases are published
automatically: when a pull request into `main` is merged, `release.yml` stamps the
version, builds the package, and publishes it with `npm publish` through
[npm trusted publishing](https://docs.npmjs.com/trusted-publishers) (OIDC, no
long-lived token; provenance is attached automatically). See
[CONTRIBUTING](https://github.com/watermarkhu/malt/blob/main/.github/CONTRIBUTING.md).

## Local docs build

The [Playground](../playground.md) runs the WebAssembly build that ships inside the
site: the Docs workflow builds it from the same commit as the documentation, so the
playground always matches the rules documented next to it. To preview the docs
locally, build it once:

```bash
mise run build-wasm
mise run serve-local-wasm   # copies crates/malt_wasm/pkg -> docs/public/assets/malt-wasm
bun run docs:dev
```
