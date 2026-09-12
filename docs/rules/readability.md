---
icon: lucide/eye
---

# Readability Improvements

**Default severity:** Info
**Auto-fix:** Yes
**Category:** Readability
**Can be disabled:** Yes

## What this rule does

A multi-check rule engine that detects readability improvements in MATLAB
code. Instead of implementing one struct per check, a single
`ReadabilityEngine` dispatches to per-check-ID logic inside its `check()`
method (and a file-level pass for MFAMB), emitting diagnostics with the
specific check ID (e.g. `ISCHR`, `IJCL`, `RPMT1`).

The 35 checks cover type-checking simplification (`isa(x, 'type')` →
dedicated predicates), dimension checks (`size(x,dim)==1` → `isrow`/`iscolumn`),
variable shadowing, unnecessary brackets, modern string functions,
character literals, formatting, control-flow style, error/warning message
IDs, input validation, output style, flip/rotate, redundant arithmetic and
boolean logic, size and logical helpers, `arguments` attributes, and
ambiguous identifiers.

## Check IDs

### ASGSL

Severity: **info** · Auto-fix: **no**

Assignment to variable might be unnecessary.

### COMNL

Severity: **info** · Auto-fix: **yes**

Newline following comma acts as a row separator. Replace the comma with a semicolon to make the row separation clearer. Alternatively, use an ellipsis (...) to continue the current row on the next line.

### SPERR

Severity: **info** · Auto-fix: **no**

ERROR takes SPRINTF-like arguments directly.

### SPWRN

Severity: **info** · Auto-fix: **no**

WARNING takes SPRINTF-like arguments directly.

### NCHKE

Severity: **info** · Auto-fix: **no**

Use NARGOUTCHK without ERROR.

### DSPSP

Severity: **info** · Auto-fix: **yes**

'disp(sprintf(...))' can usually be replaced by 'fprintf(...\n)'.

### DSPSY

Severity: **info** · Auto-fix: **no**

'display(sprintf(...))' can usually be replaced by 'fprintf(...\n)'.

### STLOW

Severity: **info** · Auto-fix: **yes**

In this comparison the call to UPPER/LOWER is unnecessary.

### FLUDLR

Severity: **info** · Auto-fix: **yes**

For readability, consider using rot90(x,2) instead of flipud(fliplr(x)) or fliplr(flipud(x)).

### RPMT1

Severity: **info** · Auto-fix: **yes**

For readability, consider using 'ones(x,y)' instead of 'repmat(1,x,y)'.

### RPMT0

Severity: **info** · Auto-fix: **yes**

For readability, consider using 'zeros(x,y)' instead of 'repmat(0,x,y)'.

### RPMTT

Severity: **info** · Auto-fix: **yes**

For readability, consider using 'true(x,y)' instead of 'repmat(true,x,y)'.

### RPMTF

Severity: **info** · Auto-fix: **yes**

For readability, consider using 'false(x,y)' instead of 'repmat(false,x,y)'.

### RPMTI

Severity: **info** · Auto-fix: **yes**

For readability, consider using 'Inf(x,y)' instead of 'repmat(Inf,x,y)'.

### RPMTN

Severity: **info** · Auto-fix: **yes**

For readability, consider using 'NaN(x,y)' instead of 'repmat(NaN,x,y)'.

### PSIZE

Severity: **info** · Auto-fix: **yes**

NUMEL(x) is usually faster than PROD(SIZE(x)).

### LOGSUM

Severity: **info** · Auto-fix: **no**

Consider using 'nnz' instead of 'sum' for logical vectors to improve readability.

### LOGL

Severity: **info** · Auto-fix: **no**

Use 'true' or 'false' instead of 'logical(1)' or 'logical(0)'.

### ISCHR

Severity: **info** · Auto-fix: **yes**

Use ISCHAR instead of comparing the class to 'char'.

### ISSTR

Severity: **info** · Auto-fix: **yes**

Use ISSTRUCT instead of comparing the class to 'struct'.

### ISLOG

Severity: **info** · Auto-fix: **yes**

Use ISLOGICAL instead of comparing the class to 'logical'.

### ISCEL

Severity: **info** · Auto-fix: **yes**

Use ISCELL instead of comparing the class to 'cell'.

### IJCL

Severity: **info** · Auto-fix: **no**

For improved robustness, consider replacing i and j by 1i.

### ISMAT

Severity: **info** · Auto-fix: **yes**

When checking if a variable is a matrix consider using ISMATRIX.

### ISROW

Severity: **info** · Auto-fix: **yes**

When checking if a variable is a row vector consider using ISROW.

### ISCOL

Severity: **info** · Auto-fix: **yes**

When checking if a variable is a column vector consider using ISCOLUMN.

### NBRAK2

Severity: **info** · Auto-fix: **yes**

Use of brackets [] is unnecessary.

### MFAMB

Severity: **info** · Auto-fix: **no**

Code Analyzer cannot determine whether VAR_NAME is a variable or a function, and assumes it is a function.

### FVINR

Severity: **info** · Auto-fix: **yes**

For readability, add Input attribute to the input arguments block.

### STREMP

Severity: **info** · Auto-fix: **yes**

For readability, use '~contains(str1, str2)' instead of 'isempty(strfind(str1, str2))'.

### STRCL1

Severity: **info** · Auto-fix: **no**

For readability, use '~contains(str1, str2)' instead of 'cellfun('isempty', strfind(str1, str2))'.

### STRCLFH

Severity: **info** · Auto-fix: **no**

For readability, use '~contains(str1, str2)' instead of 'cellfun(@isempty, strfind(str1, str2))'.

### STRIFCND

Severity: **info** · Auto-fix: **no**

For readability, use 'contains(str1, str2)' instead of 'strfind(str1, str2)'.

### CHARTEN

Severity: **info** · Auto-fix: **yes**

For readability, consider using 'newline' instead of 'char(10)'.

### SPRINTFN

Severity: **info** · Auto-fix: **yes**

For readability, consider using the 'newline' function instead of 'sprintf('\n')'.

## Automatic fixes

Rewrites the flagged construct into the clearer equivalent:

- `isa(x, 'type')` → `ischar`/`isstring`/`islogical`/`iscell`/`isnumeric`
  (ISCHR, ISSTR, ISLOG, ISCEL, ISMAT).
- `size(x, dim) == 1` → `isrow(x)`/`iscolumn(x)` (ISROW, ISCOL).
- `x * 1` → `x`, `x * 0` → `zeros(size(x))`, `x + 0` → `x`,
  `x - 0` → `x` (RPMT1, RPMT0, RPMTI, RPMTN).
- `x | true` → `true` and `x & false` → `false` (RPMTT, RPMTF).
- `prod(size(x))` → `numel(x)`, `char(10)` → `newline`,
  `sprintf('%d', x)` → `num2str(x)`, `strcmp(s, '')` → `strlength(s)==0`.
- `flipud(fliplr(x))` → `rot90(x, 2)`, `[scalar]` → `scalar`,
  `disp(sprintf(...))` → `fprintf(...)`.
- A trailing comma before a newline in a matrix becomes a semicolon (COMNL).
- An `(Input)` attribute is inserted into attribute-less `arguments`
  blocks (FVINR).

## Examples

### Incorrect

```matlab
if isa(x, 'char')
if a || true
y = flipud(fliplr(m));
n = prod(size(a));
```

### Correct

```matlab
if ischar(x)
if a
y = rot90(m, 2);
n = numel(a);
```

### Fixed

```diff
- n = prod(size(a));
+ n = numel(a);
```

## Configuration

```toml
[lint.rules.READABILITY_ENGINE]
severity = "info"
disabled_checks = ["IJCL", "NBRAK2"]
```

See [Configuration](../configuration.md) for the full parameter list and [rules.md](../rules.md) for the complete rule inventory.
