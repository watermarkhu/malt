---
icon: lucide/boxes
---

# System Object Validation Checks

**Default severity:** Warning
**Auto-fix:** No
**Category:** System Objects
**Can be disabled:** Yes
**Enabled by default:** No (opt in per rule or per category)
**Maturity:** Experimental

## What this rule does

Validates MATLAB System object usage. System objects (classes inheriting
from `matlab.System`) have specific lifecycle constraints that static
analysis can partially verify. Since full type information is not available
to a static linter, these checks use pattern-based heuristics. They detect
System object classes by their `matlab.System` superclass and inspect
their `properties` blocks and `stepImpl` / `resetImpl` methods.
All 9 checks share a single `SystemObjectsEngine` that performs
checks over `matlab.System` class definitions (file-level analysis); each
diagnostic carries the specific check ID (e.g. `SONUMIN`, `SOTUNPROP3`).

## Check IDs

### SONUMIN

Severity: **error** · Auto-fix: **no** · Maturity: **experimental**

If 'stepImpl' accepts variable number of inputs, then you must define a 'getNumInputsImpl' method.

### SONUMOUT

Severity: **error** · Auto-fix: **no** · Maturity: **experimental**

If 'stepImpl' returns variable number of outputs, then you must define a 'getNumOutputsImpl' method.

### SODEPPROP

Severity: **warning** · Auto-fix: **no** · Maturity: **experimental**

Dependent properties are not supported for MATLAB System blocks. VAR_NAME property is not included on System block.

### SOINITPROP

Severity: **warning** · Auto-fix: **no** · Maturity: **experimental**

Initialize DiscreteState property VAR_NAME within a 'resetImpl' method.

### SODFLTVAL

Severity: **error** · Auto-fix: **no** · Maturity: **experimental**

Invalid initialization of DiscreteState property VAR_NAME. Initialize property within a 'resetImpl' method.

### SORSRVDNM

Severity: **warning** · Auto-fix: **no** · Maturity: **experimental**

VAR_NAME property is a reserved name.

### SOTUNPROP1

Severity: **warning** · Auto-fix: **no** · Maturity: **experimental**

Logical attribute not supported for tunable properties on MATLAB System blocks. VAR_NAME property is made Nontunable on System block.

### SOTUNPROP3

Severity: **warning** · Auto-fix: **no** · Maturity: **experimental**

Tunable properties on MATLAB System blocks must be numeric. VAR_NAME property is made Nontunable on System block because it is a char.

### SOTUNPROP4

Severity: **warning** · Auto-fix: **no** · Maturity: **experimental**

Tunable properties on MATLAB System blocks must be numeric. VAR_NAME property is made Nontunable on System block because it is a string.

## Maturity

Experimental checks are off by default and run with `[lint] experimental = true`, the `all` preset, or an explicit rule setting (see [Experimental checks](../configuration.md#experimental-checks)).

`SONUMIN`, `SONUMOUT`, `SODEPPROP`, `SOINITPROP`, `SODFLTVAL`, `SORSRVDNM`, `SOTUNPROP1`, `SOTUNPROP3`, `SOTUNPROP4`: System object checks: silent in 13 probe shapes; verified against the documentation index only. The engine is off by default.

## Examples

### Incorrect

```matlab
classdef MySystem < matlab.System
    properties
        Flag logical = false    % SOTUNPROP1: logical tunable property
        Label char = 'x'         % SOTUNPROP3: char tunable property
    end
    properties (DiscreteState)
        Count = 0;              % SODFLTVAL: DiscreteState default value
    end
    methods (Access = protected)
        function y = stepImpl(obj, varargin)  % SONUMIN: no getNumInputsImpl
            y = 1;
        end
    end
end
```

### Correct

```matlab
classdef MySystem < matlab.System
    properties (Nontunable)
        Label char = 'x'
    end
    properties (DiscreteState)
        Count
    end
    methods (Access = protected)
        function y = stepImpl(obj, varargin)
            y = 1;
        end
        function n = getNumInputsImpl(obj)
            n = 2;
        end
        function resetImpl(obj)
            obj.Count = 0;
        end
    end
end
```

## Configuration

```toml
[lint.rules.SYSTEM_OBJECTS_ENGINE]
severity = "warning"
skip_checks = ["SONUMIN"]
```

See [Configuration](../configuration.md) for the full parameter list and [rules.md](../rules.md) for the complete rule inventory.
