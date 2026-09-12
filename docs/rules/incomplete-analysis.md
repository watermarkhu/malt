---
icon: lucide/wrench
---

# Incomplete Analysis

**Default severity:** Error
**Auto-fix:** No
**Category:** Incomplete Analysis
**Can be disabled:** Yes

## What this rule does

Implements the "Incomplete Analysis" checks from MATLAB's Code Analyzer.
These diagnostics report when the analysis itself was limited or could not
complete reliably: too many diagnostics, too many parse errors, files that
are too large, too deeply nested, or too long. They represent linter
internal limits and **cannot be disabled** by the user.

A single file-level engine (`IncompleteAnalysisEngine`) collects tree
metrics (node count, ERROR count, nesting depths) in one pass, then applies
each threshold check against the configured limits. Each diagnostic carries
the specific check ID (e.g. `TMMSG`, `MDEEP`).

Three related MATLAB checks are not emitted by this engine: `QUIT`
(analysis did not complete — the panic guard lives in the linter core),
`NOFIL` (file not found), and `RDERR` (unable to read file) are handled by
the CLI.

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

### MBIG

Severity: **error** · Auto-fix: **no**

Code analysis did not complete. File VAR_FILE is too large.

### MDOTM

Severity: **error** · Auto-fix: **no**

Unable to run code analysis. VAR_FILE has an invalid file extension.

### MDMCR

Severity: **error** · Auto-fix: **no**

Unable to run code analysis. VAR_FILE is a deployed MATLAB file.

### EOFER

Severity: **error** · Auto-fix: **no**

Code analysis did not complete. File contains too many syntax errors.

### EOFMI

Severity: **error** · Auto-fix: **no**

Invalid syntax at end of file. File is incomplete.

### MDEEP

Severity: **error** · Auto-fix: **no**

Parentheses, brackets, and braces are nested too deeply.

### DEEPC

Severity: **error** · Auto-fix: **no**

Block comments are nested too deeply.

### DEEPN

Severity: **error** · Auto-fix: **no**

Functions are nested too deeply.

### DEEPS

Severity: **error** · Auto-fix: **no**

Statements are nested too deeply.

### TEXTL

Severity: **error** · Auto-fix: **no**

Text is too long for MATLAB to parse.

## Examples

### Incorrect

```matlab
% EOFMI: file ends mid-block without a matching `end`
function f()
    if x > 0
        y = 1;
% MDEEP: parentheses nested deeper than the configured limit
z = ((((((((((((((((((((((((((((((((((((((((((1)))))))))))))))))))))))))))))))))))))))))))));
```

### Correct

```matlab
function f()
    if x > 0
        y = 1;
    end
end
z = 1;
```

## Configuration

```toml
[lint.rules.INCOMPLETE_ANALYSIS]
max_diagnostics = 10000
max_parse_errors = 1000
max_node_count = 100000
max_file_size = 1048576
max_paren_depth = 32
max_function_depth = 20
max_statement_depth = 15
max_line_length = 4096
```

See [Configuration](../configuration.md) for the full parameter list and [rules.md](../rules.md) for the complete rule inventory.
