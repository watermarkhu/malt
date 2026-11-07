# AGENTS.md

This document describes how to develop and document the `mlt` codebase for AI coding agents and human contributors.

## Project Overview

`mlt` (pronounced "melt") is an ultra-fast, extensible linter for MATLAB written in Rust. It uses `tree-sitter-matlab` for parsing and is architecturally inspired by [ruff](https://github.com/astral-sh/ruff) and [rumdl](https://github.com/rvben/rumdl).

The goal is full feature parity with MATLAB's Code Analyzer (~2,680 checks), using the same check IDs (e.g., `NOSEMI`, `AGROW`, `naming.class.casing`).

## Workspace Structure

```
mlt/
├── .agents/                # Agent guidance (this file), subagent prompts, skills
├── opencode.json           # Registers .agents/ with OpenCode (instructions + subagents)
├── Cargo.toml              # Workspace root (resolver = "2"); version placeholder 0.0.0
├── pyproject.toml          # PyPI package `mlt` (maturin, ships the CLI binary)
├── package.json            # Docs toolchain (bun + VitePress), npm scripts
├── crates/
│   ├── mlt_cli/            # CLI binary (clap, config discovery, output formatting)
│   ├── mlt_core/           # Core engine (Rule trait, RuleRegistry, Linter, Config, Diagnostic)
│   ├── mlt_rules/          # Individual lint rule implementations
│   └── mlt_wasm/           # WebAssembly bindings (npm package `mlt-wasm`)
├── python/                 # `python -m mlt` shim + wheel smoke tests
├── wasm-pkg/               # npm package README, package.json template, smoke test
└── docs/                   # VitePress documentation site (Markdown)
```

### Packages and releases

| Directory | crates.io / registry name | Rust library name |
|-----------|---------------------------|-------------------|
| `crates/mlt_core` | `mlt-lint-core` | `mlt_core` |
| `crates/mlt_rules` | `mlt-lint-rules` | `mlt_rules` |
| `crates/mlt_cli` | `mlt-lint` (binary `mlt`) | — |
| `crates/mlt_wasm` | npm `mlt-wasm` (not on crates.io) | `mlt_lib` |
| `pyproject.toml` | PyPI `mlt` | — |

The `mlt`, `mlt_core` and `mlt_wasm` crate names belong to MapLibre, hence the
`mlt-lint` prefix. Code keeps using `mlt_core::` / `mlt_rules::`.

Versions are never bumped by hand: `Cargo.toml` stays at `0.0.0` and the Release
workflow stamps the semVersie version (from the Conventional Commit title of the
pull request into `main`) before publishing. See `.github/CONTRIBUTING.md`.

### Crate Responsibilities

| Crate | Purpose |
|-------|---------|
| `mlt_core` | Defines `Rule` trait, `Category` enum, `NodeContext`/`FileContext`, `RuleRegistry`, `Linter` engine, `Config` parsing, `Diagnostic`/`Fix`/`Severity` types. No concrete rules live here. |
| `mlt_rules` | All concrete rule implementations. Uses `inventory` crate for auto-registration. Exports `all_rules(&Config)` and `active_rules(&Config)`. Depends on `mlt_core`. |
| `mlt_cli` | Binary entry point. Discovers `.mlt.toml`, builds registry, invokes linter, formats output. Depends on both `mlt_core` and `mlt_rules`. |

### Dependency Direction

```
mlt_cli → mlt_rules → mlt_core
```

`mlt_core` has zero internal dependencies. `mlt_rules` depends only on `mlt_core`. `mlt_cli` depends on both.

## Architecture

### Lint Engine Flow

1. CLI loads `.mlt.toml` → parses into `Config`
2. `mlt_rules::active_rules(&config)` → constructs only enabled rules (filtered by per-rule and per-category config)
3. `RuleRegistry::new(rules, &config)` → indexes rules by `target_node_types()`, resolves effective severity per rule (per-rule > per-category > rule default)
4. `Linter::new(registry)` → initializes tree-sitter parser with MATLAB grammar
5. `linter.lint(source, file_path)` → single-pass DFS traversal:
   - For each node: O(1) lookup of subscribed rules, dispatch `rule.check(&NodeContext)`
   - After traversal: call `rule.check_file(&FileContext)` only on rules where `has_file_check() == true`
   - Stamp effective severity (config override) onto all diagnostics
   - Return sorted diagnostics

### Rule Trait

```rust
pub trait Rule: Send + Sync {
    fn id(&self) -> &'static str;                          // e.g., "NOSEMI"
    fn description(&self) -> &'static str;                 // One-line summary
    fn severity(&self) -> Severity;                        // Default severity
    fn category(&self) -> Category;                        // Rule category for bulk config
    fn target_node_types(&self) -> &'static [&'static str]; // Node types to subscribe to

    fn can_be_disabled(&self) -> bool { true }             // false for critical checks
    fn has_file_check(&self) -> bool { false }             // true if check_file is implemented

    fn check(&self, ctx: &NodeContext) -> Vec<Diagnostic> { vec![] }      // Per-node
    fn check_file(&self, ctx: &FileContext) -> Vec<Diagnostic> { vec![] } // Per-file
}
```

- **Node-level rules**: Implement `check()`, return non-empty `target_node_types()`.
- **File-level rules**: Implement `check_file()`, set `has_file_check() → true`, return empty `target_node_types()`.
- **Hybrid**: Implement both (rare).

### Rule Categories

Rules are organized into categories matching MATLAB's Code Analyzer groups:

| Category Enum | Config Key | Description |
|---------------|-----------|-------------|
| `IncompleteAnalysis` | `incomplete-analysis` | Internal linter limits |
| `SyntaxErrors` | `syntax-errors` | Parser-level validation |
| `LanguageSpecification` | `language-specification` | Language constraint violations |
| `Bugs` | `bugs` | Likely bugs and logic errors |
| `CustomChecks` | `custom-checks` | Complexity/style metrics |
| `Naming` | `naming` | Naming conventions |
| `Compatibility` | `compatibility` | Deprecated/removed APIs |
| `ForwardCompatibility` | `forward-compatibility` | Forward compatibility |
| `GoodPractices` | `good-practices` | Best practices |
| `UnsetVariables` | `unset-variables` | Undefined variables |
| `UnusedConstructions` | `unused-constructions` | Dead code |
| `SuggestedImprovements` | `suggested-improvements` | Code improvement suggestions |
| `Readability` | `readability` | Readability improvements |
| `Formatting` | `formatting` | Code formatting |
| `Performance` | `performance` | Performance hints |
| `CodeGeneration` | `code-generation` | MATLAB Coder constraints |
| `FixedPoint` | `fixed-point` | Fixed-point toolbox |
| `Deployment` | `deployment` | MATLAB Compiler constraints |
| `SystemObjects` | `system-objects` | System object validation |
| `Unsupported` | `unsupported` | Unsupported features |
| `BehaviorChanges` | `behavior-changes` | Version behavior changes |
| `ConfigurationIssues` | `configuration-issues` | Config file validation |

### Auto-Registration

Rules self-register using the `inventory` crate. No manual editing of `lib.rs` registration arrays is needed. Each rule module adds at the bottom:

```rust
inventory::submit!(crate::RuleRegistration::new("RULE_ID", RuleStruct::from_config));
```

### Configuration System

Config file: `.mlt.toml` (discovered in CWD, or explicit `--config <path>`).

```toml
[lint]
exclude = ["vendor/**"]

[lint.categories]
performance = "off"          # Disable all performance rules
compatibility = "warn"       # Set all compatibility rules to warning

[lint.rules]
NOSEMI = "info"              # Shorthand: severity only
AGROW = "off"               # Disable rule

[lint.rules.NOSEMI]          # Full table: severity + rule-specific params
severity = "error"
ignore_functions = ["disp", "fprintf"]
```

Resolution order: per-rule > per-category > rule default.

Rules access typed parameters via `config.rule_params::<T>(rule_id)`, which deserializes the rule's TOML table into a `#[derive(Deserialize, Default)]` struct.

## Adding a New Rule

This is the most common task. Follow these steps exactly:

### 1. Create the rule module

Most categories are now **directories** (`crates/mlt_rules/src/<category>/`) holding
one file per check plus a `mod.rs` that owns the engine struct, config, the
`impl Rule` dispatch, shared helpers (`pub(crate)`), and `inventory::submit!`.
Before adding a check, look at the category's existing layout.

- **Adding a check to an existing category directory** (the common case): create
  `crates/mlt_rules/src/<category>/check_<name>.rs` containing
  `use super::*; impl <Category>Engine { pub(crate) fn check_<name>(...) }` plus
  that check's tests in a `#[cfg(test)] mod tests`. Add `mod check_<name>;` to the
  directory's `mod.rs`, and dispatch from the engine's `check()`/`check_file()`.
- **Adding a brand-new standalone rule** (rare; e.g. `NOSEMI`): create a flat
  `crates/mlt_rules/src/<rule_id_lowercase>.rs`:

```rust
//! # RULE_ID: Rule Description
//!
//! Explanation of what the rule checks.

use mlt_core::{Category, Config, Diagnostic, Fix, NodeContext, Rule, Severity};
use serde::Deserialize;

/// Rule-specific configuration (deserialized from .mlt.toml).
#[derive(Debug, Clone, Deserialize, Default)]
pub struct RuleIdConfig {
    // Add rule-specific parameters here with #[serde(default)]
}

pub struct RuleId {
    config: RuleIdConfig,
}

impl RuleId {
    pub fn from_config(config: &Config) -> Box<dyn Rule> {
        let rule_config: RuleIdConfig = config.rule_params("RULE_ID");
        Box::new(Self { config: rule_config })
    }
}

impl Rule for RuleId {
    fn id(&self) -> &'static str { "RULE_ID" }
    fn description(&self) -> &'static str { "..." }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> Category { Category::GoodPractices }
    fn target_node_types(&self) -> &'static [&'static str] { &["..."] }

    fn check(&self, ctx: &NodeContext) -> Vec<Diagnostic> {
        // Implementation here
        Vec::new()
    }
}

inventory::submit!(crate::RuleRegistration::new("RULE_ID", RuleId::from_config));
```

For multi-check engines, the engine registers once; individual checks are
`pub(crate)` methods emitted with their own check IDs. Keep shared helpers in the
category's `mod.rs` as `pub(crate)` so check files can call them via `use super::*`.

### 2. Register the module

In `crates/mlt_rules/src/lib.rs`, add only (for a brand-new flat module or category):

```rust
pub mod rule_id_lowercase;
```

No other changes needed — `inventory` handles the rest. For a directory category,
`pub mod <category>;` resolves to `<category>/mod.rs` automatically.

### 3. Add documentation

Create `docs/<rule_id_lowercase>.md` following the format of `docs/nosemi.md`:

- Title with rule ID and name
- Default severity, auto-fix status, category, can-be-disabled
- "What this rule does" section
- "Why this matters" section
- Examples: Correct / Incorrect / Fixed (with MATLAB code blocks)
- Configuration table with parameters
- Automatic fixes description
- Target node types
- Related rules

### 4. Update docs/rules.md

Add a row to the rule table in `docs/rules.md`.

### 5. Update the VitePress sidebar

Add the rule page to the `Rules` section of the sidebar in `docs/.vitepress/config.mts`:

```ts
{ text: 'RULE_ID - Name', link: '/rules/rule_id_lowercase' },
```

### 6. Verify

```bash
cargo build
cargo clippy
cargo test
```

## Key Patterns and Conventions

### Rule ID Convention

Rule IDs match MATLAB Code Analyzer check IDs exactly (e.g., `NOSEMI`, `AGROW`, `PFBNS`, `naming.class.casing`). This gives users familiar IDs and enables feature parity tracking.

### Data-Driven Rules

For large groups of similar checks (Compatibility: ~1800, Suggested Improvements: ~243), use data-driven lookup tables:

```rust
// Load from embedded TOML data file
const DATA: &str = include_str!("data/compatibility.toml");
```

A single rule module handles many check IDs by matching function names against a lookup table.

### Generic Rule Engines

For systematic patterns (Naming: 81 checks = 9 entities x 9 check types), implement a single generic engine parameterized by configuration.

### Tree-sitter Node Types

The MATLAB grammar (tree-sitter-matlab 1.3) defines these important node types:

- **Statements**: `assignment`, `function_call`, `command`, `for_statement`, `if_statement`, `while_statement`, `switch_statement`, `try_statement`, `return_statement`, `break_statement`, `continue_statement`
- **Expressions**: `binary_operator`, `boolean_operator`, `comparison_operator`, `unary_operator`, `postfix_operator`, `number`, `string`, `identifier`, `function_call`, `field_expression`, `cell`, `matrix`, `range`, `lambda`
- **Structural**: `source_file`, `block`, `function_definition`, `class_definition`, `properties`, `methods`, `arguments_statement`
- **OOP**: `attributes`, `attribute`, `enumeration`, `enum`, `events`, `property`, `superclasses`
- **Statement parents**: `source_file`, `block` (use these to determine if a node is at statement level)

**Important limitation**: `function_call` is used for both actual function calls AND array/cell indexing. The grammar cannot distinguish them.

### Checking Statement-Level Context

Many rules need to know if a node is a statement (top-level in a block) vs. a sub-expression. Pattern:

```rust
let is_statement_level = node
    .parent()
    .map(|p| ["source_file", "block"].contains(&p.kind()))
    .unwrap_or(false);
```

### Fix Construction

```rust
// Simple insertion
Fix::insert(byte_offset, ";")

// Replacement
Fix::new(start..end, "replacement_text")

// Multi-edit (atomic)
Fix::with_additional(primary_range, primary_replacement, vec![
    Fix::new(other_range, other_replacement),
])
```

### Rule Config Pattern

Every rule should define a `#[derive(Deserialize, Default)]` config struct, even if currently empty. This makes adding parameters non-breaking:

```rust
#[derive(Debug, Clone, Deserialize, Default)]
pub struct RuleIdConfig {
    #[serde(default)]
    pub some_param: Vec<String>,
}
```

## Commands

| Command | Purpose |
|---------|---------|
| `cargo build` | Build all crates |
| `cargo clippy` | Lint Rust code (must pass with zero warnings) |
| `cargo test` | Run all tests (unit tests in `mlt_core::config`) |
| `cargo run -- <file.m>` | Run the linter on a MATLAB file |
| `cargo run -- --fix <file.m>` | Apply auto-fixes |
| `cargo run -- --config path/.mlt.toml <file.m>` | Lint with explicit config |
| `bun install` | Install the docs toolchain (VitePress, lucide icons) |
| `bun run docs:dev` | Preview the docs locally |
| `bun run docs:build` | Regenerate generated tables/pages, then build the site |
| `bun run docs:gen` | Regenerate generated tables/pages only |
| `bun docs/scripts/gen_rules_docs.ts --check` | Fail if generated docs are stale (hk pre-commit) |

## Quality Standards

- Zero clippy warnings (CI gate)
- All public types and functions must have doc comments
- Rule modules must have module-level `//!` documentation with examples
- Every rule must have a corresponding `docs/<rule_id>.md` documentation page
- Tests should cover: rule fires correctly, rule does NOT fire on valid code, config parameters are respected

## Documentation System

Documentation uses [VitePress](https://vitepress.dev/):

- Config: `docs/.vitepress/config.mts` (site meta, nav, sidebar) + `docs/.vitepress/theme/`
- Content: `docs/` directory (Markdown)
- Dependencies: `package.json` `devDependencies` (`vitepress`, `lucide-vue-next`)
- Preview: `bun run docs:dev` (runs `docs:gen` first)
- Build: `bun run docs:build` (runs `docs:gen`, then `vitepress build docs`)
- **Rule pages are generated from source docstrings.** Every rule module in
  `crates/mlt_rules/src` carries a standardized `//!` header docstring (schema
  documented in `docs/scripts/SCHEMA.md`) with a ```` ```mlt ```` front-matter
  block and fixed `## Rule` / `## Fix` / `## Examples` sections. The rule index
  table in `docs/rules.md` and the per-rule pages in `docs/rules/*.md` are
  produced by `docs/scripts/gen_rules_docs.ts`. `docs:build` regenerates them
  first so they never drift from the sources.
- Editing rule docs: edit the rule's docstring (or `data/*.toml` for
  data-driven engines), then run `bun run docs:gen` to refresh the generated
  pages and the rules.md index. `bun docs/scripts/gen_rules_docs.ts --check`
  fails when generated docs are stale.

### Documentation Structure

```
docs/
├── index.md                    # Landing page
├── getting-started/
│   ├── installation.md         # How to install
│   └── quickstart.md           # Basic usage
├── usage/
│   ├── cli.md                  # CLI reference
│   ├── editors.md              # Editor integration
│   └── ci-cd.md                # CI/CD integration
├── configuration.md            # .mlt.toml schema reference
├── rules.md                    # Rules overview + generated rule index table
└── rules/<slug>.md             # One generated page per rule engine
```

### Rule Documentation Schema

Every rule module's `//!` docstring IS its documentation. Full spec:
`docs/scripts/SCHEMA.md`. Summary:

```
//! # <META_ID>: <Human Title>
//!
//! ```mlt
//! id = "<META_ID>"
//! title = "<Human Title>"
//! category = "<category-slug>"
//! severity = "<error|warning|info>"
//! fix = <true|false>
//! icon = "lucide/<name>"       # optional
//! slug = "<url-slug>"          # optional
//! data_file = "<name>.toml"    # optional — data-driven engine
//! ```
//!
//! ## Rule
//! <what the rule does>
//!
//! ## Check IDs                # engines only
//! | Check ID | Severity | Fix | Description |
//!
//! ## Fix                      # only when fix = true
//! <what the auto-fix rewrites>
//!
//! ## Examples
//! ### Correct / ### Incorrect / ### Fixed
```

When migrating a rule module to the schema, run `bun run docs:gen` and confirm
the generated page in `docs/rules/` renders the expected check anchors, then
`bun docs/scripts/gen_rules_docs.ts --check`.
