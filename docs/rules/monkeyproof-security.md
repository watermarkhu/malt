---
icon: lucide/shield
---

# MonkeyProof security rules

**Default severity:** Info
**Auto-fix:** No
**Category:** MonkeyProof Coding Standard
**Can be disabled:** Yes
**Enabled by default:** Yes
**Maturity:** Stable

> *Not part of MATLAB's Code Analyzer, and malt is not affiliated with MonkeyProof Solutions. Off unless the monkeyproof preset or category is on.*

## What this rule does

The checks of the security chapter of the
[MonkeyProof Coding Standard for MATLAB](https://doc.monkeyproofsolutions.nl/code-checker-for-matlab/monkeyproof-coding-standard-for-matlab/security/security.html) that can be checked automatically.

## Check IDs

### MP_SECURITY_1

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Do not use `eval`; avoid `evalin`, `evalc` and `feval` with a literal name.

### MP_SECURITY_2

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Do not use the `!` shell escape.

### MP_SECURITY_3

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Do not set an empty `CertificateFilename` on `weboptions` or `HTTPOptions`.

### MP_SECURITY_4

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Set `AllowDoctype` to false when reading XML.

### MP_SECURITY_5

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Do not give variables such as `username` and `password` a constant value.

### MP_SECURITY_6

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Do not hard-code values for `Username` and `Password`.

## Examples

### Incorrect

```matlab
eval(cmd);
!backup.sh
opts = weboptions("CertificateFilename", "");
doc = xmlread(file);
password = "hunter2";
opts = weboptions("Username", "ann", "Password", "hunter2");
```

### Correct

```matlab
result = fcn(x);
system("backup.sh");
opts = weboptions("CertificateFilename", "default");
doc = xmlread(file, "AllowDoctype", false);
[user, pw] = askLogin();
opts = weboptions("Username", user, "Password", pw);
```

See [Configuration](../configuration.md) for the full parameter list and [rules.md](../rules.md) for the complete rule inventory.
