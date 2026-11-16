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
  - bugs: SHOCIRF, MOCUP
  - good_practices: ERTAG, WNTAG, CTCH, CTOINW, GTARG, LTARG, SIMPT, ITERS, ADAPPREF, MIPC1,
    MCCPE (MATLAB never generates it)
  - language_spec: FVORDO, USESWNS, PFUTVR, GPNES
  - readability: LOGSUM, MFAMB, ISROW, ISCOL; performance: GRIDD; unused: VANUS, NOEFF;
    unset_variables: SUSENS, USENS
  - compatibility: IDISVARHIGH, the forward-compatibility gates (FCLEN, FCCPV, FCDQS, FCFAV,
    FCHBL, FCLFS, FCNVA) and the 897 entries MATLAB lists as "low reliability"
    (`low_reliability = true` in `data/compatibility.toml`; also on with the
    `behavior-changes` category)
  - codegen: the whole category is off; EMSCR, EMTC and EMIMP are reported only in a file
    with `%#codegen`, and the engine is gated on that directive. Deployment, system-object
    and fixed-point checks were silent in 13 shapes.
  - Verified in the Phase 2 close-out (738 probe files, plain `checkcode -id`): every entry
    above was probed in at least 4 shapes and MATLAB reported it in fewer than 3 (the
    thresholds: LTARG 2, SIMPT 1, LOGSUM 1; all others 0), and 60 low-reliability entries
    in 3 shapes each gave no report. `checkcode(..., '-allmsg')` returns the message
    catalogue and adds no reports. Taken off the list by that verification (MATLAB reports
    them by default): BDLOG2, VTPEAL, VTPIN, VTPCON, VTFIN, STRSZ, FVNIV, ELARLOG, and
    earlier BDLGI, BDSCA2, COMPNOT, COMPNOP, DUALC, PUSE, MCCSPS, STCUL.
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
- Class checks that depend on handle-ness or inherited members (MCVM, MCSNOV, MCEB, MCPO; MCNPN and MCNPR for anything but `handle`) are silent whenever the class lists a superclass, whether or not it resolves. With the path index on (stage 3.4), MCEB and MCPO also report a superclass that is a value class on the path.
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

## Phase 2 close-out (2026-11-15)

Phase 2 is closed: every engine was audited with systematic probes, the open items of each audit were worked through, and the `mathworks` off-list was re-verified (see 'Policy'). What follows is what is still open, none of it blocking.

Closed in the last pass:

- BDLOG2: MATLAB reports it only for an arithmetic operator (`+ - * / \\ ^`) at the root of a condition or `&&`/`||`/`&`/`|` operand, at the operator; the elementwise operators give BDLOG1. On the list taken off with VTPEAL, VTPIN, VTPCON, VTFIN, STRSZ (silent in `if`/`while` conditions; misses for variables and an `&` operand), FVNIV (the default names a variable assigned later) and ELARLOG (`abs`, `numel`, `length`, `floor`, `round` of one comparison).
- Parfor: PFOUS/PFTUSW ignore `save(...)` arguments, PFINCR for `max`/`min` reductions and `s = [s i]`, AGROW in a parfor (`s(end+1) = i` reported, `s = [s i]` silent); sliced outputs in lambdas (`x(i)` is PFANON at the `x`; `x(1)`, `x(i - 1)`, `x(t)` give PFSLO and a whole read PFSLRD, at the first mention).
- Flow: STCI/STCUL follow the known value that `lower`/`upper` converts; ISMT on `length(a) & b` in conditions; BDSCA1 on `if []`/`while []`; UNRCH for a catch after a nested try (depths 1 to 5); INUSD for a parent argument read in a nested function.
- Syntax and class: `f('Bad Name' = 1)` is FVAMI alone; a multi-line `"..."` is DOUQT; DEFNU is not reported for duplicated local functions; NOPRC for `x = 1 %c` and `-x + 2`; IDXCOLND is not reported in a property block (MCPLD only) or inside brackets; MCVM models loops; ADMTHDINV for `@() helper + 1`; MTAGS3 is not reported for methods blocks (ATUNK); the NOIN column for `methods (Static = TRUE)`.
- Codegen: EMSCR, EMTC, EMIMP only with `%#codegen`; a leading help comment no longer makes a function file a script.
- Formatting: NOPRT, NCOMMA, NOCOMMA, NO4LP, NOPTS and ALIGN match MATLAB on 142 shapes under `--preset all` (CTCH aside, which is off for good reason).

Open:

- Parfor: PFTUSE for a nested `for j` header and for a bare `t`; PFIIN for indexed reads of a later-defined variable; PFRUS on `s = s + i; s = s - 1` with no use; PFUNK on `y(j) = i` in a nested loop; PFUTVR stays off (MATLAB reports nothing in 10 shapes under plain `checkcode`; `--preset all` was not compared again).
- Syntax: ENDCT2 after an unclosed `if` before a `while` (the "after line N" rule is undetermined; malt gives a false SYNER at 1:1); `enumeration (Foo)` with one `end` (ENDCT2 missing, false SYNER on the classdef line); `[end + 1]` in a function file (MATLAB also gives NOPAR and SYNER); `[a;b;c] = size(x)` NOPTS and ASGLU (not emitted); the primary function of a file with broken `[end + 1]` syntax lost its MATLAB DEFNU; MATLAB reports SYNER on the SECOND `catch` of a try with two, malt on the first.
- Columns (not compared by the strict test): ATTF and MTAGS3.
- Class checks: USESWNS not reproduced in two shapes; GPNES for `global x` in a nested function whose outer function uses `x` (a miss; both stay off).
- Limits: a `switch` with 8,000 cases takes 5.9 s and 20,000 cases 52 s in release (with all categories off 0.06 s; no single engine is the cause, suspected per-node `parent()` walks across many engines: a per-file parent map or context passed down the walk is the fix); TMMSG keeps messages in generation order in MATLAB and in line order in malt (no per-engine generation-order probes yet); the MDEEP/NOSPC/MXASET/DEEPN columns were not probed; SETNU extras in nested-function bodies did not reproduce.
- Flow: NODEF in 7 default cases malt misses (`fvniv_07`, `useswns_07`, `mfamb_*`); FNCOLND 1 miss; ASGLU not emitted.
- Deferred by design: MSNU for engines other than performance and unused (needs reliable detection per ID); the 166 object compatibility entries (local type inference, plan 3.P5); `codeAnalyzerConfiguration.json` compatibility (plan step 5.8).

## Phase 3, first wave (2026-11-15)

Shared helpers (plan items 3.1, 3.2, 3.6; no output changed: 604 fixture and toolkit files and 383 extra probe files are identical to the pre-refactor binary under both presets):

- `analysis/callee.rs`: `call_path`/`command_path`/`handle_path` with an explicit dotted mode (`Whole`, `Trailing`, `Skip`), `callee_word`, `reference_name`, `is_shadowed` (enclosing scope or also the script scope) and `resolve` (variable, local or nested function, qualified, function, unknown; no consumer yet). About 20 duplicated helpers were deleted. Not migrated: `language_spec::callee_name` (reads `child(0)` of any node), the behaviour-property `name_field` (reads the `name` field of every node, so a `function_definition` named `set` is read as a call: probably a bug, fixing it changes output) and `symbols::is_qualified_call`.
- `analysis/metadata.rs` is the one attribute parser (`AttributeMeta`, `Attributes` with `get`, `has`, `is_true`, `value_text`; forms `Named`, `Bare`, `Written`). About 12 sites migrated. Class-level `~X` is invisible to `Named` (the grammar wraps it), which is why ATTOF reads the text. Not migrated: `is_set` users (mcpin, mcswa, mwkcl), `check_fv_validation` attribute names, `check_prop_validation::is_static_method` and `check_fvinr::has_output_attribute` (substring matches).
- `analysis/command.rs`: `Command::from_node` (name, arguments without quotes, `is_dotted_misread`, `is_shell_escape`). 3 of about 85 command sites migrated; the rest (compatibility `call_args`, `liveness::callee_name`/`clear_names`, control_flow, symbols, unused, unset_variables, bugs, suggested_improvements, custom_checks, the spmd/parfor commands and the per-engine `check_command` modules) are still text-based.

Definite assignment (3.3, 3.5): MATLAB (plain `checkcode -id`, 165 probes) never reports SVNODEF, SUSENS or NODEF in a script, whatever the shape (conditional definition, loop, switch, try, unknown call, `load`/`eval`/`assignin`). In a function it reports NODEF for a read after `clear` of the only definition, in an `if false`/`if 0`/`while false` body, after a `parfor` for a name defined only in the body or the loop variable, for `save('q.mat','q')` and `save f q` before a definition, and for a nested function's own dead definition; SVNODEF once per name when a nested function reads a parent variable all of whose parent definitions are dead; it is silent for a definition on only some paths (if without else, loops, switch without `otherwise`, `try` with the definition in `catch`). `eval`, `evalin`, `assignin` and an unnamed `load` define nothing; `load(f, 'x')` and `load f x` define `x`; `save('q')` alone is silent. A `parfor` body read of a later definition is PFIIN, not NODEF. A dead assignment after `return` is STOUT. `analysis/maybe_defined.rs` is a may-defined walk (union at merges, constant-dead branches and code after terminators skipped, loops by their body definitions, `parfor` bodies do not leak except sliced assignments, `try`/`catch`, `spmd`, `syms`); NODEF, SVNODEF and STOUT use it; USENS keeps the old rule and stays off.

Open: PFTUSE extras and PFUIXE/PFTUSW misses on two parfor shapes, a PFUTMP extra, NASGU for `save f q` after a `parfor` (malt reports, MATLAB is silent); bare `clear` and `clear all` are not modelled; the 3.4 remainder (validating rule names in `.malt.toml`), 3.7 (explicit confidence in `typing.rs`), the rest of the command-site migration and the three unmigrated callee helpers; the path-index stages (3.P2 to 3.P6).

## Phase 3, second wave (2026-11-15)

Helpers (output identical on 642 fixture and toolkit files and about 380 extra probe files under both presets; the only change is that PFUTMP and PFUNK messages now quote the variable name as MATLAB does): `language_spec::callee_name`, `bugs::extract_call_name` and the behaviour-property `name_field` are on `analysis/callee.rs`; `get_function_call_name` is a thin wrapper. The suspected `name_field` bug is not observable (the traversal reaches it only for `function_call` nodes). `callee::resolve` still has no consumer (SPNF and PFNF would change output when a variable shares a nested function's name) and keeps its `allow(dead_code)`. About 80 command sites are still text-based, notably `liveness::clear_names` and the compatibility `literal`, which read raw text including `%` comments and quotes (moving them changes output: needs a MATLAB decision each). Attributes: about 20 predicate sites use `Attributes` (`is_set`, `is_present`); `attributes_text_contains` keeps the two substring matchers (`arguments (Input = "Output")`, `methods (Access = Static)` differ) and `check_fv_validation::check_block_attribute_names` is not migrated. `analysis/typing.rs` is kept (MCHDT resolves a property default through an assignment made in a method): `TypeKind::Unknown` is gone and `known_kind` returns `None` when the kind is not proven; the old `global`/`persistent` arm never matched because the grammar's nodes are `global_operator` and `persistent_operator` (handling them would change MCHDT output: undecided).

Config validation (3.4): `malt_rules::config_check::config_warnings` warns once per problem with a 'did you mean' suggestion (unknown rule ID, unknown per-rule parameter, an unquoted `[lint.rules.naming.class.casing]`, which TOML reads as the rule `naming` and used to be ignored silently); `--strict-config` turns the warnings into an error with exit code 2; unknown presets and category keys were already errors. The parameter table is hand-written (15 engine structs plus the naming parameters): a new engine parameter must be added there.

Parfor and flow: PFUIXE on an empty loop, PFTUSE (no extra after `clear t`; a bare `t` statement is a read; the nested `for j` header at the iterator; an output variable with no earlier value, with the function's line; `t.x = 1` is a read and `t(2) = 5` is not), PFIIN for indexed reads of a variable defined only after the loop, PFUTMP only for top-level reads before the first set (reads under `if`, loops, `switch`, `try` are silent; a self-update gives PFUTVR), PFUNK on `y(j) = i` in a nested loop; NASGU: `save f q` is a read, `error(msg)`, `error(ME)`, `error(ME.message)` and `error(ME.identifier, 'b')` do not end the flow (UNRCH follows only `error('b')`, `error(['a' 'b'])`, `error(sprintf('b'))`); `save` with an option other than the format options (`-append`, `-mat`, `-ascii`, `-v4`, `-v6`, `-v7`, `-v7.3`, `-double`, `-tabs`, `-nocompression`) reads nothing, because `-regexp` and unknown options change what the words mean. Verified with no change: `save('f.mat')` alone, unnamed `load`, `load(f, 'x')`, `exist`/`who`, `eval`/`evalin`/`assignin`, globals, persistents, `try`/`catch`, `switch` with `return`, `break`/`continue`, bare `clear` and `clear all`.

Open: PFRUS on `s = s + i; s = s - 1`: MATLAB reports it for (+i, -1), (-i, +i), (+1, -1), (+i, i-1) and not for (+i, +2), (+i, +1), (+i, i+s), (+i, +i), (-1, -i); the candidate rule 'report when the variable has both a `+` and a `-` assignment' fits all 19 shapes except two misses; PFBNS for `v(2)`/`v(end)`; PFUTVR; PFINCR for `s = s + i; s = s / 2`; NODEF after a loop for indexed reads (`t(1)`) and a nested `j`; NOPRT for a bare name after the loop; `save f -struct s x` is not probed.

## Path index, stage 1 (2026-11-15)

`crates/malt_index` (not published yet) is the native Rust core of the replacement for maxx: an index of the MATLAB path (functions, classes, scripts, `+pkg` packages, `@cls` class folders, `private/`) with lazy tree-sitter parsing, argument and property facts, and raw docstrings. It is not used by the linter by default (the Code Analyzer is file-local); stage 3.4 adds an opt-in use, see 'Path index, stage 3.4'.

Oracle: `docs/scripts/maxx_dump.py` (run inside a maxx checkout) and the `dump` example of the crate emit the same JSON; `docs/scripts/compare_index.py` diffs them against `crates/malt_index/oracle_allowlist.json` (25 entries, each with the reason maxx and the index differ). 26 projects (maxx's tests and submodules, the toolkit, mkdocstrings fixtures, malt's fixtures) compare with no unresolved difference. The allowlisted differences are mostly maxx defects: its function query reads only the first match (so outputs after the first in `[a, b] = f` and the inputs of a function with nested functions are lost, and nested functions leak in), `~` is dropped, `Dependent` is dropped because it checks the spelling `Dependant`, class attributes are not recorded, a class folder's constructor is popped from its members, a class with a parse error is read as a script; the index also records local and nested functions, property and enumeration spans and the `varargin` kind.

Independent check against MATLAB itself: `nargin` and `nargout` of the 124 top-level function files of the toolkit agree with the index on every output count and on 123 input counts; the one mismatch is a folder that also holds a `.p` file (MATLAB prefers it).

Performance (release): the 143-file OpenTelemetry-MATLAB tree 0.04 s (maxx 0.70 s, 7 MB against 48 MB), the 145-file toolkit 0.09 s (maxx 0.88 s), the 461-file gnu-octave-statistics 2.1 s (maxx 3.4 s, 7 MB against 270 MB).

Not done (stage 2): `.malt.toml` `paths` with globs, opt-in linter use, the PyO3 module with raw docstrings, live scripts and maxx's `Expr` API, and moving griffe from maxx to mkdocstrings-matlab.

## Path index, stage 3.4 (2026-11-15)

The linter can use the path index (`crates/malt_index`) when the user turns it on: `[index] enabled = true` in `.malt.toml`, or `--index` (`--no-index` overrides). It is off by default, and with it off the output is the Code Analyzer's, byte for byte: the sample project `crates/malt_cli/tests/index_project` is linted against the output recorded before the change (`tests/index_golden/default_stdout.txt`), and the conformance suites are unchanged.

What the index adds, and only that: one check reports a class whose superclasses are value classes found on the search path (the file's own folder first, then `[index] paths`, default the working directory). A superclass that is not on the path, or whose bases are not, is unknown and gives nothing.

- **MCEB** ("Events can be defined only in a handle class.") for a class with a value superclass. Evidence: MATLAB's `checkcode` is silent on MCEB for any superclass, present or missing (probed: `sub_ev < base_v`, `sub_evh < base_h`, `sub_missing < no_such_base` silent; `classdef sub_ev` with no superclass reported at its `events` line). Loading the class in MATLAB R2026b (`obj = sub_ev()`) is an error, `MATLAB:class:nonHandleClassWithEvents` ("The class 'sub_ev' may not define events because only subclasses of handle may define events."), the same as with no superclass; a handle base loads. The extension reports what MATLAB refuses to load.
- **MCPO** is left as MATLAB has it: `checkcode` reports it only for a class with no superclass, and malt does not extend it to subclasses of value classes (the evidence was a runtime effect, not a MATLAB check).

Not done, on purpose: no consumer for the callee kind (`Callee::resolve`, an unknown `f(x)` resolving to a project function). No existing check ID changes its message or its count when a call is resolved to a project function that the rules do not already read, and the open items in this file that would need it (STRQUOT on an unknown variable, NASGU after `clearvars`) are not about function resolution. The inherited-members and output-count consumers (plan 3.4 items 3 and 4) are open. Each new finding uses an existing check ID and its MATLAB message; none is invented.

Limits: the index is built per folder of a linted file (its own folder first) and cached for the run; `[index] paths` are relative to the working directory, so a run from another folder can resolve differently. With the index on, the findings depend on the folders around a file (a base next to it, or on the configured path); that is why it is off by default and why `--no-index` exists.

Tests: `crates/malt_rules` (MCEB with a stub search path, index-on and index-off cases, handle and unknown bases); `crates/malt_cli/src/path_index.rs` (the adapter on temporary projects: handle through a project base, value chains, unknown and MathWorks bases, a base on the configured path, a file outside the path, relative paths, `[index]` parsing); `crates/malt_cli/tests/path_index.rs` (the golden default output, `--index` adds exactly the one finding, config and flag precedence, a configured path, a missing folder as a stderr warning, an invalid `[index]` table as an error).
