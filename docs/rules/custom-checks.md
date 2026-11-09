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

Configurable code-complexity and style checks from MATLAB's Code Analyzer
"Custom Checks" group, handled by a single file-level engine
(`CustomChecksEngine`).

* Metric checks (`FCNIL`, `FCNOL`, `FCNLL`, `LLMNC`, `MNCSN`, `CYCCOM`,
  `MCYCCOM`, `SCYCCOM`, `MSCYCCOM`, `ACYCCOM`, `MACYCCOM`) have a
  configurable limit; a limit of `0` disables the check.
* The `DAF*` checks and `SYSBANG` flag every use of a construct that the
  configuration disallows. They are controlled by boolean `disallow_*`
  options. As in MATLAB, all `DAF*` checks are off by default; `SYSBANG`
  is on by default.

`LLMCC` (line length excluding comments) is not implemented.

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
    try                                % DAFTC (with disallow_try_catch = true)
        x = 1;
    catch
    end
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
max_function_inputs = 7                 # FCNIL
max_function_outputs = 7                # FCNOL
max_function_lines = 200                # FCNLL
max_line_length = 120                   # LLMNC
max_nesting_depth = 5                   # MNCSN
max_cyclomatic_complexity = 15          # CYCCOM, MCYCCOM (functions)
max_script_cyclomatic_complexity = 20   # SCYCCOM, MSCYCCOM (scripts)
max_anonymous_cyclomatic_complexity = 10 # ACYCCOM, MACYCCOM
disallow_bang = true                    # SYSBANG
disallow_try_catch = false              # DAFTC
disallow_persistent = false             # DAFPV
disallow_continue = false               # DAFCO
disallow_break = false                  # DAFBR
disallow_return = false                 # DAFRT
disallow_script = false                 # DAFSC
disallow_nested_functions = false       # DAFNF
disallow_command_syntax = false         # DAFCF
disallow_anonymous_functions = false    # DAFAF
disallow_char_vectors = false           # DAFCV
disallow_cellstr = false                # DAFCVC
disallow_varargin = false               # DAFVI
disallow_varargout = false              # DAFVO
```

See [Configuration](../configuration.md) for the full parameter list and [rules.md](../rules.md) for the complete rule inventory.
