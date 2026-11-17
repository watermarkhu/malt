---
icon: lucide/wrench
---

# Incomplete Analysis

**Default severity:** Error
**Auto-fix:** No
**Category:** Incomplete Analysis
**Can be disabled:** Yes
**Enabled by default:** Yes

## What this rule does

Implements the "Incomplete Analysis" checks from MATLAB's Code Analyzer:
the limits at which the analysis stops being reliable. Too many messages
(TMMSG, reported by the linter, which sees every rule), too many parse errors
(TMSMS), parentheses nested too deeply (MDEEP), functions nested too deeply
(DEEPN), block comments nested too deeply (DEEPC), statements nested too
deeply (NOSPC), and an expression with too many nested operators (MXASET).
The thresholds are MATLAB's, measured with `checkcode` (R2026b) on generated
files. They cannot be disabled by the user. MXASET, NOSPC, MDEEP and DEEPN
stop the analysis: MATLAB reports only them for the file, so malt does too.
MDEEP and DEEPN are measured before malt's own depth guard, so a file nested
more deeply than malt walks still gets MATLAB's message.

Some MATLAB checks of this category were never reported in the probes, so
malt does not emit them: MBIG (no report up to 16 MB), TEXTL (no report for
lines or string literals of 5 MB), DEEPS (nesting beyond the NOSPC limit gives
NOSPC only), EOFER (more than 1,000 syntax errors give TMSMS only), and EOFMI
(a file ending in an unfinished statement gives SYNER). MDOTM and MDMCR are
kept: `checkcode` raises an error instead of reporting them for a file that is
not a `.m` file, so only the message text is taken from MATLAB.

malt's own safety limits are not MATLAB checks and are set under
`[lint.limits]` (see the Resource limits section of the configuration page).
Code nested beyond the depth of the parse tree (`max_tree_depth`), or with
more nested brackets than `max_nesting`, gets a `MALT_TOO_DEEP` diagnostic;
a file larger than `max_file_size` gets `MALT_TOO_BIG` and is not read. Neither
is analysed (see `Linter::lint`).

## Check IDs

### TMMSG

Severity: **error** · Auto-fix: **no**

More than 10,000 Code Analyzer messages were generated, leading to some being deleted.

### TMSMS

Severity: **error** · Auto-fix: **no**

More than 1,000 parse error messages were generated, leading to some being deleted.

### MXASET

Severity: **error** · Auto-fix: **no**

The file is too complex to analyze. Simplify the code to improve code maintainability. For example, reduce the number of operations in expressions.

### NOSPC

Severity: **error** · Auto-fix: **no**

The file is too complex to analyze. Refactor the code to improve code maintainability. For example, reduce the nesting level of conditions or functions.

### MDOTM

Severity: **error** · Auto-fix: **no**

Unable to run code analysis. VAR_FILE has an invalid file extension.

### MDMCR

Severity: **error** · Auto-fix: **no**

Unable to run code analysis. VAR_FILE is a deployed MATLAB file.

### MDEEP

Severity: **error** · Auto-fix: **no**

Parentheses, brackets, and braces are nested too deeply.

### DEEPC

Severity: **error** · Auto-fix: **no**

Block comments are nested too deeply.

### DEEPN

Severity: **error** · Auto-fix: **no**

Functions are nested too deeply.

### EOFER

Severity: **error** · Auto-fix: **no**

Code analysis did not complete. File contains too many syntax errors.

## Examples

### Incorrect

```matlab
% MDEEP: 33 nested parentheses (MATLAB accepts 32)
z = ((((((((((((((((((((((((((((((((((1))))))))))))))))))))))))))))))))));
```

### Correct

```matlab
z = ((((((((((((((((((((((((((((((((1))))))))))))))))))))))))))))))));
```

## Configuration

```toml
[lint.rules.INCOMPLETE_ANALYSIS]
max_parse_errors = 1000
max_paren_depth = 32
max_function_depth = 17
```

See [Configuration](../configuration.md) for the full parameter list and [rules.md](../rules.md) for the complete rule inventory.
