---
icon: lucide/package
---

# MATLAB Compiler Deployment Constraint Checks

**Default severity:** Warning
**Auto-fix:** No
**Category:** Deployment
**Can be disabled:** Yes

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

Severity: **error** · Auto-fix: **no**

MCC use of the CD function is problematic.

### MCPRD

Severity: **error** · Auto-fix: **no**

MCC allows only one argument in the PRINTDLG function.

### MCHLP

Severity: **warning** · Auto-fix: **no**

MCC does not permit the HELP function.

### MCKBD

Severity: **warning** · Auto-fix: **no**

MCC does not permit the KEYBOARD function.

### MCSVP

Severity: **warning** · Auto-fix: **no**

MCC does not permit the SAVEPATH function.

### MCMLR

Severity: **warning** · Auto-fix: **no**

MCC use of the MATLABROOT function is problematic.

### MCABF

Severity: **error** · Auto-fix: **no**

MCC use of absolute file names is likely to fail.

### MCMFL

Severity: **warning** · Auto-fix: **no**

MCC allows writing .m files, but they cannot be executed by the deployed application.

### MCTBX

Severity: **warning** · Auto-fix: **no**

MCC use of toolbox folder file names is likely to fail.

### MCLL

Severity: **error** · Auto-fix: **no**

MCC does not allow C++ files to be read directly using LOADLIBRARY.

## Examples

### Incorrect

```matlab
cd /tmp;              % MCCD
addpath('/abs/path'); % MCABF
doc plot;             % MCHLP
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
