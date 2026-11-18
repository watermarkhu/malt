---
icon: lucide/ban
---

# Unsupported Feature Checks

**Default severity:** Warning
**Auto-fix:** No
**Category:** Unsupported
**Can be disabled:** Yes
**Enabled by default:** No (opt in per rule or per category)
**Maturity:** Experimental

## What this rule does

Detects usage of unsupported, deprecated, or platform-specific features
that are no longer available in modern MATLAB or restricted to specific
configurations. All 13 checks share a single `UnsupportedEngine` that
dispatches node-level checks on `function_call`, `command`, `attribute`
and `field_expression` nodes; each diagnostic carries the specific check ID
(e.g. `MCADE`, `FEATUD`). Function checks fire on both call syntax
(`feature('x')`) and command syntax (`feature accel on`); `IMPKG` and
`MIPKG` fire on references and imports of `internal.matlab.*` and
`matlab.<...>.internal.*` respectively; `MCADE` fires on a `Description`
class-member attribute.

## Check IDs

### MCADE

Severity: **warning** · Auto-fix: **no** · Maturity: **experimental**

Using Description as an attribute is unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this.

### AWTIUD

Severity: **warning** · Auto-fix: **no** · Maturity: **experimental**

'awtinvoke' is unsupported and might have been changed without notice or might be removed without notice. With appropriate code changes, use javaMethodEDT instead.

### AXCHUD

Severity: **warning** · Auto-fix: **no** · Maturity: **experimental**

'axescheck' is unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this.

### FEATUD

Severity: **warning** · Auto-fix: **no** · Maturity: **experimental**

'feature' and flags passed to it are unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this.

### FNDPUD

Severity: **warning** · Auto-fix: **no** · Maturity: **experimental**

'findpackage' is unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this.

### HGCNUD

Severity: **warning** · Auto-fix: **no** · Maturity: **experimental**

'hgconvertunits' is unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this.

### IMPKG

Severity: **warning** · Auto-fix: **no** · Maturity: **experimental**

Functions in internal.matlab namespace are unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this.

### ISMBUD

Severity: **warning** · Auto-fix: **no** · Maturity: **experimental**

'ismembc' is unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this.

### MIPKG

Severity: **warning** · Auto-fix: **no** · Maturity: **experimental**

Functions in MATLAB's internal namespaces are unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this.

### SEPTUD

Severity: **warning** · Auto-fix: **no** · Maturity: **experimental**

'setptr' is unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this.

### SYDEUD

Severity: **warning** · Auto-fix: **no** · Maturity: **experimental**

'system_dependent' is unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this.

### UIRSUD

Severity: **warning** · Auto-fix: **no** · Maturity: **experimental**

'uirestore' is unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this.

### UISUUD

Severity: **warning** · Auto-fix: **no** · Maturity: **experimental**

'uisuspend' is unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this.

## Maturity

Experimental checks are off by default and run with `[lint] experimental = true`, the `all` preset, or an explicit rule setting (see [Experimental checks](../configuration.md#experimental-checks)).

13 checks (marked in the table above): Unsupported-feature checks (including fixed-point): never emitted by checkcode in the probes; verified against the documentation index only. The engine is off by default.

## Examples

### Incorrect

```matlab
v = feature('version');       % FEATUD
setptr(gcf, 'hand');          % SEPTUD
x = ismembc(a, b);            % ISMBUD
import internal.matlab.*      % IMPKG
y = matlab.internal.foo.bar;  % MIPKG
```

### Correct

```matlab
tf = ismember(a, b);
v = version;
```

## Configuration

```toml
[lint.rules.UNSUPPORTED_ENGINE]
severity = "warning"
skip_checks = ["IMPKG"]
```

See [Configuration](../configuration.md) for the full parameter list and [rules.md](../rules.md) for the complete rule inventory.
