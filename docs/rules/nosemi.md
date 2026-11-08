---
icon: lucide/code
---

# Trailing Semicolon Missing

**Default severity:** Info
**Auto-fix:** Yes
**Category:** Formatting
**Can be disabled:** Yes
**Enabled by default:** Yes

## What this rule does

Flags MATLAB statements that produce output without a trailing semicolon.
In MATLAB, such statements print their result to the console, which is
almost always unintentional in production code and can cause significant
performance degradation in loops.

## Automatic fixes

Inserts a trailing semicolon immediately after the statement, suppressing
the console output without changing program behavior.

## Examples

### Incorrect

```matlab
x = compute_value()
data = load('file.mat')
```

### Correct

```matlab
x = compute_value();
data = load('file.mat');
```

### Fixed

```diff
- x = compute_value()
+ x = compute_value();
```

## Configuration

```toml
[lint.rules.NOSEMI]
severity = "info"
ignore_functions = ["disp", "fprintf", "warning", "error"]
```

See [Configuration](../configuration.md) for the full parameter list and [rules.md](../rules.md) for the complete rule inventory.
