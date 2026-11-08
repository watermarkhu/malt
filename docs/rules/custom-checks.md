---
icon: lucide/sliders-horizontal
---

# Custom Checks

**Default severity:** Warning
**Auto-fix:** No
**Category:** Custom Checks
**Can be disabled:** Yes
**Enabled by default:** No (off in the default `mathworks` preset)

## What this rule does

Configurable code-complexity and style-metric checks from MATLAB's Code
Analyzer. All 25 checks are handled by a single file-level engine
(`CustomChecksEngine`) that performs one traversal per file and computes
all metrics simultaneously. Each check has a configurable threshold; a
threshold of `0` disables that check.

## Check IDs

### SYSBANG

Severity: **warning** · Auto-fix: **no**

Use of bang operator is disallowed by custom code analyzer configuration.

### FCNIL

Severity: **warning** · Auto-fix: **no**

Function has more than VAR_NUMBER input arguments. This makes the function difficult to understand and maintain.

### FCNOL

Severity: **warning** · Auto-fix: **no**

Function has more than VAR_NUMBER output arguments. This makes the function difficult to understand and maintain.

### FCNLL

Severity: **warning** · Auto-fix: **no**

Function has more than VAR_NUMBER lines. This makes the function difficult to understand and maintain.

### LLMNC

Severity: **warning** · Auto-fix: **no**

Line has more than VAR_NUMBER characters (including whitespaces). This makes the line difficult to understand and maintain.

### MNCSN

Severity: **warning** · Auto-fix: **no**

This control statement is deeply nested (nesting level = VAR_NUMBER) and might have more deeply nested control statements. This makes the code difficult to understand and maintain.

### DAFTC

Severity: **warning** · Auto-fix: **no**

Use of try/catch statement is disallowed by custom code analyzer configuration.

### DAFPV

Severity: **warning** · Auto-fix: **no**

Use of persistent variable is disallowed by custom code analyzer configuration.

### DAFCO

Severity: **warning** · Auto-fix: **no**

Use of continue statement is disallowed by custom code analyzer configuration.

### DAFBR

Severity: **warning** · Auto-fix: **no**

Use of break statement is disallowed by custom code analyzer configuration.

### DAFRT

Severity: **warning** · Auto-fix: **no**

Use of return statement is disallowed by custom code analyzer configuration.

### DAFSC

Severity: **warning** · Auto-fix: **no**

Use of a script is disallowed by custom code analyzer configuration.

### DAFNF

Severity: **warning** · Auto-fix: **no**

Use of a nested function is disallowed by custom code analyzer configuration.

### DAFCF

Severity: **warning** · Auto-fix: **no**

Use of command syntax to call a function is disallowed by custom code analyzer configuration.

### DAFAF

Severity: **warning** · Auto-fix: **no**

Use of an anonymous function is disallowed by custom code analyzer configuration.

### DAFCV

Severity: **warning** · Auto-fix: **no**

Use of character vector is disallowed by custom code analyzer configuration.

### DAFCVC

Severity: **warning** · Auto-fix: **no**

Use of cell array of character vectors is disallowed by custom code analyzer configuration.

### DAFVI

Severity: **warning** · Auto-fix: **no**

Use of varargin is disallowed by custom code analyzer configuration.

### DAFVO

Severity: **warning** · Auto-fix: **no**

Use of varargout is disallowed by custom code analyzer configuration.

### CYCCOM

Severity: **warning** · Auto-fix: **no**

Function has a McCabe cyclomatic complexity of more than VAR_NUMBER. This makes the function difficult to understand and maintain.

### SCYCCOM

Severity: **warning** · Auto-fix: **no**

Script has a McCabe cyclomatic complexity of more than VAR_NUMBER. This makes the script difficult to understand and maintain.

### ACYCCOM

Severity: **warning** · Auto-fix: **no**

Anonymous function has a McCabe cyclomatic complexity of more than VAR_NUMBER. This makes the anonymous function difficult to understand and maintain.

### MCYCCOM

Severity: **warning** · Auto-fix: **no**

Function has a modified cyclomatic complexity of more than VAR_NUMBER. This makes the function difficult to understand and maintain.

### MSCYCCOM

Severity: **warning** · Auto-fix: **no**

Script has a modified cyclomatic complexity of more than VAR_NUMBER. This makes the script difficult to understand and maintain.

### MACYCCOM

Severity: **warning** · Auto-fix: **no**

Anonymous function has a modified cyclomatic complexity of more than VAR_NUMBER. This makes the anonymous function difficult to understand and maintain.

## Examples

### Incorrect

```matlab
function f(a, b, c, d, e, f, g, h, i)  % FCNIL — too many inputs
    x = 1; y = 2; z = 3;               % DAFSC — too many semicolons on one line
end
```

### Correct

```matlab
function f(a, b, c)  % within the default input limit
    x = 1;
end
```

## Configuration

```toml
[lint.rules.CUSTOM_CHECKS]
severity = "warn"
max_function_inputs = 7
max_function_outputs = 7
max_function_lines = 200
max_line_length = 120
max_nesting_depth = 5
max_cyclomatic_complexity = 15
max_strict_cyclomatic_complexity = 20
max_avg_cyclomatic_complexity = 10
max_branches = 10
max_return_points = 5
max_nested_functions = 3
max_anonymous_functions = 5
max_local_variables = 20
max_local_constants = 10
max_called_functions = 30
max_semicolons_per_line = 3
max_tree_children = 50
max_persistent_variables = 10
max_conditions = 5
max_input_args_used = 10
max_output_args_used = 10
```

See [Configuration](../configuration.md) for the full parameter list and [rules.md](../rules.md) for the complete rule inventory.
