---
icon: lucide/code
---

# Extra Semicolon

**Default severity:** Info
**Auto-fix:** Yes
**Category:** Formatting
**Can be disabled:** Yes
**Enabled by default:** Yes

## What this rule does

Flags a semicolon that does nothing: one that follows another semicolon
(`x = 1;;`), or that ends a block statement or its header line (`end;`,
`if ready;`, `function f;`). A semicolon that is the only separator on its
line, as in `if ready; go(); end`, is not extra and is left alone.

A statement that is *missing* its semicolon is a different finding: see
NOPTS, NOPRT and PRTCAL.

## Automatic fixes

Deletes the extra semicolon (and the blanks in front of it).

## Examples

### Incorrect

```matlab
x = compute_value();;
if ready;
    go();
end;
```

### Correct

```matlab
x = compute_value();
if ready
    go();
end
```

### Fixed

```diff
- x = compute_value();;
+ x = compute_value();
- if ready;
+ if ready
- end;
+ end
```

See [Configuration](../configuration.md) for the full parameter list and [rules.md](../rules.md) for the complete rule inventory.
