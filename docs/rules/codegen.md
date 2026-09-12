---
icon: lucide/braces
---

# MATLAB Code Generation Constraint Checks

**Default severity:** Error
**Auto-fix:** No
**Category:** Code Generation
**Can be disabled:** Yes

## What this rule does

Detects MATLAB constructs that are unsupported or problematic for MATLAB
Coder / code generation. MATLAB code intended to be compiled to C/C++ has
many restrictions compared to general MATLAB code; this engine flags
variable-size data, growing arrays, unsupported functions, cell arrays,
try-catch, imports, nested functions, scripts, and similar constraints.
All 15 checks share a single `CodegenEngine` that dispatches node-level
checks on `function_call`, `command`, `try_statement`, `for_statement`,
`assignment`, `cell`, and `arguments_statement` nodes, plus file-level
checks for nested functions and script-mode files. Each diagnostic carries
the specific check ID (e.g. `EMFCN`, `EMSCR`).

## Check IDs

### EMVDF

Severity: **error** · Auto-fix: **no**

Code generation requires that all elements of a variable are defined before indexing into the variable.

### EMGRO

Severity: **error** · Auto-fix: **no**

For code generation, grow an array by using 'end + 1' indexing.

### EMFCN

Severity: **error** · Auto-fix: **no**

This function is not supported in code generation.

### EMCEL

Severity: **error** · Auto-fix: **no**

Fixed-point conversion does not support cell arrays.

### EMTC

Severity: **error** · Auto-fix: **no**

TRY/CATCH is unsupported for code generation.

### EMIMP

Severity: **error** · Auto-fix: **no**

Code generation does not support import statements.

### EMNST

Severity: **error** · Auto-fix: **no**

Fixed-point conversion does not support nested functions.

### EMSCR

Severity: **error** · Auto-fix: **no**

Code generation does not support scripts.

### EMPFR

Severity: **error** · Auto-fix: **no**

HDL code generation does not support parfor statements.

### EMRIFAV

Severity: **error** · Auto-fix: **no**

Code generation does not support repeating arguments with validation.

### EMLOAD

Severity: **error** · Auto-fix: **no**

The output of a call to LOAD is not assigned to a variable. For code generation, assign the output of LOAD to a variable without subscripting.

### EMS2N

Severity: **error** · Auto-fix: **no**

Code generation does not support 'str2num'. Use 'str2double' instead.

### PRMNOIN

Severity: **error** · Auto-fix: **no**

For code generation, specify a binaryOccupancyMap object in the constructor of the mobileRobotPRM object.

### LOOPPRAGMAWITHOUTFOR

Severity: **error** · Auto-fix: **no**

A coder.loop.Control transform must be immediately followed by a for loop.

### FPASE

Severity: **error** · Auto-fix: **no**

Direct assignment to a possible fixed-point type is not recommended. Use the subscripted assignment syntax 'var(:) =' instead.

## Examples

### Incorrect

```matlab
x = [];
for i = 1:10
    x(end+1) = i;   % EMGRO: growing array
end
y = {1, 2};         % EMCEL: cell array
z = str2num('1 2'); % EMS2N: use str2double
```

### Correct

```matlab
x = zeros(1, 10);
for i = 1:10
    x(i) = i;
end
y = [1, 2];
z = str2double('1 2');
```

## Configuration

```toml
[lint.rules.CODEGEN_ENGINE]
severity = "error"
skip_checks = ["EMSCR", "EMFCN"]
```

See [Configuration](../configuration.md) for the full parameter list and [rules.md](../rules.md) for the complete rule inventory.
