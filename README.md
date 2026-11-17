# malt

An ultra-fast, extensible linter for MATLAB, written in Rust.

`malt` parses MATLAB with tree-sitter and runs every rule in a
single pass over the syntax tree. Under its default configuration it reports
**exactly what MATLAB R2026b's Code Analyzer reports** — verified fixture by
fixture against a live `checkcode`, with the message text compared ([how malt
is verified](https://watermarkhu.nl/malt/verification)) — and it uses the same
check IDs (`NOSEMI`, `AGROW`, `naming.class.casing`, ...), so existing
`%#ok<...>` suppressions keep working.

Upgrading from an earlier malt? Read the
[migration note](https://watermarkhu.nl/malt/migration): the default
configuration, `--fix` and a few rule IDs changed.

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
<https://watermarkhu.nl/malt>. Two pages worth opening first:

- [How malt is verified](https://watermarkhu.nl/malt/verification) — the
  conformance suite, and which engines are verified versus best-effort
- [Migration note](https://watermarkhu.nl/malt/migration) — breaking changes
  from earlier malt releases

## License

MIT
