---
icon: lucide/align-left
---

# MonkeyProof layout and comments

**Default severity:** Info
**Auto-fix:** No
**Category:** MonkeyProof Coding Standard
**Can be disabled:** Yes
**Enabled by default:** Yes
**Maturity:** Stable

> *Not part of MATLAB's Code Analyzer, and malt is not affiliated with MonkeyProof Solutions. Off unless the monkeyproof preset or category is on.*

## What this rule does

The checks of the layout and comments chapter of the
[MonkeyProof Coding Standard for MATLAB](https://doc.monkeyproofsolutions.nl/code-checker-for-matlab/monkeyproof-coding-standard-for-matlab/layout/layout.html) that can be checked automatically.
The standard is a style guide of its own and not part of MATLAB's Code Analyzer, so the
set is off by default. Switch it on for a whole project with `preset = "monkeyproof"`, or
on top of another preset with:

```toml
[lint.categories]
monkeyproof = "warn"
```

Each check can be tuned or turned off by its own ID (`[lint.rules] MP_LAYOUT_8 = "off"`).
Comments and strings are located with the syntax tree, so a `%` inside a string, a `'`
that is a transpose, block comments and `...` continuations are told apart correctly.

## Check IDs

### MP_LAYOUT_1

Severity: **info** · Auto-fix: **no** · Maturity: **stable**

Do not use `%{ %}` block comments.

### MP_LAYOUT_2

Severity: **info** · Auto-fix: **no** · Maturity: **stable**

Indent the body of every function, also at the root of a file.

### MP_LAYOUT_3

Severity: **info** · Auto-fix: **no** · Maturity: **stable**

Surround operators with spaces; no space after a unary minus; no spaces needed in `Name=Value`.

### MP_LAYOUT_4

Severity: **info** · Auto-fix: **no** · Maturity: **stable**

Follow commas, semicolons and `if`/`while`/`for`/`switch`/`case` keywords with a space.

### MP_LAYOUT_5

Severity: **info** · Auto-fix: **no** · Maturity: **stable**

Keep lines within `max_length` characters (default 120).

### MP_LAYOUT_6

Severity: **info** · Auto-fix: **no** · Maturity: **stable**

Indent each nesting level by `indent` spaces (default 4).

### MP_LAYOUT_7

Severity: **info** · Auto-fix: **no** · Maturity: **stable**

Comments must not carry `TODO`, `FIXME`, `HACK` or similar markers (parameter `words`).

### MP_LAYOUT_8

Severity: **info** · Auto-fix: **no** · Maturity: **stable**

Align the equals signs of consecutive assignments.

### MP_LAYOUT_9

Severity: **info** · Auto-fix: **no** · Maturity: **stable**

Separate the elements of a matrix or cell row with commas placed directly after the element.

### MP_LAYOUT_11

Severity: **info** · Auto-fix: **no** · Maturity: **stable**

Combine `properties` or `methods` blocks with equal attributes and leave out default attribute values.

### MP_LAYOUT_12

Severity: **info** · Auto-fix: **no** · Maturity: **stable**

No whitespace at the end of a line.

### MP_LAYOUT_13

Severity: **info** · Auto-fix: **no** · Maturity: **stable**

No tab characters.

### MP_LAYOUT_14

Severity: **info** · Auto-fix: **no** · Maturity: **stable**

Put all `properties` blocks before all `methods` blocks of a class.

## Examples

### Incorrect

```matlab
function total = addUp(values)
total=0;
for k=1:numel(values)
  total = total+values(k); % TODO: handle NaN
end
end
```

### Correct

```matlab
function total = addUp(values)
    total = 0;
    for k = 1 : numel(values)
        total = total + values(k);
    end
end
```

See [Configuration](../configuration.md) for the full parameter list and [rules.md](../rules.md) for the complete rule inventory.
