---
icon: lucide/settings
---

# Configuration

malt is configured via a `.malt.toml` file in your project directory. This page is the complete reference for the configuration schema.

## Config File Discovery

malt looks for `.malt.toml` in the current working directory by default. You can override this with the `--config` flag:

```bash
malt --config path/to/config.toml src/**/*.m
```

If no config file is found and `--config` is not specified, malt uses the built-in **`mathworks`** preset (see [Presets](#presets)).

## Presets

Presets are named, curated sets of default category overrides. Pick one with
`[lint] preset`, or the `--preset` CLI flag, which takes priority over the
config file.

| Preset | Disabled categories | Description |
| ------ | ------------------- | ----------- |
| `mathworks` *(default)* | `custom-checks`, `naming` | Matches MATLAB Code Analyzer's factory configuration: complexity metrics and naming conventions are opt-in. |
| `all` | — | Every rule enabled, including the [default-off engines](#default-behavior). |
| `recommended` | `performance`, `readability`, `formatting`, `suggested-improvements`, `naming`, `custom-checks` | Errors and warnings on; Info-level style/suggestion checks off. |

```toml
[lint]
preset = "recommended"
```

```bash
malt --preset all src/**/*.m
```

A preset only removes categories; the specialized engines that are
[off by default](#default-behavior) stay off under `mathworks` and
`recommended`. `all` is the exception and turns them on as well.

Precedence: `--preset` > `[lint] preset` > `mathworks`. Explicit
`[lint.categories]` / `[lint.rules]` entries always win over the preset, so a
preset-disabled category can be re-enabled:

```toml
[lint]
preset = "recommended"

[lint.categories]
readability = "info"   # re-enable the readability suggestions
```

Presets are defined in `crates/malt_core/src/preset.rs`; adding one is a single
row in the `PRESETS` table.

## Schema Overview

```toml title=".malt.toml"
[lint]
# Global lint settings
exclude = ["vendor/**", "third_party/**"]
preset = "recommended"          # Preset: all | mathworks | recommended

[lint.categories]
# Per-category severity overrides
performance = "off"            # Disable every performance rule
compatibility = "warn"         # All compatibility checks warn

[lint.rules]
# Per-rule configuration (shorthand or full table)
NOSEMI = "warn"                # Shorthand: just set severity
AGROW = "off"                  # Disable a rule
COMPAT = "error"               # Escalate to error

[lint.rules.NO4LP]             # Full table: severity + rule parameters
severity = "warn"
indent_size = 2
```

## `[lint]` Section

### `exclude`

A list of glob patterns for files and directories to skip when linting.

```toml
[lint]
exclude = [
    "vendor/**",
    "third_party/**",
    "test/fixtures/**",
]
```

**Type:** Array of strings (glob patterns)
**Default:** `[]` (no exclusions)

### `preset`

The named preset to base the configuration on. See [Presets](#presets).

```toml
[lint]
preset = "recommended"
```

**Type:** string (`"all"`, `"mathworks"`, or `"recommended"`)
**Default:** `"mathworks"`

## `[lint.categories]` Section

Set the severity for **every** rule in a category at once. Keys are category slugs; values are severity strings. This is resolved *below* per-rule config: a rule-level override always wins over its category-level setting.

```toml
[lint.categories]
performance = "off"            # Disable all performance rules
compatibility = "warn"         # All compatibility rules warn
formatting = "info"            # All formatting rules are informational
```

**Available categories:**

| Category | Config Key | Includes |
| -------- | ---------- | -------- |
| Incomplete Analysis | `incomplete-analysis` | QUIT, NOFIL, RDERR, MBIG, MDEEP … |
| Syntax Errors | `syntax-errors` | BADCT, RESWD, DOUQT, NOPAR2, SYNEND … |
| Language Specification | `language-specification` | PF\*, FV\*, MC\*, AT\* checks |
| Bugs | `bugs` | IFBDUP, CTRUE, LOGEMP, INCR, DECR … |
| Custom Checks | `custom-checks` | Cyclomatic complexity, nesting, line metrics |
| Naming | `naming` | `naming.*` checks (81) |
| Compatibility | `compatibility` | Deprecated/removed function checks |
| Forward Compatibility | `forward-compatibility` | FCLEN, FCCPV, FCDQS, FCFAV … |
| Good Practices | `good-practices` | eval, error handling, OOP practices |
| Unset Variables | `unset-variables` | NODEF, PSET, USENS … |
| Unused Constructions | `unused-constructions` | NOEFF, NUSED, EQEFF, UNRCH … |
| Suggested Improvements | `suggested-improvements` | Function replacement suggestions |
| Readability | `readability` | ISCHR, IJCL, NBRAK2, STREMP … |
| Formatting | `formatting` | NOSEMI, NOCOMMA, ALIGN, NOPRT … |
| Performance | `performance` | AGROW, PFBNS, AND2, MINV … |
| Code Generation | `code-generation` | MATLAB Coder constraints |
| Fixed-Point | `fixed-point` | FPASE |
| Deployment | `deployment` | MATLAB Compiler constraints |
| System Objects | `system-objects` | SONUMIN, SOTUNPROP\* … |
| Unsupported | `unsupported` | MCADE, AWTIUD, FEATUD … |
| Behavior Changes | `behavior-changes` | Version behavior-change checks |
| Configuration Issues | `configuration-issues` | BDCFG, CFERR, BDOPT, CFIG |

## `[lint.rules]` Section

Configure individual rules. Each key is a **rule ID**. Values can be either a **severity shorthand** (string) or a **full configuration table**.

Most rule *engines* are registered under their engine ID (e.g. `GOOD_PRACTICES_ENGINE`, `LANGUAGE_SPEC_ENGINE`, `COMPAT`), and each engine exposes its individual check IDs through its parameters. See **Per-Engine Parameters** below.

### Severity Shorthand

Set a rule's severity with a single string value:

```toml
[lint.rules]
NOSEMI = "warn"       # Override severity to warning
AGROW = "error"       # Override severity to error
COMPAT = "info"       # Override severity to info
CUSTOM_CHECKS = "off" # Disable the rule entirely
```

### Valid Severity Values

| Value | Aliases | Effect |
| ----- | ------- | ------ |
| `"error"` | `"err"` | Rule violations are errors |
| `"warn"` | `"warning"` | Rule violations are warnings |
| `"info"` | `"note"` | Rule violations are informational |
| `"off"` | `"false"`, `"disabled"` | Rule is completely disabled |

### Full Table Configuration

For rules that accept parameters, use a full TOML table:

```toml
[lint.rules.NO4LP]
severity = "warn"
indent_size = 2
```

Or equivalently as an inline table:

```toml
[lint.rules]
NO4LP = { severity = "warn", indent_size = 2 }
```

The `severity` key is always optional in a full table. If omitted, the rule uses its default severity.

All other keys in the table are rule-specific parameters (see **Per-Engine Parameters**).

## Default Behavior

When no `.malt.toml` is present:

- The **`mathworks`** preset is active, so `custom-checks` and `naming` are off
- These specialized engines are **off by default** under every preset except
  `all` (opt in per rule or per category):
  - `CODEGEN_ENGINE` (`code-generation`) — MATLAB Coder constraints
  - `DEPLOYMENT_ENGINE` (`deployment`) — MATLAB Compiler constraints
  - `SYSTEM_OBJECTS_ENGINE` (`system-objects`) — `matlab.System` validation
  - `CONFIG_ISSUES_ENGINE` (`configuration-issues`) — config-function misuse
  - `SUGGESTED_IMPROVEMENTS` (`suggested-improvements`) — replacement hints
  - `UNSUPPORTED_ENGINE` (`unsupported`) — platform-specific features
- All other rules are **enabled** at their default severity
- No files are excluded
- No rule-specific parameters are set

Each rule page shows whether the rule is enabled by default. To opt in:

```toml title=".malt.toml"
[lint.categories]
code-generation = "error"   # enable the whole category

[lint.rules]
NAMING_ENGINE = "info"      # or enable a single rule
```

## Severity Override Precedence

The effective severity for each rule is resolved as:

1. **Per-rule override** (if `[lint.rules.XXXX]` specifies `severity` or uses the shorthand)
2. **Per-category override** (if `[lint.categories.XXXX]` is set for the rule's category)
3. **Rule's built-in default** (if neither override is present)

The config override applies to all diagnostics produced by that rule — rules themselves always emit their default severity, and the engine stamps the override afterward.

## Per-Engine Parameters

Multi-check engines take their parameters through their **engine ID**. Every engine also accepts `disabled_checks` (or `skip_checks`) to turn off specific check IDs.

```toml
[lint.rules.LANGUAGE_SPEC_ENGINE]
disabled_checks = ["PFEVC", "PFANSLP"]   # Turn off specific checks
```

| Engine ID | Category | Parameters |
| --------- | -------- | ---------- |
| `INCOMPLETE_ANALYSIS` | incomplete-analysis | `max_diagnostics`, `max_parse_errors`, `max_node_count` (linter-internal limits) |
| `SYNTAX_ERRORS_ENGINE` | syntax-errors | `disabled_checks` |
| `LANGUAGE_SPEC_ENGINE` | language-specification | `disabled_checks` |
| `BUGS_ENGINE` | bugs | `debug_functions`, `higher_order_functions` (extra function names to treat as debug/higher-order) |
| `CUSTOM_CHECKS` | custom-checks | `max_function_inputs`, `max_function_outputs`, `max_function_lines`, `max_line_length`, `max_nesting_depth`, `max_cyclomatic_complexity`, `max_strict_cyclomatic_complexity`, `max_avg_cyclomatic_complexity`, `max_branches`, `max_return_points`, `max_nested_functions`, `max_anonymous_functions`, `max_local_variables`, `max_local_constants`, `max_called_functions`, `max_semicolons_per_line`, `max_tree_children`, `max_persistent_variables`, `max_conditions`, `max_input_args_used`, `max_output_args_used` |
| `NAMING_ENGINE` | naming | `max`, `min`, `pattern`, `prefix`, `phrase`, `suffix`, `style` — per-entity check config via `[lint.rules.naming.<entity>.<check>]` |
| `COMPAT` | compatibility | *(data-driven lookup; no params)* |
| `GOOD_PRACTICES_ENGINE` | good-practices | `disabled_checks`, `max_variable_name_length` |
| `UNSET_VARIABLES_ENGINE` | unset-variables | `ignore` (variable names to ignore) |
| `UNUSED_ENGINE` | unused-constructions | `disabled_checks`, `ignore_patterns` |
| `READABILITY_ENGINE` | readability | `disabled_checks` |
| `FORMATTING_ENGINE` | formatting | `indent_size` (used by NO4LP) |
| `PERFORMANCE_ENGINE` | performance | `skip_checks` |
| `CODEGEN_ENGINE` | code-generation | `skip_checks` |
| `DEPLOYMENT_ENGINE` | deployment | `skip_checks` |
| `UNSUPPORTED_ENGINE` | unsupported | `skip_checks` |
| `SYSTEM_OBJECTS_ENGINE` | system-objects | `skip_checks` |
| `CONFIG_ISSUES_ENGINE` | configuration-issues | `disabled_checks` |
| `NO4LP` | formatting | `indent_size` (indent width expected for loop bodies) |
| `SUGGESTED_IMPROVEMENTS` | suggested-improvements | *(data-driven lookup; no params)* |

## Complete Example

```toml title=".malt.toml"
[lint]
# Skip generated and vendored code
exclude = [
    "codegen/**",
    "vendor/**",
    "*.generated.m",
]

[lint.categories]
# Keep compatibility warnings visible but don't gate the build on them
compatibility = "warn"

# Performance and formatting suggestions are noise during development
performance = "info"
formatting = "info"

[lint.rules]
# Escalate statements that print their result (a missing semicolon) to errors
[lint.rules.NOPTS]
severity = "error"

# Turn off a noisy sub-check of the language-spec engine
[lint.rules.LANGUAGE_SPEC_ENGINE]
disabled_checks = ["PFANSLP"]

# Raise the cyclomatic-complexity threshold for this project
[lint.rules.CUSTOM_CHECKS]
max_cyclomatic_complexity = 12
```

## Next Steps

- [Rules Reference](rules.md) — Browse all available rules
- [CLI Reference](usage/cli.md) — Command-line options
- [Quick Start](getting-started/quickstart.md) — Get up and running
