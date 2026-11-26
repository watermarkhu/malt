---
layout: home

hero:
  name: malt
  text: An ultra-fast, extensible linter for MATLAB, written in Rust
  image:
    src: /logo.png
    alt: malt
  tagline: Single-pass AST traversal powered by tree-sitter. Under its default configuration it reports what MATLAB R2026b's Code Analyzer reports. It has 2692 checks, each verified against fixtures, and lints thousands of files in milliseconds.
  actions:
    - theme: brand
      text: Quick Start
      link: /getting-started/quickstart
    - theme: alt
      text: Rules Reference
      link: /rules
    - theme: alt
      text: How it's verified
      link: /verification
    - theme: alt
      text: Playground
      link: /playground

features:
  - title: Built for speed
    details: Single-pass AST traversal using tree-sitter. Lint thousands of files in milliseconds.
    icon: ⚡
  - title: Verified against MATLAB
    details: Nearly 800 fixtures with recorded R2026b expectations, live checkcode parity in CI and compared message text. The docs say which parts are verified and which are best-effort.
    icon: ✅
    link: /verification
  - title: Auto-fix support
    details: Run malt --fix to apply safe fixes. Add --unsafe-fixes for rewrites that may change behaviour.
    icon: 🔧
  - title: Configurable
    details: TOML-based configuration, per-rule parameters, presets, and MATLAB's own codeAnalyzerConfiguration.json.
    icon: ⚙️
---

## Quick Start

```bash
# Install (prebuilt wheels on PyPI)
uv tool install malt

# Lint MATLAB files
malt path/to/file.m

# Auto-fix issues
malt --fix path/to/file.m
```

## Example Output

```text
src/compute.m:2:5 [I] NOPTS: Add a semicolon after the statement to hide the output (in a script).
src/compute.m:4:5 [I] NOPTS: Add a semicolon after the statement to hide the output (in a script).

Found 2 issues in 1 file.
```

## Design Principles

malt walks the syntax tree once. Rules subscribe to node types, and the registry dispatches each node to its subscribers with an O(1) lookup. Parsing uses `tree-sitter-matlab`, which is incremental and tolerates syntax errors.

The code is split into three crates: `malt_core` (the engine), `malt_rules` (the rules) and `malt_cli` (the command line). The architecture follows [ruff](https://github.com/astral-sh/ruff) and [rumdl](https://github.com/rvben/rumdl).

## Next Steps

- [Installation](getting-started/installation.md): build and install malt from source
- [Quick Start](getting-started/quickstart.md): lint your first file
- [Rules Reference](rules.md): every rule
- [Configuration](configuration.md): the `.malt.toml` options
- [How malt is verified](verification.md): the conformance suite, and what is verified or best-effort
- [Migration note](migration.md): breaking changes if you used an earlier malt
