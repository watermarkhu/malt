---
icon: lucide/align-left
---

# Formatting Suggestion Checks

**Default severity:** Info
**Auto-fix:** Yes
**Category:** Formatting
**Can be disabled:** Yes

## What this rule does

Suggests formatting improvements for MATLAB code. All 7 checks share a
single `FormattingEngine` that runs as a file-level rule, walking the whole
tree once and applying every enabled check per node. Each diagnostic
carries the specific check ID (e.g. `NOCOMMA`, `NO4LP`). Individual checks
can be enabled, disabled, or reconfigured through their own
`[lint.rules.*]` tables.

## Check IDs

### NOCOMMA

Severity: **info** · Auto-fix: **yes**

Extra comma is unnecessary.

### NO4LP

Severity: **info** · Auto-fix: **yes**

Parentheses are not needed in a FOR statement.

### ALIGN

Severity: **info** · Auto-fix: **yes**

This keyword might not be aligned with its matching END on line VAR_NUMBER.

### NOPTS

Severity: **info** · Auto-fix: **yes**

Add a semicolon after the statement to hide the output (in a script).

### NOPRT

Severity: **info** · Auto-fix: **yes**

Add a semicolon after the statement to hide the output (in a function).

### PRTCAL

Severity: **info** · Auto-fix: **yes**

Add a semicolon after the function call to hide the output.

### NCOMMA

Severity: **info** · Auto-fix: **yes**

Best practice is to separate output variables with commas.

## Automatic fixes

Each check rewrites the flagged construct directly:

- NOCOMMA inserts a comma between space-separated row elements (`[1 2 3]` → `[1, 2, 3]`).
- NCOMMA inserts a comma between space-separated function arguments.
- NO4LP rewrites the indentation prefix of misindented statements.
- ALIGN re-indents `elseif`/`else` clauses to the column of their `if`.
- NOPTS and NOPRT strip unnecessary parentheses (`(x)` → `x`).
- PRTCAL rewrites a string-only call to command syntax (`disp('hello')` → `disp hello`).

## Examples

### Incorrect

```matlab
x = [1, 2,];          % NOCOMMA: extra comma
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
indent_size = 4

[lint.rules.PRTCAL]
severity = "off"
```

See [Configuration](../configuration.md) for the full parameter list and [rules.md](../rules.md) for the complete rule inventory.
