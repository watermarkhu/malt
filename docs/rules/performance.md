---
icon: lucide/zap
---

# Performance Improvement Checks

**Default severity:** Info
**Auto-fix:** Yes
**Category:** Performance
**Can be disabled:** Yes

## What this rule does

A hybrid rule module implementing 41 performance-related MATLAB lint
checks from MATLAB's Code Analyzer. Most checks operate at the node level
(function calls, operators, commands), while AGROW, SAGROW, and PFBNS
require file-level traversal to detect array-growth patterns inside loops.
Each diagnostic carries the specific check ID (e.g. `AGROW`, `MINV`).

A single `PerformanceEngine` struct handles all 41 checks. Node-level
checks are dispatched by node kind (`function_call`, `command`,
`binary_operator`, `boolean_operator`, `comparison_operator`,
`unary_operator`); file-level checks traverse the full tree looking for
growth patterns inside `for_statement`/`while_statement`. The node-level
`check_*` methods and their tests live in sibling `check_*.rs` modules;
this module keeps the engine, the dispatch, the file-level growth walk
(AGROW/SAGROW/PFBNS), and the shared free helpers.

## Check IDs

### AGROW

Severity: **info** · Auto-fix: **no**

Variable appears to change size on every loop iteration. Consider preallocating for speed.

### SAGROW

Severity: **info** · Auto-fix: **no**

Variable appears to change size on every loop iteration (within a script). Consider preallocating for speed.

### AND2

Severity: **info** · Auto-fix: **yes**

When both arguments are numeric scalars, consider replacing & with && for performance.

### OR2

Severity: **info** · Auto-fix: **yes**

When both arguments are numeric scalars, consider replacing \| with \|\| for performance.

### MINV

Severity: **info** · Auto-fix: **no**

INV(A)*b can be slower and less accurate than A\b. Consider using A\b for INV(A)*b or b/A for b*INV(A).

### GFLD

Severity: **info** · Auto-fix: **no**

Use dynamic fieldnames with structures instead of GETFIELD.

### SFLD

Severity: **info** · Auto-fix: **no**

Use dynamic fieldnames with structures instead of SETFIELD.

### EXIST

Severity: **info** · Auto-fix: **no**

EXIST with two input arguments is generally faster and clearer than with one input argument.

### PFBNS

Severity: **info** · Auto-fix: **no**

The entire array or structure VAR_NAME is a broadcast variable. This might result in unnecessary communication overhead.

### CCAT

Severity: **info** · Auto-fix: **no**

For improved performance, concatenate cell arrays using [] instead of extracting cell arrays and reconstructing them.

### CCAT1

Severity: **info** · Auto-fix: **no**

{ A{I} } can usually be replaced by A(I) or A(I)', which can be much faster.

### ISMT

Severity: **info** · Auto-fix: **no**

Using ISEMPTY is usually faster than comparing LENGTH to 0.

### ISCL

Severity: **info** · Auto-fix: **no**

To improve performance, use 'isscalar' instead of length comparison.

### ST2NM

Severity: **info** · Auto-fix: **yes**

If you are operating on scalar values, consider using 'str2double' for faster performance.

### FLPST

Severity: **info** · Auto-fix: **yes**

For better performance in some cases, use SORT with the 'descend' option.

### MXFND

Severity: **info** · Auto-fix: **no**

Use FIND with the 'first' or 'last' option.

### EFIND

Severity: **info** · Auto-fix: **no**

To improve performance, replace ISEMPTY(FIND(X)) with ISEMPTY(FIND( X, 1 )).

### UDIM

Severity: **info** · Auto-fix: **no**

Instead of using transpose (' or .'), consider using a different DIMENSION input argument to VAR_NAME.

### FREAD

Severity: **info** · Auto-fix: **no**

FREAD(FID,...,'*char') is more efficient than CHAR(FREAD(...)).

### N2UNI

Severity: **info** · Auto-fix: **no**

Instead of using 'native2unicode' with 'fread', specify the character encoding scheme in the call to 'fopen'.

### TNMLP

Severity: **info** · Auto-fix: **no**

Move the toolbox function out of the loop for better performance.

### LAXES

Severity: **info** · Auto-fix: **no**

Calling AXES(h) in a loop can be slow. Consider moving the call to AXES outside the loop.

### MMTC

Severity: **info** · Auto-fix: **no**

This use of MAT2CELL should probably be replaced by a simpler, faster call to NUM2CELL.

### MRPBW

Severity: **info** · Auto-fix: **yes**

To use less memory, replace BWLABEL(bw) by LOGICAL(bw) in a call of REGIONPROPS.

### SPRIX

Severity: **info** · Auto-fix: **no**

This sparse indexing expression is likely to be slow.

### TRSRT

Severity: **info** · Auto-fix: **no**

Transposing the input to 'sort' is often unnecessary.

### GRIDD

Severity: **info** · Auto-fix: **no**

Consider replacing GRIDDATA with SCATTEREDINTERPOLANT for better performance.

### CLALL

Severity: **info** · Auto-fix: **no**

Using 'clear' with the 'all' option usually decreases code performance and is often unnecessary.

### CLCLS

Severity: **info** · Auto-fix: **no**

Using 'clear' with the 'classes' option will decrease code performance and is often unnecessary.

### CLFUNC

Severity: **info** · Auto-fix: **no**

Using 'clear' with the 'functions' option usually decreases code performance and is often unnecessary.

### CLJAVA

Severity: **info** · Auto-fix: **no**

Using 'clear' with the 'java' option usually decreases code performance and is often unnecessary.

### CLMEX

Severity: **info** · Auto-fix: **no**

Using 'clear' with the 'mex' option usually decreases code performance and is often unnecessary.

### CLEAR0ARGS

Severity: **info** · Auto-fix: **no**

Avoid using 'clear' to clear more than necessary, this decreases code performance and is usually unnecessary.

### RGXP1

Severity: **info** · Auto-fix: **no**

Using REGEXP(str, pattern, 'ONCE') is faster in this case.

### RGXPI

Severity: **info** · Auto-fix: **no**

Using REGEXPI(str, pattern, 'ONCE') is faster in this case.

### TRIM1

Severity: **info** · Auto-fix: **yes**

Use STRTRIM(str) instead of nesting FLIPLR and DEBLANK calls.

### TRIM2

Severity: **info** · Auto-fix: **yes**

Use STRTRIM(str) instead of DEBLANK(STRJUST(str,'left')).

### STTOK

Severity: **info** · Auto-fix: **no**

Use one call to 'split' instead of calling 'strtok' in a loop.

### STNCI

Severity: **info** · Auto-fix: **no**

Use STRNCMPI(str1,str2) instead of using UPPER/LOWER in a call to STRNCMP.

### STCCS

Severity: **info** · Auto-fix: **no**

It appears that STRCMPI/STRNCMPI can be replaced by a faster, case sensitive compare.

### FNDSB

Severity: **info** · Auto-fix: **yes**

For array or cell array, performance can be improved using logical indexing instead of 'find'.

## Automatic fixes

Rewrites the flagged call or operator into its faster equivalent:

- `&` → `&&` and `|` → `||` in boolean contexts (AND2, OR2).
- `str2num` → `str2double` (ST2NM), `flipud`/`fliplr` → `flip` (FLPST).
- `im2bw` → `imbinarize` (MRPBW), `deblank` → `strtrim` (TRIM1),
  `strtrim` → `strip` (TRIM2), `findstr` → `contains` (FNDSB).

## Examples

### Incorrect

```matlab
for i = 1:n
    x = [x, val];          % AGROW
end
if a & b                   % AND2
inv(A) * b                 % MINV
exist('foo', 'file')       % EXIST
```

### Correct

```matlab
x = zeros(1, n);
for i = 1:n
    x(i) = val;
end
if a && b
A \ b
isfile('foo')
```

### Fixed

```diff
- if a & b
+ if a && b
```

## Configuration

```toml
[lint.rules.PERFORMANCE_ENGINE]
severity = "info"
skip_checks = ["AGROW", "ST2NM"]
```

See [Configuration](../configuration.md) for the full parameter list and [rules.md](../rules.md) for the complete rule inventory.
