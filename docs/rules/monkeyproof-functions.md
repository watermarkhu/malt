---
icon: lucide/boxes
---

# MonkeyProof higher level rules

**Default severity:** Info
**Auto-fix:** No
**Category:** MonkeyProof Coding Standard
**Can be disabled:** Yes
**Enabled by default:** Yes
**Maturity:** Stable

> *Not part of MATLAB's Code Analyzer. Off unless the monkeyproof preset or category is on.*

## What this rule does

The checks of the higher level rules chapter of the
[MonkeyProof Coding Standard for MATLAB](https://doc.monkeyproofsolutions.nl/code-checker-for-matlab/monkeyproof-coding-standard-for-matlab/higher-level-rules/higher-level-rules.html) that can be checked automatically.

The checks are off by default. Switch them on with `preset = "monkeyproof"`, or with
`[lint.categories] monkeyproof = "warn"`, or one by one under `[lint.rules]`.

Each check reads one file. Where a rule needs more than that, the check does the part the
file shows and says so below. FCNS-23 (properties re-implemented from a superclass) and
FCNS-25 (method access differing from the superclass) are not implemented: they need the
superclass, and the path index of malt only tells whether a class is a handle class.

## Check IDs

### MP_FCNS_1

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

A function has more code lines or a higher cyclomatic complexity than the limit for its kind (main, local, method, nested).

### MP_FCNS_2

Severity: **info** · Auto-fix: **no** · Maturity: **stable**

A function or method declaration has more than five inputs or more than five outputs.

### MP_FCNS_3

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

A constructor has no output or more than one.

### MP_FCNS_4

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

The file is a script.

### MP_FCNS_5

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

A `%#ok<*ID>` directive suppresses a Code Analyzer message in the whole file.

### MP_FCNS_6

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

A `get.<Property>` or `set.<Property>` method.

### MP_FCNS_7

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

A mex function in the same folder lacks a build for a platform.

### MP_FCNS_8

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

A block is nested deeper than five levels.

### MP_FCNS_13

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

A loop variable is already the iterator of an earlier loop in the function.

### MP_FCNS_16

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

A function is nested in another function.

### MP_FCNS_18

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

A method declared by signature has no implementation.

### MP_FCNS_19

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

The inputs or outputs of a declared method differ from its implementation.

### MP_FCNS_21

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

A loop variable is read after its loop.

### MP_FCNS_22

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

A function defined in the file is called with more inputs than it takes.

### MP_FCNS_24

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

A call to a Java function, a Java package or an import of Java classes.

## Examples

### Incorrect

```matlab
function total = sumAll(a, b, c, d, e, f)   % MP_FCNS_2: six inputs
    for i = 1:3
    end
    for i = 1:3                              % MP_FCNS_13: i is used again
    end
    total = i;                               % MP_FCNS_21: i read after the loop
end
```

### Correct

```matlab
function total = sumAll(values)
    total = 0;
    for iFirst = 1:3
        total = total + values(iFirst);
    end
    for iSecond = 1:3
        total = total + values(iSecond);
    end
end
```

See [Configuration](../configuration.md) for the full parameter list and [rules.md](../rules.md) for the complete rule inventory.
