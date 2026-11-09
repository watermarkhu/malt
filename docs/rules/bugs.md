---
icon: lucide/bug
---

# Bugs

**Default severity:** Error
**Auto-fix:** Yes
**Category:** Bugs
**Can be disabled:** Yes
**Enabled by default:** Yes

## What this rule does

Detects likely bugs and logic errors in MATLAB code. All 34 checks share a
single hybrid engine (`BugsEngine`) that performs both node-level checking
(dispatched per-node during traversal) and file-level checking (full-tree
analysis after traversal). Each diagnostic carries the specific check ID
(e.g. `IFBDUP`, `FNAN`).

## Check IDs

### IFBDUP

Severity: **error** · Auto-fix: **no**

This condition has no effect because all blocks in this if statement are identical. This indicates a bug in the code. Remove the condition or change the code blocks.

### IFCDUP

Severity: **error** · Auto-fix: **no**

The statements under this elseif condition cannot be reached because it is a duplicate of the if/elseif condition on line N. This indicates a bug in the code. Remove or change the condition.

### CTRUE

Severity: **error** · Auto-fix: **no**

This logical comparison always returns true. Did you mean to use F to evaluate function argument: F(...A<=1...)? A comparison of a logical-valued function such as `any(A)` against a numeric literal that every logical value satisfies.

### CFALSE

Severity: **error** · Auto-fix: **no**

This logical comparison always returns false. Did you mean to use F to evaluate function argument: F(...A>1...)? A comparison of a logical-valued function such as `any(A)` against a numeric literal that no logical value satisfies.

### SHOCIRT

Severity: **error** · Auto-fix: **no**

The && operator is unexpected because isscalar(A && B) always returns true. (`isscalar`, `islogical`, `isreal` applied to `&&` or `

### SHOCIRF

Severity: **error** · Auto-fix: **no**

The

### DEBUGFUN

Severity: **error** · Auto-fix: **no**

Debug functions (`dbcont`, `dbquit`, `dbup`, `dbdown`, `dbstep`) are intended to be used at the command line. At runtime, they will generate an error. Remove the debug function.

### INCR

Severity: **error** · Auto-fix: **yes**

`++x` does not increment the value of x. To increase the value by 1, use x = x + 1.

### DECR

Severity: **error** · Auto-fix: **yes**

`--x` does not decrement the value of x. To decrease the value by 1, use x = x - 1.

### CMDAND

Severity: **error** · Auto-fix: **yes**

Command-syntax statement `a &b`: use 'A && B' or 'A & B' to test whether A and B are both true in MATLAB.

### CMDOR

Severity: **error** · Auto-fix: **yes**

Command-syntax statement `a

### RHSFN

Severity: **error** · Auto-fix: **no**

The expression cannot be assigned to multiple values.

### FNAN

Severity: **error** · Auto-fix: **yes**

Use ISNAN when comparing values to NaN (`x == NaN`, `x ~= NaN`).

### LOGEMP

Severity: **error** · Auto-fix: **no**

Using 'isempty' on a logical expression creates incorrect results. To determine if all the conditions are false, use '~any(..., "all")' instead.

### STCUL

Severity: **error** · Auto-fix: **no**

The comparison will likely fail due to case mismatch.

### LBODUP

Severity: **error** · Auto-fix: **no**

Since both operands are identical, the second operand has no effect on the && operation (`A && A`, `A

### FUNFUN

Severity: **error** · Auto-fix: **no**

The first input argument must be a function handle. Did you mean '@name'? (A function name passed as a string to `arrayfun`, `structfun`, `bsxfun`, …; legacy `cellfun('isempty', …)` names are allowed.)

### DEFSIZE

Severity: **error** · Auto-fix: **no**

Do not overload 'size' for fundamental data types.

### VARARG

Severity: **error** · Auto-fix: **no**

Initialize VARARGOUT with a CELL.

### STRCMPCSTR

Severity: **error** · Auto-fix: **no**

'strcmp' always returns false for string elements of a cell array. Use ["str1", "str2"] instead of {"str1", "str2"}.

### ASSRT

Severity: **error** · Auto-fix: **no**

The first input argument to 'assert' must be a condition. To always throw an error, use 'error(msg)' instead.

### BDSCA2

Severity: **error** · Auto-fix: **no**

Operands to '\|\|' and '&&' must be scalar values (matrix literal, range or sized `zeros`/`ones`/… operand). Use 'all' or 'any' to convert this value into a scalar value or use the element-wise operators '\|' or '&' instead.

### NOPRC

Severity: **error** · Auto-fix: **no**

A line break terminates the statement so it may be incomplete. Use ellipsis (...) to continue the statement. Or add a semicolon to hide the output.

### MOCUP

Severity: **error** · Auto-fix: **no**

Variable X may be cleared before the cleanup function that references X executes, resulting in an undefined variable error.

### MDUPC

Severity: **error** · Auto-fix: **no**

The case value X is a duplicate of one on line N.

### MNANC

Severity: **error** · Auto-fix: **no**

NaN never compares equal to any value, so this case will never be matched (`case NaN`).

### MULCC

Severity: **error** · Auto-fix: **no**

This case cannot be matched due to a call to UPPER or LOWER on the SWITCH value.

### MEXCEP

Severity: **error** · Auto-fix: **no**

To report an MException as a warning (`warning(E.message)`), use a format specifier to ensure the message is printed correctly. For example, 'warning(E.identifier, "%s", E.message)'.

### PFUIXE

Severity: **error** · Auto-fix: **no**

The index variable X might be used after the PARFOR loop on line N, but it is unavailable after the loop.

### PFBFN

Severity: **error** · Auto-fix: **no**

Use of this function is invalid inside a PARFOR loop because it accesses or modifies the workspace in a non-transparent way.

### PFWHOS

Severity: **error** · Auto-fix: **no**

Using "who" or "whos" without "-file" is invalid inside a PARFOR loop because it accesses the workspace in a non-transparent way.

### PFTUSE

Severity: **error** · Auto-fix: **no**

The temporary variable X is used after the PARFOR loop on line N, but its value is not available after the loop.

### PFRNC

Severity: **error** · Auto-fix: **no**

Parfor reduction variable X must be used in the same position in each assignment statement when using non-commutative reduction operations '*', '[,]', or '[;]'.

## Automatic fixes

Rewrites the flagged construct into the safe equivalent. For example:

- `&` → `&&` and `|` → `||` in boolean contexts.
- `x == NaN` → `isnan(x)` (and `x ~= NaN` → `~isnan(x)`).
- `isempty(cond)` on a logical expression → `~any(cond, "all")`.

## Examples

### Incorrect

```matlab
if x == NaN
    disp('not a number');
end
while true
    % infinite
end
```

### Correct

```matlab
if isnan(x)
    disp('not a number');
end
while true % intentional
    % ...
end
```

## Configuration

```toml
[lint.rules.BUGS_ENGINE]
severity = "error"
debug_functions = ["dbcont", "dbquit", "dbup", "dbdown", "dbstep"]
higher_order_functions = ["cellfun", "arrayfun", "structfun", "bsxfun", "spfun"]
```

See [Configuration](../configuration.md) for the full parameter list and [rules.md](../rules.md) for the complete rule inventory.
