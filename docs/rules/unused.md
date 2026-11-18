---
icon: lucide/trash-2
---

# Unused Constructions

**Default severity:** Warning
**Auto-fix:** No
**Category:** Unused Constructions
**Can be disabled:** Yes
**Enabled by default:** Yes
**Maturity:** Mixed (2 of 15 experimental)

## What this rule does

Detects unused variables, functions, methods, and dead code in MATLAB
files. A single file-level engine (`UnusedEngine`) builds a symbol table
and runs unreachability analysis, then cross-references every definition
against its usages to identify dead code. It covers 17 checks from
MATLAB's "Unused Constructions" category; each diagnostic carries the
specific check ID (e.g. `NASGU`, `UNRCH`).

## Check IDs

### NASGU

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Value assigned to variable might be unused.

### NUSED

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Global or persistent variable might be unused or unset in this function or script.

### NOEFF

Severity: **warning** · Auto-fix: **no** · Maturity: **experimental**

The operation or expression VAR_OPERATOR has no evident effect. Not emitted: MATLAB reported none in its probes.

### EQEFF

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

To assign values to variables, use =. The == operator compares equality of values.

### ASGLU

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Value assigned to variable might be unused. Consider replacing the variable with ~ instead.

### SETNU

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Variable is set, but might be unused.

### PUSE

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Persistent variable might be unused.

### PREALL

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

The preallocated value assigned to variable might be unused.

### INUSA

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Input argument might be unused after the function arguments block(s).

### INUSD

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Input argument might be unused. Consider replacing the argument with ~ instead.

### VANUS

Severity: **warning** · Auto-fix: **no** · Maturity: **experimental**

Input argument 'varargin' might be unused. Reported only when enabled (off under `mathworks`): MATLAB reports it with every check on.

### VUNUS

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

&lt;operator&gt; produces a value that might be unused.

### DEFNU

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Function might be unused.

### UNRCH

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

This statement (and possibly following ones) cannot be reached.

### MANU

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Input argument might be unused. Consider replacing the argument with ~, or make this method Static instead.

## Maturity

Experimental checks are off by default and run with `[lint] experimental = true`, the `all` preset, or an explicit rule setting (see [Experimental checks](../configuration.md#experimental-checks)).

`NOEFF`, `VANUS`: On the mathworks off-list: MATLAB's default analysis reported none of them in 4 or more probe shapes each, so their on-side (the --preset all behaviour) is not compared with MATLAB.

## Examples

### Incorrect

```matlab
function f()
    temp = compute();   % NASGU: assigned but never used
    x = 1;
    return;
    y = x;              % UNRCH: unreachable after return
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
