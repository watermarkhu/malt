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
| `--fix`               | Apply safe automatic fixes, repeating until nothing changes |
| `--unsafe-fixes`      | With `--fix`: also apply fixes that may change behaviour |
| `--config <PATH>`     | Path to config file (default: `.malt.toml` in CWD) |
| `--preset <PRESET>`   | Preset to use (`all`, `mathworks`, `recommended`); overrides `[lint] preset` |
| `-v`, `--verbose`     | Report on stderr which files are linted and the total time; `-vv` adds the time and the number of findings of each file |
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
src/compute.m:2:5 [I] NOPTS: Add a semicolon after the statement to hide the output (in a script).
```

### Apply fixes

Run with `--fix` to automatically apply fixes:

```bash
malt --fix src/compute.m
```

Output:

```text
Fixed 2 issues in src/compute.m
src/compute.m:7:1 [W] NASGU: Value assigned to variable might be unused.

Found 1 issue in 1 file.
```

`--fix` lints, applies the fixes, and lints the result again until nothing more
changes, because a fix can enable another one. A file is only rewritten when
something was fixed. What cannot be fixed automatically is reported afterwards.

Fixes come in two kinds:

- **Safe** fixes keep the meaning of the code, and `--fix` applies them.
- **Unsafe** fixes are right in the common case but can change behaviour in
  another (`str2num` → `str2double` returns `NaN` where `str2num` evaluated an
  expression; `a & b` → `a && b` errors on array operands). They are reported
  as fixable only with `--fix --unsafe-fixes`. Review the result before you commit it.

### Explicit configuration

By default, malt looks for `.malt.toml` in the current working directory. Override with `--config`:

```bash
malt --config configs/strict.toml src/**/*.m
```

If `--config` is specified and the file does not exist, malt exits with an error. If no `--config` is given and `.malt.toml` is not found, malt uses the `mathworks` preset (see [Configuration](../configuration.md#presets)).

### Verbose output

`-v` prints one line per file on stderr, in command-line order, and a closing line with the number of files and the total time. `-vv` adds the time spent on each file (reading, linting and, with `--fix`, fixing) and its number of findings. Stdout, with the diagnostics and the summary, does not change, so `-v` is safe in scripts.

```bash
malt -vv src/*.m
# malt: linted src/compute.m in 4.33 ms (3 findings)
# malt: skipped src/missing.m (NOFIL)
# malt: 2 files in 4.56 ms
```

## Exit Codes

| Code | Meaning                              |
| ---- | ------------------------------------ |
| `0`  | No diagnostics found (with `--fix`: none remain after fixing) |
| `1`  | One or more diagnostics found (with `--fix`: some could not be fixed) |
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
