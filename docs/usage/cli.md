---
icon: lucide/terminal
---

# CLI Reference

## Synopsis

```text
malt [OPTIONS] <FILES>...
```

## Arguments

| Argument   | Description              |
| ---------- | ------------------------ |
| `<FILES>` | One or more MATLAB files to lint (required) |

## Options

| Option                | Description                                      |
| --------------------- | ------------------------------------------------ |
| `--fix`               | Apply automatic fixes where available            |
| `--config <PATH>`     | Path to config file (default: `.malt.toml` in CWD) |
| `--preset <PRESET>`   | Preset to use (`all`, `mathworks`, `recommended`); overrides `[lint] preset` |
| `-h`, `--help`        | Print help information                           |
| `-V`, `--version`     | Print version information                        |

## Commands

### Lint files

Lint one or more MATLAB files and print diagnostics to stdout:

```bash
malt src/compute.m src/utils.m
```

Output format:

```text
<file>:<line>:<column> [<severity>] <rule_id>: <message>
```

Example:

```text
src/compute.m:2:5 [W] M001: Statement without trailing semicolon may produce unintended console output
```

### Apply fixes

Run with `--fix` to automatically apply fixes:

```bash
malt --fix src/compute.m
```

Output:

```text
Fixed 2 issues in src/compute.m
```

Only diagnostics with auto-fix support are applied. Issues without fixes are left unchanged and not reported in `--fix` mode.

### Explicit configuration

By default, malt looks for `.malt.toml` in the current working directory. Override with `--config`:

```bash
malt --config configs/strict.toml src/**/*.m
```

If `--config` is specified and the file does not exist, malt exits with an error. If no `--config` is given and `.malt.toml` is not found, malt uses the `mathworks` preset (see [Configuration](../configuration.md#presets)).

## Exit Codes

| Code | Meaning                              |
| ---- | ------------------------------------ |
| `0`  | No diagnostics found (or `--fix` succeeded) |
| `1`  | One or more diagnostics found        |
| `2`  | Runtime error (file not found, parse failure, bad config) |

## Shell Glob Expansion

malt does not perform its own glob expansion. Use your shell's globbing:

```bash
# Bash/Zsh — lint all .m files recursively
malt **/*.m

# Or use find
find src -name '*.m' -exec malt {} +
```

## Examples

```bash
# Lint a single file
malt main.m

# Lint all MATLAB files in a directory (using shell glob)
malt src/**/*.m

# Fix issues with a custom config
malt --fix --config .malt-strict.toml lib/**/*.m

# Check version
malt --version
```
