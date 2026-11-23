---
icon: lucide/eye
---

# Readability Improvements

**Default severity:** Info
**Auto-fix:** Yes
**Category:** Readability
**Can be disabled:** Yes
**Enabled by default:** Yes
**Maturity:** Mixed (4 of 35 experimental)

## What this rule does

A multi-check rule engine that detects readability improvements in MATLAB
code. Instead of implementing one struct per check, a single
`ReadabilityEngine` dispatches to per-check-ID logic inside its `check()`
method (and a file-level pass for MFAMB), emitting diagnostics with the
specific check ID (e.g. `ISCHR`, `IJCL`, `RPMT1`).

The 30 checks cover type-checking simplification (`strcmp(class(x), 'char')`
→ dedicated predicates), dimension checks (`size(x,dim)==1` →
`isrow`/`iscolumn`, `ndims(x)==2` → `ismatrix`), the imaginary unit,
unnecessary brackets, `strfind`-based containment tests, `char(10)` and
`sprintf('\n')` → `newline`, `repmat` of constants, `error`/`warning`
message construction, `display`/`disp` of `sprintf`, flip/rotate, logical
helpers, `arguments` attributes, and ambiguous identifiers.

## Check IDs

### COMNL

Severity: **info** · Auto-fix: **yes** · Maturity: **stable**

Newline following comma acts as a row separator. Replace the comma with a semicolon to make the row separation clearer. Alternatively, use an ellipsis (...) to continue the current row on the next line.

### SPERR

Severity: **info** · Auto-fix: **no** · Maturity: **stable**

ERROR takes SPRINTF-like arguments directly.

### SPWRN

Severity: **info** · Auto-fix: **no** · Maturity: **stable**

WARNING takes SPRINTF-like arguments directly.

### NCHKE

Severity: **info** · Auto-fix: **no** · Maturity: **experimental**

Use NARGOUTCHK without ERROR.

### DSPSP

Severity: **info** · Auto-fix: **yes** · Maturity: **stable**

'disp(sprintf(...))' can usually be replaced by 'fprintf(...\n)'.

### DSPSY

Severity: **info** · Auto-fix: **no** · Maturity: **stable**

'display(sprintf(...))' can usually be replaced by 'fprintf(...\n)'.

### STLOW

Severity: **info** · Auto-fix: **yes** · Maturity: **stable**

In this comparison the call to UPPER/LOWER is unnecessary.

### FLUDLR

Severity: **info** · Auto-fix: **yes** · Maturity: **stable**

For readability, consider using rot90(x,2) instead of flipud(fliplr(x)) or fliplr(flipud(x)).

### RPMT1

Severity: **info** · Auto-fix: **yes** · Maturity: **stable**

For readability, consider using 'ones(x,y)' instead of 'repmat(1,x,y)'.

### RPMT0

Severity: **info** · Auto-fix: **yes** · Maturity: **stable**

For readability, consider using 'zeros(x,y)' instead of 'repmat(0,x,y)'.

### RPMTT

Severity: **info** · Auto-fix: **yes** · Maturity: **stable**

For readability, consider using 'true(x,y)' instead of 'repmat(true,x,y)'.

### RPMTF

Severity: **info** · Auto-fix: **yes** · Maturity: **stable**

For readability, consider using 'false(x,y)' instead of 'repmat(false,x,y)'.

### RPMTI

Severity: **info** · Auto-fix: **yes** · Maturity: **stable**

For readability, consider using 'Inf(x,y)' instead of 'repmat(Inf,x,y)'.

### RPMTN

Severity: **info** · Auto-fix: **yes** · Maturity: **stable**

For readability, consider using 'NaN(x,y)' instead of 'repmat(NaN,x,y)'.

### LOGSUM

Severity: **info** · Auto-fix: **no** · Maturity: **stable**

Consider using 'nnz' instead of 'sum' for logical vectors to improve readability.

### LOGL

Severity: **info** · Auto-fix: **yes** · Maturity: **stable**

Use 'true' or 'false' instead of 'logical(1)' or 'logical(0)'.

### ISCHR

Severity: **info** · Auto-fix: **yes** · Maturity: **stable**

Use ISCHAR instead of comparing the class to 'char'.

### ISSTR

Severity: **info** · Auto-fix: **yes** · Maturity: **stable**

Use ISSTRUCT instead of comparing the class to 'struct'.

### ISLOG

Severity: **info** · Auto-fix: **yes** · Maturity: **stable**

Use ISLOGICAL instead of comparing the class to 'logical'.

### ISCEL

Severity: **info** · Auto-fix: **yes** · Maturity: **stable**

Use ISCELL instead of comparing the class to 'cell'.

### IJCL

Severity: **info** · Auto-fix: **yes** · Maturity: **stable**

For improved robustness, consider replacing i and j by 1i.

### ASGSL

Severity: **info** · Auto-fix: **no** · Maturity: **stable**

Assignment to variable might be unnecessary.

### ISMAT

Severity: **info** · Auto-fix: **yes** · Maturity: **stable**

When checking if a variable is a matrix consider using ISMATRIX.

### ISROW

Severity: **info** · Auto-fix: **yes** · Maturity: **experimental**

When checking if a variable is a row vector consider using ISROW.

### ISCOL

Severity: **info** · Auto-fix: **yes** · Maturity: **experimental**

When checking if a variable is a column vector consider using ISCOLUMN.

### NBRAK2

Severity: **info** · Auto-fix: **yes** · Maturity: **stable**

Use of brackets [] is unnecessary.

### MFAMB

Severity: **info** · Auto-fix: **no** · Maturity: **experimental**

Code Analyzer cannot determine whether VAR_NAME is a variable or a function, and assumes it is a function.

### FVINR

Severity: **info** · Auto-fix: **yes** · Maturity: **stable**

For readability, add Input attribute to the input arguments block.

### STREMP

Severity: **info** · Auto-fix: **yes** · Maturity: **stable**

For readability, use '~contains(str1, str2)' instead of 'isempty(strfind(str1, str2))'.

### STRCL1

Severity: **info** · Auto-fix: **yes** · Maturity: **stable**

For readability, use '~contains(str1, str2)' instead of 'cellfun('isempty', strfind(str1, str2))'.

### STRCLFH

Severity: **info** · Auto-fix: **yes** · Maturity: **stable**

For readability, use '~contains(str1, str2)' instead of 'cellfun(@isempty, strfind(str1, str2))'.

### STRIFCND

Severity: **info** · Auto-fix: **yes** · Maturity: **stable**

For readability, use 'contains(str1, str2)' instead of 'strfind(str1, str2)'.

### CHARTEN

Severity: **info** · Auto-fix: **yes** · Maturity: **stable**

For readability, consider using 'newline' instead of 'char(10)'.

### PSIZE

Severity: **info** · Auto-fix: **yes** · Maturity: **stable**

NUMEL(x) is usually faster than PROD(SIZE(x)).

### SPRINTFN

Severity: **info** · Auto-fix: **yes** · Maturity: **stable**

For readability, consider using the 'newline' function instead of 'sprintf('\n')'.

## Maturity

Experimental checks are off by default and run with `[lint] experimental = true`, the `all` preset, or an explicit rule setting (see [Experimental checks](../configuration.md#experimental-checks)).

`NCHKE`: MATLAB stayed silent on every probed shape malt reported them on (`a &b` as a command, `size` overloads, `arrayfun('f', x)`, `warning(E.message)`, a handle default or `subsindex` in a class, parfor cell and nested-loop shapes, `strcmpi` with a digit literal); PFMLTI is reported as PFFORA, NCHKE as NCHKM. Off under the mathworks preset (docs/VERIFY.md 'Recording the stable checks').

`ISROW`, `ISCOL`, `MFAMB`: On the mathworks off-list: MATLAB's default analysis reported none of them in 4 or more probe shapes each, so their on-side (the --preset all behaviour) is not compared with MATLAB.

## Automatic fixes

Rewrites the flagged construct into the clearer equivalent:

- `strcmp(class(x), 'char')` → `ischar(x)` (ISCHR, ISSTR, ISLOG, ISCEL).
- `size(x, dim) == 1` → `isrow(x)`/`iscolumn(x)` (ISROW, ISCOL);
  `ndims(x) == 2` → `ismatrix(x)` (ISMAT).
- `repmat(c, x, y)` → `ones`/`zeros`/`true`/`false`/`Inf`/`NaN(x, y)`
  (RPMT1, RPMT0, RPMTT, RPMTF, RPMTI, RPMTN).
- `char(10)` and `sprintf('\n')` → `newline`; `logical(1)` → `true`, `logical(0)` → `false`; `logical(zeros(...))` →
  `false(...)` and `logical(ones(...))` → `true(...)` (unsafe: an argument can be a
  class name, which `false` rejects) (LOGL).
- `flipud(fliplr(x))` → `rot90(x,2)` (as MATLAB's own fix writes it);
  `[scalar]` loses only its two bracket tokens, keeping the inner text,
  comments and spacing as they are (NBRAK2, MATLAB's fix is the same);
  `disp(sprintf(...))` → `fprintf(...)`.
- `isempty(strfind(s, p))` → `~contains(s, p)`, and the negated form
  rewrites as a whole (`~isempty(strfind(s, p))` → `contains(s, p)`),
  with the arguments reprinted verbatim, both as MATLAB's fix does;
  `cellfun(@isempty, strfind(s, p))` → `~contains(s, p)`;
  `if strfind(s, p)` → `if contains(s, p)`.
- `strcmpi(lower(x), y)` → `strcmpi(x, y)` (STLOW).
- `i`/`j` as the imaginary unit → `1i` (IJCL; MATLAB's fix writes `1i`
  for `j` too, and `1i` and `1j` are the same constant).
- A trailing comma before a newline in a matrix becomes a semicolon (COMNL).
- An `(Input)` attribute is inserted into attribute-less `arguments`
  blocks (FVINR).

Fixes that can change behaviour (STLOW, STREMP, STRCL1, STRCLFH, STRIFCND,
ISROW, ISCOL, IJCL) are unsafe and need `--unsafe-fixes`.

## Examples

### Incorrect

```matlab
if strcmp(class(x), 'char')
y = flipud(fliplr(m));
z = repmat(0, 3, 4);
```

### Correct

```matlab
if ischar(x)
y = rot90(m, 2);
z = zeros(3, 4);
```

### Fixed

```diff
- z = repmat(0, 3, 4);
+ z = zeros(3, 4);
```

## Configuration

```toml
[lint.rules.READABILITY_ENGINE]
severity = "info"
disabled_checks = ["IJCL", "NBRAK2"]
```

See [Configuration](../configuration.md) for the full parameter list and [rules.md](../rules.md) for the complete rule inventory.
