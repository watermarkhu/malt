---
icon: lucide/align-left
---

# Formatting Suggestion Checks

**Default severity:** Info
**Auto-fix:** Yes
**Category:** Formatting
**Can be disabled:** Yes
**Enabled by default:** Yes
**Maturity:** Stable

## What this rule does

Suggests formatting improvements for MATLAB code. All 6 checks share a
single `FormattingEngine` that runs as a file-level rule, walking the whole
tree once and applying every enabled check per node. Each diagnostic
carries the specific check ID (e.g. `NOCOMMA`, `NO4LP`). Individual checks
can be enabled, disabled, or reconfigured through their own
`[lint.rules.*]` tables.

## Check IDs

### NOCOMMA

Severity: **info** · Auto-fix: **yes** · Maturity: **stable**

Extra comma is unnecessary.

### NO4LP

Severity: **info** · Auto-fix: **yes** · Maturity: **stable**

Parentheses are not needed in a FOR statement.

### ALIGN

Severity: **info** · Auto-fix: **yes** · Maturity: **stable**

This keyword might not be aligned with its matching END on line VAR_NUMBER.

### NOPTS

Severity: **info** · Auto-fix: **yes** · Maturity: **stable**

Add a semicolon after the statement to hide the output (in a script).

### NOPRT

Severity: **info** · Auto-fix: **yes** · Maturity: **stable**

Add a semicolon after the statement to hide the output (in a function).

### NCOMMA

Severity: **info** · Auto-fix: **yes** · Maturity: **stable**

Best practice is to separate output variables with commas.

## Automatic fixes

Each check rewrites the flagged construct directly:

- NOCOMMA removes a comma that repeats a separator (`x = 1;,` → `x = 1;`).
- NCOMMA inserts a comma between space-separated output variables (`[a b]` → `[a, b]`).
- NO4LP removes the parentheses around the `for` header (`for (i = 1:n)` → `for i = 1:n`).
- ALIGN re-indents the `end` to the column of its keyword (only when the keyword starts its line).
- NOPTS and NOPRT add the missing semicolon as MATLAB's own fix does: a
  comma separating the statement from the next is replaced
  (`x = 1, y = 2` → `x = 1; y = 2;`), otherwise the semicolon is inserted
  after the statement (`x = 5` → `x = 5;`). Inserting in front of a comma
  would produce `x = 1;, y = 2`, which the Code Analyzer reports as
  NOCOMMA (probed).

## Examples

### Incorrect

```matlab
x = 1;,                % NOCOMMA: extra comma
[a b] = f();          % NCOMMA: separate outputs with commas
function g()
    y = compute()     % NOPRT: semicolon to hide output (in a function)
end
```

### Correct

```matlab
x = [1, 2];
[a, b] = f();
function g()
    y = compute();
end
```

### Fixed

```diff
- x = [1, 2,];
+ x = [1, 2];
- [a b] = f();
+ [a, b] = f();
- y = compute()
+ y = compute();
```

## Configuration

Each check is configured independently through its own rule table:

```toml
[lint.rules.NOCOMMA]
severity = "info"

[lint.rules.NO4LP]
severity = "info"
```

See [Configuration](../configuration.md) for the full parameter list and [rules.md](../rules.md) for the complete rule inventory.
