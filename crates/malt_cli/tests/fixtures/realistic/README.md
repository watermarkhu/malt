# Realistic MATLAB fixtures

Idiomatic code used to catch false positives. `crates/malt_cli/tests/realistic.rs`
lints each file with the default preset and pins the set of check IDs it may
report. A new false positive should become a new file here (or a new case in an
existing one) before it is fixed.

| File | Exercises | Expected IDs and why |
| --- | --- | --- |
| `script1.m` | Script: `load`, `fprintf`, dynamic fields, `clear` | `CLEAR0ARGS` (`clear` with no arguments) |
| `script2.m` | Script: `tic`/`toc`, `containers.Map`, `cellfun`, `end:-1:1` | none (script workspace values are not dead stores) |
| `func1.m` | Function: `inputParser`, `varargin`, `catch err`, `switch` | none |
| `func2.m` | Function: `arguments` block, `parfor`, nested function, `switch`, lambda | `FVINR`, `PFNF` (nested function called in `parfor`) |
| `cls1.m` | Class: `properties`, `arguments` block, static method | ``FVINR` |
| `test1.m` | `matlab.unittest.TestCase` with `Test`/`TestMethodSetup` | none |
| `gui1.m` | GUIDE-style callbacks and `varargout` | `DEFNU`, `INUSD`, `VANUS` (unused callback arguments and callback) |
| `legacy1.m` | `global`, `persistent`, `nargin`, `varargin` parsing, `eval` | `AGROW`, `ERTAG` |
| `legacy2.m` | Old string handling: `findstr`, `\|`, growing char arrays | `AGROW`, `NASGU`, `OR2` |
| `num1.m` | Conjugate gradient: `r' * r` products | none |
| `num2.m` | Monte Carlo script: logical `sum`, tables, `rand(N, 2)` | `LOGSUM`, `STRNU` |
| `coder1.m` | `%#codegen`, `coder.inline`, `persistent` | none |

Rules for this directory:

- Keep files idiomatic; do not write code only to trigger a finding.
- Say why each expected ID is correct (the table above is the record).
- These files are excluded from the whitespace hooks (see `hk.pkl`).
