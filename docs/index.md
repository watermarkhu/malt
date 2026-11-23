---
layout: home

hero:
  name: malt
  text: An ultra-fast, extensible linter for MATLAB, written in Rust
  tagline: Single-pass AST traversal powered by tree-sitter. Under its default configuration it reports exactly what MATLAB R2026b's Code Analyzer reports — 2692 checks, verified fixture by fixture. Lint thousands of files in milliseconds.
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
    details: Nearly 800 fixtures with recorded R2026b expectations, live checkcode parity in CI, message text compared. What is verified and what is best-effort is documented.
    icon: ✅
    link: /verification
  - title: Auto-fix support
    details: Rules can provide automatic fixes. Run malt --fix for safe fixes; --fix --unsafe-fixes opts into rewrites that may change behaviour.
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

- **Single-pass traversal** — The AST is walked exactly once. Rules subscribe to node types and are dispatched via an O(1) registry lookup.
- **tree-sitter powered** — Leverages `tree-sitter-matlab` for fast, incremental, error-tolerant parsing.
- **Workspace architecture** — Separated into `malt_core` (engine), `malt_rules` (rule implementations), and `malt_cli` (interface).
- **Inspired by the best** — Architecture modeled after [ruff](https://github.com/astral-sh/ruff) and [rumdl](https://github.com/rvben/rumdl).

## Next Steps

- [Installation](getting-started/installation.md) — Build and install malt from source
- [Quick Start](getting-started/quickstart.md) — Get up and running in minutes
- [Rules Reference](rules.md) — Explore all available linting rules
- [Configuration](configuration.md) — Customize malt for your project with `.malt.toml`
- [How malt is verified](verification.md) — The conformance suite, and what is verified versus best-effort
- [Migration note](migration.md) — Breaking changes if you used an earlier malt
