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
| READSZK | MATLAB is silent on `KeyValueLimit` in nine probes (`readstruct`, `set`/`get` on `0` and `groot`, property assignment); not emitted (the test asserts silence). |
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
| OR2, AND2 | Bare calls outside the probed list (`isempty`, `ischar`, `strcmp`, `numel`, `length`, `size`, `any`, `max`, `min`, `mean`, `isnan`, `isfield`) are treated as unknown and stay silent. |
| MCVM | The "later read" of `obj` is textual (sibling `if`/`switch` branches excluded); loops are not modelled. |
| RHSFN | `[a,b] = v(...)` is reported only when `v` holds a numeric literal; chains through other variables are not followed. |
| STRNU | Conflicting probes: one set had MATLAB reporting `x(end+1).f = v` on an undefined `x`, a later set (cd3_g03, cd3_g04) had it silent; malt is silent. Re-probe before changing. |
| AGROW | `x(numel(x)+1)` on a parameter is unprobed. |
| Message text | The test compares file, line and ID, not the text of dynamic messages (for example MATLAB quotes names in MCNPN: `'foo' is referenced...`). |
| Statement right after a stray `end` | MATLAB reports one SYNER and no NASGU for the statement after a stray `end`; malt adds a second SYNER on that statement (grammar-level, open). |
| Signed literal in kind conditions | `year(-5)` is reported by MATLAB and malt is silent: a `unary_operator` arm in `argument_kinds` also made `display(-3)` fire, which MATLAB does not report (the existing test), so the arm was removed. Command syntax (`display x`, `year x`) matches MATLAB. |
| `[h,a] = legend;` | MATLAB reports LEGMOP; malt only visits calls and commands, so the bare form with several outputs is missed. |
| Class member reads | `obj.Y(3)` (parenthesised read in a setter, MCSUP) and `obj.Undecl{1}` (MCNPN) were not probed. |
| `x = f(a.b=1);` | MATLAB reports SYNER at the `=`; malt reports nothing (only the statement-level call form is handled). |
| NASGU | MATLAB's constant propagation is only partly implemented (`1 == 1` is not folded, as in MATLAB; other cases are untested). |
| CMDAND, CMDOR | No MATLAB trigger found in six command-syntax probes; the message matches the official text. |
| PFTUSE | PFUIXE is reported at the loop header with the read's line (aligned); PFTUSE for a nested `for j` header is still silent in malt. |
| DEFNU | `localfunctions` counts as a use of every local function (function-based test files) and a bare validator in an `arguments` block (`{mustBeSquareMatrix}`) as a use of that function; the dynamic-call exemption covers `eval`, `feval`, `str2func`; `evalin`, `evalc`, `run` are unprobed. |
| FNDEF | The column is always 1 (MATLAB points at the name). |
| SYNER | Shows malt's error-node text where MATLAB names the offending token. |

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
- SMTHGF, SMTHFA and SMTHFT are reported only through `groot` (`set(groot, ...)`, `get(groot, ...)`); `set(0, ...)` and `feature(...)` are silent. SMTHG: `set` and `get` only on a figure (`gcf`, `figure(...)`, or a variable assigned from them); a property assignment `x.GraphicsSmoothing = v` is reported unless `x` is assigned `gca` (a file-wide scan, because the assignment itself mentions `x`).
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

Open after the cleanup pass (re-probe before changing):

- Declarations: `enumeration (Foo)` in a class with one `end` (MATLAB ENDCT2 and ATUNK, malt SYNER on the classdef); `e = [1 2 3](end)` in a property block also gets an IDXCOLND; the columns of NOIN and ATTF for `methods (Static = TRUE)`.
- Class checks: MCVM's "later read" of `obj` is textual (loops not modelled); MTAGS3 for methods blocks; ADMTHDINV misses `@() helper + 1`; ATTF for the logical attribute names `Transient`, `WeakHandle`, `NonCopyable`, `AbortSet`, `SetObservable`, `GetObservable`, `HandleCompatible` is unprobed; USESWNS, FVSORO position and GPNES positives are unprobed; REDEF on a duplicate constructor prototype in a methods block.
- MSNU for language_spec directives (BRKFOR, WTXT, CTOINE) needs `active_check_ids` (see 'Suppression directives and MSNU').

Closed in the cleanup pass: MCMTP (the attribute only, in a test class, non-static), MCCBD, MCSCF, ATUNK on enumeration blocks, MCCSPS (reported, off the off-list), MCNPR and MCNPN in constructors, `Static = TRUE` is not static, IDXCOLND inside `[ ]`/`{ }` is RESWD.

## Parser fork

malt depends on `tree-sitter-matlab` 1.3.3 plus the fork branch `malt-parser-fixes` (watermarkhu/tree-sitter-matlab): `arguments` attribute values, `opts.?pkg.Class = 1`, `end` as an expression outside an index and four-element ranges now parse. The fork also accepts `if end > 1`, which MATLAB rejects; malt reports it as SYNER. Fork changes follow the upstream CONTRIBUTING rules (minimal, no new node types, no generated files in source commits) so they can be sent upstream. Open in the grammar (not fixed here; the fork is pinned in `Cargo.toml`): a three-part dotted command with an argument (`comm.gpu.AWGNChannel arg1`) gives SYNER and swallows the next line, so a handle on that line (`h = @comm.gpu.AWGNChannel;`) is missed; MATLAB gives one SYNER for the statement.

## Phase 2.3 audit gaps (syntax_errors)

Closed: REDEF scoped per function body, REDEF for duplicate methods and constructors, REDEFGG once per name, CLIS/SOFOC/CLTWO lines, SEMFU, BDFIL limit (2048 characters), UNRCH after a constant-true `if` and for `try, catch end`, SYNER/NOPAR/EOLPAR/ENDPAR token positions, EOFER for an unseparated `end` in scripts, RESWD only for `end` inside `[ ]`/`{ }`, BADNOT/BADNOTLHS, BADCH/BADCT, STRIN/DOUQT pairing, FVSYN/MCPLD ordering tokens, keywords used as values (`x = for;`) are SYNER not ENDCT, a `;` row in an output list is LHROW only. `i += 1;` is SYNER in scripts only (silent in functions in the fixtures).

Open (after the cleanup pass): `f('Bad Name' = 1)` still adds NOPAR2 and SYNER next to the FVAMI MATLAB reports; `[a;b;c] = size(x)` (MATLAB LHROW only; malt NOPAR2 and SYNER); a multi-line `"..."` DOUQT shape; ALIGN and ENDCT2 after an unclosed `if` before a `while`; DEFNU on a duplicated local function; NOPRC for `x = 1 %c` then `-x` and `x = 1` then `-x + 2`; a bare `%#ok` claimed by a wildcard gets MSNU in MATLAB; NOPAR vs NOPAR2 on `f(123=1)`; the EOFER filter, the loop-header SEPEX, IDXCOLND in brackets, NUSED once per name, NOIN static and the FNDEF header have no unit tests (verified by probe diff only).

Closed in the cleanup pass: one-line `for`/`while` SEPEX, EOFER silencing other checks (`EOFER_KEPT_IDS`), M3COL at the last colon, `i += 1` (SYNER) and `x += 1` (REDEF when assigned, never SYNER), `[end]`/`{end}`/`[end 1]` (RESWD), `function [a, ~]`, `[~ ~] = f()`, `a (1 2)`, BADCH for leading underscores, `1e+` (BADFP only), FVACS vs FVAMI by the quoted name, REDEF messages quote names, a second `global n` is REDEFGG only.

## Naming checks (Phase 2.4)

MATLAB reports `naming.*` only with a code analyzer configuration: `checkcode(file, '-config=<path.json>')` (note the `=`), or, through `codeIssues`, a `codeAnalyzerConfiguration.json` in a `resources/` folder of the file's folder or a parent. Plain `checkcode(file)` is silent, so the naming engine is off under `mathworks` and has no default `maxLength`/`minLength`. The schema (`schemaVersion` >= 1.1.0; entities class, enumeration, event, function, localFunction, method, nestedFunction, property, variable; `maxLength`/`minLength` 1 to 2048 inclusive; `regularExpression` full match; `casing` one of lowerCamelCase, UpperCamelCase, lowercase_with_underscores, UPPERCASE_WITH_UNDERSCORES, lowercase, UPPERCASE; prefix/suffix/phrase a string or non-empty array, case-sensitive) comes from MATLAB's `matlab.codeanalysis.validateConfiguration` messages. Variables are reported once per scope at the first occurrence; the constructor is the class, not a method; `arguments`-block entries are variables.

Open: malt reads its own TOML keys, not MATLAB's JSON file (a loader would also have to handle `checks`, `ignoredFiles`, `baseConfiguration`); parent-scope variables read inside nested functions, script-level local functions, `checks`-based severity overrides and the array semantics of required prefix/suffix are not probed.

## Formatting checks (Phase 2.5)

NOPTS/NOPRT: echoed expressions of every kind are reported (strings, matrices, cells, handles, lambdas, `~x`, ranges); a bare name assigned later in its scope is a variable, `ans` only when assigned somewhere; silent in `parfor` bodies and when indexing an input argument, a copy of one, or a variable started as `[]`. The column is the operator for binary, comparison and range expressions, the quote for postfix, the bracket for indexing. ALIGN covers `if`, `for`, `while`, `switch` and `try` (not `spmd`, `function`, `else`/`elseif`); a column difference of 1 is silent and a tab counts as one column. NCOMMA is silent after an ignored output (`[~ a]`).

Open: ALIGN after an unclosed `if` before a `while`; MSNU for formatting directives; the `--preset all` side of formatting was not compared.

## Incomplete analysis (Phase 2.7)

MATLAB's real limits (probed by bisection): MDEEP 33 nested brackets/calls, DEEPN 18 nested functions (reported at the deepest header), DEEPC the 8th nested `%{`, NOSPC weighted nesting (if 4, switch 8, for/while 3, try 2 quarter-levels) at >= 994, MXASET 998 nested binary operators, TMSMS above 1000 parse errors, TMMSG a budget of 10,000 messages. MATLAB never reports MBIG (1 to 16 MB files), TEXTL (5,000,000 character lines), DEEPS or an EOFER from this category: those malt checks were deleted. `MALT_TOO_DEEP` is malt's own parse-tree safety limit (1,000 levels native, 200 wasm), not a MATLAB message.

Open: loop nests of 500+ and operator chains of 999+ hit the safety limit first (MATLAB: NOSPC/MXASET); TMMSG weights messages in MATLAB (about 3 units for NOPTS); a single statement with 1,500 errors gives TMSMS in MATLAB only; performance: a 320-deep `for` nest takes about 23 s and a 50,000-statement file more than five minutes (suspect NASGU liveness); the docs still list the removed `max_*` keys of the incomplete-analysis engine (old keys parse without error).

## Suppression wildcards, NOPRC and syntax follow-ups

`%#ok<*ID>` suppresses exactly that ID (exact, case-sensitive; no globs, `*all`, or trimming) in the whole file, wherever it sits, including `*MSNU`; wildcards are removed before line directives, a named directive whose message a wildcard removed gets MSNU, and an unused wildcard gets none. NOPRC is reported at the `=` of a one-line assignment without a semicolon that ends on the line above a statement starting with a unary `+`/`-` (not after a call, command, comment or blank line). FNSWA is not emitted (MATLAB never reports it): a leading `_` in a function name is BADCH. `case 1 y = 1;` is SEPEXR, not NOSEMI; FVAMI/FVACS survive a dropped error statement; `[~ ~] = f()` is BADNOTLHS then BADNOT.

MSNE is emitted for a directive element that is not a valid ID (case-sensitive, no spaces; `%#ok<>` and wildcards are silent) from `malt_core/src/known_ids.rs`: the declared IDs, the IDs in 'Official IDs malt does not emit', a few special IDs and the `naming.*` pattern (IDs only, no message text). A bare `%#ok` that covers no message gets MSNU. MATLAB's MSNU does not depend on the `mathworks` off-list: `%#ok<ITERS>`, `%#ok<NOEFF>`, `%#ok<NOPTS>` and `%#ok<NOSEMI>` all get MSNU when nothing on their statement reports them, so the real limit is detection reliability.

Open: MSNU is still emitted only for the performance and unused engines (activating an engine is unsafe wherever a default-on check has a detection gap: a missed finding becomes a false MSNU); a bare `%#ok` claimed by a wildcard gets MSNU in MATLAB; BADCH/other leftovers are in the sections above.

## Performance and depth limits (Phase 2 follow-up)

Release timings after four passes (12 cores; the 1-CPU column in parentheses): 50,000 straight-line statements 2.1 s (5.0 s), 20,000 0.7 s, a 320-deep `for` nest 0.3 s, 2,000 functions in one file 0.2 s, the 336 fixtures 0.25 s (1.6 s), the 145 toolkit files 0.3 s (1.7 s), one tiny file 6 ms (was 18 ms), 1,000 tiny files 0.07 s, a 12,000-line mixed file 1.2 s default and 1.8 s with `--preset all` (was 22 s). Output is byte-identical to the previous binary on the fixtures, the toolkit and the generated inputs under both presets.

What made the difference: the symbol table is built once per file (`SymbolTable::shared`, per parse epoch) and the builder carries an ancestor stack (tree-sitter's `parent()` and `next_sibling()` search from the root); `dead_stores` runs once per file; the formatting, codegen, unset-variable and reaching analyses index each scope once instead of rescanning per node; the two large data tables (`compatibility.toml`, `suggested_improvements.toml`) are turned into static Rust tables by `crates/malt_rules/build.rs` (the TOML stays the source of truth), which saves about 13 ms of startup per run; the CLI lints files in parallel when not fixing (output in command-line order; `--fix` stays serial); trees of 20,000+ nodes run the engines' file checks on worker threads (opt-in through `Linter::set_file_check_threads`, off for the library, tests and wasm; peak memory rises from 110 to 200 MB); `lto = "thin"` in the release profile. Rule order is pinned in `RULE_ORDER` (`crates/malt_rules/src/lib.rs`) because `inventory` returns rules in link order and diagnostics at the same position depend on it; a test fails when an engine is missing from the list. Fat LTO gave nothing over thin LTO for 2.5 times the build time; `panic = "abort"` is not possible because a panicking rule must become QUIT.

Remaining: about 40 full-tree walks at 1 to 6% each (UNRCH, NASGU, the compatibility generic walk, `check_error_nodes`, `check_logical_usage`); `Node::kind()` string conversion (about 15%) behind 1,255 call sites; parfor/spmd checks rescan the file once per loop; the wasm build and debug-build stack use were not checked.

`Rule::check_limits` runs before the depth guard: NOSPC (weighted nesting >= 994; 331 nested `for` weigh 993, 332 fire) and MXASET (>= 998 nested binary operators) are then the only output, as in MATLAB; `MALT_TOO_DEEP` remains for other deep trees. Open: 2,000 nested brackets (MATLAB: MDEEP), deeply nested anonymous functions (MATLAB: NOSPC, threshold unprobed) and nested functions (DEEPN) still give `MALT_TOO_DEEP`; debug-build stack use and the wasm build were not checked.

## Compatibility data and unused/unset_variables (Phases 2.6 and 2.8)

About 1,700 of the 1,837 compatibility entries were probed in the shapes their conditions imply (plain call, argument shapes, argument and output counts, script vs function, handle, command syntax, shadowing by a variable). Fixed: 21 entries that matched the wrong function name (their message named another function), about 60 missing or wrong conditions, 17 `pattern = "object"` entries MATLAB never reports on a plain call, 9 entries set to `pattern = "unverified"` because MATLAB was silent in every form (DFCNCHK, MSYSTEM, OBJMPOOL, JAVFM, JAVCT, PERFREF, V6ON, JAPIMATHWORKS, PRJET), and the engine now matches handle references (`@f`), bare dotted names and a bare identifier on the right of a multi-output assignment (`[h, a] = legend;`). New conditions: `first_arg_any`, `first_arg_not_text`, `unless_sole_arg`, `unless_all_args`, `second_arg = "not_string"`, `second_arg_any` (case-sensitive; a literal or a variable assigned only such literals) and `second_arg = "optimoptions"` through a variable. MATLAB reports `solve(p, solver)` (OPTMSLV, OPTMNVP) only for the solvers `linprog` and `intlinprog`, and OPTMOPT for an `optimoptions` value whatever the solver. None of the 897 low-reliability entries was reported by MATLAB in any probe form, so the flags stay.

Unused and unset: UNRCH folds `while 1` without a break, bare `exit`/`quit`, `if` with every branch erroring, dead `catch` clauses and chains of false conditions; PUSE is reported by default (off the `mathworks` off-list) for a persistent that is written but never read; STOUT ignores dead assignments; Static methods and constructors give INUSD, not MANU; VUNUS reports `x{1};`. NOEFF and VANUS are gated off (`NOEFF_REPORTED`, `VANUS_REPORTED`): MATLAB reports neither in any probe, and a `%#ok<NOEFF>` still gets MSNU.

Open: the 166 `object` entries need class tracking (plan stage 3.P5); the MDFRD, MDFCHL and MDFSVA entries of the suggested-improvements data depend on the same work (MATLAB reports them on a variable assigned from `mdf(...)`). Closed in the handle, pair-list and property pass: BUFSIZE with `Bufsize` as a name in a complete pair list (`pairs_from`), CNNCGD with a text first argument (`first_arg_not_text`), QAMDEPM/QAMDEPD with a number-literal phase (`number_arg_at`: `pi/4` is silent), JAPIMATHWORKS for any member of `com.mathworks` (`function_prefix`), SMTHGF/SMTHFA/SMTHFT only through `groot`, SMTHG on `set`/`get` of a figure and on `x.GraphicsSmoothing = v` (unless `x` is `gca`), the handle flag on 549 entries (`handle = true`; the others are silent on `@name`), and the severity and message text of 168 and 92 entries aligned with the reference list (three "has been removed" entries moved from Suggested Improvements to Compatibility at Error). Still open: `g = gcf; set(g, 'GraphicsSmoothing', 'off')` on one line is missed (the two-line form is reported); the three-part dotted command in the Parser fork section; NASGU after a bare `clearvars` (MATLAB reports `x = 1; clearvars; x = 2;`; malt skips the scope); MCNPR for `obj.x = a` in a constructor; PFRUS for a reduction with no later use; `x = 1; x += 1;` in a script (MATLAB REDEF, malt SYNER).

## Suggested-improvements data (Phase 2.9)

About 245 entries probed in the shapes their messages imply (no arguments, one to three arguments, the literal strings of the message, each argument kind, command syntax, bare name, handle, shadowing by a variable, parfor/spmd, script vs function file). The usage conditions are now data: arity (`min_args`/`max_args`), literals (`literal_args`, `literal_at`, `literal_exact`, `literal_has_value`), argument kinds (including `handle` and `struct`), `min_outputs`, `function_only`, `statement_only`, `not_assigned`, `arg_through_variables`; dotted names are references in value positions (statement, argument, operand, `if` condition, matrix element, handle), not in command form. About 21 entries are `pattern = "unverified"` (MATLAB never reports them in any probe form: OOPS, EMTAG, EMXTR, NVREPLA, NVREPLM, VIDREAD, NUMCH, GETFSP, GETFMT, SETFMT, SETFSP, INTRPP, HGSTGT, FISADR, MINELV, the LEGACY* entries); the RISK*, MASSO and IMSSO entries could not be probed (toolboxes not installed). IMPORTDYN and MATCH2 were split into two entries each.

Open: STRQUOT on a parameter of a function file (`function f(p); y = string(p);` is reported by MATLAB; `string(x)` with `x = 'abc'` is silent in both file kinds; `string('abc')` is reported in a script) needs an argument kind for unknown variables, not done; `year(-5)` (see Known gaps); MDFRD, MDFCHL and MDFSVA need class tracking (plan stage 3.P5); DATE is reported once more than MATLAB in the handle probe; SETFMT (`set(0, 'Format', 'short')`, an entry this data marks as unverified) is reported by MATLAB and not by malt; DINVHCRM (one `set(groot|0, ...)` shape reported by MATLAB, not reproduced); EVLCS through a variable is a known gap. Closed in the handle and command-shadowing pass: `@name` per entry (`handle = false` on 58 entries, e.g. `@now`, `@hour`); a command with a shadowing variable (`corner abc`, `display abc`, `nargchk abc`, `ezplot abc`, `cornermetric abc`, `ezgraph3`, `ezcontourf`, `ezmeshc`, `ezsurfc`) only for the nine entries with `shadowed_command = true`, all others silent (`hour abc`, `gcat seed`); the eight DLAB*/DNUMLABS entries now only in this table (Info); TNOW1 with extra `datetime` arguments already matches MATLAB; IMPORTDYN (`not_all_arg_kinds`: silent only when every argument is a single-quoted literal); MDFLT1 numeric third argument only; command syntax in kind conditions (`display x`, `year x`) already matches. `sigwin.hamming seed` is a SYNER in MATLAB only.

## Parser follow-up and the second cleanup wave (2026-11-15)

Fork rev `d377bdf`: a dotted name followed by a word at a statement start (`comm.gpu.AWGNChannel arg1`) is scanned as a command whose first argument is the glued rest of the dotted name, so the error stays on its line (MATLAB: SYNER at the word, the next lines analysed as usual); malt reports the SYNER at the word. The 481 fixture and toolkit trees do not change. Handles of dotted names are reported at the name after the `@`. `phase2/` fixtures (EOFER filter, one-line `for`/`while` SEPEX, a repeated `global`, `Static = TRUE`, `function [a, ~]`, the dotted command) are part of the strict test.

Closed in the flow wave: a shared constant resolver (`reaching_values`/`resolved_values` in `analysis/reaching.rs`: `eval` through a variable, ERTXT, VERLESSMATLAB, PFEVB/SPEVB with `b = 'base'`, FVAL through copies, RHSFN on indexed values), NASGU after a bare `clearvars` (clears do not kill: `clear all; eval(t)` is silent), `load(f, 'x')` stores, SAGROW for `x(size(x,2)+1)`, `case` expressions are scanned for uses. `save('a.mat','x')` and `save f x` are uses (already right); AGROW on a parameter and CTPCT already matched; `v = 'abc'; [a,b] = v(2)` is RHSFN in MATLAB.

Open: `c = true; if c ... else ... end` (MATLAB folds the dead branch: ERTXT `cd_er06`, RHSFN `cd_rh09`); UNRCH after `error(x)` with a variable argument and inside `while false`; `[end + 1]` and friends: MATLAB also reports NOPTS on the statement (its recovery echoes despite the `;`) and no NBRAK2, malt reports NBRAK2 and no NOPTS; CFALSE for functions other than any/isequal/isreal, STLOW for a letterless literal, the AND2/OR2 bare-call list, the UNRPWR column, PFTUSE/PFUIXE lines, `year(-5)`; DEFNU after an unclosed `arguments` block and with `localfunctions`.

## Phase 2 completion (2026-11-15)

The parfor, limits/naming/docs and flow-leftover agents finished (the first two had stopped at the session limit). Closed:

- Flow: dead branches fold through `strcmp`, `strcmpi`, `ischar` and a copy of their result (not through `==` or `isempty`, which MATLAB leaves live); OR2 reports `size`; the UNRPWR column is at the power operator before the sign; `year(-5)` and the signed number kind for the date entries; STRQUOT for a variable with exactly one reaching text value, in a function only (silent on a parameter and in a script); NOPRT on `[end + 1]` in functions. Already matching, no change: UNRCH after `error(x)` and in `while false`, CFALSE, STLOW, SMTHG on one line, the DEFNU cases.
- BDLOG2 (found while integrating): MATLAB reports it only for an arithmetic operator (`+ - * / \ ^`) at the root of a condition or `&&`/`||`/`&`/`|` operand, at the operator (`if z + 1` column 6, `if (z + 1) * 2` at the `*`); `if 1`, `if x` with `x = 5` and `if t` give nothing, and the elementwise `.* ./ .^` give BDLOG1. malt had a "bare scalar value" rule that MATLAB does not have.
- Parfor: PFBNS (65 probes: `M(i, i)` and `data(i + k)`, handles are calls), PFUNK, PFRUS for `s = [s i]`, PFOUS (earlier reads are uses, struct-array field writes are outputs), PFIIN after `clear k`, PFSLO/PFSLRD at the first mention in the body, SPBFN for `run`, PFTUSW for function outputs. A sliced output in a lambda: `x(i)` is PFANON at the `x` inside the lambda; `x(1)`, `x(i - 1)`, `x(t)` give PFSLO and a whole read PFSLRD, both at the first mention. PFUIXW is no longer emitted (MATLAB reports a read after the loop as PFUIXE).
- Limits: a top-level function weighs 5 units for NOSPC (247 `if`s silent, 248 fire); TMMSG weights NOPTS, FNDEF, INUSD at 3 units and NASGU, UNRCH, AGROW, ALIGN, COMPNOT, COMPNOP at 1; MDEEP, anonymous-function NOSPC and DEEPN match.
- Naming: functions in a script are `localFunction`; requiredPrefix/requiredSuffix arrays are any-of. MATLAB's configuration uses the top-level key `naming` with the entities inside and `requiredPrefix`/`requiredSuffix` (`entities`, `checks`, `prefix` and `suffix` are rejected by `validateConfiguration`).

Open:

- Flow: STCI on `strcmp(lower(b), 'x')` with `b = 'x'` (malt reports, MATLAB is silent, other single-letter cases report); in function files `[end + 1]` also has SYNER at `]`, NOPAR and DEFNU in MATLAB; ISMT on `length(a) & b` and BDSCA1 on `if []`/`while []` are MATLAB-only.
- Parfor: `save(file, 'a')` after the loop is not a use for MATLAB; PFINCR misses `s = max(s, i); s = min(s, i)` and `s = s + i; s = [s i]`; PFTUSE for a nested `for j`; `data` first defined after the loop (PFBNS and PFIIN); 14 NODEF reports on reads after the loop of a variable assigned only inside it; PFUTVR stays off (`checkcode -id` emits it; enabling it needs two false positives, `uv01`/`uv02`, fixed and 16 misses closed).
- Limits: TMMSG keeps messages in generation order in MATLAB, not line order; a `switch` with 20,000 cases takes 52 s in release (per-case `parent()` walks); UNRCH on `catch` clauses after nested `try/catch/end` (1,194 misses); SETNU in nested-function bodies and INUSD for a parent argument read in a nested function (malt extras); MDEEP columns not re-checked.
- Formatting `--preset all`: only 29 script shapes probed (15 NOPTS and 4 ALIGN match); NOPRT, NCOMMA, NOCOMMA and NO4LP are unprobed under `all`.
- Still deferred by design: MSNU for the other engines (needs reliable detection per ID), the 166 object compatibility entries (path index, plan stage 3.P5), `codeAnalyzerConfiguration.json` compatibility (plan step 5.8).

## Neighbouring files and the path (plan stage 3.P1, probed 2026-11-15)

Question: does the Code Analyzer depend on files next to the analysed file, or on the MATLAB path? Method: each scenario linted with its neighbouring files present (A) and absent (B), through both entry points (`checkcode(file, '-id')` and the editor's `codeIssues(file)`), in three environments: the folder not on the path with another current folder (E0), the folder on the path (E1), the folder as the current folder (E2). About 150 comparisons over 20 scenarios, with real findings in the compared files (NOPTS, NOPRT, DWVRD, FSTR, NODEF). Result: **no difference in any comparison.**

Scenarios: a sibling function shadowing a deprecated function (`findstr`) and a removed one (`wavread`, also on the path and as the current folder: still DWVRD); a sibling function called with the wrong number of inputs or outputs, with `[~, b] =`, with `varargout`; a bare undefined name `y = und;` (no NODEF in either case: MATLAB treats it as a possible call); statement-level calls of sibling functions with and without outputs, with and without a semicolon, in scripts and functions (NOPTS/NOPRT identical); sibling scripts; `private/`, `+pkg` and `@class` folders; static methods and handles of a sibling class; command-syntax calls; a variable named like a sibling function; a subclass of a sibling handle class and of a sibling value class, reading and writing an inherited property and a property that exists nowhere (`obj.Missing`): MATLAB reports nothing in any variant.

Conclusions:

- The checks probed are file-local. Neighbouring files, the path, the current folder and the installed toolboxes do not change what MATLAB reports (only MATLAB, Fixed-Point Designer, Coder, Compiler, Copilot and Parallel Computing Toolbox are installed here, yet the DSP and Communications entries are reported by name).
- Class checks are silent whenever a superclass is listed, whether or not the superclass is visible next to the file. Resolving superclasses through a path index would produce findings MATLAB never reports, so the planned "handle-ness through project classes" consumer (3.P4) must not change the default output.
- So the path index is not needed for `checkcode` parity. Its value is elsewhere: the Python module that replaces maxx (3.P6), opt-in analyses beyond MATLAB (unresolved calls, project-wide symbols), and editor features. The 166 object compatibility entries need local type inference (`g = gcf; g.Prop`), not a path index.
- Not probed: MATLAB Projects (`.prj`) and `resources/` project configuration, `checkcode` with `-config`, other releases.
