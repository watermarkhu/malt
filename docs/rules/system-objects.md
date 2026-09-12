---
icon: lucide/boxes
---

# System Object Validation Checks

**Default severity:** Warning
**Auto-fix:** No
**Category:** System Objects
**Can be disabled:** Yes

## What this rule does

Validates MATLAB System object usage. System objects (classes inheriting
from `matlab.System`) have specific lifecycle constraints that static
analysis can partially verify. Since full type information is not available
to a static linter, these checks use pattern-based heuristics — detecting
System object patterns by class inheritance, lifecycle method calls
(`step`, `setup`, `release`, `reset`), and property access conventions.
All 9 checks share a single `SystemObjectsEngine` that dispatches node-level
checks on `function_call` nodes plus file-level class analysis; each
diagnostic carries the specific check ID (e.g. `SONUMIN`, `SOTUNPROP3`).

## Check IDs

### SONUMIN

Severity: **error** · Auto-fix: **no**

If 'stepImpl' accepts variable number of inputs, then you must define a 'getNumInputsImpl' method.

### SONUMOUT

Severity: **error** · Auto-fix: **no**

If 'stepImpl' returns variable number of outputs, then you must define a 'getNumOutputsImpl' method.

### SODEPPROP

Severity: **warning** · Auto-fix: **no**

Dependent properties are not supported for MATLAB System blocks. VAR_NAME property is not included on System block.

### SOINITPROP

Severity: **warning** · Auto-fix: **no**

Initialize DiscreteState property VAR_NAME within a 'resetImpl' method.

### SODFLTVAL

Severity: **error** · Auto-fix: **no**

Invalid initialization of DiscreteState property VAR_NAME. Initialize property within a 'resetImpl' method.

### SORSRVDNM

Severity: **warning** · Auto-fix: **no**

VAR_NAME property is a reserved name.

### SOTUNPROP1

Severity: **warning** · Auto-fix: **no**

Logical attribute not supported for tunable properties on MATLAB System blocks. VAR_NAME property is made Nontunable on System block.

### SOTUNPROP3

Severity: **warning** · Auto-fix: **no**

Tunable properties on MATLAB System blocks must be numeric. VAR_NAME property is made Nontunable on System block because it is a char.

### SOTUNPROP4

Severity: **warning** · Auto-fix: **no**

Tunable properties on MATLAB System blocks must be numeric. VAR_NAME property is made Nontunable on System block because it is a string.

## Examples

### Incorrect

```matlab
classdef MySystem < matlab.System
    properties
        Gain = rand();      % SODFLTVAL: function-call default value
        Flag logical = false % SOTUNPROP1: logical tunable property
    end
    methods
        function step(obj)   % SORSRVDNM: reserved method name
        end
    end
end
step(); % SONUMIN: step() called without inputs
```

### Correct

```matlab
classdef MySystem < matlab.System
    properties
        Gain = 1;
        Flag = false;
    end
    methods
        function stepImpl(obj) % use the *Impl override, not 'step'
        end
    end
end
step(obj, input); % pass the object and the input signal
```

## Configuration

```toml
[lint.rules.SYSTEM_OBJECTS_ENGINE]
severity = "warning"
skip_checks = ["SONUMIN"]
```

See [Configuration](../configuration.md) for the full parameter list and [rules.md](../rules.md) for the complete rule inventory.
