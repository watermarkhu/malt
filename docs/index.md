---
layout: home

hero:
  name: malt
  text: An ultra-fast, extensible linter for MATLAB, written in Rust
  tagline: Single-pass AST traversal powered by tree-sitter, with 2,680+ MATLAB Code Analyzer checks. Lint thousands of files in milliseconds.
  actions:
    - theme: brand
      text: Quick Start
      link: /getting-started/quickstart
    - theme: alt
      text: Rules Reference
      link: /rules
    - theme: alt
      text: Playground
      link: /playground

features:
  - title: Built for speed
    details: Single-pass AST traversal using tree-sitter. Lint thousands of files in milliseconds.
    icon: ⚡
  - title: Extensible rule system
    details: Clean Rule trait architecture inspired by ruff and rumdl. Add new rules with minimal boilerplate.
    icon: 🧩
  - title: Auto-fix support
    details: Rules can provide automatic fixes. Run malt --fix to apply them in one shot.
    icon: 🔧
  - title: Configurable
    details: TOML-based configuration. Enable, disable, or override severity per rule. Per-rule parameters.
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
src/compute.m:2:5 [W] M001: Statement without trailing semicolon may produce unintended console output
src/compute.m:4:5 [W] M001: Statement without trailing semicolon may produce unintended console output

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
