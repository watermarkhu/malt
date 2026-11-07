---
icon: lucide/trash-2
---

# Unused Constructions

**Default severity:** Warning
**Auto-fix:** No
**Category:** Unused Constructions
**Can be disabled:** Yes
**Enabled by default:** Yes

## What this rule does

Detects unused variables, functions, methods, and dead code in MATLAB
files. A single file-level engine (`UnusedEngine`) builds a symbol table
and runs unreachability analysis, then cross-references every definition
against its usages to identify dead code. It covers 17 checks from
MATLAB's "Unused Constructions" category; each diagnostic carries the
specific check ID (e.g. `NASGU`, `UNRCH`, `MSNU`).

## Check IDs

### NASGU

Severity: **warning** · Auto-fix: **no**

Value assigned to variable might be unused.

### NUSED

Severity: **warning** · Auto-fix: **no**

Global or persistent variable might be unused or unset in this function or script.

### NOEFF

Severity: **warning** · Auto-fix: **no**

The operation or expression VAR_OPERATOR has no evident effect.

### EQEFF

Severity: **warning** · Auto-fix: **no**

To assign values to variables, use =. The == operator compares equality of values.

### ASGLU

Severity: **warning** · Auto-fix: **no**

Value assigned to variable might be unused. Consider replacing the variable with ~ instead.

### SETNU

Severity: **warning** · Auto-fix: **no**

Variable is set, but might be unused.

### PUSE

Severity: **warning** · Auto-fix: **no**

Persistent variable might be unused.

### PREALL

Severity: **warning** · Auto-fix: **no**

The preallocated value assigned to variable might be unused.

### INUSA

Severity: **warning** · Auto-fix: **no**

Input argument might be unused after the function arguments block(s).

### INUSD

Severity: **warning** · Auto-fix: **no**

Input argument might be unused. Consider replacing the argument with ~ instead.

### VANUS

Severity: **info** · Auto-fix: **no**

Input argument 'varargin' might be unused.

### DEFNU

Severity: **warning** · Auto-fix: **no**

Function might be unused.

### UNRCH

Severity: **warning** · Auto-fix: **no**

This statement (and possibly following ones) cannot be reached.

### MANU

Severity: **warning** · Auto-fix: **no**

Input argument might be unused. Consider replacing the argument with ~, or make this method Static instead.

### VUNUS

Severity: **warning** · Auto-fix: **no**

VAR_OPERATOR produces a value that might be unused.

### MSNU

Severity: **warning** · Auto-fix: **no**

A Code Analyzer message was once suppressed here, but the message is no longer generated.

### MSNE

Severity: **info** · Auto-fix: **no**

No Code Analyzer check is found for this check ID.

## Examples

### Incorrect

```matlab
function f()
    temp = compute();   % NASGU: assigned but never used
    x = 1;
    return;
    y = x;              % UNRCH: unreachable after return
    s.field = 1;        % MSNU: set but never read
end
```

### Correct

```matlab
function f()
    x = compute();
    disp(x);
    s.field = x;
    disp(s.field);
end
```

## Configuration

```toml
[lint.rules.UNUSED_ENGINE]
severity = "warning"
ignore_patterns = ["_*", "unused*"]
disabled_checks = ["VANUS"]
```

See [Configuration](../configuration.md) for the full parameter list and [rules.md](../rules.md) for the complete rule inventory.
