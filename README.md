# mlt

An ultra-fast, extensible linter for MATLAB, written in Rust.

`mlt` (pronounced "melt") parses MATLAB with tree-sitter and runs every rule in a
single pass over the syntax tree. It aims for parity with MATLAB's Code Analyzer
and uses the same check IDs (`NOSEMI`, `AGROW`, `naming.class.casing`, ...), so
existing `%#ok<...>` suppressions keep working.

## Install

| Channel | Command |
| ------- | ------- |
| PyPI | `pip install mlt` or `uv tool install mlt` (or run once with `uvx mlt`) |
| crates.io | `cargo install mlt-lint` |
| npm (WebAssembly) | `npm install mlt-wasm` |

PyPI and crates.io install the same `mlt` command; the npm package is a
WebAssembly library for browsers and Node.js.

## Usage

```bash
mlt src/                 # lint a directory
mlt --fix src/foo.m      # apply automatic fixes
mlt --preset all src/    # enable every rule
```

Configure rules in a `.mlt.toml` at the project root:

```toml
[lint]
preset = "recommended"

[lint.rules]
NOSEMI = "off"
```

## Documentation

Full documentation, the rule reference and an in-browser playground are at
<https://watermarkhu.nl/mlt>.

## License

MIT
