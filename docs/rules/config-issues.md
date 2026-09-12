---
icon: lucide/cog
---

# Configuration Issues

**Default severity:** Error
**Auto-fix:** No
**Category:** Configuration Issues
**Can be disabled:** Yes

## What this rule does

Detects MATLAB-side configuration issues such as invalid parameter names
passed to configuration functions, wrong argument counts, and invalid
option values. The engine matches `function_call` and `command` nodes
against known configuration function patterns and emits a diagnostic with
the specific check ID (`BDCFG`, `CFERR`, `BDOPT`, `CFIG`).

## Check IDs

### BDCFG

Severity: **error** · Auto-fix: **no**

Code Analyzer configuration file is invalid. Factory configuration is used instead. Run matlab.codeanalysis.validateConfiguration(VAR_NAME) to identify specific issues.

### CFERR

Severity: **error** · Auto-fix: **no**

Cannot open or read the Code Analyzer settings from file VAR_FILE. Using default settings instead.

### BDOPT

Severity: **error** · Auto-fix: **no**

Option VAR_NAME is ignored because it is invalid.

### CFIG

Severity: **error** · Auto-fix: **no**

The Code Analyzer settings file, VAR_FILE, has an error on line VAR_NUMBER.

## Examples

### Incorrect

```matlab
set_param(gcs, 'NotARealParam', 'value');
cfg = coder.config('something');
opts = optimset('NotAnOption', 1);
```

### Correct

```matlab
set_param(gcs, 'SimulationCommand', 'start');
cfg = coder.config('lib');
opts = optimset('Display', 'off');
```

## Configuration

```toml
[lint.rules.CONFIG_ISSUES_ENGINE]
disabled_checks = []
```

See [Configuration](../configuration.md) for the full parameter list and [rules.md](../rules.md) for the complete rule inventory.
