# malt

An ultra-fast, extensible linter for MATLAB, written in Rust.

`malt` parses MATLAB with tree-sitter and runs every rule in a
single pass over the syntax tree. It aims for parity with MATLAB's Code Analyzer
and uses the same check IDs (`NOSEMI`, `AGROW`, `naming.class.casing`, ...), so
existing `%#ok<...>` suppressions keep working.

## Install

| Channel | Command |
| ------- | ------- |
| PyPI | `pip install malt` or `uv tool install malt` (or run once with `uvx malt`) |
| crates.io | `cargo install malt-cli` |
| npm (WebAssembly) | `npm install malt-wasm` |

PyPI and crates.io install the same `malt` command; the npm package is a
WebAssembly library for browsers and Node.js.

## Usage

```bash
malt src/                 # lint a directory
malt --fix src/foo.m      # apply automatic fixes
malt --preset all src/    # enable every rule
```

Configure rules in a `.malt.toml` at the project root:

```toml
[lint]
preset = "recommended"

[lint.rules]
NOSEMI = "off"
```

## Documentation

Full documentation, the rule reference and an in-browser playground are at
<https://watermarkhu.nl/malt>.

## License

MIT
