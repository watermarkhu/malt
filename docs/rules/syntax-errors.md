---
icon: lucide/x-circle
---

# Syntax Errors

**Default severity:** Error
**Auto-fix:** Yes
**Category:** Syntax Errors
**Can be disabled:** Yes
**Enabled by default:** Yes
**Maturity:** Stable

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

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

Parse error at VAR_RESERVED_WORD: usage might be invalid MATLAB syntax.

### BDFIL

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

Invalid MATLAB file name. MATLAB file names must start with a letter, contain only letters, numbers, or underscores, and have no more than VAR_NUMBER characters.

### BADNE

Severity: **error** · Auto-fix: **yes** · Maturity: **stable**

'Not Equals' is spelled ~= in MATLAB, not !=.

### BADOT

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

Use of two dots (..) is an invalid MATLAB construction.

### TWOCM

Severity: **error** · Auto-fix: **yes** · Maturity: **stable**

A comma cannot immediately follow another comma.

### CLIS

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

Defining a class in script is not allowed.

### CLTWO

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

Only one class definition is allowed per file, and it must come at the head of the file.

### SOFOC

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

Statement outside a class definition is not allowed.

### SEMFU

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

Script file must contain executable code. Remove empty statements to make this file a function file.

### FNDOT

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

Function name can only contain dots if it is a class method.

### FNSWA

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

Function name must start with alphabetic character.

### NOPAR2

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

A VAR_NAME might be missing a closing VAR_NAME, causing invalid syntax at VAR_NAME on line VAR_NUMBER.

### NOPAR

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

A VAR_NAME might be missing a closing VAR_NAME, causing invalid syntax at VAR_NAME.

### EOLPAR

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

A VAR_NAME might be missing a closing VAR_NAME, causing invalid syntax at end of line.

### ENDPAR

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

A VAR_NAME might be missing a closing VAR_NAME, causing invalid syntax at end of file.

### ENDCT

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

An END might be missing, possibly matching VAR_RESERVED_WORD.

### ENDCT2

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

An END might be missing (after line VAR_RESERVED_WORD), possibly matching VAR_NUMBER.

### ENDCT3

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

An END might be missing (before VAR_RESERVED_WORD on line VAR_NUMBER), possibly matching VAR_RESERVED_WORD.

### ENDCT4

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

A METHODS block or END might be missing before the function definition. This might be causing additional error messages.

### NOLHS

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

Left side of an assignment is empty.

### BADCH

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

Invalid text character(s).

### BADSP

Severity: **error** · Auto-fix: **yes** · Maturity: **stable**

Invalid text character(s). The text contains an unsupported non-ASCII whitespace character.

### BADCT

Severity: **error** · Auto-fix: **yes** · Maturity: **stable**

Unicode explicit directional formatting characters are not supported.

### REDEF

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

The current use of VAR_NAME is inconsistent with its previous use or definition (line VAR_NUMBER).

### SEPEXR

Severity: **error** · Auto-fix: **yes** · Maturity: **stable**

Use a newline, semicolon, or comma before this statement.

### SEPEXC

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

Use comma to separate between input arguments.

### SBTMP

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

Invalid array indexing or function call. Chaining outputs after parenthesis is not supported.

### FVSYN

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

Invalid function argument syntax at VAR_RESERVED_WORD.

### FVAPN

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

Move name-value arguments that use the name=value syntax to the end of the argument list.

### FVACI

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

Use of name-value arguments in cell indexing is not supported.

### FVACS

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

Using a character vector or string as a name in name=value syntax is not supported. Remove the quotes around the name.

### FVAMI

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

Name in name-value argument syntax must be a valid MATLAB identifier.

### UNSET

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

Invalid use of VAR_OPERATOR on the left side of an assignment.

### LHROW

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

The left side of an assignment cannot have multiple rows (';').

### RESWD

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

Invalid use of a reserved word.

### SYNEND

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

Invalid use for END operator.

### MCPLD

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

Invalid property syntax at VAR_RESERVED_WORD.

### VTPOD

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

Specify validation in the following order: size, then class, then functions.

### BADNOT

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

Using ~ to ignore a value is not permitted in this context.

### BADNOTLHS

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

Invalid use of logical not operator (~) on left side of an assignment. To use ~ to ignore function outputs, separate output variables with commas.

### STRIN

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

A quoted character vector is unterminated.

### DOUQT

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

A double quoted string is unterminated.

### INBLK

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

A block comment is unterminated at the end of the file.

### BADFP

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

Invalid floating-point constant.

### BADHBH

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

Invalid digit in hexadecimal literal. Supported hexadecimal digits are 0-9 and A-F. Supported type suffixes are u8, u16, u32, u64, and s8, s16, s32, s64.

### BADHBB

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

Invalid digit in binary literal. Supported binary digits are 0 and 1. Supported type suffixes are u8, u16, u32, u64, and s8, s16, s32, s64.

### BADHBHT

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

Hexadecimal literal has too many digits for specified type suffix.

### BADHBBT

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

Binary literal has too many digits for specified type suffix.

### HEXTOOLONG

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

Hexadecimal literal has too many digits.

### BINARYTOOLONG

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

Binary literal has too many digits.

## Automatic fixes

Five checks carry automatic fixes:

- `BADNE` rewrites `!=` to `~=`, the MATLAB not-equal operator.
- `SEPEXR` inserts a `;` between two statements that share a line with no
  separator.
- `TWOCM` deletes the second comma of `,,` (`[1,,2]` → `[1,2]`, as MATLAB's
  fix does). `[1,,2]` is a syntax error in MATLAB, so no running code
  changes meaning.
- `BADSP` replaces a non-ASCII blank in code (a no-break space, an em space)
  with an ASCII space. A line or paragraph separator gets no fix.
- `BADCT` deletes a directional formatting character from a comment. In a
  string it changes the value, so that fix is unsafe.

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
