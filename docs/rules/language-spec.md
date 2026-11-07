---
icon: lucide/book-marked
---

# Language Specification Errors

**Default severity:** Error
**Auto-fix:** No
**Category:** Language Specification
**Can be disabled:** Yes
**Enabled by default:** Yes

## What this rule does

Enforces MATLAB language-specification constraints from the Code Analyzer.
All 165 checks share a single file-level engine (`LanguageSpecEngine`) that
performs a full DFS traversal with context stacks for parfor/spmd/loop
detection, then runs class, arguments-block, global/persistent, and
script-level checks over extracted file metadata. Each diagnostic carries
the specific check ID (e.g. `PFPF`, `FVAPN`, `MCFIL`).

The checks are grouped into families:

- **PF\*** — parfor restrictions, banned functions, and sliced-variable /
  reduction classification.
- **SP\*** — spmd block restrictions (`SPNST`, `SPRET`, `SPGP`, and the
  transparency group `SPDEC`, `SPEVC`, `SPLD`, `SPNF`, `SPSV`, `SPWHOS`,
  `SPBFN`).
- **AT\*** — class, events, property, and method attribute validation.
- **CLS\* / NOPRV** — class-file structure checks.
- **VTP\*** — property validation function checks.
- **MC\*** — class/OOP rules: naming, set/get signatures, constructor and
  superclass calls, Constant/WeakHandle properties.
- **FV\*** — `arguments` block validation (plus `TINVALDIM`,
  `TTOOFEWDIMS`).
- **Miscellaneous** — `FCONV`, `FCONF`, `GPFST`, `GPNES`, `NPERS`,
  `ROWLN`, `FCNANS`, `CLANS`, `BRKFOR`, `CONTFOR`, `IDXCOLND`, `ERTXT`,
  `WTXT`, `MHERIT`, `NCHKOS`, `CTOINE`, `CTORO`, `USESWNS`.

## Check IDs

### PFPF

Severity: **error** · Auto-fix: **no**

parfor loops cannot be used inside other parfor loops.

### PFSPMD

Severity: **error** · Auto-fix: **no**

spmd statements cannot be used inside parfor loops.

### PFBRK

Severity: **error** · Auto-fix: **no**

break statements cannot be used inside a parfor loop.

### PFRTN

Severity: **error** · Auto-fix: **no**

return statements cannot be used inside a parfor loop.

### PFGLOB

Severity: **error** · Auto-fix: **no**

Global variable declarations are not supported in parfor loops.

### PFPERS

Severity: **error** · Auto-fix: **no**

Persistent variable declarations are not supported in parfor loops.

### PFFORA

Severity: **error** · Auto-fix: **no**

Assigning to for loop variables is not supported in parfor loops.

### PFXST

Severity: **error** · Auto-fix: **no**

Assigning to the parfor loop index variable is not supported in parfor loops.

### PFNF

Severity: **error** · Auto-fix: **no**

Nested functions cannot be called from within parfor loops.

### FPFORP

Severity: **error** · Auto-fix: **no**

'fprintf' is writing to a file that is opened with read permission only. Open a file using 'fopen(...,'W',...)' instead.

### FWFORP

Severity: **error** · Auto-fix: **no**

'fwrite' is writing to a file that is opened with read permission only. Open a file using 'fopen(...,'W',...)' instead.

### PFANON

Severity: **error** · Auto-fix: **no**

Using a sliced output variable in an anonymous function is not supported in parfor loops.

### PFANSLP

Severity: **error** · Auto-fix: **no**

'ans' is not supported as a parfor loop variable.

### PFANSNS

Severity: **error** · Auto-fix: **no**

'ans' is not supported as a for loop variable in parfor loops.

### PFCEL

Severity: **error** · Auto-fix: **no**

The function VAR_NAME does not support cell arrays (argument VAR_NUMBER).

### PFCTXT

Severity: **error** · Auto-fix: **no**

When indexing a sliced variable with a nested for loop variable, the sliced variable must be inside the for loop that defines the range of the for loop variable.

### PFEVC

Severity: **error** · Auto-fix: **no**

EVALIN('caller') and ASSIGNIN('caller') are invalid inside of a PARFOR loop.

### PFFRNG

Severity: **error** · Auto-fix: **no**

When indexing a sliced variable with a nested for loop variable, the range of the for loop variable must be a row vector of positive constant numbers or variables.

### PFFSUB

Severity: **error** · Auto-fix: **no**

Indexing a nested for loop variable is not supported in parfor loops.

### PFINCR

Severity: **error** · Auto-fix: **no**

Using different reduction functions with the same reduction variable is not supported in parfor loops.

### PFINPT

Severity: **error** · Auto-fix: **no**

'inputname' is not supported in parfor loops.

### PFLD

Severity: **error** · Auto-fix: **no**

'load' must assign to an output variable in parfor loops.

### PFMLTI

Severity: **error** · Auto-fix: **no**

When indexing a sliced variable with a nested for loop variable, the for loop variable must not be assigned other than by its for statement.

### PFNACK

Severity: **error** · Auto-fix: **no**

'narginchk' and 'nargoutchk' cannot be used in parfor loops.

### PFNAIO

Severity: **error** · Auto-fix: **no**

'nargin' and 'nargout' require a function argument in parfor loops.

### PFNAR

Severity: **error** · Auto-fix: **no**

Subtracting reduction variable VAR_NAME from expressions is not supported in parfor loops.

### PFRFH

Severity: **error** · Auto-fix: **no**

The parfor reduction function VAR_NAME must either be a function name or a broadcast variable.

### PFRNG

Severity: **error** · Auto-fix: **no**

The range of a PARFOR statement must be consecutive integers.

### PFSLO

Severity: **error** · Auto-fix: **no**

Variable VAR_NAME is indexed using the parfor loop variable, but it is not a valid sliced output variable.

### PFSLRD

Severity: **error** · Auto-fix: **no**

Parfor loop variable VAR_NAME is accessed with an invalid combination of sliced indexing expressions and non-indexed reads. It is not valid to access the whole value of a sliced output variable.

### PFSLW

Severity: **error** · Auto-fix: **no**

Parfor loop variable VAR_NAME has multiple sliced accesses, but they do not all have the same list of subscripts. Each access to a sliced variable must use precisely the same list of subscripts.

### PFSV

Severity: **error** · Auto-fix: **no**

SAVE cannot be called in a PARFOR loop without the '-fromstruct' option.

### PFUNK

Severity: **error** · Auto-fix: **no**

Unable to classify variable VAR_NAME in the body of the parfor loop.

### PFUTMP

Severity: **error** · Auto-fix: **no**

Temporary variable VAR_NAME must be set inside the parfor loop before it is used.

### PFUTVR

Severity: **error** · Auto-fix: **no**

Variable VAR_NAME may have been intended as a reduction variable, but is an uninitialized temporary.

### PFVARS

Severity: **error** · Auto-fix: **no**

Parfor loop contains too many variables.

### PFVSUB

Severity: **error** · Auto-fix: **no**

Indexing parfor loop variables is not supported in parfor loops.

### ATUNK

Severity: **error** · Auto-fix: **no**

Unknown attribute name.

### ATLAB

Severity: **error** · Auto-fix: **no**

Attribute 'Input' and 'Output' must not be assigned a value or negated.

### ATNAS

Severity: **error** · Auto-fix: **no**

Set attribute to a single meta-class object or a cell array of meta-class objects.

### ATNPI

Severity: **error** · Auto-fix: **no**

Set attribute to 'public', 'private', 'protected', 'immutable', or a cell array of meta-classes instead.

### ATNPP

Severity: **error** · Auto-fix: **no**

Set attribute to 'public', 'private', 'protected', or a cell array of meta-classes instead.

### ATPPI

Severity: **error** · Auto-fix: **no**

The attribute value is unexpected. Use 'public', 'private', 'protected', 'immutable', or a cell array of meta-classes instead.

### ATPPP

Severity: **error** · Auto-fix: **no**

The attribute value is unexpected. Use 'public', 'private', 'protected', or a cell array of meta-classes instead.

### ATAS

Severity: **error** · Auto-fix: **no**

The attribute value is unexpected. Use a single meta-class object or a cell array of meta-class objects.

### ATVIZE

Severity: **error** · Auto-fix: **no**

The 'Visible' attribute is invalid for classes and events. Use the '~Hidden' attribute instead or omit the attribute since 'Hidden' is false by default.

### CLSAT

Severity: **error** · Auto-fix: **no**

Specify class attributes before the name of the class.

### CLSUNK

Severity: **error** · Auto-fix: **no**

This class, or one of its superclasses, could not be found on MATLAB's path.

### NOPRV

Severity: **error** · Auto-fix: **no**

A class definition cannot be inside a private directory.

### VTPCON

Severity: **error** · Auto-fix: **no**

For properties, validation functions must only use the property being validated or literals.

### VTPEAL

Severity: **error** · Auto-fix: **no**

Specify at least one input argument for validator.

### VTPIN

Severity: **error** · Auto-fix: **no**

Validation function must use the property as an input.

### SPNST

Severity: **error** · Auto-fix: **no**

PARFOR or SPMD cannot be used inside an SPMD block.

### SPRET

Severity: **error** · Auto-fix: **no**

VAR_RESERVED_WORD statement cannot be used inside an SPMD block.

### SPGP

Severity: **error** · Auto-fix: **no**

Setting the GLOBAL or PERSISTENT variable VAR_NAME in an SPMD block might fail because the set happens on a worker machine.

### MCFIL

Severity: **error** · Auto-fix: **no**

Class name VAR_NAME and file name do not agree: VAR_FILE. Update the class name and constructor, if defined, or change the file name to match the class name.

### MCDIR

Severity: **error** · Auto-fix: **no**

Class name VAR_NAME and @directory name do not agree: VAR_FILE.

### MCRED

Severity: **error** · Auto-fix: **no**

Property, event, or enumeration names must be different from the name of the class VAR_NAME.

### MCCBD

Severity: **error** · Auto-fix: **no**

Constructor must be fully defined in the class definition file.

### MCS2I

Severity: **error** · Auto-fix: **no**

Set Methods must have exactly two inputs.

### MCS1O

Severity: **error** · Auto-fix: **no**

Set Methods must have at most one output.

### MCG1I

Severity: **error** · Auto-fix: **no**

Get methods must have exactly one input.

### MCG1O

Severity: **error** · Auto-fix: **no**

Get methods must have exactly one output.

### MCEB

Severity: **error** · Auto-fix: **no**

Events can be defined only in a handle class.

### MCANI

Severity: **error** · Auto-fix: **no**

Abstract property VAR_NAME cannot be initialized.

### MCASC

Severity: **error** · Auto-fix: **no**

Abstract property VAR_NAME cannot be used in a Sealed class.

### MCSGA

Severity: **error** · Auto-fix: **no**

Set or get method must be defined in a METHODS block with no attributes.

### MCSGP

Severity: **error** · Auto-fix: **no**

The method VAR_NAME does not refer to a valid property name.

### MABSEAC

Severity: **error** · Auto-fix: **no**

Instance properties and methods are illegal in classes that are both Sealed and Abstract.

### MABSEAM

Severity: **error** · Auto-fix: **no**

A method cannot be both Abstract and Sealed.

### MCAPP

Severity: **error** · Auto-fix: **no**

Private property cannot be Abstract.

### MCCBS

Severity: **error** · Auto-fix: **no**

A superclass constructor is being called, but VAR_NAME is not a declared superclass name.

### MCCBU

Severity: **error** · Auto-fix: **no**

This superclass constructor is called after a use of the constructed object.

### MCCMC

Severity: **error** · Auto-fix: **no**

Constructor for superclass can only be called once.

### MCCSOP

Severity: **error** · Auto-fix: **no**

Unable to modify Constant property VAR_NAME.

### MCGSA

Severity: **error** · Auto-fix: **no**

Method VAR_NAME tries to set or get an abstract property.

### MCMIO

Severity: **error** · Auto-fix: **no**

Method has too many inputs or outputs.

### MCMSP

Severity: **error** · Auto-fix: **no**

Private method cannot be Abstract.

### MCMTP

Severity: **error** · Auto-fix: **no**

TestParameterDefinition methods must be Static, so that they can be called at test suite creation time to set test parameter values.

### MCPIN

Severity: **error** · Auto-fix: **no**

Unable to initialize class property to an instance of the class itself.

### MCPSG

Severity: **error** · Auto-fix: **no**

Set or get method must be fully defined in the class definition file.

### MCSCC

Severity: **error** · Auto-fix: **no**

To call the superclass constructor, the name of the subclass constructor VAR_NAME must match the name of the subclass VAR_NAME.

### MCSCF

Severity: **error** · Auto-fix: **no**

A superclass constructor must be assigned to the first constructor output argument.

### MCSCM

Severity: **error** · Auto-fix: **no**

To call a superclass method, the method name VAR_NAME must match the name of the subclass method VAR_NAME.

### MCSCN

Severity: **error** · Auto-fix: **no**

Method VAR_NAME tries to set a constant property.

### MCSCO

Severity: **error** · Auto-fix: **no**

A superclass constructor must be called using the first constructor output argument.

### MCSCT

Severity: **error** · Auto-fix: **no**

Superclass constructor call must not be conditionalized or be part of another expression.

### MCSMO

Severity: **error** · Auto-fix: **no**

Returning multiple outputs from a superclass object initialization is not supported.

### MCSWA

Severity: **error** · Auto-fix: **no**

A sealed class cannot specify allowed subclasses.

### MTAGS3

Severity: **error** · Auto-fix: **no**

Cannot use the Access attribute when using the SetAccess or GetAccess attribute.

### MTMAT

Severity: **error** · Auto-fix: **no**

Attribute can only be set once.

### MWKCL

Severity: **error** · Auto-fix: **no**

A WeakHandle property must restrict its type using a class validation.

### MWKCT

Severity: **error** · Auto-fix: **no**

Specifying both WeakHandle and Constant attributes on the same property is not supported.

### MWKREF

Severity: **error** · Auto-fix: **no**

Specifying both WeakHandle and Dependent attributes is invalid. A dependent property does not store a value.

### FVNST

Severity: **error** · Auto-fix: **no**

Arguments blocks in nested function declarations are not supported.

### FVAPN

Severity: **error** · Auto-fix: **no**

Move name-value arguments that use the name=value syntax to the end of the argument list.

### FVATF

Severity: **error** · Auto-fix: **no**

Attribute values in arguments blocks must be logical constants.

### FVBTN

Severity: **error** · Auto-fix: **no**

Use of this function is not supported in arguments blocks.

### FVDAN

Severity: **error** · Auto-fix: **no**

Using the same name as both a name-value argument structure and as a positional argument is not supported.

### FVDAP

Severity: **error** · Auto-fix: **no**

Positional argument can only be declared once.

### FVDNF

Severity: **error** · Auto-fix: **no**

Name-value argument can only be declared once.

### FVDREP

Severity: **error** · Auto-fix: **no**

Multiple Repeating arguments blocks are not supported.

### FVIDV

Severity: **error** · Auto-fix: **no**

Specifying validation or default value for ignored arguments is not supported.

### FVIOA

Severity: **error** · Auto-fix: **no**

Specifying both 'Input' and 'Output' attributes on the same arguments block is not supported.

### FVMCL

Severity: **error** · Auto-fix: **no**

Specifying multiple name-value structures using .? syntax and a class name is not supported.

### FVNDE

Severity: **error** · Auto-fix: **no**

When specifying name-value arguments using a class name, it is illegal to specify default values for the arguments.

### FVNIV

Severity: **error** · Auto-fix: **no**

This variable is not an input to the function and cannot be used in an arguments block.

### FVNREP

Severity: **error** · Auto-fix: **no**

Name-value arguments are not supported in a Repeating arguments block.

### FVNSC

Severity: **error** · Auto-fix: **no**

Use of nested functions is not supported in arguments blocks.

### FVNVL

Severity: **error** · Auto-fix: **no**

When specifying name-value arguments using a class name, it is illegal to specify validation for the arguments.

### FVOBI

Severity: **error** · Auto-fix: **no**

Declare all input argument blocks before all output arguments blocks.

### FVOCON

Severity: **error** · Auto-fix: **no**

For output arguments, validation functions must only use the argument being validated or literals.

### FVOND

Severity: **error** · Auto-fix: **no**

Use of name-value arguments in default values is not supported.

### FVONV

Severity: **error** · Auto-fix: **no**

Use of name-value arguments without dotted name in the validation is not supported.

### FVOOD

Severity: **error** · Auto-fix: **no**

Specifying a default value for an output argument is not supported.

### FVOOI

Severity: **error** · Auto-fix: **no**

Use of ignored arguments in output arguments block is not supported.

### FVOON

Severity: **error** · Auto-fix: **no**

Using name-value argument as output argument is not supported.

### FVORDI

Severity: **error** · Auto-fix: **no**

Ignored input arguments are not allowed after a Repeating arguments block or name-value arguments.

### FVORDN

Severity: **error** · Auto-fix: **no**

Positional arguments must be defined before name-value arguments.

### FVORDO

Severity: **error** · Auto-fix: **no**

Repeating output arguments must be defined after required output arguments.

### FVORDP

Severity: **error** · Auto-fix: **no**

Positional arguments must be defined in the following order: required, optional, and repeating.

### FVORM

Severity: **error** · Auto-fix: **no**

Declaring multiple repeating output arguments is not supported.

### FVOVREP

Severity: **error** · Auto-fix: **no**

Output argument varargout can only be used inside a Repeating output arguments block.

### FVREPD

Severity: **error** · Auto-fix: **no**

Default values are not supported in a Repeating arguments block.

### FVREPO

Severity: **error** · Auto-fix: **no**

Repeating input arguments block containing varargin must not have other arguments.

### FVSOR

Severity: **error** · Auto-fix: **no**

Input arguments block declarations and the function line must contain the same input arguments in the same order, including ignored arguments.

### FVSORO

Severity: **error** · Auto-fix: **no**

Output arguments block declarations and the function line must contain the same output arguments in the same order.

### FVUBD

Severity: **error** · Auto-fix: **no**

Argument is referenced before it is declared in the arguments block.

### FVVCON

Severity: **error** · Auto-fix: **no**

For input arguments, validation functions must only use previously declared positional arguments, the argument being validated, or literals.

### FVVIN

Severity: **error** · Auto-fix: **no**

Validation function must use the argument as an input.

### FVVREP

Severity: **error** · Auto-fix: **no**

varargin can only be used inside repeating input arguments block.

### TINVALDIM

Severity: **error** · Auto-fix: **no**

Each dimension must be a nonnegative integer number or a colon.

### TTOOFEWDIMS

Severity: **error** · Auto-fix: **no**

Specify at least two dimensions for size.

### FCONV

Severity: **error** · Auto-fix: **no**

Unable to define variable VAR_NAME because it has the same name as the script.

### FCONF

Severity: **error** · Auto-fix: **no**

Unable to define local function VAR_NAME because it has the same name as the file.

### GPFST

Severity: **error** · Auto-fix: **no**

A GLOBAL or PERSISTENT declaration must precede first use.

### GPNES

Severity: **error** · Auto-fix: **no**

A GLOBAL or PERSISTENT declaration must be in the outermost function where it is used.

### NPERS

Severity: **error** · Auto-fix: **no**

A PERSISTENT declaration is not valid in scripts.

### ROWLN

Severity: **error** · Auto-fix: **no**

All matrix rows must be the same length.

### FCNANS

Severity: **error** · Auto-fix: **no**

Using ANS as a function name is not supported.

### CLANS

Severity: **error** · Auto-fix: **no**

Using ANS as a class name is not supported.

### BRKFOR

Severity: **error** · Auto-fix: **no**

BREAK statement can only be used in a FOR or WHILE loop.

### CONTFOR

Severity: **error** · Auto-fix: **no**

CONTINUE statement can only be used in a FOR or WHILE loop.

### IDXCOLND

Severity: **error** · Auto-fix: **no**

The END operator must be used within an array index expression.

### CTOINE

Severity: **error** · Auto-fix: **no**

Use of constructed object as input to constructor is not supported.

### CTORO

Severity: **error** · Auto-fix: **no**

Class constructors must be declared with at least one output argument.

### ERTXT

Severity: **error** · Auto-fix: **no**

Specify an error message with the message identifier.

### MHERIT

Severity: **error** · Auto-fix: **no**

Deriving from the built-in MATLAB VAR_NAME class is not supported.

### NCHKOS

Severity: **error** · Auto-fix: **no**

NARGINCHK does not return any values.

### SPBFN

Severity: **error** · Auto-fix: **no**

Use of this function is invalid inside an SPMD block because it accesses or modifies the workspace in a non-transparent way.

### SPDEC

Severity: **error** · Auto-fix: **no**

The bounds on the number of workers an SPMD block can use must be a nonnegative integer.

### SPDEC3

Severity: **error** · Auto-fix: **no**

An SPMD block can only specify a lower and upper bound for the number of workers to use.

### SPEVC

Severity: **error** · Auto-fix: **no**

EVALIN('caller') and ASSIGNIN('caller') are invalid inside of an SPMD block.

### SPLD

Severity: **error** · Auto-fix: **no**

To avoid a transparency violation, assign the output of LOAD to a variable in SPMD blocks.

### SPNF

Severity: **error** · Auto-fix: **no**

The nested function VAR_NAME cannot be called from within an SPMD block.

### SPSV

Severity: **error** · Auto-fix: **no**

SAVE cannot be called in an SPMD block without the '-fromstruct' option.

### SPWHOS

Severity: **error** · Auto-fix: **no**

Using "who" or "whos" without "-file" is invalid inside an SPMD block because it accesses the workspace in a non-transparent way.

### USESWNS

Severity: **error** · Auto-fix: **no**

Variable must be explicitly defined before first use.

### WTXT

Severity: **error** · Auto-fix: **no**

Specify a warning message with the message identifier.

## Examples

### Incorrect

```matlab
parfor i = 1:10          % PFPF: nested parfor
    parfor j = 1:10
    end
    break                % PFBRK: break inside parfor
end

function y = f(a)
    arguments (Input, Output)   % FVIOA
        a (1,1) double = 1      % FVOOD: default on output
    end
end
```

### Correct

```matlab
parfor i = 1:10
    x(i) = i;
end

function y = f(a)
    arguments (Input)
        a (1,1) double
    end
    arguments (Output)
        y (1,1) double
    end
    y = a;
end
```

## Configuration

```toml
[lint.rules.LANGUAGE_SPEC_ENGINE]
severity = "error"
disabled_checks = ["PFPF", "FVIOA"]
```

See [Configuration](../configuration.md) for the full parameter list and [rules.md](../rules.md) for the complete rule inventory.
