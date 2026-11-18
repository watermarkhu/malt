---
icon: lucide/package
---

# MATLAB Compiler Deployment Constraint Checks

**Default severity:** Warning
**Auto-fix:** No
**Category:** Deployment
**Can be disabled:** Yes
**Enabled by default:** No (opt in per rule or per category)
**Maturity:** Experimental

## What this rule does

Detects MATLAB constructs that are problematic in compiled, deployed
applications built with `mcc`. Deployed MATLAB applications run in a
restricted MCR environment where certain functions are unavailable or
behave differently. All 10 checks share a single `DeploymentEngine` that
dispatches node-level checks on `function_call` and `command` nodes; each
diagnostic carries the specific check ID (e.g. `MCCD`, `MCTBX`) and its
own severity.

## Check IDs

### MCCD

Severity: **warning** · Auto-fix: **no** · Maturity: **experimental**

MCC use of the CD function is problematic.

### MCPRD

Severity: **error** · Auto-fix: **no** · Maturity: **experimental**

MCC allows only one argument in the PRINTDLG function.

### MCHLP

Severity: **error** · Auto-fix: **no** · Maturity: **experimental**

MCC does not permit the HELP function.

### MCKBD

Severity: **error** · Auto-fix: **no** · Maturity: **experimental**

MCC does not permit the KEYBOARD function.

### MCSVP

Severity: **error** · Auto-fix: **no** · Maturity: **experimental**

MCC does not permit the SAVEPATH function.

### MCMLR

Severity: **warning** · Auto-fix: **no** · Maturity: **experimental**

MCC use of the MATLABROOT function is problematic.

### MCABF

Severity: **warning** · Auto-fix: **no** · Maturity: **experimental**

MCC use of absolute file names is likely to fail.

### MCMFL

Severity: **warning** · Auto-fix: **no** · Maturity: **experimental**

MCC allows writing .m files, but they cannot be executed by the deployed application.

### MCTBX

Severity: **warning** · Auto-fix: **no** · Maturity: **experimental**

MCC use of toolbox folder file names is likely to fail.

### MCLL

Severity: **warning** · Auto-fix: **no** · Maturity: **experimental**

MCC does not allow C++ files to be read directly using LOADLIBRARY.

## Maturity

Experimental checks are off by default and run with `[lint] experimental = true`, the `all` preset, or an explicit rule setting (see [Experimental checks](../configuration.md#experimental-checks)).

`MCCD`, `MCPRD`, `MCHLP`, `MCKBD`, `MCSVP`, `MCMLR`, `MCABF`, `MCMFL`, `MCTBX`, `MCLL`: MATLAB Compiler checks: never emitted by checkcode in the probes; verified against the documentation index only. The engine is off by default.

## Examples

### Incorrect

```matlab
cd /tmp;                  % MCCD
fopen('/abs/data.bin');   % MCABF
help plot;                % MCHLP
```

### Correct

```matlab
% Bundle resources with the deployed app and use relative paths.
result = processFile('data.bin');
```

## Configuration

```toml
[lint.rules.DEPLOYMENT_ENGINE]
severity = "warning"
skip_checks = ["MCTBX"]
```

See [Configuration](../configuration.md) for the full parameter list and [rules.md](../rules.md) for the complete rule inventory.
