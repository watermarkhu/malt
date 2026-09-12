---
icon: lucide/x-circle
---

# Syntax Errors

**Default severity:** Error
**Auto-fix:** Yes
**Category:** Syntax Errors
**Can be disabled:** Yes

## What this rule does

Detects syntax errors in MATLAB source files. The engine combines
tree-sitter `ERROR`/`MISSING` node analysis with semantic checks the parser
cannot catch on its own: invalid operators, malformed file and function
names, structural violations, unterminated strings and comments, and
invalid numeric literals.

A single file-level engine (`SyntaxErrorsEngine`) dispatches all 48 checks
in one pass over the parse tree, plus targeted source scans that skip over
comments and strings. Each diagnostic carries the specific check ID
(e.g. `SYNER`, `BADNE`, `SEPEXR`).

## Check IDs

### SYNER

Severity: **error** · Auto-fix: **no**

Parse error at VAR_RESERVED_WORD: usage might be invalid MATLAB syntax.

### BDFIL

Severity: **error** · Auto-fix: **no**

Invalid MATLAB file name. MATLAB file names must start with a letter, contain only letters, numbers, or underscores, and have no more than VAR_NUMBER characters.

### BADNE

Severity: **error** · Auto-fix: **yes**

'Not Equals' is spelled ~= in MATLAB, not !=.

### BADOT

Severity: **error** · Auto-fix: **no**

Use of two dots (..) is an invalid MATLAB construction.

### TWOCM

Severity: **error** · Auto-fix: **no**

A comma cannot immediately follow another comma.

### CLIS

Severity: **error** · Auto-fix: **no**

Defining a class in script is not allowed.

### CLTWO

Severity: **error** · Auto-fix: **no**

Only one class definition is allowed per file, and it must come at the head of the file.

### SOFOC

Severity: **error** · Auto-fix: **no**

Statement outside a class definition is not allowed.

### SEMFU

Severity: **error** · Auto-fix: **no**

Script file must contain executable code. Remove empty statements to make this file a function file.

### FNDOT

Severity: **error** · Auto-fix: **no**

Function name can only contain dots if it is a class method.

### FNSWA

Severity: **error** · Auto-fix: **no**

Function name must start with alphabetic character.

### NOPAR2

Severity: **error** · Auto-fix: **no**

A VAR_NAME might be missing a closing VAR_NAME, causing invalid syntax at VAR_NAME on line VAR_NUMBER.

### EOLPAR

Severity: **error** · Auto-fix: **no**

A VAR_NAME might be missing a closing VAR_NAME, causing invalid syntax at end of line.

### ENDPAR

Severity: **error** · Auto-fix: **no**

A VAR_NAME might be missing a closing VAR_NAME, causing invalid syntax at end of file.

### ENDCT

Severity: **error** · Auto-fix: **no**

An END might be missing, possibly matching VAR_RESERVED_WORD.

### ENDCT2

Severity: **error** · Auto-fix: **no**

An END might be missing (after line VAR_RESERVED_WORD), possibly matching VAR_NUMBER.

### ENDCT3

Severity: **error** · Auto-fix: **no**

An END might be missing (before VAR_RESERVED_WORD on line VAR_NUMBER), possibly matching VAR_RESERVED_WORD.

### ENDCT4

Severity: **error** · Auto-fix: **no**

A METHODS block or END might be missing before the function definition. This might be causing additional error messages.

### EOFMI

Severity: **error** · Auto-fix: **no**

Invalid syntax at end of file. File is incomplete.

### NOLHS

Severity: **error** · Auto-fix: **no**

Left side of an assignment is empty.

### BADCH

Severity: **error** · Auto-fix: **no**

Invalid text character(s).

### BADSP

Severity: **error** · Auto-fix: **no**

Invalid text character(s). The text contains an unsupported non-ASCII whitespace character.

### BADCT

Severity: **error** · Auto-fix: **no**

Unicode explicit directional formatting characters are not supported.

### REDEF

Severity: **error** · Auto-fix: **no**

The current use of VAR_NAME is inconsistent with its previous use or definition (line VAR_NUMBER).

### SEPEXR

Severity: **error** · Auto-fix: **yes**

Use a newline, semicolon, or comma before this statement.

### SBTMP

Severity: **error** · Auto-fix: **no**

Invalid array indexing or function call. Chaining outputs after parenthesis is not supported.

### FVSYN

Severity: **error** · Auto-fix: **no**

Invalid function argument syntax at VAR_RESERVED_WORD.

### FVACI

Severity: **error** · Auto-fix: **no**

Use of name-value arguments in cell indexing is not supported.

### FVACS

Severity: **error** · Auto-fix: **no**

Using a character vector or string as a name in name=value syntax is not supported. Remove the quotes around the name.

### FVAMI

Severity: **error** · Auto-fix: **no**

Name in name-value argument syntax must be a valid MATLAB identifier.

### UNSET

Severity: **error** · Auto-fix: **no**

Invalid use of VAR_OPERATOR on the left side of an assignment.

### LHROW

Severity: **error** · Auto-fix: **no**

The left side of an assignment cannot have multiple rows (';').

### RESWD

Severity: **error** · Auto-fix: **no**

Invalid use of a reserved word.

### SYNEND

Severity: **error** · Auto-fix: **no**

Invalid use for END operator.

### MCPLD

Severity: **error** · Auto-fix: **no**

Invalid property syntax at VAR_RESERVED_WORD.

### BADNOT

Severity: **error** · Auto-fix: **no**

Using ~ to ignore a value is not permitted in this context.

### BADNOTLHS

Severity: **error** · Auto-fix: **no**

Invalid use of logical not operator (~) on left side of an assignment. To use ~ to ignore function outputs, separate output variables with commas.

### STRIN

Severity: **error** · Auto-fix: **no**

A quoted character vector is unterminated.

### DOUQT

Severity: **error** · Auto-fix: **no**

A double quoted string is unterminated.

### INBLK

Severity: **error** · Auto-fix: **no**

A block comment is unterminated at the end of the file.

### BADFP

Severity: **error** · Auto-fix: **no**

Invalid floating-point constant.

### BADHBH

Severity: **error** · Auto-fix: **no**

Invalid digit in hexadecimal literal. Supported hexadecimal digits are 0-9 and A-F. Supported type suffixes are u8, u16, u32, u64, and s8, s16, s32, s64.

### BADHBB

Severity: **error** · Auto-fix: **no**

Invalid digit in binary literal. Supported binary digits are 0 and 1. Supported type suffixes are u8, u16, u32, u64, and s8, s16, s32, s64.

### BADHBHT

Severity: **error** · Auto-fix: **no**

Hexadecimal literal has too many digits for specified type suffix.

### BADHBBT

Severity: **error** · Auto-fix: **no**

Binary literal has too many digits for specified type suffix.

### HEXTOOLONG

Severity: **error** · Auto-fix: **no**

Hexadecimal literal has too many digits.

### BINARYTOOLONG

Severity: **error** · Auto-fix: **no**

Binary literal has too many digits.

### VTPOD

Severity: **error** · Auto-fix: **no**

Specify validation in the following order: size, then class, then functions.

## Automatic fixes

Two checks carry automatic fixes:

- `BADNE` rewrites `!=` to `~=`, the MATLAB not-equal operator.
- `SEPEXR` inserts a `;` between two statements that share a line with no
  separator.

## Examples

### Incorrect

```matlab
x = 1 != 2;      % BADNE: `!=` is not MATLAB
a = 1 b = 2;     % SEPEXR: statements on one line without separator
if x > 0
    y = 'unterminated;   % STRIN
```

### Correct

```matlab
x = 1 ~= 2;
a = 1; b = 2;
if x > 0
    y = 'terminated';
end
```

### Fixed

```diff
- x = 1 != 2;
+ x = 1 ~= 2;
- a = 1 b = 2;
+ a = 1; b = 2;
```

## Configuration

```toml
[lint.rules.SYNTAX_ERRORS_ENGINE]
disabled_checks = []
```

See [Configuration](../configuration.md) for the full parameter list and [rules.md](../rules.md) for the complete rule inventory.
