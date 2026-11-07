---
icon: lucide/download
---

# Installation

Every channel installs the same `mlt` command-line tool.

## From PyPI (recommended)

The `mlt` wheels bundle a prebuilt binary for Linux (glibc and musl, x86_64 and
aarch64), macOS (Intel and Apple silicon), and Windows (x64), so no Rust
toolchain is needed:

```bash
uv tool install mlt     # or: pipx install mlt / pip install mlt
uvx mlt src/            # run once without installing
```

`python -m mlt` works too, which is handy inside virtual environments.

## From crates.io

The crates are published under the `mlt-lint` name (the `mlt` crate name
belongs to another project); the installed binary is still `mlt`:

```bash
cargo install mlt-lint
```

This needs a [Rust toolchain](https://rustup.rs/) and a C compiler (for the
tree-sitter parser).

## From source

```bash
git clone https://github.com/watermarkhu/mlt.git
cd mlt
cargo install --path crates/mlt_cli
```

To build without installing, run `cargo build --release`; the binary is at
`target/release/mlt`.

## In the browser or Node.js

The WebAssembly build is published to npm as
[`mlt-wasm`](https://www.npmjs.com/package/mlt-wasm). See
[WebAssembly](../usage/wasm.md).

## Verify Installation

```bash
mlt --version
```

## Next Steps

- [Quick Start](quickstart.md) — Get up and running with mlt
- [CLI Reference](../usage/cli.md) — Full command reference
- [Configuration](../configuration.md) — Customize mlt for your project
