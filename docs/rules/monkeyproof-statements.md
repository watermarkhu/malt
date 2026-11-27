---
icon: lucide/code
---

# MonkeyProof statements and expressions

**Default severity:** Info
**Auto-fix:** No
**Category:** MonkeyProof Coding Standard
**Can be disabled:** Yes
**Enabled by default:** Yes
**Maturity:** Stable

> *Not part of MATLAB's Code Analyzer. Off unless the monkeyproof preset or category is on.*

## What this rule does

The checks of the statements and expressions chapter of the
[MonkeyProof Coding Standard for MATLAB](https://doc.monkeyproofsolutions.nl/code-checker-for-matlab/monkeyproof-coding-standard-for-matlab/statements/statements.html) that can be checked automatically.

The checks are off by default. Switch them on with `preset = "monkeyproof"`, or with
`[lint.categories] monkeyproof = "warn"`, or one at a time under `[lint.rules]`.
Every check works on one file's syntax tree, so it reports what the file itself
shows and stays silent when it would have to guess. Where a check cannot see
everything, the limit is given here.

`MP_STAT_17` (all called functions are known to MATLAB) is not implemented: it needs
MATLAB's own function list, toolboxes and the user's path, which a single file does
not provide.

## Check IDs

### MP_STAT_1

Severity: **info** · Auto-fix: **no** · Maturity: **stable**

A constant (`MAX_SIZE = 100`, capitals, literal value) is defined after other code in a function body. Only the top level of the body is looked at.

### MP_STAT_2

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

A number literal in an expression that is not in the allowed list.

### MP_STAT_3

Severity: **info** · Auto-fix: **no** · Maturity: **stable**

A well-known function, or a function of the same file, is used without parentheses (`x = rand * 2`, `tic`).

### MP_STAT_4

Severity: **info** · Auto-fix: **no** · Maturity: **stable**

More than one statement starts on a line.

### MP_STAT_5

Severity: **info** · Auto-fix: **no** · Maturity: **stable**

An `if` has neither `elseif` nor `else`, and is not error or input handling.

### MP_STAT_6

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

A `switch` has no `otherwise`.

### MP_STAT_7

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

An arithmetic expression is an operand of a logical operator, or a logical expression an operand of an arithmetic one.

### MP_STAT_8

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Different logical operators are mixed without parentheses (`a && b \|\| c`).

### MP_STAT_9

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Different arithmetic operators are mixed without parentheses (`a * b + c`).

### MP_STAT_10

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

An `assert` call lacks an error identifier and a message as its second and third inputs.

### MP_STAT_11

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

A `global` declaration.

### MP_STAT_12

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

A condition of `if`, `elseif` or `while` that is always true or always false.

### MP_STAT_13

Severity: **info** · Auto-fix: **no** · Maturity: **stable**

A field is added to a struct after the struct was used. Only structs created by `struct(...)` in the same block are followed.

### MP_STAT_18

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

A `try` has no `catch`, an empty `catch`, or a `catch` that never raises an error.

### MP_STAT_19

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

`==` or `~=` with a fractional number, `pi` or the result of a floating-point function.

### MP_STAT_21

Severity: **info** · Auto-fix: **no** · Maturity: **stable**

A number written without a zero before the decimal point (`.5`).

### MP_STAT_22

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

The variable `ans` is read or written by name.

### MP_STAT_23

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

A call of a debugger function such as `keyboard` or `dbstop`.

### MP_STAT_24

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

An `if` with an `elseif` has no `else`.

### MP_STAT_25

Severity: **error** · Auto-fix: **no** · Maturity: **stable**

The constructor of an abstract class defined in the same file is called.

### MP_STAT_26

Severity: **info** · Auto-fix: **no** · Maturity: **stable**

A statement in command syntax with arguments (`hold on`).

### MP_STAT_27

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

A function that reads or writes another workspace (`evalin`, `assignin`, `who`, `clc`, ...).

## Examples

### Incorrect

```matlab
function out = scale(in)
out = in * 1.8 + 32;          % MP_STAT_2, MP_STAT_9
if out == .5, out = 0; end    % MP_STAT_19, MP_STAT_21, MP_STAT_4, MP_STAT_5
hold on                       % MP_STAT_26
end
```

### Correct

```matlab
function out = scale(in)
FACTOR = 1.8;
OFFSET = 32;
out = (in * FACTOR) + OFFSET;
if abs(out - 0.5) < 1e-9
    out = 0;
else
    % Keep the converted value.
end
hold("on")
end
```

## Configuration

Each parameter belongs to the table of its check. A list parameter replaces the
default list when it is set.

```toml
[lint.rules.MP_STAT_2]
allowed = [0, 1, 2]        # values that are not magic, by absolute value
check_indices = false      # also report numbers inside x(5) on an assigned name

[lint.rules.MP_STAT_3]
functions = ["rand", "randn", "tic", "toc", "clock", "now", "cputime", "date", "pwd",
             "computer", "version", "mfilename", "gcf", "gca", "figure", "clc", "clf",
             "drawnow", "beep", "pause"]
local_functions = true     # also functions defined in the same file

[lint.rules.MP_STAT_5]
exit_functions = ["error", "throw", "rethrow", "throwAsCaller"]
input_checks = ["isempty", "isfield", "isa", "ischar", "isnumeric"]  # shortened here

[lint.rules.MP_STAT_12]
allow_endless_loop = false # accept `while true`

[lint.rules.MP_STAT_18]
error_functions = ["error", "rethrow", "throw", "throwAsCaller"]

[lint.rules.MP_STAT_19]
functions = ["sqrt", "sin", "cos", "exp", "log", "mean"]  # shortened here

[lint.rules.MP_STAT_23]
functions = ["keyboard", "dbstop", "dbclear", "dbcont", "dbstep", "dbquit", "dbup",
             "dbdown", "dbstack", "dbstatus", "dbtype"]

[lint.rules.MP_STAT_26]
allow = ["hold", "format"] # command names to accept

[lint.rules.MP_STAT_27]
functions = ["evalin", "assignin", "inputname", "who", "whos", "evalc", "clc"]
special_cases = true       # exist(x, "var"), load without output, clear / clear all
```

See [Configuration](../configuration.md) for the full parameter list and [rules.md](../rules.md) for the complete rule inventory.
