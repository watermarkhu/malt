# Conformance with MATLAB's Code Analyzer

What is verified against MATLAB R2026b (`checkcode`, run through the MATLAB MCP
server), the policy that follows from it, and what is not covered. This file
describes the current state only: settle an item, then delete its row. The history
of how it got here is in git and in the plan's progress log.

## How conformance is measured

| What | Where |
| --- | --- |
| Run the Code Analyzer over a file list | `docs/scripts/matlab_probe.m` |
| Compare (file, line, ID) with malt, per check | `docs/scripts/diff_against_matlab.ts` |
| Compare IDs, severities and messages with MATLAB's own list (`checkcode -allmsg`) | `docs/scripts/make-allmsg-csv.ts` + `docs/scripts/check_against_mathworks.ts` |
| Fixture regression test | `crates/malt_cli/tests/matlab_conformance.rs` |

`tests/fixtures/matlab_expected.tsv` holds what MATLAB reports for every fixture
(path, line, ID: facts only, no message text). The test fails on any difference, a
missed finding or an extra one: malt matches MATLAB on all 336 fixtures under the default configuration: the 209
the rules were fitted to, and `corpus1/` to `corpus4/` (31, 31, 33 and 32 fresh files,
written independently of the rules). A new fixture needs its MATLAB expectations: run
`docs/scripts/matlab_probe.m` over it and append the rows (relative path, line, ID;
line 0 becomes 1) to `matlab_expected.tsv`.

MATLAB's own message text and full check list are MathWorks content: keep them outside
the repository (`~/.local/share/malt-reference/`: `matlab-allmsg.txt` from
`checkcode(file, '-id', '-allmsg')`, and `matlab-checks.csv` made from it).

## Policy

- MATLAB's real behaviour is the ground truth. When it cannot be reproduced, the check
  is not emitted rather than guessed.
- The `mathworks` preset mirrors MATLAB's factory configuration. Checks MATLAB's default
  analysis never reports are off under it and on under `--preset all` (or with an explicit
  rule or category setting):
  - bugs: STCUL, SHOCIRF, MOCUP
  - good_practices: ERTAG, WNTAG, CTCH, BDLOG2, CTOINW, GTARG,
    LTARG, SIMPT, STRSZ, ELARLOG, ITERS, ADAPPREF, MIPC1, VTFIN, MCCPE (MATLAB never
    generates the last two)
  - language_spec: FVNIV, FVORDO, USESWNS, PFUTVR, VTPEAL, VTPIN, VTPCON, GPNES
  - readability: LOGSUM, MFAMB; performance: GRIDD; unused: VANUS, NOEFF, PUSE;
    unset_variables: SUSENS, USENS
  - compatibility: IDISVARHIGH, the forward-compatibility gates (FCLEN, FCCPV, FCDQS, FCFAV,
    FCHBL, FCLFS, FCNVA) and the 897 entries MATLAB lists as "low reliability"
    (`low_reliability = true` in `data/compatibility.toml`; also on with the
    `behavior-changes` category)
- The suggested-improvements engine is on by default, as in MATLAB.
- Severities follow MathWorks' documentation index. For RHSFN (error) and PFRNI (warning)
  the numeric levels of `checkcode -allmsg` say otherwise and `checkcode` exposes no
  severity field, so the editor's behaviour cannot be read from here.
- The conformance test only runs the default configuration: the `--preset all` side of the
  policy (the checks above that are on under it) is not compared with MATLAB.

## Official IDs malt does not emit

| ID | Why |
| --- | --- |
| PRTCAL | Never fired in any probe (built-in calls, local functions, functions in files, methods; scripts and functions) or in the fixtures. |
| XCREDS | Fires for `password`/`Password` assigned a non-empty char literal but not for `PASSWORD`, `pwd`, `token` or `apiKey`; the case asymmetry is unexplained. |
| INVALIDCHECKDISABLE, INVALIDCHECKENABLE, INVALIDCHECKPLACEMENT, INVALIDCHECKPRAGMA | `checkcode` ignores `%#check` pragmas; probably emitted by the editor's analyzer only. |
| FVUBD, FVONV | MATLAB reports a forward reference in a validator as FVVCON only, and never reports FVONV. |
| VIDAF, VIDFH, FVOOOV, VINONA | Did not fire in about 40 probes (validators with anonymous functions or handles give FVVCON, FVOCON, VTPCON, FVSYN, FVOON or MCPLD). |
| FVATF, FVNIV, CTOINE, MWKCL | MATLAB never reports them in 11 to 21 probes each (a value on an Input/Output attribute is ATLAB; FVNIV probes gave FVSOR). Gated off (`*_REPORTED = false`), code kept. |
| VTPEAL, VTPIN, VTPCON | `checkcode -allmsg` reports none of them in about 30 probes; off under `mathworks`, still emitted under `--preset all`. |
| SHVAI, SHVAU | MATLAB reported neither in 12 nested-function probes, including the shape malt flagged (gated off: `SHVAI_REPORTED`, `NOT_REPRODUCED`). |
| CLSUNK | MATLAB is silent on a missing superclass; the name heuristic was removed. |
| LNGNM | Never reported in about 35 probes with names of 63 to 120 characters (gated off: `NOT_REPRODUCED`). |
| BADOT, SYNEND, EOFMI | No evidence in MATLAB's output; the syntax engine no longer emits them. |
| TNMLP, STRSET | Not reproducible (TNMLP: loops over many functions; STRSET: the message needs a name that was not found). |
| DEBUGMSG, LLMCC | Need a custom analyzer configuration whose format is not established. |
| BDCFG, CFERR, BDOPT, CFIG | Configuration-file errors; malt does not read MATLAB configuration files. |
| CABE, SCABE, ACABE, MCABE, SMCABE, AMCABE | Cyclomatic-complexity metrics, reported only with `-cyc` or `-modcyc`. |
| MSNE | MATLAB reports an unknown ID in a `%#ok<ID>` directive; malt needs the complete set of official IDs (or of its own declared IDs at runtime) to decide, and would otherwise report false ones. |
| EMNODEF, EMBRK, EMCNT, EMRTN, EMWHL, EMS2N | MATLAB Coder checks, see below. |

## Not verifiable through `checkcode`

The Coder (code generation), Compiler (deployment) and Fixed-Point checks are in MATLAB's
list, but a `%#codegen` probe with `eval` and `global` returned only base-product messages
with MATLAB Coder installed. Until a probe shows `checkcode` emits them, the `codegen`,
`deployment` and `fixed-point` engines are verified against the documentation index only.
Eight message templates differ for that reason (EMVDF, EMGRO, EMFCN, EMRIFAV, EMLOAD,
PRMNOIN, LOOPPRAGMAWITHOUTFOR, FPASE).

## Suppression directives and MSNU

`%#ok<ID>` directives are read from comment nodes as MATLAB does: the comment must start with `%#ok`, only the first directive per comment counts, and directives in strings, after `...`, `%%#ok` or mid-comment do not count; a bare `%#ok` suppresses everything on its line. MSNU ("a message was once suppressed here") is emitted for an ID named in a directive that is *active* in the engine and produced no diagnostic on its line (`Rule::active_check_ids`, implemented by the performance and unused engines; other engines never cause an MSNU). Risk for the unused engine: NASGU has known gaps, so a directive on an unhandled NASGU case gets a false MSNU. Risk: a finding malt misses where MATLAB reports it makes malt report MSNU for the user's directive. A bare `%#ok` that suppresses nothing (MATLAB reports MSNU) and GRIDD (MATLAB reports MSNU although malt has it off under `mathworks`) are not emitted.

## Known gaps

No fixture fails on these.

| Gap | Note |
| --- | --- |
| Overfitting | The first fresh corpus (`corpus1/`, 31 files) measured generalisation before any fixing: 44 of MATLAB's 45 findings matched (recall 98%) and malt reported 28 more (precision about 61%): DEFNU in function-based test files, data entries firing on ordinary calls, class checks with an unresolvable superclass, and a few flow checks. All were fixed and the files joined the fixtures. `corpus2/` (31 files, different themes, unseen) then measured 24 of 27 findings matched, 3 misses and 7 extra (recall 89%, precision 77%): MHERM, the date-function family (TYEAR), DISPLAYPROG, `persistent` redefinitions (REDEF, GPFST), LOAD in anonymous functions, PROP, NASGU with `rethrow`; all fixed. `corpus3/` (33 messy, student, legacy and other-language-style files, MATLAB reports 160 findings on them) measured 134 matched, 26 misses and 12 extra before fixing: SAGROW and AGROW growth in loops, OR2/AND2 (see the precedence rule below), NASGU/PREALL and `syms`, ROWLN, MSNU, FNDSB, VERLESSMATLAB and single rows; all fixed. The number to watch is the first-measurement difference count per MATLAB finding of each new corpus: 29/45, 10/27, 38/160 (0.64, 0.37, 0.24). `corpus4/` (32 files: old-era code, formatting variety, class-heavy code; MATLAB reports 174 findings) measured 170 matched, 4 misses and 9 extra (13 differences, 0.075 per finding) before fixing: logical predicates in AND2/OR2, MCPO/MCVM class gates, RHSFN with a shadowed function, block-comment openers (INBLK), a leading blank line or comment before `function` making the file a script (DEFNU), MSNU for NASGU, NOPTS, STRNU, CTPCT; all fixed. The per-finding rate across the four corpora is 0.64, 0.37, 0.24, 0.075 (plan step 4.1). |
| BDSCA2 in call arguments | MATLAB reports `disp([1 0] && x)` and `f(x && [1 0])` as BDSCA2; malt does not visit a `&&` inside function-call arguments, so it is silent there (a miss, not an extra). |
| SAGROW | `x(size(x,2)+1) = v` is reported by MATLAB; only `numel`/`length` are handled. |
| VERLESSMATLAB | A version held in a variable (`vv = '9.10'; verLessThan('matlab', vv)`) is reported by MATLAB and missed by malt (needs constant propagation); MATLAB reports it for dotted versions of 9.9 or higher, in every position. |
| OR2, AND2 | Bare calls outside the probed list (`isempty`, `ischar`, `strcmp`, `numel`, `length`, `size`, `any`, `max`, `min`, `mean`, `isnan`, `isfield`) are treated as unknown and stay silent. |
| DEEPC | malt reports a nested `%{`; MATLAB was silent on nested block comments in the probes (depth 3 not probed). |
| CTPCT | MATLAB reports too many arguments (`sprintf('%s %d', 'a', 1, 2)`); malt only handles zero specifiers or zero arguments. |
| MCVM | The "later read" of `obj` is textual (sibling `if`/`switch` branches excluded); loops are not modelled. |
| RHSFN | `[a,b] = v(...)` is reported only when `v` holds a numeric literal; chains through other variables are not followed. |
| STRNU | MATLAB reports `x(end+1).f = v` on an undefined `x`; malt is silent. |
| AGROW | `x(numel(x)+1)` on a parameter is unprobed. |
| Message text | The test compares file, line and ID, not the text of dynamic messages (for example MATLAB quotes names in MCNPN: `'foo' is referenced...`). |
| Statement right after a stray `end` | tree-sitter parses it as ERROR tokens plus a number, so malt misses MATLAB's NASGU for `z = 1;` there. |
| `eval` through a variable | MATLAB reports EVLCS for `t = 'x = 1;'; eval(t)` and EVLEQ for `t = ['x = ' num2str(v)]; eval(t)`; malt is silent on both. `['a.b = ' num2str(v) ';']` is EVLEQ in MATLAB and EVLDOT in malt. |
| Command syntax in kind conditions | `display x` and `year x` are reported by MATLAB; command arguments never match the `arg_kinds` conditions, so malt is silent. `year(-5)` was not probed. |
| Loop with try/catch and `break` | malt can report an unused `line = readline(s)` in a `while` loop whose `try`/`catch` contains `break`, where MATLAB is silent. |
| `[h,a] = legend;` | MATLAB reports LEGMOP; malt only visits calls and commands, so the bare form with several outputs is missed. |
| `load(fileName, 'x')`, `save f x` | A variable file name drops the first string, so `'x'` is lost as a store of `x` (NASGU); the command form of `save` is not handled. |
| Class member reads | `obj.Y(3)` (parenthesised read in a setter, MCSUP) and `obj.Undecl{1}` (MCNPN) were not probed. |
| `x = f(a.b=1);` | MATLAB reports SYNER at the `=`; malt reports nothing (only the statement-level call form is handled). |
| Self-call after an unclosed `arguments` block | The statement is not visited by the symbol table, so DEFNU can report the primary function where MATLAB is silent. |
| NASGU | MATLAB's constant propagation is only partly implemented (`1 == 1` is not folded, as in MATLAB; other cases are untested). |
| UNRPWR | Condition inferred from the tree shape; the column is one off from MATLAB's. |
| CMDAND, CMDOR | No MATLAB trigger found in six command-syntax probes; the message matches the official text. |
| PFTUSE, PFUIXE | Message agrees; MATLAB reports at the assignment or `parfor` line, malt at the use after the loop. |
| DEFNU | `localfunctions` counts as a use of every local function (function-based test files) and a bare validator in an `arguments` block (`{mustBeSquareMatrix}`) as a use of that function; the dynamic-call exemption covers `eval`, `feval`, `str2func`; `evalin`, `evalc`, `run` are unprobed. |
| FNDEF | The column is always 1 (MATLAB points at the name). |
| SYNER | Shows malt's error-node text where MATLAB names the offending token. |
| `analysis/control_flow.rs` | The `elseif` condition is still matched as `kind() == "condition"` (line ~762); `condition` is a field name in this grammar. |
| Message tables | Explanations removed from the Check IDs tables to match MATLAB's templates (SHOCIRT, DEBUGFUN, CTRUE, CHAIN, ...) are not yet moved into the `## Rule` sections. |

## Compatibility data: conditions and object members

Entries in `data/compatibility.toml` can carry `min_args`, `max_args`, `args_any`,
`args_all`, `unless_args`, `second_arg`, `script_only` and a `pattern` (`generic`, `object`,
`unverified`; only entries without a pattern are looked up by name). The `object` entries
(`tcpclient`, `visadev`, `Session` members, INTLPOUTA) are reported by MATLAB only on a
variable of that class and are not matched until class tracking exists. A variable or local
function with the same name shadows a removed function. `unverified` entries (INTRPC,
ADTPATH, MATPOOL, SESSION\*) were never reported by MATLAB in the probes (some need toolboxes
that are not installed); LENEMP and LEGPVPAIR are `unverified` (MATLAB reported neither on any `legend` call form). About 90 serialport, tcpserver, udpport and bluetooth member entries are `object` patterns: MATLAB reports nothing on a plain constructor call. `min_outputs` limits an entry to calls with several outputs (LEGMOP). The suggested-improvements data has its own condition fields
(`literal_args`, `arg_kinds`, `arg_at`, `max_args`, `within_parfor`, `not_in_call`, ...).

## Rules worth knowing

- NASGU is a liveness analysis with constant folding (`analysis/liveness.rs`): `if 1`,
  `if true` and `k = 1; if k` kill the dead branch; a `for` loop joins the values before and
  after its body; `parfor` bodies run once, `spmd` is a sequence, a lambda reads its captures
  at creation, `x += v` silences `x`, `load('a.mat','x')` stores `x`; dynamic calls do not stop
  the analysis (`clearvars` and `run` still do).
- A file whose root is an ERROR node that starts with `function` is analysed as a function
  (symbol table, FNDEF, NASGU).
- `condition` is a field name in this grammar, not a node kind: read it with
  `child_by_field_name("condition")`.
- After a token MATLAB cannot parse, it suppresses the other checks on that statement
  (separator to the next `;`, `,` or newline); `malt_core::linter` does the same. A
  number-literal error drops every SYNER in its statement; an unterminated string drops the
  other checks on its statement.
- Statements after `error(...)`, `return`, `break`, `continue` and after `while true` without
  `break` are unreachable and many checks skip them; this is probed per check (STCMP and
  FLUDLR still fire after `error`).
- A block comment opener is a line that is only `%{` (whitespace allowed) and the closer a line that is only `%}`; `%{ text`, `x = 1; %{` and `%} text` are plain comments (INBLK).
- A file is a function file when its first statement is a `function`, ignoring comments and blank lines (an earlier version let a leading blank line or help comment turn the file into a script).
- AND2/OR2 are silent when an operand is a known-logical predicate: `isstruct`, `isnumeric`, `iscell`, `ischar`, `islogical`, `isa`, `isequal`, `strcmp`, `strcmpi`, `isempty`, `isobject`, `isinteger`, `isfloat`, `~` of those, `true`/`false`; `exist`, `isfield`, `isreal`, `isvector`, `any`, `all`, `contains`, `isfile`, `xor`, `isnan` are reported.
- A `%#ok` directive on line L covers messages from the first line of its logical statement (joined by `...` or a bracket newline) through L; a directive on the first line does not cover later lines, and MSNU follows the same association.
- BDLOG1 fires only for a transposed condition (`if c'`), never for a plain variable condition.
- In an `arguments` block, a declaration with a plain name followed by a package-qualified class (`obj pkg.Class`) is positional, not name-value.
- `name = value` call arguments are not uses of `name`; the value side still is.
- AGROW: `x(end+1).f = v` is reported (SAGROW in a script); growth of `x{k}(end+1)` is silent after `cell`, `zeros`, `struct`.
- NOPRT/NOPTS are silent for calls of a variable holding an anonymous function or handle; the column for an assignment is the `=`.
- COMPNOT/COMPNOP/DUALC are reported by default: COMPNOT is `== 0`/`~= 1` and COMPNOP `== 1`/`~= 0` on `any`, `all`, `isequal`, `isreal`; `== true`/`false` is silent.
- CTPCT: too many arguments (count not a multiple of the specifier count) or any argument with zero specifiers is reported; too few is silent.
- PROP and CPROP: see the check docstrings; CPROP is a bare read or call of a property name in a method that never uses the name as a variable.
- MCPO/MOBSRV: `SetObservable = false` is silent. MCHDP/MCHDT: only `containers.Map()` as a handle default.
- Parfor reductions (PFRIN/PFRUS) are reported once per variable, at its first reduction.
- BDLGI/BDSCI/BDSCA2 are on by default; ATPPI/ATPPP fire only for invalid attribute values (`Private`, bad `SetAccess`); `(x > 1) & (y < 2)` is AND2; a classdef method named `ans` gives MTHANS only.
- tree-sitter-matlab gives `|` and `&` higher precedence than the comparison operators: `L > 1 | L < 0.1` parses as `L > (1 | L) < 0.1`. `analysis/elementwise.rs` recovers the true operands (MATLAB: comparison, then `&`, then `|`); use it in every check that reads the operands of `|` or `&` (OR2, AND2, LBODUP use it; FNAN and CHAIN carry older patches).
- MATLAB reports OR2/AND2 unless an operand is known to be logical (a comparison of values of known class, `isempty`/`ischar`/`strcmp` results, `true`/`false` variables); comparisons of `input`, indexed arrays that grow, struct fields and most function results do not count as known.
- FNDSB in its variable form (`idx = find(L)`) is reported at the single direct-subscript use of `idx`, inside a function only; the direct form `A(find(...))` is reported everywhere.
- Class checks that depend on handle-ness or inherited members (MCVM, MCSNOV, MCEB, MCPO; MCNPN and MCNPR for anything but `handle`) are silent whenever the class lists a superclass, whether or not it resolves.
- NODEF is reported for a read with no reaching definition when the name is assigned somewhere
  later in the function, and silent otherwise. INUSD uses the entry value's liveness: an input
  overwritten before any read, or read only after `error(...)`, is unused.

## Real third-party code

`tests/external_toolkit.rs` lints MathWorks' matlab-agentic-toolkit (145 `.m` files, commit `b58be8124631cccb03a67aa79dcef1c9720243e2`) when `MALT_EXTERNAL_TOOLKIT` points to a clone; the code is licensed for MathWorks products only and is not vendored. `toolkit_expected.tsv` holds facts only. First measurement: 34 MATLAB findings, all matched, 83 extra (precision 29%); after fixing, 0 differences. Agent-written corpora measured an easier problem than real code.

## Phase 2 audit gaps (good_practices)

Open after the audit and the follow-up pass (probe files are not kept; re-probe before changing):

- Syntax: `try, x = 1; catch ME end` is SYNER in malt (MATLAB: SEPEX plus ENDCT2); EOFER for `if x, disp(1) end` and `while x x=0 end`; NBRAK2 for `[1:2:3]`; NOSEMI on `case 2, y = 2;`.
- MSNU for `end %#ok<TRYNC>`, `%#ok<NOPRT>`, SHVAU/ITERS, the second of two own-line directives and a bare `%#ok`: needs `active_check_ids` on more engines (false-MSNU risk).
- BDSCA2 inside call arguments (`disp([1 0] && x)`) is missed. BDLGI/BDSCI use a reaching-definitions rule inferred from about 60 probes: a variable is reported only when every reaching definition is of the reported kind.
- ADMTHDINV misses `@() helper + 1`.
- Parfor: PFBNS rule unclear (6 misses); PFUNK for `k=i; v(k)=1`; PFRUS/PFRIN/PFVSUB/PFUTMP extras when a reduction variable is read whole; PFOUS misses struct-field sliced writes; PFIIN after `clear k`; PFINCR for `s = [s i]`; PFUTVR is off by design.
- CFALSE is limited to `any`, `isequal`, `isreal` (others unprobed); STLOW for a letterless literal; INUSD message text includes the argument name; STRNU loop-store fallback is a heuristic.

## Phase 2.2 audit gaps (language_spec)

Open after the audit (re-probe before changing):

- Parser-driven (tree-sitter-matlab gives ERROR nodes for valid MATLAB): `arguments` attribute values and `opts.?pkg.Class = 1` are parse in the fork (the earlier same-length source rewrite is gone); `y = end + 1;` no longer gives SYNER. Still open: `arguments (Input,⏎ Repeating)` line, `enumeration (Foo)` with one `end`, `e = [1 2 3](end)` in a property block (MCPLD), `[end]`/`{end}` (RESWD), `function [a, ~]` (BADNOT), `i += 1;` (SYNER missed), EOFER, UNRCH after `if 1, return, end`.
- Class checks: MCMTP false positive (non-static `TestParameterDefinition`); MCCBD (constructor prototype in a methods block), MCSCF, and ATUNK for enumeration attributes are missed; `methods (Static = TRUE)` is NOIN in MATLAB; MATLAB does emit MCCSPS on `ClassName.K = v` in a method.
- FNCOLND (`disp(end)`), UNRCH after `if 1 return end`, FCONV position (first reference), REDEF for duplicate local functions and constructors, REDEFGG, NBRAK2 on `[narginchk]` and `[1:2:3]`, `function [a, ~]` (MATLAB: syntax error), `a (1 2)` dimensions, ERTXT through a variable.
- MSNU for language_spec directives (BRKFOR, WTXT, CTOINE) needs `active_check_ids`.
- PUSE: MATLAB reports it on 7 probes but it is on the `mathworks` off-list; probe before deciding.
- FVSORO position in an output block, GPNES unprobed positively.

## Parser fork

malt depends on `tree-sitter-matlab` 1.3.3 plus the fork branch `malt-parser-fixes` (watermarkhu/tree-sitter-matlab): `arguments` attribute values, `opts.?pkg.Class = 1`, `end` as an expression outside an index and four-element ranges now parse. The fork also accepts `if end > 1`, which MATLAB rejects; malt reports it as SYNER. Fork changes follow the upstream CONTRIBUTING rules (minimal, no new node types, no generated files in source commits) so they can be sent upstream.

## Phase 2.3 audit gaps (syntax_errors)

Closed: REDEF scoped per function body, REDEF for duplicate methods and constructors, REDEFGG once per name, CLIS/SOFOC/CLTWO lines, SEMFU, BDFIL limit (2048 characters), UNRCH after a constant-true `if` and for `try, catch end`, SYNER/NOPAR/EOLPAR/ENDPAR token positions, EOFER for an unseparated `end` in scripts, RESWD only for `end` inside `[ ]`/`{ }`, BADNOT/BADNOTLHS, BADCH/BADCT, STRIN/DOUQT pairing, FVSYN/MCPLD ordering tokens, keywords used as values (`x = for;`) are SYNER not ENDCT, a `;` row in an output list is LHROW only. `i += 1;` is SYNER in scripts only (silent in functions in the fixtures).

Open: FVAMI on `f('Bad Name' = 1)` (the linter drops the finding because the statement has an ERROR node); FNSWA fires on `function y = _x()` (MATLAB: BADCH; MATLAB never reports FNSWA in the probes); NOSEMI on `case 1 y = 1;`; `u = (1:2:3:4)` M3COL column and `1:2:3:4:5`; `[a;b;c] = size(x)` LHROW and NOPAR2; `[~ ~] = f()` BADNOT at the second `~`; `x = 1e+;` MISSING-node SYNER; a multi-line `"..."` DOUQT shape; EOFER also silences other checks in MATLAB; `IDXCOLND` on `[end + 1]`; NOPAR vs NOPAR2 on `f(123=1)`; REDEF messages do not quote names.
