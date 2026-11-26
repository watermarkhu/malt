---
icon: lucide/bug
---

# Bugs

**Default severity:** Error
**Auto-fix:** Yes
**Category:** Bugs
**Can be disabled:** Yes
**Enabled by default:** Yes
**Maturity:** Mixed (7 of 33 experimental)

## What this rule does

Detects likely bugs and logic errors in MATLAB code. All 34 checks share a
single hybrid engine (`BugsEngine`) that performs both node-level checking
(dispatched per-node during traversal) and file-level checking (full-tree
analysis after traversal). Each diagnostic carries the specific check ID
(e.g. `IFBDUP`, `FNAN`).

CTRUE and CFALSE: a comparison of a function that returns a logical value with a
numeric literal, such as `any(A) <= 1`, is constant, because a logical is only 0
or 1. SHOCIRT and SHOCIRF: a predicate applied to the result of a short-circuit
operator, such as `isscalar(A && B)`, is constant, because `&&` and `||` always
give a logical scalar (or an error). DEBUGFUN: a debug function, which is meant
for the Command Window, is called in code that runs at runtime, where it fails.

## Check IDs

### IFBDUP

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

This condition has no effect because all blocks in this if statement are identical. This indicates a bug in the code. Remove the condition or change the code blocks.

### IFCDUP

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

The statements under this &lt;reserved word&gt; condition cannot be reached because it is a duplicate of the &lt;reserved word&gt; condition on line &lt;number&gt;. This indicates a bug in the code. Remove or change the condition.

### CTRUE

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

This logical comparison always returns true. Did you mean to use &lt;name&gt; to evaluate function argument: &lt;name&gt;(...&lt;name&gt;...)?

### CFALSE

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

This logical comparison always returns false. Did you mean to use &lt;name&gt; to evaluate function argument: &lt;name&gt;(...&lt;name&gt;...)?

### SHOCIRT

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

The &lt;name&gt; operator is unexpected because &lt;name&gt;(A &lt;name&gt; B) always returns true.

### SHOCIRF

Severity: **error** · Auto-fix: **no** · Maturity: **experimental**

The &lt;name&gt; operator is unexpected because &lt;name&gt;(A &lt;name&gt; B) always returns false.

### DEBUGFUN

Severity: **error** · Auto-fix: **yes** · Maturity: **stable**

Debug functions are intended to be used at the command line. At runtime, they will generate an error. Remove the debug function.

### INCR

Severity: **error** · Auto-fix: **yes** · Maturity: **stable**

++x operation does not increment the value of x. To increase the value by 1, use x = x + 1.

### DECR

Severity: **error** · Auto-fix: **yes** · Maturity: **stable**

--x operation does not decrement the value of x. To decrease the value by 1, use x = x - 1.

### CMDAND

Severity: **error** · Auto-fix: **yes** · Maturity: **experimental**

Use 'A && B' or 'A & B' to test whether A and B are both true in MATLAB.

### CMDOR

Severity: **error** · Auto-fix: **yes** · Maturity: **experimental**

Use 'A

### RHSFN

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

The expression cannot be assigned to multiple values.

### FNAN

Severity: **error** · Auto-fix: **yes** · Maturity: **stable**

Use ISNAN when comparing values to NaN.

### LOGEMP

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

Using 'isempty' on a logical expression creates incorrect results. To determine if all the conditions are false, use '~any(..., "all")' instead.

### STCUL

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

The comparison will likely fail due to case mismatch.

### LBODUP

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

Since both operands are identical, the second operand has no effect on the &lt;reserved word&gt; operation. This indicates a bug in the code. Change one of the operands or remove the &lt;reserved word&gt; operation.

### FUNFUN

Severity: **error** · Auto-fix: **yes** · Maturity: **experimental**

The first input argument must be a function handle. Did you mean '@&lt;name&gt;'?

### DEFSIZE

Severity: **error** · Auto-fix: **no** · Maturity: **experimental**

Do not overload 'size' for fundamental data types.

### VARARG

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

Initialize VARARGOUT with a CELL.

### STRCMPCSTR

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

'strcmp' always returns false for string elements of a cell array. Use ["str1", "str2"] instead of {"str1", "str2"}.

### ASSRT

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

The first input argument to 'assert' must be a condition. To always throw an error, use 'error(msg)' instead.

### BDSCA2

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

Operands to '\|\|' and '&&' must be scalar values.  Use 'all' or 'any' to convert this value into a scalar value or use the element-wise operators '\|' or '&' instead.

### NOPRC

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

A line break terminates the statement so it may be incomplete. Use ellipsis (...) to continue the statement. Or add a semicolon to hide the output.

### MOCUP

Severity: **error** · Auto-fix: **no** · Maturity: **experimental**

Variable &lt;name&gt; may be cleared before the cleanup function that references &lt;name&gt; executes, resulting in an undefined variable error.

### MDUPC

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

The case value &lt;name&gt; is a duplicate of one on line &lt;number&gt;.

### MNANC

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

NaN never compares equal to any value, so this case will never be matched.

### MULCC

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

This case cannot be matched due to a call to UPPER or LOWER on the SWITCH value.

### MEXCEP

Severity: **error** · Auto-fix: **no** · Maturity: **experimental**

To report an MException as a warning, use a format specifier to ensure the message is printed correctly. For example, 'warning(E.identifier, "%s", E.message)'.

### PFUIXE

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

The index variable &lt;name&gt; might be used after the PARFOR loop on line &lt;number&gt;, but it is unavailable after the loop.

### PFBFN

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

Use of this function is invalid inside a PARFOR loop because it accesses or modifies the workspace in a non-transparent way.

### PFWHOS

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

Using "who" or "whos" without "-file" is invalid inside a PARFOR loop because it accesses the workspace in a non-transparent way.

### PFTUSE

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

The temporary variable &lt;name&gt; is used after the PARFOR loop on line &lt;number&gt;, but its value is not available after the loop.

### PFRNC

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

Parfor reduction variable &lt;name&gt; must be used in the same position in each assignment statement when using non-commutative reduction operations '*', '[,]', or '[;]'.

## Maturity

Experimental checks are off by default and run with `[lint] experimental = true`, the `all` preset, or an explicit rule setting (see [Experimental checks](../configuration.md#experimental-checks)).

`SHOCIRF`, `MOCUP`: On the mathworks off-list: MATLAB's default analysis reported none of them in 4 or more probe shapes each, so their on-side (the --preset all behaviour) is not compared with MATLAB.

`CMDAND`, `CMDOR`, `FUNFUN`, `DEFSIZE`, `MEXCEP`: MATLAB stayed silent on every probed shape malt reported them on (`a &b` as a command, `size` overloads, `arrayfun('f', x)`, `warning(E.message)`, a handle default or `subsindex` in a class, parfor cell and nested-loop shapes, `strcmpi` with a digit literal); PFMLTI is reported as PFFORA, NCHKE as NCHKM. Off under the mathworks preset (docs/VERIFY.md 'Recording the stable checks').

## Automatic fixes

Rewrites the flagged construct into the safe equivalent. For example:

- `&` → `&&` and `|` → `||` in boolean contexts.
- `x == NaN` → `isnan(x)` (and `x ~= NaN` → `~isnan(x)`); with an
  elementwise operator behind it the rest is kept, as MATLAB's fix does
  (`if n == NaN | n > 3` → `if isnan(n) | n > 3`).
- `cellfun('name', c)` → `cellfun(@name, c)` when the first argument is a
  literal that is a plain name (FUNFUN). Unsafe: a name that is a variable
  stops working.
- A debug call such as `dbstep();` that has its line to itself is deleted
  (DEBUGFUN). Unsafe: it is code the author wrote.

`isempty(cond)` on a logical expression (LOGEMP) has no fix: `~any(cond, "all")`
is true when no element is set, `isempty` only when there are no elements.

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
