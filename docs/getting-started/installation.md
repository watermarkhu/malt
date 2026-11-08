---
icon: lucide/download
---

# Installation

Every channel installs the same `malt` command-line tool.

## From PyPI (recommended)

The `malt` wheels bundle a prebuilt binary for Linux (glibc and musl, x86_64 and
aarch64), macOS (Intel and Apple silicon), and Windows (x64), so no Rust
toolchain is needed:

```bash
uv tool install malt     # or: pipx install malt / pip install malt
uvx malt src/   # run once without installing
```

`python -m malt` works too, which is handy inside virtual environments.

## From crates.io

The CLI is published as the `malt-cli` crate (the bare `malt` crate name belongs
to another project); the installed binary is still `malt`:

```bash
cargo install malt-cli
```

This needs a [Rust toolchain](https://rustup.rs/) and a C compiler (for the
tree-sitter parser).

## From source

```bash
git clone https://github.com/watermarkhu/malt.git
cd malt
cargo install --path crates/malt_cli
```

To build without installing, run `cargo build --release`; the binary is at
`target/release/malt`.

## In the browser or Node.js

The WebAssembly build is published to npm as
[`malt-wasm`](https://www.npmjs.com/package/malt-wasm). See
[WebAssembly](../usage/wasm.md).

## Verify Installation

```bash
malt --version
```

## Next Steps

- [Quick Start](quickstart.md) — Get up and running with malt
- [CLI Reference](../usage/cli.md) — Full command reference
- [Configuration](../configuration.md) — Customize malt for your project
