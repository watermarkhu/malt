---
icon: lucide/play
---

# Quick Start

Get up and running with malt in minutes.

## Basic Usage

### Lint MATLAB files

```bash
# Lint a single file
malt src/compute.m

# Lint multiple files
malt src/compute.m src/utils.m lib/helpers.m
```

### Auto-fix issues

```bash
# Apply automatic fixes
malt --fix src/compute.m
```

## Create a Configuration File

Create a `.malt.toml` file in your project root to customize behavior:

```toml title=".malt.toml"
[lint]
exclude = ["vendor/**", "test/fixtures/**"]

[lint.rules]
M001 = "warn"
```

malt automatically discovers `.malt.toml` in the current working directory. You can also specify a config file explicitly:

```bash
malt --config path/to/.malt.toml src/compute.m
```

## Example Configuration

```toml title=".malt.toml"
[lint]
# Exclude directories from linting
exclude = ["vendor/**", "third_party/**"]

[lint.rules]
# Set severity per rule
M001 = "error"

[lint.rules.M001]
# Rule-specific parameters
severity = "warn"
ignore_functions = ["disp", "fprintf", "warning", "error"]
```

## Understanding Output

malt outputs diagnostics in a clear, grep-friendly format:

```text
src/compute.m:2:5 [W] M001: Statement without trailing semicolon may produce unintended console output
src/compute.m:4:5 [W] M001: Statement without trailing semicolon may produce unintended console output

Found 2 issues in 1 file.
```

Each line shows:

- **File path** and **line:column** position
- **Severity** in brackets (`[E]` error, `[W]` warning, `[I]` info)
- **Rule ID** (e.g., M001)
- **Description** of the issue

## Exit Codes

| Code | Meaning                     |
| ---- | --------------------------- |
| `0`  | No issues found             |
| `1`  | Diagnostics found           |
| `2`  | Configuration or runtime error |

## Next Steps

- [CLI Reference](../usage/cli.md) — Full command-line options
- [Configuration](../configuration.md) — Detailed config file reference
- [Rules Reference](../rules.md) — Explore all available rules
