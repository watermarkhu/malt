---
icon: lucide/type
---

# Naming Checks

**Default severity:** Info
**Auto-fix:** No
**Category:** Naming
**Can be disabled:** Yes
**Enabled by default:** No (off in the default `mathworks` preset)

## What this rule does

A single generic engine that implements all 81 naming-convention checks
from MATLAB's Code Analyzer. The check IDs are the Cartesian product of
9 entity types × 9 check types:

- **Entities**: class, function, localFunction, method, nestedFunction,
  property, event, enumeration, variable
- **Check types**: maxLength, minLength, regularExpression, requiredPrefix,
  disallowedPrefix, disallowedPhrase, requiredSuffix, disallowedSuffix,
  casing

Rule IDs follow the pattern `naming.<entity>.<checkType>` (e.g.
`naming.class.casing`). The `NamingEngine` registers once with inventory
as `"NAMING_ENGINE"` and uses `has_file_check() = true`: it walks the tree
once to extract all named entities, then applies every enabled check to
each matching entity, emitting diagnostics with the specific rule ID.

Only `maxLength` (default 63, MATLAB's `namelengthmax`) and `minLength`
(default 2) produce diagnostics without user configuration; the other
seven check types require configuration.

## Check IDs

### naming.class.maxLength

Severity: **info** · Auto-fix: **no**

Naming check for class maxLength

### naming.class.minLength

Severity: **info** · Auto-fix: **no**

Naming check for class minLength

### naming.class.regularExpression

Severity: **info** · Auto-fix: **no**

Naming check for class regularExpression

### naming.class.requiredPrefix

Severity: **info** · Auto-fix: **no**

Naming check for class requiredPrefix

### naming.class.disallowedPrefix

Severity: **info** · Auto-fix: **no**

Naming check for class disallowedPrefix

### naming.class.disallowedPhrase

Severity: **info** · Auto-fix: **no**

Naming check for class disallowedPhrase

### naming.class.requiredSuffix

Severity: **info** · Auto-fix: **no**

Naming check for class requiredSuffix

### naming.class.disallowedSuffix

Severity: **info** · Auto-fix: **no**

Naming check for class disallowedSuffix

### naming.class.casing

Severity: **info** · Auto-fix: **no**

Naming check for class casing

### naming.function.maxLength

Severity: **info** · Auto-fix: **no**

Naming check for function maxLength

### naming.function.minLength

Severity: **info** · Auto-fix: **no**

Naming check for function minLength

### naming.function.regularExpression

Severity: **info** · Auto-fix: **no**

Naming check for function regularExpression

### naming.function.requiredPrefix

Severity: **info** · Auto-fix: **no**

Naming check for function requiredPrefix

### naming.function.disallowedPrefix

Severity: **info** · Auto-fix: **no**

Naming check for function disallowedPrefix

### naming.function.disallowedPhrase

Severity: **info** · Auto-fix: **no**

Naming check for function disallowedPhrase

### naming.function.requiredSuffix

Severity: **info** · Auto-fix: **no**

Naming check for function requiredSuffix

### naming.function.disallowedSuffix

Severity: **info** · Auto-fix: **no**

Naming check for function disallowedSuffix

### naming.function.casing

Severity: **info** · Auto-fix: **no**

Naming check for function casing

### naming.localFunction.maxLength

Severity: **info** · Auto-fix: **no**

Naming check for localFunction maxLength

### naming.localFunction.minLength

Severity: **info** · Auto-fix: **no**

Naming check for localFunction minLength

### naming.localFunction.regularExpression

Severity: **info** · Auto-fix: **no**

Naming check for localFunction regularExpression

### naming.localFunction.requiredPrefix

Severity: **info** · Auto-fix: **no**

Naming check for localFunction requiredPrefix

### naming.localFunction.disallowedPrefix

Severity: **info** · Auto-fix: **no**

Naming check for localFunction disallowedPrefix

### naming.localFunction.disallowedPhrase

Severity: **info** · Auto-fix: **no**

Naming check for localFunction disallowedPhrase

### naming.localFunction.requiredSuffix

Severity: **info** · Auto-fix: **no**

Naming check for localFunction requiredSuffix

### naming.localFunction.disallowedSuffix

Severity: **info** · Auto-fix: **no**

Naming check for localFunction disallowedSuffix

### naming.localFunction.casing

Severity: **info** · Auto-fix: **no**

Naming check for localFunction casing

### naming.method.maxLength

Severity: **info** · Auto-fix: **no**

Naming check for method maxLength

### naming.method.minLength

Severity: **info** · Auto-fix: **no**

Naming check for method minLength

### naming.method.regularExpression

Severity: **info** · Auto-fix: **no**

Naming check for method regularExpression

### naming.method.requiredPrefix

Severity: **info** · Auto-fix: **no**

Naming check for method requiredPrefix

### naming.method.disallowedPrefix

Severity: **info** · Auto-fix: **no**

Naming check for method disallowedPrefix

### naming.method.disallowedPhrase

Severity: **info** · Auto-fix: **no**

Naming check for method disallowedPhrase

### naming.method.requiredSuffix

Severity: **info** · Auto-fix: **no**

Naming check for method requiredSuffix

### naming.method.disallowedSuffix

Severity: **info** · Auto-fix: **no**

Naming check for method disallowedSuffix

### naming.method.casing

Severity: **info** · Auto-fix: **no**

Naming check for method casing

### naming.nestedFunction.maxLength

Severity: **info** · Auto-fix: **no**

Naming check for nestedFunction maxLength

### naming.nestedFunction.minLength

Severity: **info** · Auto-fix: **no**

Naming check for nestedFunction minLength

### naming.nestedFunction.regularExpression

Severity: **info** · Auto-fix: **no**

Naming check for nestedFunction regularExpression

### naming.nestedFunction.requiredPrefix

Severity: **info** · Auto-fix: **no**

Naming check for nestedFunction requiredPrefix

### naming.nestedFunction.disallowedPrefix

Severity: **info** · Auto-fix: **no**

Naming check for nestedFunction disallowedPrefix

### naming.nestedFunction.disallowedPhrase

Severity: **info** · Auto-fix: **no**

Naming check for nestedFunction disallowedPhrase

### naming.nestedFunction.requiredSuffix

Severity: **info** · Auto-fix: **no**

Naming check for nestedFunction requiredSuffix

### naming.nestedFunction.disallowedSuffix

Severity: **info** · Auto-fix: **no**

Naming check for nestedFunction disallowedSuffix

### naming.nestedFunction.casing

Severity: **info** · Auto-fix: **no**

Naming check for nestedFunction casing

### naming.property.maxLength

Severity: **info** · Auto-fix: **no**

Naming check for property maxLength

### naming.property.minLength

Severity: **info** · Auto-fix: **no**

Naming check for property minLength

### naming.property.regularExpression

Severity: **info** · Auto-fix: **no**

Naming check for property regularExpression

### naming.property.requiredPrefix

Severity: **info** · Auto-fix: **no**

Naming check for property requiredPrefix

### naming.property.disallowedPrefix

Severity: **info** · Auto-fix: **no**

Naming check for property disallowedPrefix

### naming.property.disallowedPhrase

Severity: **info** · Auto-fix: **no**

Naming check for property disallowedPhrase

### naming.property.requiredSuffix

Severity: **info** · Auto-fix: **no**

Naming check for property requiredSuffix

### naming.property.disallowedSuffix

Severity: **info** · Auto-fix: **no**

Naming check for property disallowedSuffix

### naming.property.casing

Severity: **info** · Auto-fix: **no**

Naming check for property casing

### naming.event.maxLength

Severity: **info** · Auto-fix: **no**

Naming check for event maxLength

### naming.event.minLength

Severity: **info** · Auto-fix: **no**

Naming check for event minLength

### naming.event.regularExpression

Severity: **info** · Auto-fix: **no**

Naming check for event regularExpression

### naming.event.requiredPrefix

Severity: **info** · Auto-fix: **no**

Naming check for event requiredPrefix

### naming.event.disallowedPrefix

Severity: **info** · Auto-fix: **no**

Naming check for event disallowedPrefix

### naming.event.disallowedPhrase

Severity: **info** · Auto-fix: **no**

Naming check for event disallowedPhrase

### naming.event.requiredSuffix

Severity: **info** · Auto-fix: **no**

Naming check for event requiredSuffix

### naming.event.disallowedSuffix

Severity: **info** · Auto-fix: **no**

Naming check for event disallowedSuffix

### naming.event.casing

Severity: **info** · Auto-fix: **no**

Naming check for event casing

### naming.enumeration.maxLength

Severity: **info** · Auto-fix: **no**

Naming check for enumeration maxLength

### naming.enumeration.minLength

Severity: **info** · Auto-fix: **no**

Naming check for enumeration minLength

### naming.enumeration.regularExpression

Severity: **info** · Auto-fix: **no**

Naming check for enumeration regularExpression

### naming.enumeration.requiredPrefix

Severity: **info** · Auto-fix: **no**

Naming check for enumeration requiredPrefix

### naming.enumeration.disallowedPrefix

Severity: **info** · Auto-fix: **no**

Naming check for enumeration disallowedPrefix

### naming.enumeration.disallowedPhrase

Severity: **info** · Auto-fix: **no**

Naming check for enumeration disallowedPhrase

### naming.enumeration.requiredSuffix

Severity: **info** · Auto-fix: **no**

Naming check for enumeration requiredSuffix

### naming.enumeration.disallowedSuffix

Severity: **info** · Auto-fix: **no**

Naming check for enumeration disallowedSuffix

### naming.enumeration.casing

Severity: **info** · Auto-fix: **no**

Naming check for enumeration casing

### naming.variable.maxLength

Severity: **info** · Auto-fix: **no**

Naming check for variable maxLength

### naming.variable.minLength

Severity: **info** · Auto-fix: **no**

Naming check for variable minLength

### naming.variable.regularExpression

Severity: **info** · Auto-fix: **no**

Naming check for variable regularExpression

### naming.variable.requiredPrefix

Severity: **info** · Auto-fix: **no**

Naming check for variable requiredPrefix

### naming.variable.disallowedPrefix

Severity: **info** · Auto-fix: **no**

Naming check for variable disallowedPrefix

### naming.variable.disallowedPhrase

Severity: **info** · Auto-fix: **no**

Naming check for variable disallowedPhrase

### naming.variable.requiredSuffix

Severity: **info** · Auto-fix: **no**

Naming check for variable requiredSuffix

### naming.variable.disallowedSuffix

Severity: **info** · Auto-fix: **no**

Naming check for variable disallowedSuffix

### naming.variable.casing

Severity: **info** · Auto-fix: **no**

Naming check for variable casing

## Examples

### Incorrect

```matlab
myvar1 = 1;       % naming.variable.casing — style configured, e.g. camelCase
classdef myClass  % naming.class.casing — style configured, e.g. PascalCase
end
```

### Correct

```matlab
myVar1 = 1;
classdef MyClass
end
```

## Configuration

Each of the 81 checks can be configured individually through its rule ID:

```toml
[lint.rules."naming.class.casing"]
severity = "error"
style = "PascalCase"

[lint.rules."naming.function.maxLength"]
severity = "warn"
max = 32

[lint.rules."naming.variable.disallowedPrefix"]
prefix = "temp"
```

See [Configuration](../configuration.md) for the full parameter list and [rules.md](../rules.md) for the complete rule inventory.
