---
icon: lucide/alert-triangle
---

# Unset Variables

**Default severity:** Warning
**Auto-fix:** No
**Category:** Unset Variables
**Can be disabled:** Yes
**Enabled by default:** Yes
**Maturity:** Mixed (2 of 5 experimental)

## What this rule does

Detects variables that might not be defined before use. A single file-level
engine (`UnsetVariablesEngine`) builds a symbol table and runs
definite-assignment analysis to find variables used on code paths where
they may never have been assigned. It covers 6 checks from MATLAB's "Unset
Variables" category; each diagnostic carries the specific check ID (e.g.
`NODEF`, `STOUT`).

## Check IDs

### NODEF

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Variable might be used before it is defined.

### USENS

Severity: **warning** · Auto-fix: **no** · Maturity: **experimental**

Explicitly initialize this variable to avoid a potential uninitialized variable, or use a valid syntax for function call on line VAR_NUMBER.

### SUSENS

Severity: **warning** · Auto-fix: **no** · Maturity: **experimental**

Variable is used, but might be unset (within a script).

### SVNODEF

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Variable might not have a value, because variable definition may not be executed.

### STOUT

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Function return value might be unset.

## Maturity

Experimental checks are off by default and run with `[lint] experimental = true`, the `all` preset, or an explicit rule setting (see [Experimental checks](../configuration.md#experimental-checks)).

`USENS`, `SUSENS`: On the mathworks off-list: MATLAB's default analysis reported none of them in 4 or more probe shapes each, so their on-side (the --preset all behaviour) is not compared with MATLAB.

## Examples

### Incorrect

```matlab
function f()
    disp(x);    % NODEF: x may never be defined before use
end

function y = g()
    if flag
        y = 1;
    end         % STOUT: output not assigned on all paths
end
```

### Correct

```matlab
function f()
    x = 0;
    disp(x);
end

function y = g()
    y = 0;
    if flag
        y = 1;
    end
end
```

## Configuration

```toml
[lint.rules.UNSET_VARIABLES_ENGINE]
severity = "warning"
ignore = ["myGlobalWorkspaceVar"]
```

See [Configuration](../configuration.md) for the full parameter list and [rules.md](../rules.md) for the complete rule inventory.
