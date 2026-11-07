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

Detects likely bugs and logic errors in MATLAB code. All 35 checks share a
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

The statements under this VAR_RESERVED_WORD condition cannot be reached because it is a duplicate of the VAR_RESERVED_WORD condition on line VAR_NUMBER. This indicates a bug in the code. Remove or change the condition.

### CTRUE

Severity: **error** · Auto-fix: **no**

This logical comparison always returns true. Did you mean to use VAR_NAME to evaluate function argument: VAR_NAME(...VAR_NAME...)?

### CFALSE

Severity: **error** · Auto-fix: **no**

This logical comparison always returns false. Did you mean to use VAR_NAME to evaluate function argument: VAR_NAME(...VAR_NAME...)?

### SHOCIRT

Severity: **error** · Auto-fix: **no**

The VAR_NAME operator is unexpected because VAR_NAME(A VAR_NAME B) always returns true.

### SHOCIRF

Severity: **error** · Auto-fix: **no**

The VAR_NAME operator is unexpected because VAR_NAME(A VAR_NAME B) always returns false.

### DEBUGFUN

Severity: **error** · Auto-fix: **no**

Debug functions are intended to be used at the command line. At runtime, they will generate an error. Remove the debug function.

### INCR

Severity: **error** · Auto-fix: **no**

++x operation does not increment the value of x. To increase the value by 1, use x = x + 1.

### DECR

Severity: **error** · Auto-fix: **no**

--x operation does not decrement the value of x. To decrease the value by 1, use x = x - 1.

### CMDAND

Severity: **error** · Auto-fix: **yes**

Use 'A && B' or 'A & B' to test whether A and B are both true in MATLAB.

### CMDOR

Severity: **error** · Auto-fix: **yes**

Use 'A \|\| B' or 'A \| B' to test whether either A or B is true in MATLAB.

### RHSFN

Severity: **error** · Auto-fix: **no**

The expression cannot be assigned to multiple values.

### FNAN

Severity: **error** · Auto-fix: **yes**

Use ISNAN when comparing values to NaN.

### LOGEMP

Severity: **error** · Auto-fix: **yes**

Using 'isempty' on a logical expression creates incorrect results. To determine if all the conditions are false, use '~any(..., "all")' instead.

### STCUL

Severity: **error** · Auto-fix: **no**

The comparison will likely fail due to case mismatch.

### LBODUP

Severity: **error** · Auto-fix: **no**

Since both operands are identical, the second operand has no effect on the VAR_RESERVED_WORD operation. This indicates a bug in the code. Change one of the operands or remove the VAR_RESERVED_WORD operation.

### FUNFUN

Severity: **error** · Auto-fix: **no**

The first input argument must be a function handle. Did you mean '@VAR_NAME'?

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

Operands to '\|\|' and '&&' must be scalar values. Use 'all' or 'any' to convert this value into a scalar value or use the element-wise operators '\|' or '&' instead.

### NOPRC

Severity: **error** · Auto-fix: **no**

A line break terminates the statement so it may be incomplete. Use ellipsis (...) to continue the statement. Or add a semicolon to hide the output.

### MOCUP

Severity: **error** · Auto-fix: **no**

Variable VAR_NAME may be cleared before the cleanup function that references VAR_NAME executes, resulting in an undefined variable error.

### MDUPC

Severity: **error** · Auto-fix: **no**

The case value VAR_NAME is a duplicate of one on line VAR_NUMBER.

### MNANC

Severity: **error** · Auto-fix: **yes**

NaN never compares equal to any value, so this case will never be matched.

### MULCC

Severity: **error** · Auto-fix: **no**

This case cannot be matched due to a call to UPPER or LOWER on the SWITCH value.

### MEXCEP

Severity: **error** · Auto-fix: **no**

To report an MException as a warning, use a format specifier to ensure the message is printed correctly. For example, 'warning(E.identifier, "%s", E.message)'.

### PFUIXE

Severity: **error** · Auto-fix: **no**

The index variable VAR_NAME might be used after the PARFOR loop on line VAR_NUMBER, but it is unavailable after the loop.

### PFBFN

Severity: **error** · Auto-fix: **no**

Use of this function is invalid inside a PARFOR loop because it accesses or modifies the workspace in a non-transparent way.

### PFWHOS

Severity: **error** · Auto-fix: **no**

Using "who" or "whos" without "-file" is invalid inside a PARFOR loop because it accesses the workspace in a non-transparent way.

### PFTUSE

Severity: **error** · Auto-fix: **no**

The temporary variable VAR_NAME is used after the PARFOR loop on line VAR_NUMBER, but its value is not available after the loop.

### PFRNC

Severity: **error** · Auto-fix: **no**

Parfor reduction variable VAR_NAME must be used in the same position in each assignment statement when using non-commutative reduction operations '*', '[,]', or '[;]'.

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
debug_functions = ["keyboard", "dbstop", "dbclear", "dbcont", "dbquit", "dbup", "dbdown"]
higher_order_functions = ["cellfun", "arrayfun", "structfun", "bsxfun", "spfun"]
```

See [Configuration](../configuration.md) for the full parameter list and [rules.md](../rules.md) for the complete rule inventory.
