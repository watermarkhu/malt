---
icon: lucide/play
---

# Quick Start

## Basic usage

### Lint MATLAB files

```bash
# Lint a single file
malt src/compute.m

# Lint multiple files
malt src/compute.m src/utils.m lib/helpers.m

# Lint a folder, and everything below it
malt src/

# Lint the current folder
malt
```

Which files a folder run picks up, and which it skips, is set in [`.malt.toml`](../configuration.md#files-and-folders).

### Auto-fix issues

```bash
# Apply automatic fixes
malt --fix src/compute.m
```

## Create a configuration file

Put a `.malt.toml` file in your project root to change malt's behavior:

```toml title=".malt.toml"
[lint]
exclude = ["vendor/**", "test/fixtures/**"]

[lint.rules]
NOPTS = "warn"
```

malt looks for `.malt.toml` in the current working directory. To use another file, pass it explicitly:

```bash
malt --config path/to/.malt.toml src/compute.m
```

## Example configuration

```toml title=".malt.toml"
[lint]
# Exclude directories from linting
exclude = ["vendor/**", "third_party/**"]

[lint.rules]
# Set severity per rule
NOPTS = "error"

[lint.rules.GOOD_PRACTICES_ENGINE]
# Rule-specific parameters
severity = "warn"
max_variable_name_length = 32
```

## Reading the output

Each diagnostic is one grep-friendly line:

```text
src/compute.m:2:5 [I] NOPTS: Add a semicolon after the statement to hide the output (in a script).
src/compute.m:4:5 [I] NOPTS: Add a semicolon after the statement to hide the output (in a script).

Found 2 issues in 1 file.
```

Each line shows:

- the file path and the line:column position
- the severity in brackets (`[E]` error, `[W]` warning, `[I]` info)
- the rule ID, such as NOPTS
- a description of the issue

## Exit codes

| Code | Meaning                     |
| ---- | --------------------------- |
| `0`  | No issues found             |
| `1`  | Diagnostics found           |
| `2`  | Configuration or runtime error |

## Next steps

- [CLI Reference](../usage/cli.md)
- [Configuration](../configuration.md)
- [Rules Reference](../rules.md)
