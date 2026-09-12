---
icon: lucide/ban
---

# Unsupported Feature Checks

**Default severity:** Warning
**Auto-fix:** No
**Category:** Unsupported
**Can be disabled:** Yes

## What this rule does

Detects usage of unsupported, deprecated, or platform-specific features
that are no longer available in modern MATLAB or restricted to specific
configurations. All 13 checks share a single `UnsupportedEngine` that
dispatches node-level checks on `function_call` and `command` nodes; each
diagnostic carries the specific check ID (e.g. `MCADE`, `FEATUD`).

## Check IDs

### MCADE

Severity: **warning** · Auto-fix: **no**

Using Description as an attribute is unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this.

### AWTIUD

Severity: **warning** · Auto-fix: **no**

'awtinvoke' is unsupported and might have been changed without notice or might be removed without notice. With appropriate code changes, use javaMethodEDT instead.

### AXCHUD

Severity: **warning** · Auto-fix: **no**

'axescheck' is unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this.

### FEATUD

Severity: **warning** · Auto-fix: **no**

'feature' and flags passed to it are unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this.

### FNDPUD

Severity: **warning** · Auto-fix: **no**

'findpackage' is unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this.

### HGCNUD

Severity: **warning** · Auto-fix: **no**

'hgconvertunits' is unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this.

### IMPKG

Severity: **warning** · Auto-fix: **no**

Functions in internal.matlab namespace are unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this.

### ISMBUD

Severity: **warning** · Auto-fix: **no**

'ismembc' is unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this.

### MIPKG

Severity: **warning** · Auto-fix: **no**

Functions in MATLAB's internal namespaces are unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this.

### SEPTUD

Severity: **warning** · Auto-fix: **no**

'setptr' is unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this.

### SYDEUD

Severity: **warning** · Auto-fix: **no**

'system_dependent' is unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this.

### UIRSUD

Severity: **warning** · Auto-fix: **no**

'uirestore' is unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this.

### UISUUD

Severity: **warning** · Auto-fix: **no**

'uisuspend' is unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this.

## Examples

### Incorrect

```matlab
deploytool();           % MCADE: ADE function
s = serial('COM1');     % SEPTUD: use serialport instead
v = feature('version'); % FEATUD: undocumented internal function
import pkg.sub.*;       % IMPKG: unsupported import context
```

### Correct

```matlab
sp = serialport('COM1', 9600);
v = version;
```

## Configuration

```toml
[lint.rules.UNSUPPORTED_ENGINE]
severity = "warning"
skip_checks = ["IMPKG"]
```

See [Configuration](../configuration.md) for the full parameter list and [rules.md](../rules.md) for the complete rule inventory.
