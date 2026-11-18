---
icon: lucide/check-circle
---

# Good Practices

**Default severity:** Warning
**Auto-fix:** Yes
**Category:** Good Practices
**Can be disabled:** Yes
**Enabled by default:** Yes
**Maturity:** Mixed (12 of 106 experimental)

## What this rule does

Encourages recommended coding practices for MATLAB code. All 106 checks are
handled by a single hybrid engine (`GoodPracticesEngine`). Node-level checks
cover simple pattern matches (error handling, `eval` usage, string
comparisons, parfor/spmd usage, redundant comparisons), while file-level
checks use metadata extraction and the symbol table for context-aware
analysis (last-statement detection, variable shadowing, class and property
validation). Each diagnostic carries the specific check ID (e.g. `TRYNC`,
`EVLCS`, `PFRNI`).

CHAIN: a chained comparison such as `a < b < c` is evaluated by MATLAB as
`(a < b) < c`, which compares a logical value with `c`; the check reports the
outermost comparison of a chain once.

The checks span error handling, string and comparison idioms, `eval`/dynamic
code, parfor/spmd parallel practices, structure and field access, OOP and
class properties, logical-usage patterns, shared variables, arity, and App
Designer methods.

## Check IDs

### TRYNC

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

TRY statement should have a CATCH statement to check for unexpected errors.

### CTCH

Severity: **warning** · Auto-fix: **no** · Maturity: **experimental**

Best practice is for CATCH to be followed by an identifier that gets the error information.

### WLAST

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

WARNING('') does not reset the warning state. Use LASTWARN('') instead.

### WNTAG

Severity: **warning** · Auto-fix: **no** · Maturity: **experimental**

The first argument of WARNING should be a message identifier. Using a message identifier allows users better control over the message.

### ERTAG

Severity: **warning** · Auto-fix: **no** · Maturity: **experimental**

The first argument of ERROR should be a message identifier.

### STCMP

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Use STRCMP instead of == or ~= to compare character vectors, or convert character vectors to string scalars for direct comparison.

### STCI

Severity: **warning** · Auto-fix: **yes** · Maturity: **stable**

Use STRCMPI(str1,str2) instead of using UPPER/LOWER in a call to STRCMP.

### STISA

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Consider using ISA instead of comparing the class name.

### STRNU

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

This variable, apparently a structure, is changed but the value might be unused.

### EVLCS

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

'eval' is inefficient and makes code less clear. Call the statement directly.

### EVLDOT

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

'eval' is inefficient and makes code less clear. Use dynamic field names to access structure fields or object properties instead.

### EVLEQ

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

'eval' is inefficient and makes code less clear. Assign to the variable directly.

### EVLSYS

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

'eval' is inefficient and makes code less clear. To make calls to the operating system use the system function instead.

### EVLDUAL

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

This use of 'eval' is unnecessary and can be removed. Call the evaluated function directly using parentheses. For example, use 'load(filename)' instead of 'eval(['load ' filename])'.

### EVLSEQVAR

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Using 'eval' to dynamically assign variables is not recommended.

### NOANS

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Using ANS as a variable is not recommended as ANS is frequently overwritten by MATLAB.

### LOAD

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

To avoid conflicts with functions on the path, specify variables to load from file.

### SEPEX

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Consider using newline, semicolon, or comma before this statement for readability.

### NBRAK1

Severity: **warning** · Auto-fix: **yes** · Maturity: **stable**

If you intend to specify expression precedence, use parentheses () instead of brackets [].

### LNGNM

Severity: **warning** · Auto-fix: **no** · Maturity: **experimental**

Names longer than VAR_NUMBER characters are not supported. This name has been truncated to VAR_NUMBER characters.

### CHAIN

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Expressions like a &lt;name&gt; b &lt;name&gt; c are interpreted as (a &lt;name&gt; b) &lt;name&gt; c. Typically, to test a &lt;name&gt; b &lt;name&gt; c mathematically, if all arguments are numeric scalars, use (a &lt;name&gt; b) && (b &lt;name&gt; c), otherwise use (a &lt;name&gt; b) & (b &lt;name&gt; c).

### DISPLAY

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Overloading DISPLAY is not recommended.

### FNDEF

Severity: **warning** · Auto-fix: **yes** · Maturity: **stable**

Function name VAR_NAME is known to MATLAB by its file name: VAR_FILE.

### NOIN

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Method VAR_NAME should either be a static method or have at least one input argument.

### VALST

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

VAR_NAME must be the last argument in the argument list.

### PROP

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

VAR_NAME is also the name of a property, which may be confusing. Use obj.PropertyName syntax to reference the property, or rename this variable to improve readability.

### CPROP

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

VAR_NAME is also the name of a property, which may be confusing. Use obj.PropertyName syntax to reference the property, or rename the property to improve readability.

### FVAL

Severity: **warning** · Auto-fix: **yes** · Maturity: **stable**

Calling functions using 'feval' is usually not necessary. Call the function directly instead.

### FNCOLND

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Consider explicitly defining the array, and then using the END operator to index into it.

### COMNC

Severity: **warning** · Auto-fix: **yes** · Maturity: **stable**

Comment with percent (%) following comma acts as a row separator. Replace the comma with a semicolon to make the row separation clearer. Alternatively, replace the percent (%) with an ellipsis (...) to add a comment inside a row.

### ITERS

Severity: **warning** · Auto-fix: **no** · Maturity: **experimental**

The Code Analyzer type analysis may be incorrect here.

### LOGPROD

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Using 'prod' on a logical expression is hard to understand and might be incorrect. Consider using 'all' instead.

### LOGMIN

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Using 'min' on a logical expression is hard to understand and might be incorrect. Consider using 'all' instead.

### LOGMAX

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Using 'max' on a logical expression is hard to understand and might be incorrect. Consider using 'any' instead.

### ELARLOG

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

The VAR_NAME operator in the expression VAR_NAME(A VAR_NAME B) is unexpected. Should this be VAR_NAME(A) VAR_NAME B?

### SHOCIRAA

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Using the VAR_NAME operator in the expression VAR_NAME(A VAR_NAME B) is probably unintended.

### FXSETL

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

&lt;name&gt; is also the loop variable for an outer loop. Rename this variable, so this loop does not affect the outer loop variable.

### ISEQ

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

The context surrounding this expression expects a scalar value. Consider using 'isequal', which returns a scalar value, instead of '==' or '~='.

### BDSCA1

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Unexpected use of non-scalar value in a context which expects a scalar value. Use 'all' or 'any' to convert this value into a scalar value.

### SUBSASGN

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Output of 'subsasgn' should be assigned to a variable.

### XPORD

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Use parentheses to clarify the evaluation order of chained exponents.

### UNRPWR

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Combining exponentiation operators with unary operators (such as logical negation) without parentheses will not be supported in a future release. Use parentheses to clarify the evaluation order.

### ADAPPREF

Severity: **warning** · Auto-fix: **no** · Maturity: **experimental**

Use app as the first argument for VAR_NAME.

### KEYBOARDFUN

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Consider removing 'keyboard' function once you have finished debugging. This function may have security implications.

### GVMIS

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Global variables are inefficient and make errors difficult to diagnose. Use a function with input variables instead.

### PFEVB

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Using EVALIN('base') or ASSIGNIN('base') inside a PARFOR loop refers to the worker machines' base workspaces.

### PFGP

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Avoid assigning to GLOBAL or PERSISTENT variable VAR_NAME inside a PARFOR loop.

### PFGV

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Avoid using GLOBAL variable VAR_NAME in a PARFOR loop.

### PFIIN

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

The input variable VAR_NAME should be initialized before the PARFOR loop.

### PFOUS

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

The output variable VAR_NAME might not be used after the PARFOR loop.

### PFRNI

Severity: **warning** · Auto-fix: **yes** · Maturity: **stable**

The parfor loop can only use a step size of 1 or -1.

### PFRIN

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

The reduction variable VAR_NAME might not be set before the PARFOR loop.

### PFRUS

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

The reduction variable VAR_NAME might not be used after the PARFOR loop.

### PFTUSW

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

The temporary variable VAR_NAME might be used after the PARFOR loop on line VAR_NUMBER. The value set on this line is not available after the loop.

### PFUIXW

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

The index variable VAR_NAME might be used after the PARFOR loop on line VAR_NUMBER. The value set on this line is not available after the loop.

### SPEVB

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Using EVALIN('base') or ASSIGNIN('base') inside an SPMD block refers to the worker machines' base workspaces.

### SPGV

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Using the GLOBAL or PERSISTENT variable VAR_NAME in an SPMD block might fail because it is accessed on a worker machine.

### DSPMDA

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Distributed array must be created outside of an SPMD block.

### DUALC

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Command might be prematurely ended by comma.

### RMFLD

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

RMFIELD output must be assigned back to the structure.

### RMWRN

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

The warning with tag VAR_NAME has been removed from MATLAB, so this statement has no effect.

### STFLD

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

SETFIELD output must be assigned back to the structure.

### STRSZ

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Use STRCMP to compare character vectors that can have different sizes.

### ATTF

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

The Code Analyzer is unable to determine if the expression assigned to the VAR_NAME attribute evaluates to true or false.

### ATTOF

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Setting the class attribute Abstract to false is not recommended.

### MCPO

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

VAR_NAME property has no effect in a value class.

### MCSAC

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

SetAccess cannot be set on Constant properties.

### MOBSRV

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Using SetObservable or GetObservable on a Constant property has no effect.

### MDEPIN

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Default values should not be assigned to dependent properties because dependent properties do not store the values.

### MCCPI

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Initialize the Constant property or make it an Abstract Constant property.

### MGMD

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

'get' method should be implemented for each dependent property that does not also have private 'GetAccess' attribute.

### MCCPE

Severity: **warning** · Auto-fix: **no** · Maturity: **experimental**

Attempting to call a property or event VAR_NAME as a function.

### MTHANS

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Using ANS as a method name is not recommended as ANS is frequently overwritten by MATLAB.

### MHERM

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Parenthesize the multiplication of VAR_NAME and its transpose to ensure the result is Hermitian.

### MNUML

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

To create a square matrix, use VAR_NAME(numel(...), numel(...)). Alternatively, use VAR_NAME(size(...)) to create an array with same size as input array.

### COMPNOP

Severity: **warning** · Auto-fix: **yes** · Maturity: **stable**

This logical comparison simplifies to VAR_NAME(...). Did you mean to use VAR_NAME to evaluate function argument: VAR_NAME(...VAR_NAME...)?

### COMPNOT

Severity: **warning** · Auto-fix: **yes** · Maturity: **stable**

This logical comparison simplifies to ~VAR_NAME(...). Did you mean to use VAR_NAME to evaluate function argument: VAR_NAME(...VAR_NAME...)?

### M3COL

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Using three colons (a:b:c:d) in an expression is probably unintended.

### BDLGI

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Variable might be set by a nonlogical operator.

### BDLOG1

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

A scalar logical value is expected in the conditional expression. Use 'any' or 'all' to reduce the array to a logical scalar.

### BDLOG2

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

A scalar logical value is expected in the conditional expression. Use 'any' or 'all' to reduce the array to a logical scalar, or compare the scalar value to 0.

### BDSCI

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Variable might be set by a nonscalar operator.

### MCHDP

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

A property default value that is a handle will cause all instances to share the same object data. To avoid sharing, create the property value in the constructor. For intentional sharing, consider using a Constant property.

### MCHDT

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Declaring the value of a property as a handle might cause all instances to share the same default handle. To avoid sharing, create the handle for this property in the constructor. To express that sharing is intentional, use the Constant property attribute.

### SHVAU

Severity: **warning** · Auto-fix: **no** · Maturity: **experimental**

Confusing usage of name VAR_NAME on lines VAR_NUMBER and VAR_NUMBER. Initialize VAR_NAME before line VAR_NUMBER to make it a shared variable or rename VAR_NAME on line VAR_NUMBER to disambiguate.

### GTARG

Severity: **warning** · Auto-fix: **no** · Maturity: **experimental**

Function might be called with too many arguments.

### LTARG

Severity: **warning** · Auto-fix: **no** · Maturity: **experimental**

Function might be called with too few arguments.

### CTPCT

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

The format might not agree with the argument count.

### FXSETA

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Assigning a value to a loop variable within the loop statements is confusing and may not have an effect. The value of &lt;name&gt; will be reset on the next iteration of the loop.

### SIMPT

Severity: **warning** · Auto-fix: **no** · Maturity: **experimental**

This import statement runs before any other code in function VAR_NAME. Consider placing it at the top of the function body.

### TLEV

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

VAR_NAME could be very inefficient unless it is a top-level statement in its function.

### UNONC

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Assign the onCleanup output argument to a variable. Do not use the tilde operator (~) in place of a variable.

### MIPC1

Severity: **warning** · Auto-fix: **no** · Maturity: **experimental**

Calling the computer function with 'arch' returns 'win64', 'glnxa64', or 'maca64'.

### SUBSINDEX

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Do not overload 'subsindex' for fundamental data types.

### VTFIN

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

VAR_NAME should be the first input argument to the VAR_NAME function.

### CTOINW

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Use of constructed object as input to constructor is not necessary.

### FXUP

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Outer loop variable VAR_NAME is set inside a nested function.

### ADMTHDINV

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Use VAR_NAME(app, ...) to call this function.

### ADPROP

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

VAR_NAME is also the name of a property, which may be confusing. Use app.PropertyName syntax to reference the property, or change one of the names to improve readability.

### MCNPN

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

VAR_NAME is referenced but is not a property, method, or event name defined in this class.

### MCNPR

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

VAR_NAME is not a property, but is the target of an assignment.

### MCSNOV

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Set function in value class must return the modified object.

### MCSOH

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Set function in handle class does not need to return the modified object.

### MCVM

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Value class method that modifies the object must return the modified object.

### MCCSPS

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

Constant property VAR_NAME is not modified. 'VAR_NAME.VAR_NAME' creates a struct named VAR_NAME with a field named VAR_NAME.

### MCSUP

Severity: **warning** · Auto-fix: **no** · Maturity: **stable**

The set method for the property VAR_NAME should not access another property (VAR_NAME).

## Maturity

Experimental checks are off by default and run with `[lint] experimental = true`, the `all` preset, or an explicit rule setting (see [Experimental checks](../configuration.md#experimental-checks)).

`CTCH`, `WNTAG`, `ERTAG`, `ITERS`, `ADAPPREF`, `MCCPE`, `GTARG`, `LTARG`, `SIMPT`, `MIPC1`: On the mathworks off-list: MATLAB's default analysis reported none of them in 4 or more probe shapes each, so their on-side (the --preset all behaviour) is not compared with MATLAB.

`LNGNM`, `SHVAU`: MATLAB never reported them in 11 to 35 probes each; the check is gated off in the engine (not reproduced), the code is kept.

## Automatic fixes

Rewrites the flagged construct into the cleaner equivalent:

- the function declaration takes the file name (`function r = other(x)`
  in `mine.m` → `function r = mine(x)`), as MATLAB's auto fix does; the
  runtime resolves the function by its file name, so the declared name
  is dead text and the rewrite cannot change behaviour (FNDEF).
- `call(...) == true` → `call(...)` (COMPNOP) and
  `call(...) ~= true` / `call(...) == false` → `~call(...)` (COMPNOT).
- `parfor i = 1:step:end` → `parfor i = 1:end` (PFRNI).
- `%comment` → `% comment` (COMNC).
- `(scalar)` → `scalar` (NBRAK1).
- `feval('f', x)` and `feval(@f, x)` → `f(x)` (FVAL), the text MATLAB's own fix
  writes (`@pkg.fn` and `"a.b"` become `pkg.fn(...)` and `a.b(...)`). The call
  is the same call, so the fix is safe, and it is withheld when the name (or
  its package) is a variable or a function of the file, where the direct call
  would index the variable instead (MATLAB's fix does not look: on a script
  with `psd = 5; feval('psd', 1)` it writes `psd(1)`). A variable or an
  anonymous function as the first argument gets no fix.
- `strcmp(lower(s), 'abc')` → `strcmpi(s, 'abc')` (STCI): unsafe, because
  `lower`/`upper` and the case-insensitive comparison differ on some
  non-ASCII text; when both arguments convert, only the first conversion
  goes (`strcmpi(a, lower(b))`), as MATLAB's fix writes it.

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
