---
icon: lucide/check-circle
---

# Good Practices

**Default severity:** Warning
**Auto-fix:** Yes
**Category:** Good Practices
**Can be disabled:** Yes
**Enabled by default:** Yes

## What this rule does

Encourages recommended coding practices for MATLAB code. All 106 checks are
handled by a single hybrid engine (`GoodPracticesEngine`). Node-level checks
cover simple pattern matches (error handling, `eval` usage, string
comparisons, parfor/spmd usage, redundant comparisons), while file-level
checks use metadata extraction and the symbol table for context-aware
analysis (last-statement detection, variable shadowing, class and property
validation). Each diagnostic carries the specific check ID (e.g. `TRYNC`,
`EVLCS`, `PFRNI`).

The checks span error handling, string and comparison idioms, `eval`/dynamic
code, parfor/spmd parallel practices, structure and field access, OOP and
class properties, logical-usage patterns, shared variables, arity, and App
Designer methods.

## Check IDs

### TRYNC

Severity: **warning** · Auto-fix: **no**

TRY statement should have a CATCH statement to check for unexpected errors.

### CTCH

Severity: **warning** · Auto-fix: **no**

Best practice is for CATCH to be followed by an identifier that gets the error information.

### WLAST

Severity: **warning** · Auto-fix: **no**

WARNING('') does not reset the warning state. Use LASTWARN('') instead.

### WNTAG

Severity: **warning** · Auto-fix: **no**

The first argument of WARNING should be a message identifier. Using a message identifier allows users better control over the message.

### ERTAG

Severity: **warning** · Auto-fix: **no**

The first argument of ERROR should be a message identifier.

### MEXCEP

Severity: **warning** · Auto-fix: **no**

To report an MException as a warning, use a format specifier to ensure the message is printed correctly. For example, 'warning(E.identifier, "%s", E.message)'.

### STCMP

Severity: **warning** · Auto-fix: **no**

Use STRCMP instead of == or ~= to compare character vectors, or convert character vectors to string scalars for direct comparison.

### STCI

Severity: **warning** · Auto-fix: **no**

Use STRCMPI(str1,str2) instead of using UPPER/LOWER in a call to STRCMP.

### STISA

Severity: **warning** · Auto-fix: **no**

Consider using ISA instead of comparing the class name.

### STRNU

Severity: **warning** · Auto-fix: **no**

This variable, apparently a structure, is changed but the value might be unused.

### EVLCS

Severity: **warning** · Auto-fix: **no**

'eval' is inefficient and makes code less clear. Call the statement directly.

### EVLDOT

Severity: **warning** · Auto-fix: **no**

'eval' is inefficient and makes code less clear. Use dynamic field names to access structure fields or object properties instead.

### EVLEQ

Severity: **warning** · Auto-fix: **no**

'eval' is inefficient and makes code less clear. Assign to the variable directly.

### EVLSYS

Severity: **warning** · Auto-fix: **no**

'eval' is inefficient and makes code less clear. To make calls to the operating system use the system function instead.

### EVLDUAL

Severity: **warning** · Auto-fix: **no**

This use of 'eval' is unnecessary and can be removed. Call the evaluated function directly using parentheses. For example, use 'load(filename)' instead of 'eval(['load ' filename])'.

### EVLSEQVAR

Severity: **warning** · Auto-fix: **no**

Using 'eval' to dynamically assign variables is not recommended.

### NOANS

Severity: **warning** · Auto-fix: **no**

Using ANS as a variable is not recommended as ANS is frequently overwritten by MATLAB.

### LOAD

Severity: **warning** · Auto-fix: **no**

To avoid conflicts with functions on the path, specify variables to load from file.

### SEPEX

Severity: **warning** · Auto-fix: **no**

Consider using newline, semicolon, or comma before this statement for readability.

### NBRAK1

Severity: **warning** · Auto-fix: **yes**

If you intend to specify expression precedence, use parentheses () instead of brackets [].

### LNGNM

Severity: **warning** · Auto-fix: **no**

Names longer than VAR_NUMBER characters are not supported. This name has been truncated to VAR_NUMBER characters.

### CHAIN

Severity: **warning** · Auto-fix: **no**

Expressions like a < b < c are interpreted as (a < b) < c. Use (a < b) && (b < c) instead.

### DISPLAY

Severity: **warning** · Auto-fix: **no**

Overloading DISPLAY is not recommended.

### FNDEF

Severity: **warning** · Auto-fix: **no**

Function name VAR_NAME is known to MATLAB by its file name: VAR_FILE.

### NOIN

Severity: **warning** · Auto-fix: **no**

Method VAR_NAME should either be a static method or have at least one input argument.

### VALST

Severity: **warning** · Auto-fix: **no**

VAR_NAME must be the last argument in the argument list.

### PROP

Severity: **warning** · Auto-fix: **no**

VAR_NAME is also the name of a property, which may be confusing. Use obj.PropertyName syntax to reference the property, or rename this variable to improve readability.

### CPROP

Severity: **warning** · Auto-fix: **no**

VAR_NAME is also the name of a property, which may be confusing. Use obj.PropertyName syntax to reference the property, or rename the property to improve readability.

### FVAL

Severity: **warning** · Auto-fix: **no**

Calling functions using 'feval' is usually not necessary. Call the function directly instead.

### FNCOLND

Severity: **warning** · Auto-fix: **no**

Consider explicitly defining the array, and then using the END operator to index into it.

### COMNC

Severity: **warning** · Auto-fix: **yes**

Comment with percent (%) following comma acts as a row separator. Replace the comma with a semicolon to make the row separation clearer. Alternatively, replace the percent (%) with an ellipsis (...) to add a comment inside a row.

### ITERS

Severity: **warning** · Auto-fix: **no**

The Code Analyzer type analysis may be incorrect here.

### LOGPROD

Severity: **warning** · Auto-fix: **no**

Using 'prod' on a logical expression is hard to understand and might be incorrect. Consider using 'all' instead.

### LOGMIN

Severity: **warning** · Auto-fix: **no**

Using 'min' on a logical expression is hard to understand and might be incorrect. Consider using 'all' instead.

### LOGMAX

Severity: **warning** · Auto-fix: **no**

Using 'max' on a logical expression is hard to understand and might be incorrect. Consider using 'any' instead.

### ELARLOG

Severity: **warning** · Auto-fix: **no**

The VAR_NAME operator in the expression VAR_NAME(A VAR_NAME B) is unexpected. Should this be VAR_NAME(A) VAR_NAME B?

### SHOCIRAA

Severity: **warning** · Auto-fix: **no**

Using the VAR_NAME operator in the expression VAR_NAME(A VAR_NAME B) is probably unintended.

### UNRPWR

Severity: **warning** · Auto-fix: **no**

Consider using parentheses to explicitly specify operator precedence.

### ADAPPREF

Severity: **warning** · Auto-fix: **no**

Use app as the first argument for VAR_NAME.

### KEYBOARDFUN

Severity: **warning** · Auto-fix: **no**

Consider removing 'keyboard' function once you have finished debugging. This function may have security implications.

### GVMIS

Severity: **warning** · Auto-fix: **no**

Global variables are inefficient and make errors difficult to diagnose. Use a function with input variables instead.

### PFEVB

Severity: **warning** · Auto-fix: **no**

Using EVALIN('base') or ASSIGNIN('base') inside a PARFOR loop refers to the worker machines' base workspaces.

### PFGP

Severity: **warning** · Auto-fix: **no**

Avoid assigning to GLOBAL or PERSISTENT variable VAR_NAME inside a PARFOR loop.

### PFGV

Severity: **warning** · Auto-fix: **no**

Avoid using GLOBAL variable VAR_NAME in a PARFOR loop.

### PFIIN

Severity: **warning** · Auto-fix: **no**

The input variable VAR_NAME should be initialized before the PARFOR loop.

### PFOUS

Severity: **warning** · Auto-fix: **no**

The output variable VAR_NAME might not be used after the PARFOR loop.

### PFRNI

Severity: **warning** · Auto-fix: **yes**

The parfor loop can only use a step size of 1 or -1.

### PFRIN

Severity: **warning** · Auto-fix: **no**

The reduction variable VAR_NAME might not be set before the PARFOR loop.

### PFRUS

Severity: **warning** · Auto-fix: **no**

The reduction variable VAR_NAME might not be used after the PARFOR loop.

### PFTUSW

Severity: **warning** · Auto-fix: **no**

The temporary variable VAR_NAME might be used after the PARFOR loop on line VAR_NUMBER. The value set on this line is not available after the loop.

### PFUIXW

Severity: **warning** · Auto-fix: **no**

The index variable VAR_NAME might be used after the PARFOR loop on line VAR_NUMBER. The value set on this line is not available after the loop.

### SPEVB

Severity: **warning** · Auto-fix: **no**

Using EVALIN('base') or ASSIGNIN('base') inside an SPMD block refers to the worker machines' base workspaces.

### SPGV

Severity: **warning** · Auto-fix: **no**

Using the GLOBAL or PERSISTENT variable VAR_NAME in an SPMD block might fail because it is accessed on a worker machine.

### DSPMDA

Severity: **warning** · Auto-fix: **no**

Distributed array must be created outside of an SPMD block.

### DUALC

Severity: **warning** · Auto-fix: **no**

Command might be prematurely ended by comma.

### RMFLD

Severity: **warning** · Auto-fix: **no**

RMFIELD output must be assigned back to the structure.

### RMWRN

Severity: **warning** · Auto-fix: **no**

The warning with tag VAR_NAME has been removed from MATLAB, so this statement has no effect.

### STFLD

Severity: **warning** · Auto-fix: **no**

SETFIELD output must be assigned back to the structure.

### STRSZ

Severity: **warning** · Auto-fix: **no**

Use STRCMP to compare character vectors that can have different sizes.

### ATTF

Severity: **warning** · Auto-fix: **no**

The Code Analyzer is unable to determine if the expression assigned to the VAR_NAME attribute evaluates to true or false.

### ATTOF

Severity: **warning** · Auto-fix: **no**

Setting the class attribute Abstract to false is not recommended.

### MCPO

Severity: **warning** · Auto-fix: **no**

VAR_NAME property has no effect in a value class.

### MCSAC

Severity: **warning** · Auto-fix: **no**

SetAccess cannot be set on Constant properties.

### MOBSRV

Severity: **warning** · Auto-fix: **no**

Using SetObservable or GetObservable on a Constant property has no effect.

### MDEPIN

Severity: **warning** · Auto-fix: **no**

Default values should not be assigned to dependent properties because dependent properties do not store the values.

### MCCPI

Severity: **warning** · Auto-fix: **no**

Initialize the Constant property or make it an Abstract Constant property.

### MGMD

Severity: **warning** · Auto-fix: **no**

'get' method should be implemented for each dependent property that does not also have private 'GetAccess' attribute.

### MCCPE

Severity: **warning** · Auto-fix: **no**

Attempting to call a property or event VAR_NAME as a function.

### MTHANS

Severity: **warning** · Auto-fix: **no**

Using ANS as a method name is not recommended as ANS is frequently overwritten by MATLAB.

### MHERM

Severity: **warning** · Auto-fix: **no**

Parenthesize the multiplication of VAR_NAME and its transpose to ensure the result is Hermitian.

### MNUML

Severity: **warning** · Auto-fix: **no**

To create a square matrix, use VAR_NAME(numel(...), numel(...)). Alternatively, use VAR_NAME(size(...)) to create an array with same size as input array.

### COMPNOP

Severity: **warning** · Auto-fix: **yes**

This logical comparison simplifies to VAR_NAME(...). Did you mean to use VAR_NAME to evaluate function argument: VAR_NAME(...VAR_NAME...)?

### COMPNOT

Severity: **warning** · Auto-fix: **yes**

This logical comparison simplifies to ~VAR_NAME(...). Did you mean to use VAR_NAME to evaluate function argument: VAR_NAME(...VAR_NAME...)?

### M3COL

Severity: **warning** · Auto-fix: **no**

Using three colons (a:b:c:d) in an expression is probably unintended.

### BDLGI

Severity: **warning** · Auto-fix: **no**

Variable might be set by a nonlogical operator.

### BDLOG1

Severity: **warning** · Auto-fix: **no**

A scalar logical value is expected in the conditional expression. Use 'any' or 'all' to reduce the array to a logical scalar.

### BDLOG2

Severity: **warning** · Auto-fix: **no**

A scalar logical value is expected in the conditional expression. Use 'any' or 'all' to reduce the array to a logical scalar, or compare the scalar value to 0.

### BDSCI

Severity: **warning** · Auto-fix: **no**

Variable might be set by a nonscalar operator.

### MCHDP

Severity: **warning** · Auto-fix: **no**

A property default value that is a handle will cause all instances to share the same object data. To avoid sharing, create the property value in the constructor. For intentional sharing, consider using a Constant property.

### MCHDT

Severity: **warning** · Auto-fix: **no**

Declaring the value of a property as a handle might cause all instances to share the same default handle. To avoid sharing, create the handle for this property in the constructor. To express that sharing is intentional, use the Constant property attribute.

### SHVAU

Severity: **warning** · Auto-fix: **no**

Confusing usage of name VAR_NAME on lines VAR_NUMBER and VAR_NUMBER. Initialize VAR_NAME before line VAR_NUMBER to make it a shared variable or rename VAR_NAME on line VAR_NUMBER to disambiguate.

### GTARG

Severity: **warning** · Auto-fix: **no**

Function might be called with too many arguments.

### LTARG

Severity: **warning** · Auto-fix: **no**

Function might be called with too few arguments.

### CTPCT

Severity: **warning** · Auto-fix: **no**

The format might not agree with the argument count.

### FXSETA

Severity: **warning** · Auto-fix: **no**

Loop index variable is changed inside of a `for` loop

### SIMPT

Severity: **warning** · Auto-fix: **no**

This import statement runs before any other code in function VAR_NAME. Consider placing it at the top of the function body.

### TLEV

Severity: **warning** · Auto-fix: **no**

VAR_NAME could be very inefficient unless it is a top-level statement in its function.

### UNONC

Severity: **warning** · Auto-fix: **no**

Assign the onCleanup output argument to a variable. Do not use the tilde operator (~) in place of a variable.

### MIPC1

Severity: **warning** · Auto-fix: **no**

Calling the computer function with 'arch' returns 'win64', 'glnxa64', or 'maca64'.

### SUBSINDEX

Severity: **warning** · Auto-fix: **no**

Do not overload 'subsindex' for fundamental data types.

### VTFIN

Severity: **warning** · Auto-fix: **no**

VAR_NAME should be the first input argument to the VAR_NAME function.

### CTOINW

Severity: **warning** · Auto-fix: **no**

Use of constructed object as input to constructor is not necessary.

### FXUP

Severity: **warning** · Auto-fix: **no**

Outer loop variable VAR_NAME is set inside a nested function.

### ADMTHDINV

Severity: **warning** · Auto-fix: **no**

Use VAR_NAME(app, ...) to call this function.

### ADPROP

Severity: **warning** · Auto-fix: **no**

VAR_NAME is also the name of a property, which may be confusing. Use app.PropertyName syntax to reference the property, or change one of the names to improve readability.

### MCNPN

Severity: **warning** · Auto-fix: **no**

VAR_NAME is referenced but is not a property, method, or event name defined in this class.

### MCNPR

Severity: **warning** · Auto-fix: **no**

VAR_NAME is not a property, but is the target of an assignment.

### MCSNOV

Severity: **warning** · Auto-fix: **no**

Set function in value class must return the modified object.

### MCSOH

Severity: **warning** · Auto-fix: **no**

Set function in handle class does not need to return the modified object.

### MCVM

Severity: **warning** · Auto-fix: **no**

Value class method that modifies the object must return the modified object.

### MCCSPS

Severity: **warning** · Auto-fix: **no**

Constant property VAR_NAME is not modified. 'VAR_NAME.VAR_NAME' creates a struct named VAR_NAME with a field named VAR_NAME.

### MCSUP

Severity: **warning** · Auto-fix: **no**

The set method for the property VAR_NAME should not access another property (VAR_NAME).

## Automatic fixes

Rewrites the flagged construct into the cleaner equivalent:

- `call(...) == true` → `call(...)` (COMPNOP) and
  `call(...) ~= true` / `call(...) == false` → `~call(...)` (COMPNOT).
- `parfor i = 1:step:end` → `parfor i = 1:end` (PFRNI).
- `%comment` → `% comment` (COMNC).
- `(scalar)` → `scalar` (NBRAK1).

## Examples

### Incorrect

```matlab
if isa(x, 'double') == true      % COMPNOP
    disp('double');
end
parfor i = 1:2:10                % PFRNI
    y(i) = i;
end
%comment with no space           % COMNC
x = (5);                         % NBRAK1
```

### Correct

```matlab
if isa(x, 'double')              % COMPNOP fix applied
    disp('double');
end
parfor i = 1:10                  % PFRNI fix applied
    y(i) = i;
end
% comment with space             % COMNC fix applied
x = 5;                           % NBRAK1 fix applied
```

### Fixed

```diff
- if isa(x, 'double') == true
+ if isa(x, 'double')
- parfor i = 1:2:10
+ parfor i = 1:10
```

## Configuration

```toml
[lint.rules.GOOD_PRACTICES_ENGINE]
severity = "warning"
max_variable_name_length = 63
disabled_checks = []
```

See [Configuration](../configuration.md) for the full parameter list and [rules.md](../rules.md) for the complete rule inventory.
