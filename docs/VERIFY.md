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
| Fixture regression test, and the fixture list | `crates/malt_cli/tests/matlab_conformance.rs`; how fixtures are added: `crates/malt_cli/tests/fixtures/README.md` |
| Recorded expectations against live `checkcode`, and the release | `crates/malt_cli/tests/matlab_live.rs` (see 'Live checkcode check') |

`tests/fixtures/matlab_expected.tsv` holds what MATLAB reports for every fixture
(path, line, ID: facts only, no message text), recorded with the release named in
`tests/fixtures/matlab_release.txt`. The test fails on any difference, a
missed finding or an extra one: malt matches MATLAB on all 1,133 fixtures under the default configuration, among them
`corpus1/` to `corpus4/` (31, 31, 33 and 32 fresh files, written independently of the rules). A new fixture needs
its MATLAB expectations and a line in `matlab_files.txt`; `crates/malt_cli/tests/fixtures/README.md` gives the steps.

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
  - good_practices: ERTAG, WNTAG, CTCH, CTOINW (the `C(C(x))` call form only; a constructor taking its own output is reported), GTARG, LTARG, SIMPT, ITERS, ADAPPREF, MIPC1,
    MCCPE (MATLAB never generates it)
  - language_spec: FVORDO, USESWNS, PFUTVR, GPNES
  - readability: LOGSUM (the general `sum(<logical>)`; the slice-and-literal shapes MATLAB reports are on), MFAMB, ISROW, ISCOL; performance: GRIDD; unused: VANUS, NOEFF;
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
- CFALSE is limited to `any`, `isequal`, `isreal` (others unprobed); STLOW for a letterless literal; STRNU loop-store fallback is a heuristic.

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

## Cross-check against MATLAB (path index)

`crates/malt_cli/tests/matlab_namespace.rs` checks the path index (`malt_index`) against MATLAB on the same folders. It runs in two phases.

1. **Generation.** `tests/matlab/generate_namespace_facts.m` (a script, no arguments) runs `matlab_facts.m` on the committed fixture and writes `target/matlab/namespace_facts.json`. For each name of the folders it records what MATLAB says: the file `which` resolves to and `which -all` in order, `nargin` and `nargout` of function files, and for classes the superclasses, attributes, properties, methods (with inputs and outputs), events and enumeration members. The folders it used are recorded too (`roots`). No project code runs: names are listed with `what` and `dir`, and classes read with `meta.class.fromName`.
2. **Comparison.** The Rust test reads the facts file and compares it with the index built from the recorded folders, in both directions: every name the index lists is a name MATLAB lists and resolves to the same file; every name `what` lists is in the index; the facts both sides have agree (docstrings and spans are not compared).

**Why two phases: licensing.** On a public repository, only the `matlab-actions/run-build`, `run-tests` and `run-command` actions license MATLAB automatically. A `matlab -batch` started from `cargo test` is not licensed in CI. So CI generates the file with `matlab-actions/run-command` and then runs `cargo test` with `MALT_MATLAB_FACTS` set, which starts no MATLAB. The later live `checkcode` facts will be generated by one more run-command step, and read by their own test with the same contract.

### The facts file contract

- `MALT_MATLAB_FACTS` is the path of one facts file. A relative path is relative to the repository root (cargo runs tests in the crate directory). A directory is not accepted: the comparison needs one file.
- With it set, MATLAB is not started. The file's `roots` give the folders, so `MALT_MATLAB_EXTRA_PATHS` is not read in this mode (the test prints a note if it is set).
- With it set and the file unreadable: `MALT_MATLAB_REQUIRE=1` fails; otherwise the test prints a `SKIP` line with the reason and passes. A file that is not valid JSON always fails.
- Without it, the test finds MATLAB (`MALT_MATLAB`, else `matlab` on `PATH`) and runs the same generator script in a temporary directory. Without MATLAB it prints `SKIP` and passes; `MALT_MATLAB_REQUIRE=1` fails.

### Generating the file

From the repository root, with MATLAB on `PATH` (as CI does):

```sh
matlab -batch "run('crates/malt_cli/tests/matlab/generate_namespace_facts.m')"
MALT_MATLAB_FACTS=target/matlab/namespace_facts.json MALT_MATLAB_REQUIRE=1 \
  cargo test -p malt-cli --locked --test matlab_namespace -- --nocapture
```

The generator reads two optional environment variables: `MALT_NAMESPACE_FACTS_OUT` (the file to write; default `target/matlab/namespace_facts.json`) and `MALT_MATLAB_EXTRA_PATHS` (an OS path list of local folders, added in front of the fixture, each with its plain subfolders as `genpath` lists them). The test without `MALT_MATLAB_FACTS` runs the very same script, so both ways produce the same facts.

The fixture is `tests/matlab/project`: argument forms (`~`, `varargin`, `varargout`, arguments blocks), scripts with local functions, packages (`+a/+b`), an old-style class folder (`@oldcls`), `private/`, two folders that define one name (in front of the path) and one at the end, and classdef classes with each property and method attribute, events, an enumeration, handle and value inheritance and abstract members.

Accepted differences are in `tests/matlab/namespace_allowlist.json`. Each entry has a `reason` (an empty reason fails the test). It matches on the entity, the field (a glob, `|` separates several, `{class}` is the entity's class name) and optionally the MATLAB value (`matlab`, a glob) and a required file state (`requires`: `enumeration`, `p-file`, `arguments-metadata`). A difference that matches no entry fails the test. An entry that matches nothing is printed as a warning; the extra-project entries are unused on the fixture alone. The accepted groups:

- MATLAB's own additions: the implicit `empty` method of every classdef, implicit constructors (only where the index finds no declared constructor, `declares_constructor`), implicit methods of enumeration classes and of `matlab.unittest`/`matlab.perftest` test classes, `loadobj` of a UI component container;
- MATLAB's metadata: `varargin` and `varargout` as the only input and output of a method with an arguments block (the index reads the source), a constant property declared with the `MethodSetupParameter` attribute, and the `SetAccess` default of an enumeration class's property (`immutable`);
- MATLAB's resolution: a `.p` file next to the `.m` (MATLAB runs the `.p`, so its file and `nargin` are the `.p`'s); a method of a class folder inside a package (no name resolves it: `which('mlog.Logger.addMessage')` is empty); an old-style method under its dotted name (`which('oldcls.getv')` is empty, `which('@oldcls/getv')` is the file);

Real index bugs the test found, both fixed with unit tests in `crates/malt_index/tests/parse.rs`: abstract methods (a signature with no body) were not methods; `Access` with a class list (`{?Cls}`, `?Cls`) was `private` and is now `restricted`.

Measured on R2026b: the fixture alone gives 24 names, 43 accepted, 0 unaccepted. With the toolkit at `MALT_EXTERNAL_TOOLKIT` and the advanced-logger `advancedLogger` folder as `MALT_MATLAB_EXTRA_PATHS` (204 names), 240 accepted, 0 unaccepted. The CI job `matlab` (Qualify) runs both phases, with `MALT_MATLAB_REQUIRE=1`.

## Live checkcode check

`matlab_expected.tsv` is a recording: it was true of R2026b when it was written. The live check asks a
running MATLAB the same question over the same fixtures and compares the answers, so it finds
expectations gone stale and MATLAB's reports drifting across releases. `matlab_conformance.rs` proves
malt equals the recording; this check proves the recording equals live MATLAB. Together, malt equals
live MATLAB.

Two phases, as for the path index. Generation: `tests/matlab/generate_checkcode.m` runs
`checkcode(file, '-id', '-struct')` (the default configuration, as `matlab_probe.m` does) over every
`.m` file under `tests/fixtures`, at any depth, in one MATLAB session (about 5 seconds for the 507
files on R2026b). It writes `target/matlab/checkcode.tsv` in the format of `matlab_expected.tsv` (path
relative to the fixtures folder, line with 0 read as 1, ID) and `target/matlab/checkcode.release`
(`version('-release')`, beside the TSV). A file `checkcode` fails on becomes a `PROBE_ERROR` row, so
it is a difference. Beside the TSV it also writes `checkcode_messages.tsv` (MATLAB's message text per
finding, MathWorks content: under `target/` only, never committed) and `checkcode_files.txt` (every file
checked). `MALT_CHECKCODE_OUT` changes the TSV's location; the two other files go beside it.

Comparison: `tests/matlab_live.rs` reads the TSV named by `MALT_MATLAB_CHECKCODE` (MATLAB is not
started), or, without it, finds MATLAB (`MALT_MATLAB`, else `matlab` on `PATH`) and runs the same script
in a temporary directory. Without MATLAB it prints `SKIP`; `MALT_MATLAB_REQUIRE=1` fails. Two tests:

- **Release**: the release that wrote the live file must be the one in `tests/fixtures/matlab_release.txt`
  (`R2026b`). A different release fails with its own message (`expectations recorded with R2026b, live
  MATLAB is R2025b: re-record or pin the CI release`). Reports are expected to drift across releases, so
  this is not a bug in malt; it means the recording has to be re-made, or the CI release pinned again.
- **Files**: the files MATLAB checked (`checkcode_files.txt`, `MALT_MATLAB_CHECKCODE_FILES`) are the files of
  `fixtures/matlab_files.txt`. A fixture with no finding is checked too, so it must be listed.
- **Messages**: for every finding both sides report, the message text is the same: malt's message and
  MATLAB's (`checkcode_messages.tsv`, `MALT_MATLAB_CHECKCODE_MESSAGES`), after `normalise_message` (tabs and
  line breaks as spaces, trimmed). A message repeated on one line counts once. Quoting is part of the text:
  MATLAB quotes the names in FNDEF, MCFIL, PFRUS, REDEF and similar messages, and not in INUSD, NODEF,
  PREALL or SVNODEF. The accepted differences are in `tests/matlab/message_allowlist.tsv` (path, line, ID and a
  reason); an entry that matches no difference fails. Only SYNER is accepted (its token, see Known gaps).
- **Reports**: the live (path, line, ID) findings equal the recorded ones. The failure lists the
  findings recorded but no longer reported (`-`) and the new ones (`+`), counted per check ID and per file,
  then the first 40. A `-` is a stale expectation or a fixture change; a `+` is either a stale expectation
  or a change in MATLAB. Neither is accepted silently: fix the recording (see below) or the fixture.

Run it locally, from the repository root, with MATLAB on `PATH`:

```sh
matlab -batch "run('crates/malt_cli/tests/matlab/generate_checkcode.m')"
MALT_MATLAB_CHECKCODE=target/matlab/checkcode.tsv \
MALT_MATLAB_CHECKCODE_MESSAGES=target/matlab/checkcode_messages.tsv \
MALT_MATLAB_CHECKCODE_FILES=target/matlab/checkcode_files.txt MALT_MATLAB_REQUIRE=1 \
  cargo test -p malt-cli --locked --test matlab_live -- --nocapture
```

Or `cargo test -p malt-cli --locked --test matlab_live -- --nocapture` alone: the test starts MATLAB
itself. CI (the `matlab` job of Qualify) runs the generator in its own `matlab-actions/run-command` step
after the path-index generator, then the test with `MALT_MATLAB_REQUIRE=1`. The release is checked on
every run there, so the CI release pin (`release: R2026b` in `setup-matlab`) and `matlab_release.txt` must
agree.

Licensing: on a public repository only the matlab-actions `run-*` actions license MATLAB in CI, so MATLAB
runs only inside `run-command` there, and the test reads the file. Locally, `matlab -batch` works with a
licence that the machine holds.

When the check reports differences, read the report first and decide whether the fixture or MATLAB
changed. Then bring `matlab_expected.tsv` in line with the live rows in `target/matlab/checkcode.tsv`
(path, line, ID; the `-` and `+` lines of the report are the rows to remove and to add). A change of
release is a separate change: update `matlab_release.txt` and the header of `matlab_expected.tsv`
together, after a live run on that release. No finding difference is allowlisted (message text has its own list, see 'Messages').

## MATLAB configuration file (plan step 5.8)

What MATLAB R2026b does with `resources/codeAnalyzerConfiguration.json`, probed with
`matlab.codeanalysis.validateConfiguration` (about 270 files), `codeIssues` and
`checkcode(file, '-id', '-config=<path>')`. Facts only: key names, enum values and the
numbers that the messages give. The probe files and the message catalogue are not kept
in the repository.

### Format, as probed

- **Discovery.** `codeIssues` reads the file of the file's folder or the closest parent
  folder with a `resources` folder: a file in a subfolder without its own configuration
  inherits the parent's. `checkcode(file)` reads no configuration (the factory checks
  run even beside a disabling file). `checkcode(..., '-config=<path>')` overrides a
  folder configuration. `validateConfiguration` accepts any path; outside `resources`
  it only adds a `NotVisible` note.
- **Top level.** `schemaVersion` is optional and a string `M.m.p`: `1.0.0` and `1.1.0`
  are accepted, `1.9.0` gives a warning, `2.0.0` and a number are errors. `naming` is
  applied under `1.0.0` too. `baseConfiguration` is `factory` or `closestParentFolder`;
  absent, it is `closestParentFolder` (a child without it keeps the parent's disable,
  `factory` ignores the parent, and a child can re-enable a parent-disabled check with
  `enabled: true`). The fields of a check merge: a child's `severity` alone does not
  re-enable a disabled check. An unknown top-level key is a warning.
- **`ignoredFiles`.** A string or an array of strings, relative to the folder that holds
  `resources`. `..` is an error, a leading `/` is an info (stripped). `sub/*.m` skips the
  files of `sub` only; `s1.m` skips the top-level file only. `codeIssues` honours it
  (also for a file named explicitly); `checkcode` does not.
- **`checks`.** Each value is an object (`false` is an error). `enabled` is a boolean.
  `severity` is `"error"`, `"warning"` or `"info"`, case-sensitive: `"warn"` and `"Error"`
  are warnings and are ignored (the check keeps its severity). `limit` is a nonnegative
  integer (`"x"`, `-1`, `1.5`, `true` are errors). `messageText` is accepted. Any other
  field is a warning. An unknown check ID is a warning. `codeIssues` reports the severity
  of a finding from the file: `error` gave an error, `warning` a warning.
- **Thresholds.** `FCNLL`, `CYCCOM` and `MNCSN` report with a `limit` (`FCNLL` 3 lines
  and `CYCCOM` 2 flagged the expected functions; a large function with no `limit` gave
  nothing). `enabled: true` without a `limit` reports nothing (probed on a 2100-line
  function, a 300-branch function and a 300-branch script). `limit` on a check without a
  threshold is a warning (`CABE`).
- **Disallow checks.** `DAFSC` (script) and `DAFAF` (anonymous function) report with
  `enabled: true`. `SYSBANG`, `DAFTC`, `DAFPV`, `DAFCO`, `DAFBR`, `DAFRT`, `DAFCF`, `DAFNF`,
  `DAFCV`, `DAFCVC`, `DAFVI`, `DAFVO` are mapped by their message text in
  `crates/malt_core/src/matlab_config.rs`, not probed.
- **`naming`.** A group object for each of `class`, `enumeration`, `event`, `function`,
  `localFunction`, `method`, `nestedFunction`, `property`, `variable` (another key is a
  warning). A check is a plain value, not an object: `maxLength`/`minLength` an integer
  1 to 2048 (0 and 2049 are warnings; `"5"`, `-1` and `5.5` errors), `regularExpression`
  a string (an invalid pattern is a warning, `""` an error), `casing` one of the six
  styles (`camelCase`, `PascalCase`, `snake_case`, `UPPER_CASE` are warnings). The affix
  checks are `requiredPrefix`, `disallowedPrefix`, `disallowedPhrase`, `requiredSuffix`,
  `disallowedSuffix`: a non-empty string or a non-empty array of strings (an empty array,
  a number or a mixed array are errors). Verified in `codeIssues` and `checkcode`:
  `maxLength` 6 flagged `longVariableName` (16 characters) and `v_count` (7) on their
  lines; `casing` `lowerCamelCase` flagged `v_count`; `requiredPrefix` `v_` and
  `disallowedSuffix` `x` flagged the expected names; `regularExpression` `[a-z_]+` on a
  function name stayed silent.
- **Custom checks.** `checks.<name>.rule` with `template`. `functionCall` needs
  `functionNames` (a string or an array); `variableName` needs `variableNames`. Other
  template names are warnings; a template field missing is an error; a name that is not
  a MATLAB identifier, or that is a built-in ID, is an error; a `rule` without a
  template is an error. `functionCall` with `disp` reported an `error` at the call
  (verified; malt does not implement templates).
- **Invalid files.** Malformed JSON, a top-level array, a `checks` that is not an object,
  and a duplicate key are errors. `codeIssues` throws `Invalid Configuration file`;
  `checkcode -config` reports a `CFIG` at line 0 and analyses the file with the factory
  checks. The four "configuration file" IDs in the table above are therefore emitted by
  MATLAB only for `checkcode -config`; malt does not emit them.
- **Message texts** are not recorded here (MathWorks content). The message catalogue is
  `resources/CodeAnalyzer/en/caConfig.xml` in the MATLAB installation.

### What malt implements

| JSON | malt | Verified by |
| ---- | ---- | ----------- |
| `checks.<ID>.enabled` (any built-in ID) | drops that check's findings (`CheckSetting`, applied in the linter before the message limit) | `c01`, `c11`, `c15` |
| `checks.<ID>.severity` | the check's findings get that severity | `c02`, `c03`, `c13` |
| `checks.<threshold>.limit` with `enabled` | the `CUSTOM_CHECKS` parameter: `FCNIL` `max_function_inputs`, `FCNOL` `max_function_outputs`, `FCNLL` `max_function_lines`, `LLMNC` `max_line_length`, `MNCSN` `max_nesting_depth`, `CYCCOM`/`MCYCCOM` `max_cyclomatic_complexity`, `SCYCCOM`/`MSCYCCOM` `max_script_cyclomatic_complexity`, `ACYCCOM`/`MACYCCOM` `max_anonymous_cyclomatic_complexity` | `c04`, `c05`, `c06`, `c16` (`FCNLL`, `CYCCOM`; the rest mapped only) |
| `checks.DAF*`, `checks.SYSBANG` with `enabled` | the `disallow_*` flag of `CUSTOM_CHECKS` | `c10` (`DAFSC`, `DAFAF`) |
| `naming.<group>.<check>` | the parameters of `naming.<group>.<check>` (`max`, `min`, `pattern`, `prefix`, `suffix`, `phrase`, `style`) and `NAMING_ENGINE` on | `c07`, `c08`, `c09` |
| `ignoredFiles` | the CLI skips matching files (`crates/malt_cli/src/ignored.rs`) | `c14`, the `ignored_files_*` test |
| `schemaVersion`, `baseConfiguration` | validated; the base is always malt's `mathworks` preset | `c17` and the unit tests |

Malt's own TOML is layered over the JSON: per rule and per category key, the TOML wins;
check settings come from the JSON only. The precedence (`--config` JSON alone; `.malt.toml`
with `[lint] matlab_config`, JSON first; `.malt.toml` alone; `resources/...` in the
working directory; defaults) is in `docs/configuration.md`.

### MATLAB comparison

`crates/malt_cli/tests/matlab_config/expected.tsv` holds the recorded `codeIssues` output
of 20 cases (`c00` to `c19`): line, check ID and severity, no message text. The
`checkcode -config=` output agreed with `codeIssues` on IDs and lines in every case with a
readable configuration, except `c14` (`ignoredFiles`, which `checkcode` ignores). `c16` and `c17`
are refused by `codeIssues` and analysed with the factory checks by `checkcode`.
`tests/matlab_config.rs` runs malt on each case and compares, and a live variant (`live_matlab_agrees_with_malt_on_every_case`, with
the usual `common::matlab` skip) runs `codeIssues` now. Both pass, except for the known
gaps `c18` and `c19`.

### Gaps

- **Default-off checks outside custom checks and naming.** `checks.ERTAG.enabled: true`
  with `severity: error` made MATLAB report `ERTAG` on the line; malt reports nothing,
  because the engine keeps the check off under `mathworks` (`off_under_mathworks` in
  `malt_rules`). Closing it needs a per-check hook in the engines (a malt_rules change).
- **`MNCSN` position.** MATLAB reports the control statement that exceeds the limit
  (line 7 for a limit of 2 in `c19`); malt reports the function's line. A malt_rules fix.
- **Custom checks (`rule`), `messageText`, `formatting`.** Warned and ignored. A custom
  check's findings are missing from malt's output (`c18`).
- **Parent folders.** MATLAB searches the parents of the file's folder; malt reads one
  file per run (the working directory's, or `--config`).
- **Not verified against MATLAB.** The mapping of `LLMNC`, `FCNIL`, `FCNOL`, `SCYCCOM`,
  `ACYCCOM`, `MCYCCOM`, `MSCYCCOM`, `MACYCCOM`, `SYSBANG` and the other `DAF*` checks.
  Their message texts give the threshold; the probes did not cover them.
- **Earlier rows now superseded** (not edited here, as this section is appended): the
  `DEBUGMSG`/`LLMCC` and `BDCFG`/`CFERR`/`BDOPT`/`CFIG` rows of 'Official IDs malt does
  not emit', the paragraph 'Naming checks (Phase 2.4)' ("the schema ... comes from
  validateConfiguration messages") and the Phase 2 close-out line that defers the JSON
  configuration.

## Phase 4, rounds (2026-11-17)

Each round is ten new files of the session's own writing (scripts, functions, local and nested functions, classdefs with events and enumerations, a package folder, a GUI callback file, a deliberate syntax error in round 2), in 40 to 200 lines, drawn under `/tmp/probe/round<N>/`. MATLAB ran once per round over the set (`matlab_probe`), malt with the default preset, and the findings were compared per file, line and ID, then per message text. A round is clean only if the comparison showed no difference before any fix. The exit criterion (three consecutive clean rounds) is not met: round 1 is clean, rounds 2 to 4 are not, and the session stopped at four rounds.

| Round | Files | MATLAB findings | Before fixes | Result |
| --- | --- | --- | --- | --- |
| 1 | 10 (34 to 72 lines; two extended before the diff) | 33 | none (33 matched, message text identical) | clean |
| 2 | 10 (incl. `+c_geo/area.m`, `broken_if`) | 47 | MISS EXIST (`exist name` command form), MISS MINV (`M = inv(A); M*b`), EXTRA FNCOLND (`y(min(2, end))`) | fixed F1 to F3 |
| 3 | 10 (abstract base, image pipeline, nested config, dates, stats, tables, varargout, legacy, error flow) | 50 | MISS NASGU at `err = [];` before `try ... catch err` (`r3_error_flow.m:28`) | fixed F4, F5 |
| 4 | 10 (kernel, enumeration state machine, event bus, config parser, codegen, script harness, log rotation, error-heavy, grid plot, arguments block) | 42 | EXTRA SBTMP on `obj.Subscribers{k}(value)` | fixed F8 |

After the fixes all four rounds compare with zero differences. Round 2 to 4 are recorded as not clean. Round 1 was also re-run after its two short files were extended, before any fix.

Probes between the rounds (same procedure, minimal variants, one MATLAB call each) found the SPERR, catch-variable and throw cases that round 3 and 4 touched, and the SPERR identifier form. They are fixtures under `phase3/` (`nasgu_p4_*`, `sperr_p4_*`, `throw_p4_*`, `sbtmp_p4_*`, `exist_p4_*`, `fncolnd_p4_*`, `minv_p4_*`).

### Fixes (MATLAB evidence)

- **F1 EXIST, command form** (`performance/check_command.rs`). `exist foo`, `exist 'foo'`, `exist foo.bar` at statement level give EXIST in scripts and functions (probes ex02, ex03, ex05, ex06, ex08, ex11). `exist foo var` and `if exist foo` give none (ex04, ex10).
- **F2 MINV on a variable** (`performance/check_binary_operator.rs`). `M = inv(A); c = M*b;` and `c = b*M;` give MINV at the variable (column of `M`: 5 and 7; mi01, mi02, mi03, mi09 inside an `if`, mi13 after a later `M = 2`, mi07 for the direct form, mi08 twice). Silent for a copy `N = M` (mi05), a reassignment before the product (mi06), `pinv` (mi10), `(M)*b` (mi11), `.*` (mi14). The rule uses `reaching_values`, so anything it cannot follow stays silent.
- **F3 FNCOLND inside an index** (`good_practices/check_fncolnd.rs`). An `end` is the index of a variable when any enclosing argument list is a variable, field or brace index, however deep: `y(min(2, end))`, `y(foo(end))`, `s.data(min(2, end))`, `c{min(2, end)}`, `z(min(2, end))` (fn01, fn08, fn09, fn10, fn11). MATLAB reports only when no enclosing list belongs to a variable: `foo(min(2, end))` at column 16 and `min(end, 3)` at column 9 (fn04, fn07).
- **F4 NASGU and a catch variable** (`analysis/liveness.rs`, `try_statement`). `catch err` stores the error when the catch body starts, so a value of `err` before the try is never read there. The name is removed from the catch's live set (e1 to e4, e13, c2, c3, c5). `try` bodies that can finish normally still keep it live (c1, c4, c7).
- **F5 throw ends the flow** (`analysis/liveness.rs`). `throw(MException(...))` ends the flow like `error` (t1, t2, t6, t7, t8: NASGU for a store before it, UNRCH after it, as MATLAB). `throw(ME)` with a variable keeps the old rule (the scope is not analysed), and `obj.throw(...)` is a method, not a terminator.
- **F6 SPERR and SPWRN, identifier form** (`readability/check_error_warning.rs`). `error('a:b', sprintf(...))` and `warning('a:b', sprintf(...))` are reported at the call whatever the sprintf arguments, format only or not (s1, s3, s4, s5, s9, s10, i3, i8, i10, i13, i14, j1, j2, j6, j7, j9, j10, k1 to k5). The identifier is colon-separated components, each starting with a letter and holding letters, digits or underscores (`a_b:c`, `a1:b`, `a:B2`, `Mod:Sub:Id`). Not reported: `'Value %d'`, `'a:b c'`, `'1a:b'`, `'a:1b'`, `'a:_b'`, `'a.b:c'`, `'a:'`, `':b'` (i2, i4, i11, i12, i15, j3 to j5, s2, s6, s11). The unit test `sperr_plain_error_calls_do_not_fire` asserted `error('MyComp:myID', sprintf(...))` silent; MATLAB reports it (k1 to k3), so that case moved to the fires test.
- **F7 SPERR after an unconditional error** (`readability/mod.rs`). The `error` dispatch skips unreachable code, as `warning` already did (i18: MATLAB reports UNRCH, not SPERR, on the statement after an `error`).
- **F8 SBTMP on a cell index after a field** (`syntax_errors/check_call_syntax.rs`). `obj.Subscribers{k}(value);` and `s.f{1}(v);` are a cell index followed by a call and are silent; the chained call `s.f(1)(2)` is still SBTMP (b1, b4, b5, b9; b2 `c{k}(value)` was already silent).

### Known gaps found in these rounds (not fixed)

- **CTPCT for a non-identifier first argument.** `error('msg', x)`, `error('A_1:b-2', x)`, `error('a:', x)`, `error(':b', x)`, `error('1a:b', x)`, `error('a.b:c', x)` and `warning('msg', sprintf(...))` give CTPCT in MATLAB ("The format might not agree with the argument count"); malt is silent on most of them (i1, i4, i5, i6, i11, i12, i17). Those files are not fixtures.
- **SPERR with a non-literal first argument.** `error(['a:b'], sprintf(...))` gives SPERR (and NBRAK2) in MATLAB (i16); a parameter used as the identifier, `error(id, sprintf(...))`, gives SPERR too (i9, j8). malt is silent on both: the condition for a variable is not established (one shape each), and a bracket literal is not followed.
- **LTARG for a bare `exist` in a condition.** `if exist foo ... end` gives LTARG ("Function might be called with too few arguments.") in MATLAB, a zero-argument call (ex10); malt gives SEPEX only. The file is not a fixture.
- **`run` and a store before it.** `x = 1; run('foo.m'); x = 2;` gives NASGU for `x = 1` in MATLAB (t10). malt skips the scope because `run` is in `DYNAMIC_CALLS` and the reason for that is not known; not changed. Not a fixture.
- **`throw(ME)` with a variable.** MATLAB analyses the scope (t9: NASGU for `y = 1` in a try body whose catch rethrows `ME`); malt skips it, as before F5. Not a fixture.
- **MINV for a parameter operand.** `function c = f(M, b); c = M*b;` was not probed; malt is silent, which is the safe side.

## Phase 4, rounds 5 onwards (2026-11-17)

Same procedure as the rounds above: ten or more new files of the session's own writing, drawn under `/tmp/probe/round<N>/`, 30 to 75 lines each (round 6 has one deliberate syntax error), one MATLAB `matlab_probe` run per round, malt with the default `mathworks` preset (the diff script's own default is `all`; it was run with `--preset mathworks`), compared per file, line and ID and then per message text. A round is clean only if the comparison showed no difference before any fix. Probe sets were run once per fix, one MATLAB call each, 3 to 27 tiny files per hypothesis.

| Round | Files | MATLAB findings | Before fixes | Result |
| --- | --- | --- | --- | --- |
| 5 | 10 (queue simulation, polyline with nested functions, value class with operators, parfor sweep, bit checksums, log regexp, sensor file IO, timer panel, ODE and least squares, plugin registry with eval) | 44 | PFBNS extra at the `parfor` header (`parfor k = 1:numel(omegas)`, parallel_sweep:16); PFBNS text unquoted (MATLAB `'options'`); MISS DSPSP on `disp(msg)` with `msg = sprintf(...)` in a function (timer_panel:23); MISS NODEF on `days = days(d2 - d1)` (an output read before its assignment, timer_panel:40) | not clean; fixed F9 to F13 |
| 6 | 12: 11 fixtures, plus one deliberate syntax error (`parse_error`, not a fixture) | 39 | MISS PREALL and EXTRA NASGU on `out = zeros(...)` overwritten on every `switch` branch (legacy_matrix:16); the syntax-error file differs in parser recovery (known gap G1) | not clean; fixed F14 |
| 7 | 10 (struct helpers, interval merge, value class `Money`, package class `+retail/Order`, sloppy loops, memoised recursion with nested function, integration and fit, binary codec, table join, test script) | 46 | EXTRA SFLD and GFLD on `setfield(units.mass, ...)` and `getfield(units.mass, ...)` (a field-expression base; unit_convert:29, 31) | not clean; fixed F15 |
| 8 | 11: 10 fixtures, plus `r8_matrix_report` (not a fixture: known gap G2) | 41 | MISS LOGSUM on `sum(M(:, i) > 3)` (matrix_report:30, mathworks preset, gap G2); EXTRA NODEF on the output `obj` of a classdef constructor (`addlistener(obj, ...)`, config_gui_state:20) | not clean; fixed F16 |
| 9 | 11 | 33 | none (33 matched, message text identical) | clean |

Consecutive clean rounds: one (round 9). Rounds 5, 6, 7 and 8 are not clean, so the count restarted and the exit criterion (three consecutive clean rounds) is not met this session. Round 9 was run after the fixes of rounds 5 to 8 and needed none.

### Fixes (MATLAB evidence)

- **F9 PFBNS and the loop header** (`performance/mod.rs`, `check_broadcast_variables`). The header runs once on the client, so only the loop body is scanned for broadcast reads. `parfor k = 1:numel(v)` with `out(k) = v(k) * 2` in the body is silent (probes pfb_01, pfb_03, pfb_07); a whole read in the body still fires (pfb_02, pfb_04, pfb_05).
- **F10 PFBNS message text** (`performance/mod.rs`, the description and the rule docstring). MATLAB quotes the name: `The entire array or structure 'v' is a broadcast variable.` Live check: the message agrees.
- **F11 DSPSP on a variable** (`readability/check_disp.rs`, `sprintf_value_read_once`). In a function, `disp(s)` gives DSPSP when the only definition reaching it is `s = sprintf(...)`, and that value has no other use: no second read (a `disp`, `fprintf`, `t = s`, a bare `s` statement, a handle `@() s`), not an output or input, and nothing else reaches it (probes dsp_02, d2a, d2n, e07, e11, e16, e21, e25, e26, e22, d2f, d2j, d2m). Silent for two reads (d2b, d2d, e02), an output (d2k, e-series), a reassignment before the read (d2c), `sprintf` inside an array (dsp_06), a script (dsp_01, dsp_05, dsp_09, dsp_12), and a bare `s` statement after the `disp` (c04, c11). Fix: none (the value is built elsewhere).
- **F12 NODEF for an output read before its assignment** (`analysis/maybe_defined.rs`, `analyze_may_defined`). Only the inputs are defined on entry; outputs are not. MATLAB reports `y = y + x`, `disp(max(y, x))`, `numel(y)`, `y.field`, `y(1)`, and a read on the branch that does not assign (`if x > 0 y = 1; else y = y; end`) (o1, o2, o4, o5, o9, o10, o11-o12 for locals, f5, f9, nod_01, nod_02, nod_05, nod_06, nod_08). Silent for a read after a conditional assignment (f2, the USENS path is unchanged) and for `y(2) = x`, `y(end+1) = x`, `r(3) = v` (o13, g2, c12).
- **F13 NODEF for a call or index of a name the function assigns** (`analysis/maybe_defined.rs`, `collect_called_names`; `unused_variables/check_function_scopes.rs`). A name the function assigns is a variable everywhere in it: `k = w(1) + v` before `w = [1 2]`, `size(v)` before `size = 2`, `c(1)` before `c = {v}`, `min(v, 2)` before `min = 3`, `x = y(1)` in an `if` before `y` is assigned are NODEF (c01, c02, c03, c06, c07, c08). Silent when the name is never assigned (`size(v)` alone, `v(1)` with `v` an input). The superclass constructor call `obj@Base(...)` is not a read (corpus1 c_events_class, corpus4 l_circle).
- **F14 PREALL on a dead preallocation** (`unused/check_nasgu.rs`, `unused/check_preall.rs`). A dead preallocation that a later full assignment of the same name overwrites on some path is PREALL, not NASGU, in any block: every `switch` branch with `otherwise` (pa2, pa8, pa12, legacy:16), `if`/`else` (pa1, pb6), `try`/`catch` (pa10), one branch only (pb2), a dead value read after the branches (pa7). A preallocation nothing overwrites stays NASGU (pb1); a read before the overwrite is live and silent (pa3, pa5); a conditional overwrite of a live value is silent. The same-block rule (`is_overwritten_preallocation`) is unchanged. Unreachable code is still silent.
- **F15 GFLD and SFLD argument shape** (`performance/check_function_call.rs`, `fieldfn_applies`). Only the single-level form on a variable: `getfield` with two arguments, `setfield` with three, and a struct argument that is a variable or an index of one (`s(2)`, `c{1}`). Silent on a field expression (`u.m`, `s.inner`; sf2, sf7, gf2, g7, g10, g13), the path forms (`getfield(s, 'a', 'b')`, `setfield(s, 'a', 'b', 2)`; gf6, g4, g5, g11), and a base that is not a variable (`getfield(f(1), 'a')`; g2). A name the scope assigns anywhere counts as a variable, even when the call comes first (`v = getfield(s, 'a')` with `s = ...` on the next line: GFLD, fix/performance__t2 and getfield_p8).
- **F16 NODEF in a classdef constructor** (`unset_variables/check_function_scopes.rs`). The output of a classdef constructor is the object, created before the body: a read of it is not NODEF (probed as for STOUT; `addlistener(obj, ...)` before any assignment, and the r8 fixture).
- Also changed: `varargout` is a variable when it is an output (`check_function_scopes.rs`): `varargout{1} = varargout{1}` gives NODEF (f6). An input `varargout` is still defined on entry.

### Known gaps found in these rounds (not fixed)

- **G1, the parser's recovery after an unclosed `[`** (`r6_parse_error`, not a fixture). `weights = [0.2 0.5 0.3;` on line 9, then ordinary statements: MATLAB gives SYNER on the first `=` after the break (line 10), NOPTS on the statements after it (lines 11 and 12). malt gives ENDPAR at the `[` (9:11) and SYNER on lines 10, 11 and 12. The recovery is in tree-sitter's error tree (the fork, not to be edited here); no rule-level change was made.
- **G2, LOGSUM under the `mathworks` preset, indexed operands** (`r8_matrix_report:30`, not a fixture). MATLAB reports `sum(x(:) op c)` and `sum(x(:, k) op c)` with a numeric literal `c` (`sum(M(:, i) > 3)`, `sum(v(:) > 3)`, `sum(v(:) <= 1)`, `sum(v(:, 2) > 0.5)`, `sum(x(:) <= 1)` in a function). It is silent on `sum(v > 3)`, `sum(v(1:3) > 3)`, `sum(v(2) > 3)`, `sum(diff(v) == 1)`, `sum(v(:) > w(:))`, `sum(s.a > 3)`, `sum(v(:) > 3, 1)`, `sum(v(v > 3))` (probes r8l, MATLAB 2026b). malt's LOGSUM is off under `mathworks` (the preset is in malt_core, which this phase does not touch). Under `all` malt reports every shape above, including the silent ones, so turning it on would add false positives; the shape needs a rule that is on under `mathworks` only for this form.
- **G3, NODEF for a function name used as a variable through a nested call inside a script.** Not probed. Scripts give no NODEF on a variable read before its assignment (`total = total + 1` in a script: silent in MATLAB, malt silent), which is consistent with the rules; the nested-call shape in a script was not checked.
- **G4, PREALL where a dead value is overwritten only inside an unreachable branch.** Not probed; malt reports nothing there.

### Gates

- `cargo fmt --all`, `cargo clippy --all-targets --locked -- -D warnings`: clean.
- `MALT_MATLAB_REQUIRE=1 cargo test --workspace --locked --no-fail-fast`: every test passes, including the conformance test (753 fixtures) and the live check (2795 recorded findings agree with MATLAB 2026b, 0 message differences, 20 accepted). `MALT_EXTERNAL_TOOLKIT` was not set: `/tmp/probe/corpus5_src/repo` does not exist on this machine.
- `bun docs/scripts/gen_rules_docs.ts --check`: passes after `bun run docs:gen` (the PFBNS text in `docs/rules/performance.md`).

## Phase 4, rounds 10 onwards (2026-11-17)

Same procedure as the rounds above: ten new files of the session's own writing per round, drawn under `/tmp/probe/round<N>/`, one `matlab_probe` run per round, malt with `--preset mathworks`, compared per file, line and ID and then per message text. Round 10 was classdef-heavy (property validation, dependent properties, events, enumeration with constructor arguments, abstract base, a package class, operator overloading and `subsref`, a `matlab.unittest` class, `inputParser`, an `arguments` block, `containers.Map`, handle copy semantics); round 11 was old-style and sloppy code (globals, `eval`, `findstr`, `str2num`, date functions, growth in loops, `inv(A) * b`, GUI callbacks with a nested function, `ode45` with events, error handling, `persistent` caches). Class and function names carry the fixture's letter prefix in the fixtures (`a_temperature_sensor`), as in the earlier corpora.

| Round | Files | MATLAB findings | Before fixes | Result |
| --- | --- | --- | --- | --- |
| 10 | 10 (`corpus14/`) | 5 | none (5 matched, message text identical) | clean |
| 11 | 10 (`corpus15/`; nine first, a tenth added before the diff) | 34 | none (34 matched, message text identical) | clean |

Consecutive clean rounds: three (rounds 9, 10 and 11), so the exit criterion of the phase is met and no further round was run. Round 10 found little to compare (well-written class code gives MATLAB five findings), so it is the weakest of the three; round 11 is the stronger test. No malt change was needed, so there are no new fixes and no new known gaps. The files joined the fixtures as `corpus14/` and `corpus15/` (20 files; 773 in all).

## 5.2 DEFNU and callbacks (2026-11-17)

Whether the Code Analyzer treats GUIDE- and App Designer-shaped functions (`*_Callback`, `*_CreateFcn`, `*_DeleteFcn`, `*_OpeningFcn`, `*_OutputFcn`, `*_KeyPressFcn`, `*_ButtonDownFcn`, `*_ResizeFcn`, `*_CloseRequestFcn`, `startupFcn`, `*ButtonPushed`, `*ValueChanged`, `*CellEditCallback`) as used: does DEFNU fire on an uncalled callback, and INUSD/MANU on its typical `hObject, eventdata, handles` / `app, event` arguments. 40 tiny probe files of the session's own writing under `/tmp/probe/cb/` (GUIDE-style function files without `gui_mainfcn` boilerplate, scripts with local functions, classdefs with private callback methods, nested functions, minimal variants), one `matlab -batch` call over the set (`docs/scripts/matlab_probe.m`; the paired `.fig` for one probe was created in the same call), compared with malt per (file, line, ID) and message text.

| Case (probe file) | MATLAB IDs |
| --- | --- |
| Function file, uncalled `pushbutton1_Callback(hObject, eventdata, handles)` (a01) | DEFNU + 3x INUSD |
| Same, body reads `handles` (a02) or `hObject` (e05) | DEFNU + 2x INUSD |
| Uncalled `figure1_CreateFcn` (a03), `<gui>_OpeningFcn` with `varargin` (a04) | DEFNU + 3x INUSD |
| `varargout = <gui>_OutputFcn` (a05) | DEFNU + 3x INUSD + STOUT (`varargout` never assigned) |
| `_KeyPressFcn`, `_ButtonDownFcn` (a06); `_ResizeFcn`, `_CloseRequestFcn`, `_DeleteFcn` (a07) | DEFNU + 3x INUSD each |
| `startupFcn()` (a08) | DEFNU |
| `appButtonPushed`, `slider1ValueChanged`, `table1CellEditCallback`, `edit1ValueChanged` (a09) | DEFNU + 2x INUSD each |
| Plain `helper(a, b)` uncalled (a10) | DEFNU + 2x INUSD (the same shape: no exemption) |
| Callback called by name (a11) | no DEFNU; INUSD x3 (the body reads nothing) |
| `uicontrol('Callback', 'name')` (a12), `set(h, 'WindowKeyPressFcn', 'name')` (a13), `uibutton(f, 'push', 'ButtonPushedFcn', 'name')` (a16) | DEFNU + 2x INUSD: a callback STRING is not a use |
| `uicontrol('Callback', @pushed)` (a14), `@(src,evt) mycb(src,evt)` (a15) | 2x INUSD: a handle or an anonymous call IS a use |
| Script local function `pushbutton1_Callback` (b01), plain `helper` (b02) | DEFNU + INUSD x3/2, as in a function file |
| classdef, `methods (Access = private)`, `%`/`%% Callbacks that handle component events` header, `appButtonPushed(app, event)` using `app` (c01, c02, c07) | INUSD for `event` only: methods are outside DEFNU, and the header exempts nothing |
| Same with `app` and `event` both unused (c03) | INUSD x2 |
| Private `startupFcn(app)` (c04), plain `helper2(app)` (c05) | MANU both: no `startupFcn` exemption |
| Callback referenced as `@app.pushed` (c06); full App Designer shape with `createCallbackFcn` and a `matlab.apps.AppCompatActivityBase` superclass (c08) | INUSD for `event` only |
| Nested `pushbutton1_Callback` (d01), plain nested `helper` (d02) | DEFNU + INUSD x3/2 |
| GUI-named file with a paired `.fig` (e01), any other file name (e02) | DEFNU + 3x INUSD: a paired `.fig` and the file name change nothing |
| `%% Callbacks that handle component events` header in a function file (e03) | DEFNU + 3x INUSD |
| `handles.count = handles.count + 1;` in the body (e04) | DEFNU + 2x INUSD, no NASGU for the store |
| Plain `helper(hObject, eventdata, handles)` (e06) | DEFNU + 3x INUSD: the argument names are not exempt either |
| Callback-named + plain subfunctions, both uncalled (e07) | DEFNU for both |
| Lowercase `pushbutton1_callback` (e08) | DEFNU + 3x INUSD |
| 62-character callback name (e09) | DEFNU + 3x INUSD, no LNGNM |
| Main function named differently from the file (e01, e10) | FNDEF, as for any file |
| `helper@app(app)` inside `appButtonPushed` of a `handle` subclass (e11) | MCSCM at the call (name mismatch) + INUSD for `event`: the `name@app(app)` form counts as a read of `app` |

**Established rule.** MATLAB has no callback exemption of any kind. A function that looks like a GUIDE or App Designer callback is analysed exactly like any other local function: DEFNU fires when it is never called, in function files, scripts with local functions and nested functions alike, whatever suffix or case its name carries; INUSD fires for each unread input, `hObject`/`eventdata`/`handles` no more exempt than `a`/`b`; MANU fires for a private `startupFcn(app)` like any method's sole input. Nothing silences them: not a paired `.fig`, not the file name, not a `%` or `%% Callbacks that handle component events` header (function file or classdef), not the privacy of the methods block. A use is a call by name, a handle `@f`, or an anonymous function that calls `f`; a callback registered as a STRING (`uicontrol`, `set`, `uibutton`) is not a use. Real GUIDE files are silent only through their `str2func`/`feval` boilerplate, which the existing `DYNAMIC_CALLS` rule in `check_defnu.rs` already models.

**Decision.** No malt change. Before any fix, malt reported exactly MATLAB's 88 findings on the 40 probes — identical per (file, line, ID) and message text — so there was no false positive to remove and no condition to gate on. The behaviour is pinned so a name-based exemption cannot be added later: unit tests in `unused/check_defnu.rs` (uncalled suffixes, the header comment, string vs handle registration), `check_unused_inputs.rs` (callback argument names, classdef callback methods) and `check_manu.rs` (`startupFcn`), each naming its probe, and the 18 fixtures `phase3/defnu_p52_*.m` with their MATLAB expectations appended (2874 recorded findings on 791 files, live check agreeing).

Known gaps:

- INUSA (an input of a callback-named function declared in an `arguments` block) was not probed; no exemption is expected, since INUSD and MANU have none.
- The paired-`.fig` probe covered one shape (a GUIDE-style function file whose stem matches the `.fig`). The `.fig` is binary and cannot be a fixture, so the fixtures pin the name-independent behaviour only.
- GUIDE's real `guidata`/`gui_mainfcn` boilerplate was not re-probed; its silencing is attributed to the already-modelled dynamic calls (`str2func`, `feval`), not to anything callback-specific.

## WASM parity (plan step 5.7, 2026-11-17)

The browser build (`malt-wasm`) must report what the CLI reports. Two checks keep it so:

- `crates/malt_wasm` has a host test that runs every conformance fixture (`matlab_files.txt`, 791 files) through the `Linter.check` the browser calls and compares the (file, line, ID) findings with the recorded MATLAB expectations. It found one real bug, now fixed: `new Linter()` and `new Linter(null)` built their configuration from a derived `Default`, which turned `%#ok` suppression off (extra AGROW findings, missing MSNU), while `new Linter({})` had it on. The default is now the same as an empty object, with a unit test and a smoke-test case.
- `wasm-pkg/smoke-test.mjs` runs the built package in Node on realistic cases with the IDs the CLI reports, covers the three constructor forms, `fix`, the file-name-dependent checks, a malformed TOML, and a 5,000-level nesting that must not trap.

One difference remains, on purpose: the wasm depth guard is 200 levels, the native one 1,000. The one fixture that nests 200 levels (`fix/incomplete__di200.m`) reports `MALT_TOO_DEEP` in wasm and is allowlisted in the host test. The guard stays low because of the measured stack: in the Node/V8 build a chain of about 750 binary operators (`x = 1+1+…`) traps (`memory access out of bounds`), while parentheses, calls, matrices, cells, `if`/`for` nesting, field chains and lambdas ran to 5,000 levels without a trap. Raising the guard to the native 1,000 is therefore not safe; with 200, every shape tested up to 5,000 levels ends in a diagnostic or a normal result, never a trap. Other engines (Safari, Firefox) have different stack sizes and were not measured.

## Resource limits

The `[lint.limits]` table of `.malt.toml` (`max_file_size`, `max_tree_depth`, `max_nesting`, `max_threads`; see [Resource limits](configuration.md#resource-limits)) was added after a pathological input made a malt process grow until the machine hung. The defaults come from these measurements: the release CLI (`target/release/malt`, one file per run, so one worker), each run under `timeout 60` and `ulimit -v 12000000`, inputs generated in `/tmp` (never committed), peak RSS from `/usr/bin/time -v`, 12 cores, WSL2. "n" is the nesting depth, the number of operands, or the megabytes. A run that hit the 60 s cap was killed and its peak is a lower bound.

| Input | n | File size | Peak RSS | Wall time | Result |
| --- | --- | --- | --- | --- | --- |
| `x = ((((1))))` | 8,000 | 16 KB | 12 MB | 0.01 s | MDEEP (MATLAB's own check) |
| same | 64,000 | 128 KB | 32 MB | 0.07 s | MDEEP |
| same | 256,000 | 512 KB | 103 MB | 0.25 s | MDEEP |
| `[[[[1]]]]` | 64,000 | 128 KB | 45 MB | 0.11 s | MDEEP |
| `{ { { 1 } } }` | 64,000 | 128 KB | 44 MB | 0.10 s | MDEEP |
| nested `if` / `for` in a function | 200 | 2 KB | 11 MB | 0.05 s / 0.28 s | analyzed |
| nested `if` / `for` / `while` / `try` / `switch` | 400 to 30,000 | up to 600 KB | up to 80 MB | up to 0.28 s | NOSPC (MATLAB's own check) |
| nested `@() @() …`, `a.b.b.b…`, nested functions | 30,000 | 60 to 650 KB | 29 to 95 MB | up to 0.28 s | MALT_TOO_DEEP or MATLAB's limit |
| `%{` nested blocks | 30,000 | 180 KB | 9 MB | 0.01 s | DEEPC |
| chain `1+1+1+…` | 400,000 | 800 KB | 212 MB | 0.55 s | MXASET |
| one line `x = [1 1 1 …];` | 1,000 | 2 KB | 10 MB | 0.02 s | analyzed |
| same | 100,000 | 200 KB | 142 MB | 1.0 s | analyzed |
| same | 1,000,000 | 2 MB | 1,045 MB | 13 s | analyzed |
| same | 4,000,000 | 8 MB | 3,630 MB | 60 s (killed) | not finished |
| ordinary statements, 10,003 findings | 1 MB | 1 MB | 218 MB | 2.5 s | analyzed |
| same | 4 MB | 4 MB | 789 MB | 10 s | analyzed |
| same | 8 MB | 8 MB | 1,447 MB | 29 s | analyzed |
| same | 16 MB | 16 MB | 2,549 MB | 51 s | analyzed |
| same | 50 MB | 50 MB | 7,616 MB | 60 s (killed) | not finished |
| empty lines | 1e5, 1e6, 1e7 | 0.1 to 10 MB | 9 to 19 MB | up to 0.54 s | analyzed |

What the table decides:

- **Nesting is cheap, size is not.** Parsing and the MATLAB limit checks cost about 0.4 KB of memory per level, linear to 256,000 levels (100 MB), and MATLAB's own checks (MDEEP, NOSPC, MXASET, DEEPC) answer for the nesting shapes before malt's analysis runs. The nesting bomb of the incident did not reproduce as a memory blow-up in the native CLI on any shape tried. What does scale is the input size: about 180 bytes of memory per input byte for ordinary statements (time super-linear, 2.5 s per MB growing to 3.5 s per MB) and about 500 bytes per byte for one huge matrix. Empty lines are nearly free.
- **`max_file_size`: 5 MiB native, 2 MiB wasm.** At 5 MiB the worst measured shape (single-line matrix) needs about 2.5 GB and a statement file about 0.9 GB and 15 s, which a CI runner survives; at 16 MB the statement file already needs 2.5 GB and 50 s, and 50 MB did not finish within 60 s at 7.6 GB. 5 MiB is far above real MATLAB files (the largest fixture is below 1 MiB; MATLAB's own MBIG would start at 16 MB and MATLAB never reports it). The browser tab gets 2 MiB because its memory is smaller and a tab that hangs is worse than a skipped file.
- **`max_tree_depth` stays 1,000 native and 200 wasm** (see 'WASM parity' above for why 200).
- **`max_nesting`: 10,000 native, 1,000 wasm.** A backstop that stops the parse before it starts: 10,000 levels cost under 15 MB to parse, so the default is generous, and it is above MATLAB's MDEEP limit (33) and the tree-depth limit so that moderately deep input still gets MATLAB's own message. The scan counts brackets only, because `end` is ambiguous before the parse (`x(end)`); a test scans every conformance fixture and refuses none at either default.
- **`max_threads`: the number of cores, as a total.** Before, each of the `cores` file workers also started `cores` check threads for a large file (all with 256 MiB of reserved stack), so one run could start cores squared threads. Now the workers and the per-file check threads share the budget (`workers * threads per file <= max_threads`); the memory of one run is bounded by the number of files in flight times the memory of a file. A budget of 1 runs on one thread. The many-worker memory sum was not measured (one file per run).

Not verified: the effect of `max_threads` on peak memory with several large files in flight, the wasm limits in a browser (the host test and the V8 depth measurement above stand in), and shapes other than those listed (for example thousands of long lines of nested anonymous functions).

## 5.5 Curated safe autofixes (2026-11-17)

The fixes were re-curated with MATLAB's own fix as the oracle: for every fixture
finding that MATLAB's Code Analyzer marks fixable, `codeIssues(file).fix(ID)` was
applied to a copy of the fixture and its text compared with malt's fix for the same
ID alone (the default configuration, that ID's fixes only, repeated until stable, as
`--fix` does). Method: one `matlab -batch` call per bounded chunk, `codeIssues(...,
CodeAnalyzerConfiguration="factory")`, `try`/`catch` per file, `Fixability` read per
finding (`auto` = MATLAB offers a fix), the fixed text saved only from the copies.
`checkcode` exposes no fixability; `codeIssues` is the only programmatic window onto
it, and its `fix` is what the editor's quick-fix does.

**Inventory.** 791 fixtures (the pre-5.5 set), 3,011 findings, 950 auto-fixable ones,
53 check IDs with at least one auto-fixable finding, 521 unique (fixture, ID) pairs.
Differential: 452 pairs byte-identical between malt's fix and MATLAB's, 61 where malt
has no fix (see the table), 7 deliberate differences, 1 pair where malt reports but
deliberately does not fix the assigned form (DSPSP).

**The table.** malt's fix class before → after, and whether MATLAB's own fix produces
the same text on the fixtures:

| ID | MATLAB fixable | malt before | malt after | Agreement |
| --- | --- | --- | --- | --- |
| NOPTS, NOPRT | yes (auto) | safe, `;` inserted before a following comma (`x = 1;, y = 2`) | safe, the comma separator is **replaced** (`x = 1; y = 2;`) | now identical (43+64 pairs); the old text was valid MATLAB but `checkcode` reports NOCOMMA on it (probed) |
| NOSEMI | yes | safe, deletes the `;` plus the blanks in front | unchanged (kept deliberately) | malt is **deeper**: `end;;` converges to `end` (MATLAB's fix leaves `end;`, which its own analyzer still flags); blank-taking differs on `y = 2; ;` (4 pairs) |
| NBRAK2 | yes | safe, replaced the whole bracket expression with the element text — **comments inside the brackets were lost** (found by the differential on `y = [a  % comment ...]`) | safe, strips only the two bracket tokens; inner text, comments and spacing kept | now identical (10 pairs) |
| NBRAK1, NOCOMMA, NCOMMA, ASGLU, FVINR, BADNE, TRIM2, RPMT1, MRPBW, ISCEL, ISCHR, STLOW | yes | safe rewrites | unchanged | identical on every pair |
| FLUDLR | yes | safe, `rot90(x, 2)` (spacing) | safe, `rot90(x,2)` | identical (1 pair) |
| DSPSP | yes | safe, statement form only | unchanged | malt fixes the statement form identically; MATLAB also fixes `w = disp(sprintf(...))` to `w = fprintf(...)` — which **assigns `fprintf`'s return count to `w`**, a behaviour change; malt keeps the conservative scope |
| FNDEF | yes (128 pairs, the largest) | no fix | **safe**: the declaration is renamed to the file name; the runtime resolves by file name, so the declared name is dead text (recursion, handles and calls all resolve as before); a header broken by a later syntax error is handled too (the name token carries the fix there) | identical on all 128 |
| MCFIL | yes (4) | no fix | **unsafe**: renames the `classdef` and every constructor to the file name in one edit; a method that was never the constructor becomes one, so constructing the object starts running it | identical text, unsafe by the reasoning above |
| DLABINDEX, DNUMLABS, DGOP, CAXIS, ISDIR, MLNT, DEGTORAD, RADTODEG, WRITEFIS, GCAT, GPLUS, LAB* | yes | no fix | **safe**, data-driven (`renames_call = true` in `suggested_improvements.toml`): the callee token is renamed, arguments untouched; the entries that need code changes (`-opengl`, `hold('all')`, `datetime(now, ...)`) carry no fix | identical (DLABINDEX 6, DNUMLABS 3, DGOP 1, CAXIS 2, ISDIR 2 pairs, probed shapes) |
| AND2, OR2 | yes (auto) | unsafe | unsafe; **one finding, one operator** (probed: MATLAB's AND2 fix on `if a & b | c` gives `if a && b | c`) | identical per finding; `(a | b) | c` differs because `codeIssues` reports a second, parenthesised finding that `checkcode` and malt do not (2 pairs, documented) |
| FNAN | yes | unsafe, no fix when a logical operator followed (`n == NaN | n > 3`) | unsafe, the rest is kept as MATLAB's fix keeps it (`if isnan(n) | n > 3`; the operator stays elementwise — MATLAB's OR2 fix is the one that doubles it) | now identical (4 pairs) |
| STREMP | yes | unsafe, `~isempty(strfind(a,b))` → `~(~contains(a, b))` (double negation, wrong anchor) | unsafe, the negation is rewritten whole (`contains(a,b)`), arguments reprinted verbatim | identical (5 pairs) |
| IJCL | yes | unsafe, `j` → `1j` | unsafe, `i` and `j` both → `1i` (what MATLAB's fix writes; `1i` and `1j` are the same constant) | identical (5 pairs) |
| STRCL1, STRCLFH, STRIFCND | **no** (manual) | unsafe rewrites | unchanged (kept: the `contains` rewrite returns a logical array where `isempty(strfind(...))` returned a scalar on cell input, so it is not provably equivalent) | MATLAB offers no fix; documented divergence |
| INUSD | yes | unsafe (`~`) | unchanged | identical text (75 pairs); unsafe stays: `inputname(k)` of a `~` input changes, and MATLAB's fix shares that risk |
| str2num→str2double, fliplr(sort(x)), the performance rewrites | **no** | unsafe / removed | unchanged | MATLAB offers no fix; kept unsafe where an equivalent exists, removed where none does (`deblank`, `strtrim`, `findstr`, `im2bw`, `x * 1`, `isa(x,'double')` — see `tests/unsafe_fixes.rs`) |

**Where MATLAB offers a fix and malt does not:** see 'Second pass' below; the first
pass listed 61 pairs in 21 IDs, of which 41 now have a fix.

**Regression tests.** `crates/malt_cli/tests/fix_matlab.rs` + `tests/fixtures/fix_matlab/`
commit MATLAB's fixed text for 77 pairs, two or three per check ID (`<fixture>.<ID>.m`;
only fixtures without a parse error, because a fixed copy of a syntax-error fixture
would need allow-listed SYNER messages): the test applies malt's fix for that ID
(default configuration, Safe and Unsafe) and requires byte equality;
a second test re-generates MATLAB's fixes (`tests/matlab/generate_fixes.m`, the same
two-phase skip/REQUIRE contract as the checkcode generator, `MALT_MATLAB_FIXES`) and
compares them with the committed ones, so the set cannot drift from live MATLAB.
The expectation files are lint-able and are part of the conformance fixtures
(`matlab_files.txt`, 879 files now). New fixtures for the changed and new fixes:
`fix/formatting__sepcomma.m`, `fix/formatting__noprtsep.m`,
`fix/readability__nbrak2fix.m`, `fix/readability__fludlrfix.m`,
`fix/goodpractices__fndefix.m`, `fix/languagespec__mcfilfix.m`,
`fix/suggested__renames.m`, `fix/bugs__fnanrest.m`, `fix/readability__strempneg.m`,
`fix/readability__ijclfix.m`, each with its MATLAB expectations appended
(2,953 recorded findings on the first-pass 811 files). The `fix_safety` harness covers them all; `--fix` on any
of them stays idempotent.

Known gaps found by the probes, not fixed: `codeIssues` reports an OR2/AND2 for the
parenthesised inner operand of `(a | b) && c` that `checkcode` does not (malt matches
`checkcode`, so its fix does not fire there); OR2 chained without parentheses
(`a | b | c`) has no fixture at the `checkcode` level; MATLAB's fix on the one-line
`if a & b | c` shape doubles only the reported operator, but `codeIssues` reports
both operators there, and its AND2/OR2 pair-fixes then produce `a && b || c`.

### Second pass (2026-11-18)

Method as above, on the same 521 (fixture, ID) pairs, now with a differential that applies
each ID's fixes alone, Safe and Unsafe, and compares with MATLAB's text; the shapes
were probed on small files (`codeIssues` on a copy, `fix(ID)`, bytes read back).

**Counts.** 521 pairs: **494 identical** (342 with a Safe fix, 152 with an Unsafe one),
**20 with no fix** (NUSED 9, ASSRT 3, LOGEMP 3, NOPRC 3, EQEFF 1, FVAL 1), **7 deliberate
differences** (NOSEMI 4 deeper, OR2 2 parenthesised-operand report, DSPSP 1 assigned form).
Before: 452 identical, 61 no fix, 7 differences, 1 DSPSP. The FVAL pair left is
`fix/compat__a.m`: `psd` is a variable there, MATLAB's fix writes `psd(1)` and would
index it; malt withholds the fix (a deliberate difference, tested).

**Decisions per ID** (MATLAB's text, then why):

| ID | Decision | Evidence and reason |
| --- | --- | --- |
| MSNU | **Safe** | Comment-only edit. One ID: the whole `%#ok<ID>` (the blank in front of it is left), or only `#ok<ID>` when a remark follows (`% note`); several IDs: the ID and the comma after it (the one in front for the last), blanks stay (`<NOPRT, AGROW>` leaves a blank in front of `AGROW`); bare `%#ok` goes whole. A list whose every ID is unused gets no fix (unprobed). 4 pairs + probes ms1-ms8. |
| FVAL | **Safe** | `feval('f', x)`, `feval(@f, x)` → `f(x)`: the callee replaces `feval`, the first argument goes with its comma and following blanks (`feval( @sin , 1 )` → `sin( 1 )`, `feval(@sin)` → `sin()`); `@pkg.fn` and `"a.b"` too. Same call: resolved at run time either way, arguments untouched, `inputname` the same (probed), outputs by context. Withheld when the name or its package is a variable/function of the file, and for a variable or anonymous-function first argument (no fix there). 8 pairs. |
| TWOCM | **Safe** | second comma deleted; `[1,,2]` is a syntax error in MATLAB (eval probed), so no running code changes. 1 pair + probes. |
| STRQUOT | **Safe** | `string('abc')` → `"abc"`, literal only; `''` → `'`, `"` → `""`, blanks inside the parentheses stay (probed). |
| FORMATNOI | **Safe** | `format` → `format("default")`; the documented meaning of the bare command; not offered when `format` is shadowed (engine rule). |
| ISMT, ISCL | **Unsafe** | `length(x) == 0` / `0 == length(x)` → `isempty(x)`, `~= 0` and `> 0` → `~isempty(x)`, `if length(x)` → `if ~isempty(x)` (`~~` under a `~`), `x & b` shapes keep the operator; `length`/`numel(x) == 1` → `isscalar(x)`. Identical on 10 pairs and probes; a class can overload `length`, `numel`, `isempty`. |
| EFIND | **Unsafe** | `, 1` inserted before the `)` of `find`; identical on 4 pairs. MATLAB's fix ignores a local `function find`, so malt withholds it there; a class can overload `find`. |
| FNDSB | **Unsafe** | `x(find(L))` → `x(L)`, `idx = find(L)` → `idx = L`; `L` must be logical. 3 pairs. |
| STCI, STNCI | **Unsafe** | strcmp → strcmpi / strncmp → strncmpi and the first `lower`/`upper` call is removed; with two conversions only the first goes (`strcmpi(a, lower(b))`). Case folding differs for non-ASCII text. 3 + 1 pairs, probes sn1, sc2. |
| LOGL (zeros/ones) | **Unsafe** | `logical(zeros(n, 1))` → `false(n, 1)`, `ones` → `true`; `zeros(2, cls)` with a class name is a valid call whose `false(2, cls)` is an error (probed). |
| TNOW2, HOLDALL, PRTOG, PRTPT | **Unsafe** | `datetime("now")`; `hold on` (same in R2026b, probed: colour order and line count; not before R2014b); `-image`, `-vector` (argument only, command and call form). Text identical, behaviour documented rather than proven. |
| NUSED | **no fix** | MATLAB's text is ragged: only the unused names go (`global a b c` with `b` used keeps the blanks around the removed names), a whole statement goes with its terminator but its indentation stays (`  ` + next line), and a trailing comment on the declaration line is deleted. Copying that is not a fix to ship; also `eval`, `exist` and `who` can see the declaration. 9 pairs. |
| ASSRT | **no fix** | `assert('msg')` → `error('msg')`: the assertion passes silently for a nonempty char (probed), the replacement always throws. |
| EQEFF | **no fix** | `a == 1;` → `a = 1;`: turns a no-op into an assignment by guess. |
| LOGEMP | **no fix** | `isempty(x == 1)` → `~any(x == 1, "all")`: the result differs exactly in the case that is the bug; an intent-based rewrite. |
| NOPRC | **no fix** | adds `...`, which joins the next line into the statement (`y = 3` + `-x` becomes `y = 3 - x`): a behaviour change by design. |

**Regression set.** `fixtures/fix_matlab/` holds 77 pairs now (10 before); `fix_matlab.rs`
compares Safe and Unsafe fixes. `fix/secondpass__msnu.m` joins the `fix_safety` harness
(the MSNU shapes) and the 77 fixed texts are conformance fixtures (879 files, 4,037
recorded findings on them). `tests/second_pass_fixes.rs` pins the Safe shapes and what is
withheld; `tests/unsafe_fixes.rs` the new Unsafe ones.

**Detection gaps.** `codeIssues` reports an OR2/AND2 on the parenthesised inner operand of
`(a | b) && c` that `checkcode` does not. It only affects two OR2 pairs (listed above), does
not change malt's `checkcode` parity and needs no action. malt's MSNU does not report a
directive naming an ID that malt switches off by policy (NOPRT under the default preset):
MATLAB does, so the fixtures use active IDs for MSNU.

## Rule maturity (plan 6.3)

Every check is `stable` or `experimental` (metadata, `malt_core::Maturity`). The classification is
one table, `crates/malt_rules/src/data/maturity.toml`, with a reason per entry; everything not
listed is stable. `malt_rules::maturity` reads it (`engine_maturity`, `check_maturity`,
`experimental_reason`), and `docs/scripts/gen_rules_docs.ts` renders the Maturity column of
`docs/rules.md`, the per-check maturity and the `## Maturity` section of each rule page, and the
count in the check-inventory paragraph.

**Rule.** STABLE: MATLAB's Code Analyzer reported the check on a recorded fixture
(`matlab_expected.tsv`) or in a recorded configuration case (`tests/matlab_config/expected.tsv`),
or the check is part of the default analysis of the `mathworks` preset, which the conformance suite
and the live checkcode parity cover as a whole. EXPERIMENTAL: MATLAB reported it nowhere that was
recorded and it is off under `mathworks`: verified against the documentation index only, silent
in the probes, or low-reliability. So a default-on check is never experimental, and the default
output cannot change.

**Behaviour.** `[lint] experimental = true|false` (default `false`; `docs/configuration.md`,
'Experimental checks'). The experimental checks are the ones that are off by default already
(rule default off, or the `mathworks` off-list, low-reliability and gate entries): the flag selects
the experimental variant of the preset (`PresetDef::with_experimental`, the same preset with
`enables_default_off` on and its disabled categories unchanged), so every engine that reads
`preset.enables_default_off` honours it without a change. `--preset all` is unchanged (it already
runs everything), explicit per-rule and per-category entries still win (also `"off"` over the flag),
`naming` and `custom-checks` stay opt-in by category. `Config::set_preset` keeps the flag over a
`--preset` override; the WebAssembly `LinterConfig` has an `experimental` field and
`Linter.from_toml` reads the key. An invalid value is a parse error (exit code 2).

**Evidence method.** `bun docs/scripts/gen_rules_docs.ts --maturity-report` counts, per engine, the
unique check IDs, how many the table marks experimental, how many appear in a recorded MATLAB run
(`matlab_expected.tsv`, 1,313 IDs since 'Recording the stable checks', plus the 19 recorded configuration cases), and the stable ones without a
recording. `--check` fails when an experimental check has a recorded MATLAB finding or when the table
names an unknown engine or ID. A Rust test (`maturity::tests::default_output_has_no_experimental_check`)
fails when the default output reports an experimental ID.

| Engine | Checks | Stable | Experimental | Recorded by MATLAB | Stable without a recording |
| --- | --- | --- | --- | --- | --- |
| `BUGS_ENGINE` | 33 | 26 | 7 | 26 | 0 |
| `CODEGEN_ENGINE` | 14 | 0 | 14 | 0 | 0 |
| `COMPAT` | 1832 | 709 | 1123 | 709 | 0 |
| `CUSTOM_CHECKS` | 25 | 5 | 20 | 5 | 0 |
| `DEPLOYMENT_ENGINE` | 10 | 0 | 10 | 0 | 0 |
| `FORMATTING_ENGINE` | 6 | 6 | 0 | 6 | 0 |
| `GOOD_PRACTICES_ENGINE` | 106 | 91 | 15 | 91 | 0 |
| `INCOMPLETE_ANALYSIS` | 10 | 9 | 1 | 6 | 3 |
| `LANGUAGE_SPEC_ENGINE` | 161 | 148 | 13 | 141 | 7 |
| `NAMING_ENGINE` | 81 | 4 | 77 | 4 | 0 |
| `NOSEMI` | 1 | 1 | 0 | 1 | 0 |
| `PERFORMANCE_ENGINE` | 40 | 38 | 2 | 38 | 0 |
| `READABILITY_ENGINE` | 35 | 31 | 4 | 31 | 0 |
| `SUGGESTED_IMPROVEMENTS` | 241 | 209 | 32 | 209 | 0 |
| `SYNTAX_ERRORS_ENGINE` | 50 | 50 | 0 | 38 | 12 |
| `SYSTEM_OBJECTS_ENGINE` | 9 | 0 | 9 | 0 | 0 |
| `UNSET_VARIABLES_ENGINE` | 5 | 3 | 2 | 3 | 0 |
| `UNSUPPORTED_ENGINE` | 13 | 0 | 13 | 0 | 0 |
| `UNUSED_ENGINE` | 15 | 13 | 2 | 13 | 0 |
| **Total** | 2687 | 1343 | 1344 | 1321 | 22 |

Experimental groups (reasons in the table file):

| Group | IDs | Why |
| --- | --- | --- |
| Coder, Compiler, System Object, Unsupported engines | 14 + 10 + 9 + 13 | `checkcode` never emitted one in the probes; documentation index only; engines off by default |
| Naming, except `naming.variable.casing`, `maxLength`, `requiredPrefix`, `disallowedSuffix` | 77 | MATLAB reports naming only through a configuration; only these four have recorded cases (c07 to c09) |
| Custom checks without a recorded case (everything but FCNLL, CYCCOM, MNCSN, DAFSC, DAFAF) | 20 | mapped from message text, not compared with a live configuration |
| `mathworks` off-list (SHOCIRF, MOCUP, ERTAG, WNTAG, CTCH, ..., VANUS, NOEFF, SUSENS, USENS) | 26 | MATLAB's default analysis reported none in 4 or more probe shapes each |
| Forward-compatibility gates and IDISVARHIGH | 8 | never reported by default |
| Gated off, not reproduced (SHVAI, SHVAU, LNGNM, FVATF, CTOINE, MWKCL) | 6 | silent in 11 to 35 probes each |
| Compatibility entries: low-reliability (MATLAB's own list: 897, plus 17 generic checks MATLAB was silent on), `object`, `unverified` pattern | 914 + 183 + 13, as unique IDs | low-reliability flag; `object` needs class tracking; `unverified` silent in every probe |
| Suggested-improvement entries with `pattern = "unverified"` or `"object"`: 23 IDs, plus 9 never matched (`MASSO`, `IMSSO`, the seven `RISK*`) | 32 | the 37 `unverified`-pattern entries counted earlier were 13 compatibility IDs and 24 suggested-improvement entries; the second set was never in the table and is listed by ID now |
| MATLAB silent on every probed shape malt reported (CMDAND, CMDOR, DEFSIZE, FUNFUN, MEXCEP, MCHDT, SUBSINDEX, PFCEL, PFFRNG, PFMLTI, STCCS, NCHKE) | 12 | see 'Recording the stable checks' |
| Declared, never emitted (FEGLO, ROS\*, RMWRN, TMMSG, ATNAS, ATNPI, ATNPP) | 7 | no code path, empty denylist or unreachable arm; ROS needs a toolbox that is not installed |

**Counts.** 2,687 unique IDs in the engine tables (the 2,692 of the Rules Reference add the IDs emitted outside the engines): 1,343 stable and 1,344 experimental. Of the 1,343 stable IDs 1,321 have a
recorded MATLAB finding; 22 are stable without a recording (before 'Recording the stable checks':
1,413 stable, 301 recorded, 1,112 without a recording). The IDs that are declared but never emitted
(the code-gated ones such as `PRTCAL` or `XCREDS`, listed in 'Official IDs malt does not emit') are
not all in the table: only the ones with an engine entry or a listed ID are there.

**Verified by the byte-identity run.** A release binary of HEAD (before the change) and the new one
gave identical output over all 879 fixture files for the default, `--preset all` and
`--preset recommended` runs (4,221, 5,161 and 2,690 lines), and `experimental = true` with
`preset = "all"` equals `all`; the flag alone adds findings only.

## Dogfooding (plan 6.2, 2026-11-18)

malt run on real MATLAB code, with the default (`mathworks`) preset and with `--preset all`, and the default run compared with MATLAB R2026b's `checkcode` file by file (`matlab_probe.m` in one `matlab -batch` call of at most 170 files, then `diff_against_matlab.ts --preset mathworks` per (file, line, ID), then the message text). The code is read-only input under `/home/watermarkhu/repositories` and `/tmp/probe/corpus5_src`; none of it is copied into the repository. The fixtures of this step are the session's own minimal reductions (`corpus16/`, 20 files).

| Codebase (names only) | .m files | Role |
| --- | --- | --- |
| OpenTelemetry-MATLAB (`maxx/submodule`) | 143 | MathWorks-style OOP code, test classes, packages |
| advanced-logger (`maxx/submodule`) | 35 | classes, packages, tests (diffed together with mkdocstrings-matlab/test as `lm`, 50 files) |
| matlab (matlabdomain, textmate, sphinx-matlab, ...) | 170 | parser and documentation test data: many deliberate syntax errors |
| mkdocstrings-matlab/test | 15 | class folders, package folders |
| matlab-agentic-toolkit (re-cloned, depth 1) | 145 | current MathWorks skills code |
| gnu-octave-statistics (every fourth file) | 116 | Octave code: robustness input only. MATLAB reports a syntax error (`endfunction`, `#` comments, ...) in all 116 files, so there is no meaningful diff; malt does not crash or stall |

`zed-matlab` holds no `.m` file. Times are for one `malt` call over the codebase, release build: 12 to 640 ms (default) and 13 to 740 ms (`all`); the largest input is 2.6 MB over 624 files. Memory stayed under 3 GB used.

### Counts

Default preset (findings, files with a finding, distinct rules; top ten by count):

| Codebase | Findings | Files | Rules | Top ten |
| --- | --- | --- | --- | --- |
| otel | 2 | 1 of 143 | 1 | NBRAK2 2 |
| logger | 4 | 2 of 35 | 4 | DATST 1, INUSD 1, MSNE 1, VERLESSMATLAB 1 |
| matlab | 278 | 78 of 170 | 40 | SYNER 53, INUSD 48, MANU 23, NOPRT 21, INUSA 21, NASGU 13, NOPTS 12, STOUT 10, ENDCT2 9, NOSEMI 8 |
| mkdoc | 32 | 10 of 15 | 8 | INUSD 12, INUSA 8, STOUT 4, MCFIL 3, MANU 2, MCDIR 1, MCNPR 1, FVORDN 1 |
| toolkit | 34 | 20 of 145 | 13 | STOUT 7, AGROW 6, INUSA 6, BDFIL 4, INUSD 2, NOCOMMA 2, NASGU 1, MSNU 1, CCAT1 1, FNDEF 1 |
| octave | 16,842 | 116 of 116 | 69 | SYNER 14,405, NOPAR 497, FINSNI 265, IJCL 235, ENDCT2 223, ENDCT3 158, NBRAK2 153, UNRCH 134, BADNE 96, EOLPAR 83 |
| overall (without octave) | 350 | | 51 | INUSD 63, SYNER 53, INUSA 35, MANU 25, NOPRT 21, STOUT 21, NASGU 14, NOPTS 12, ENDCT2 9, NOSEMI 8 |

`--preset all`:

| Codebase | Findings | Files | Rules | Top ten |
| --- | --- | --- | --- | --- |
| otel | 2,089 | 108 of 143 | 13 | FCDQS 1,815, LLMNC 105, FCFAV 62, FCCPV 35, FCNVA 34, CTCH 15, LOGSUM 9, CYCCOM 4, MCYCCOM 4, SIMPT 2 |
| logger | 208 | 20 of 35 | 15 | FCDQS 160, FCCPV 16, FCFAV 12, LLMNC 4, SIMPT 4, FCNVA 2, MCCD 2, CYCCOM 1, MCYCCOM 1, ERTAG 1 |
| matlab | 375 | 101 of 170 | 45 | SYNER 53, INUSD 48, FCDQS 46, MANU 23, NOPRT 21, INUSA 21, FCFAV 17, NASGU 13, LLMNC 13, NOPTS 12 |
| mkdoc | 39 | 10 of 15 | 10 | INUSD 12, INUSA 8, FCFAV 6, STOUT 4, MCFIL 3, MANU 2, MCDIR 1, MCNPR 1, FCDQS 1, FVORDN 1 |
| toolkit | 2,377 | 116 of 145 | 36 | FCDQS 1,918, FCNVA 168, FCFAV 96, LLMNC 44, CTCH 31, CYCCOM 22, MCYCCOM 20, LOGSUM 17, STOUT 7, FCNIL 6 |
| octave | 19,520 | 116 of 116 | 84 | SYNER 14,405, FCDQS 2,231, NOPAR 497, FINSNI 265, IJCL 235, ENDCT2 223, ENDCT3 158, NBRAK2 153, ERTAG 145, UNRCH 134 |
| overall (without octave) | 5,088 | | 75 | FCDQS 3,940, FCNVA 204, FCFAV 193, LLMNC 166, FCCPV 65, INUSD 63, SYNER 53, CTCH 46, INUSA 35, CYCCOM 27 |

The full per-rule table is not kept here (75 rules under `all`); to reproduce: run `malt --preset all` over the codebases and count the third field.

### MATLAB diff of the default preset

Per codebase, (file, line, ID) triples, before the fixes of this step and after:

| Codebase | MATLAB findings | Before: both / matlab-only / malt-only | After |
| --- | --- | --- | --- |
| otel (143 files) | 1 | 1 / 0 / 7 | 1 / 0 / 0 |
| logger + mkdoc (50) | 26 | 25 / 1 / 5 | 26 / 0 / 2 |
| matlab (170) | 258 | 188 / 70 / 55 | 202 / 56 / 50 |
| toolkit (145) | 34 | 34 / 0 / 0 | 34 / 0 / 0 |
| octave (116, robustness) | 13,925 | not recorded | 1,903 / 12,022 / 10,308 |

What remains after the fixes, none of it a rule defect:

- **Parser-caused (recorded, grammar not edited).** The 56 + 50 differences of `matlab` are in files made of deliberate syntax errors (`t41FunctionDefinitions`, `t54LineContinuationInAnonymousFunctions`, `t57Operators`, `t23LineContinuationCall`, `t30functionContinuation`, `vscode-textmate/.../test.m`, `ClassWithErrors`, `f_inputargs_error`): SYNER, ENDCT2/3, EOLPAR, NOPAR2, EOFER, BADCH, RESWD and the findings that follow from the recovery (DEFNU, NASGU, NOPTS on code after the break). Same for Octave: every file is a syntax-error file for MATLAB.
- **MCATP (4)**: `name@char = v` property restrictions; the tree-sitter grammar has no node for them. **IMPKEY (1)**: `import module.submodule.class.function`, a keyword inside the import name.
- **A file that is not UTF-8** (`f_with_latin_1.m`): MATLAB reads it and reports INUSD. Fixed after the run: `malt_cli` reads such a file as Latin-1 (every byte a character), analyzes it and leaves it alone under `--fix` (it would have to be written in another encoding); test in `crates/malt_cli/tests/nofil_rderr.rs`.
- **`%#ok<INUSL>`** (`advanced-logger/test/+mlog/+test/TestLogger.m:522`): MATLAB accepts `INUSL` as a suppression ID that suppresses INUSD (probes j6, j7: both unused inputs of `function f(a, b) %#ok<INUSL>` are silent), although `INUSL` is not one of the 2,728 messages of `checkcode -allmsg`. malt reports INUSD and an MSNE for `INUSL` (2 extra findings). The fix is in `malt_core` (`known_ids` and the suppression map: an alias INUSL for INUSD), which this step did not touch. Open: which other legacy IDs MATLAB accepts in `%#ok<...>` is unknown, so MSNE for an ID outside malt's list is a false-positive risk in general.

### Fixes (MATLAB evidence; probe sets of 3 to 22 files, one MATLAB call each)

Each is a unit test and, for the default-analysis ones, a `corpus16/` fixture (20 files, 899 in all; `MALT_MATLAB_REQUIRE=1 cargo test -p malt-cli --test matlab_conformance --test matlab_live` passes, message text included).

- **D1 REDEF on `get.X` next to `set.X`** (`syntax_errors/check_statements.rs`). The duplicate-method check keyed methods by the part after the dot, so every property with both accessors got a REDEF error (a defect from Phase 4). The key keeps the `get.`/`set.` prefix (probes b1, b2, b5: silent; 143 OpenTelemetry files had 2 such errors).
- **D2 MCSUP for a Dependent property's setter** (`good_practices/check_oop_practice.rs`). Silent when the property being set is Dependent, with or without a get method, with or without an output (d3 to d8, e1 to e5, f1 to f5); reported for a non-Dependent property (d1, d2, d6, e3, e4, e6). A value-class setter without an output that reads a private property was still reported by MATLAB in two probes (b3, b4); malt is silent there (a miss).
- **D3 ATUNK and the superclass** (`language_spec/check_class_attributes.rs`). MATLAB reports an unknown attribute only in a class without a superclass, with `handle`, or with a `matlab.mixin` class; silent with `matlab.System`, `ComponentContainer`, any unknown base and `SomeBase & handle` (g1 to g12, h1 to h9). `UsedInUpdate` in a `ComponentContainer` was the dogfooding case.
- **D4 FXSETA and `[a(k), b] = f()`** (`good_practices/mod.rs`, `assigns_loop_variable`). An index inside an output element is a read of the loop variable; the element `[a, k]` still writes it (OpenTelemetry `processAttributes.m`: 2 extras).
- **D5 NASGU for `varargin`** (`unused/check_nasgu.rs`). A dead store to `varargin` is NASGU (k1 to k7: `varargin = {}`, `varargin = varargin{2:nargin}`, in a branch, after a read); malt had exempted it with the always-used names, which also caused a false MSNU on `%#ok<NASGU>` (advanced-logger).
- **D6 MCFIL inside an `@directory`** (`language_spec/check_mcfil.rs`). The `@Qux/Qux.m` holding `classdef Bar` is MCDIR and MCFIL; `@Zed/other.m` holding `classdef Zed` is MCFIL only. The old guard silenced MCFIL in every `@` folder. The MCDIR text was also wrong: MATLAB quotes the class name and ends with the absolute path of the file (`Class name 'Bar' and @directory name do not agree: '/abs/@Qux/Qux.m'.`), not the folder name.
- **D7 INUSD for an input that is also an output** (`unused/check_unused_inputs.rs`, `analysis/liveness.rs`). `function out = f(out, x)` with `out = x + 1` is INUSD (m1 to m6); the entry-value analysis keeps `out` used when it reaches the exit (`if x, out = 1; end`, an empty body). Where the analysis cannot follow the scope, silent as before (f_with_utf8 and f_with_latin_1 of matlabdomain).
- **D8 VARARG right-hand sides** (`bugs/mod.rs`, 22 probes). Reported for a number, text, `[]`, a function handle, `ones(...)`, `struct()` and `true`; silent for a variable, an operator, an index, a field, `x'`, any other call (`cellfun`, `num2cell`), `[x, y]`. The old rule reported every non-cell value (matlabdomain: 3 extras).
- **D9 CTPCT for `error` and `warning`** (`good_practices/check_ctpct.rs`, 20 probes). The first argument is the format unless it is an identifier (colon-separated components starting with a letter) or, for `warning`, a control word (`on off query error verbose backtrace debug`); an identifier first makes the second argument the format: `error('msg', x)`, `error('ErrorID', x, y)`, `error('A:b', 'fmt', 1)`, `warning('msg', 1, 2)` are CTPCT; `warning('off', 'all')`, `error('Mod:sub', x)`, `error('msg %d', 1)` are not. This closes the Phase 4 gap 'CTPCT for a non-identifier first argument'.
- **D10 MANU for a returned input** (`unused/check_manu.rs`): `function obj = f(obj)` whose body only calls `error(...)` is MANU (matlabdomain `ClassWithStrings`); a body that returns `obj` unchanged is not.
- **D11 CTOINW for a constructor that takes its own output** (`good_practices/check_misc_general.rs`). `function obj = C(obj, args)`: reported at the input, any position, `~` before it, any output name; a method that is not the constructor is silent (q1 to q9). The `C(C(x))` call form was never reported and stays off under `mathworks` (pseudo check `CTOINW_CALL`). CTOINW leaves the experimental list of the maturity table.
- **D12 NBRAK1 and NBRAK2 for an indexed field** (`good_practices/check_nbrak1.rs`, `readability/check_unnecessary_brackets.rs`). `[obj.Value( end ) ]* n` is NBRAK1 and `[obj.Value(end)]` NBRAK2; `[obj.a] * n` stays silent (a comma-separated list).
- **D13 NOSEMI after a method `end;`** (`nosemi.rs`): inside a `methods` block the `;` is a child of `methods`.
- **D14 UNRCH after a `switch`** (`analysis/control_flow.rs`): a `switch` with `otherwise` whose every branch ends the flow ends it, like `if`/`else`; without `otherwise` the next statement is reachable.
- **D15 LBODUP only for `&&` and `||`** (`bugs/check_short_circuit.rs`). MATLAB is silent on `a & a`, `a | a`, `x(1) & x(1)`, `1 & 1`, `1 &1` and the same in an `if` (16 probes). The older unit test that expected `b | b` was wrong; `Operand::text` (dead now) is gone.
- **D16 VTPOD** (`syntax_errors/check_validation_order.rs`). A class written before the size (`a double (1,1)`) is VTPOD at the class name, in an `arguments` block and in a `properties` block (the parser leaves the class as an ERROR node before `dimensions`, which malt reported as MCPLD); error severity. VTPOD joins the syntax table again (see `docs/migration.md`).
- **D17 HGSTGT for a superclass** (`suggested_improvements.rs`). `classdef C < hgsetget` and `< handle & hgsetget` are HGSTGT at the name; calls and bare names stayed silent in 49 earlier probes.
- **D18 LOGSUM, the shapes MATLAB reports** (`readability/check_logical.rs`, 16 probes; closes G2 of the Phase 4 rounds). `sum` of a comparison (`> < >= <= ~= ==`) of a slice with a positive number literal, on either side: `v(:)`, `v(:, k)`, `v(1, :)`. Silent for `v(:) > w`, `v(:) > -1`, `v(:, :)`, `&`, a char, `v > 3`, `v(2) > 3`. They are on by default; the general `sum(<logical>)` is the pseudo check `LOGSUM_ANY`, off under `mathworks`. LOGSUM leaves the experimental list.

Defects found only through `--preset all` (no MATLAB comparison; unit tests):

- **MCCPE** (10 reports in otel and logger): a name the function takes as an input or assigns is a variable (`Time(idx, 1) = v`; a test-method parameter that has the name of a `TestParameter` property).
- **FDTAG** (13 reports): fired on every `findall`; now only with one of the six `Exploration.*` tags the message names (`args_any`).
- **GTARG and LTARG** (9 reports): a `...` continuation was counted as an argument; a comma-separated list argument (`varargin{:}`, `s.f`) is an unknown count (only too many can still be told); a function with an `arguments` block (optional inputs, name-value) is not compared; the table of builtin arities is switched off (`USE_BUILTIN_TABLE = false`: `var(X, 0, 1, 'omitnan')` and `ones(n, m, 3, "uint8")` were reported, and the table is unverified).

### Noise under `--preset all`: legitimate or not

| Rule | Reports (no octave) | Verdict |
| --- | --- | --- |
| FCDQS, FCNVA, FCFAV, FCCPV (forward-compatibility gates) | 3,940 + 204 + 193 + 65 = 4,402 of 5,088 | Legitimate only with a target release; malt has none, so they fire on every double-quoted string, `Name=Value`, arguments block and property validation. The checks are right, the gating is missing. **Demotion candidate** (or: report only when a minimum release is configured). They are already experimental. |
| LLMNC (166), CYCCOM and MCYCCOM (27 + 25), FCNIL, FCNLL | custom checks | Legitimate thresholds from MATLAB's list (120 characters, 15, 7 inputs, 200 lines); not compared with a live configuration. Stay experimental. |
| CTCH (46), ERTAG, WNTAG, SIMPT | good practices | Legitimate (the message text agrees with the catalogue); MATLAB never reports them by default. Stay experimental. |
| LOGSUM (26) | | Now split (D18); the rest is LOGSUM_ANY: reports shapes MATLAB is silent on (G2). **Demotion candidate** for the general form. |
| MCCPE, FDTAG, GTARG, LTARG | | Defects, fixed above. GTARG/LTARG on builtins: **demotion candidate** (unverified table, off now). |

### Known gaps found here (not fixed)

- `%#ok<INUSL>`: suppression alias and MSNE (see above; `malt_core`).
- A non-UTF-8 file is read as Latin-1 and analyzed (not fixed by `--fix`); its columns count bytes, as MATLAB's do for Latin-1 files (unprobed for multi-byte text).
- `[obj.Value(end)]` (NBRAK2) and the other bracket forms were probed on indexed fields only; `[s.a(1).b]` is untested and silent.
- MCSUP: a value-class setter of a Dependent property without an output that reads a private property is reported by MATLAB (b3, b4) and silent in malt.
- A file with `end;` after its main function (`function p1(a) ... end;`, one probe) gets DEFNU and SEMFU in MATLAB and NOSEMI only in malt; SEMFU is not emitted.
- GTARG/LTARG are compared only for functions of the same file without an `arguments` block; a function with one is never compared.
- The MCDIR message names the file by `std::path::absolute` of the path malt was given; MATLAB's path may differ when symbolic links are involved.

### Gates

`cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings`, `MALT_MATLAB_REQUIRE=1 cargo test --workspace --exclude malt-py --locked --no-fail-fast` (conformance 1,133 fixtures, live check: recorded findings and message text agree with MATLAB 2026b), `bun docs/scripts/gen_rules_docs.ts --check` and `bun run docs:build` pass.

## Recording the stable checks

Follow-up of 'Rule maturity (plan 6.3)'. The principle of that table is: STABLE means MATLAB reported the check in a recorded run. At the start, 1,413 of the 2,687 unique IDs were `stable`, but 1,112 of them only because they belong to the default analysis, with no finding in `matlab_expected.tsv` (718 compatibility entries, 195 suggested improvements, 199 ordinary engine checks). The goal: for as many as possible, ask MATLAB R2026b and record the answer, so that every ID ends as RECORDED (MATLAB reports the same ID on the same line with the same message), DISAGREEMENT (MATLAB is silent where malt reports: fix the rule, or make malt silent and move the ID to `experimental`) or NOT TRIGGERABLE.

### Method

1. **Inventory.** `bun docs/scripts/gen_rules_docs.ts --unrecorded` lists `engine<TAB>ID` of every stable check without a recorded MATLAB finding (the same set as the last column of `--maturity-report`).
2. **Triggers.** The data-driven entries were generated from the data files (a call of the entry's function with the arguments its conditions need: `min_args`, `args_any`, `literal_args`, `arg_kinds`, `number_arg_at`, `pairs_from`, `min_outputs`, `shadowed_command`) and packed 50 per file (`recorded/compat_NN.m`, `recorded/si_NN.m`). The engine checks came from the unit tests: every string literal of every test module (5,300 sources, with the file name the test passes, because several checks depend on it) was linted with the default preset, and the smallest sources on which malt reports a wanted ID were taken to MATLAB. The limits (nesting, operator and bracket depth, 8 nested block comments), the parfor shapes and the syntax shapes were written by hand from the MATLAB probes recorded in the unit tests.
3. **MATLAB.** `matlab -batch` with `docs/scripts/matlab_probe.m` over at most 60 files per call (`timeout 600`), then a comparison per (file, line, ID) and per message text against malt's default and `--preset all` output.
4. **Fixtures.** Files where MATLAB and malt agree on every finding became fixtures under `crates/malt_cli/tests/fixtures/recorded/` (234 files, 1,230 recorded rows): a minimal set cover of the IDs for the harvested files, the packed files for the data-driven entries, and `silent/` for the shapes MATLAB is silent on, which pin the fixes below. `matlab_files.txt` and `matlab_expected.tsv` were regenerated with `generate_checkcode.m` (README of the fixtures folder). The live message check agrees on all of them (0 differences, 21 accepted as before).

### Counts

| | Before | After |
| --- | --- | --- |
| Stable IDs | 1,413 | 1,343 |
| Experimental IDs | 1,274 | 1,344 |
| Stable, MATLAB finding recorded (fixtures or configuration cases) | 301 | 1,321 |
| Stable, no recording | 1,112 | 22 |
| IDs in `matlab_expected.tsv` | 288 | 1,313 |
| Fixture files | 899 | 1,133 |

Of the 1,112: 1,020 are now recorded (compatibility 12 to 709 IDs, suggested improvements 46 to 209, the engines 243 to 403), 70 moved to `experimental` and 22 stay. The three states:

- **Recorded** (1,020 IDs). 656 data-driven compatibility IDs with a function name (650 on the first run, the rest after fixing the shape: `optimoptions('fmincon', 'CheckGradients', true)`, `savefig(gcf, 'f', 'compact')`, `solve(p, 'linprog')` with `p = optimproblem`, `com.mathworks.mde.desk.MLDesktop.getInstance`) and 38 generic compatibility IDs; 160 suggested-improvement IDs (`iscellstr(x)` with an unknown `x` and `datetime(today, 'ConvertFrom', 'datenum')` needed their own shape; a zero-argument call of a `nan*` function is LTARG in MATLAB, so every plain trigger has one argument) and the three `nargchk` forms (NCHKM, NCHKN, NCHKO); 160 engine IDs from harvested and hand-written files (`engines/`, `engines2/`, `engines3/`).
- **Disagreement** (below): 29 IDs made silent and moved to `experimental`, and about 25 rules corrected.
- **Not triggerable** (below): 41 moved to `experimental` as never emitted, 22 stay stable without a recording.

### Disagreements and fixes

MATLAB stayed silent, or reported another ID, where malt reported. The false positives cost more than the misses, so each was narrowed to the shape MATLAB reports, or switched off.

| IDs | MATLAB | Change |
| --- | --- | --- |
| FPRENAME, XPCRENAME, SLRTRENAME, PSRENAME, DCRENAME, SERENAME | only `ver('fixpoint')` and `verLessThan(...)`; `license('test', ...)` is silent | reported in `ver` and `verLessThan` only |
| HHCNA, HHCWE | only `hereHDLMConfiguration(...)` | `hereHDLMReader` no longer fires |
| FROPT, FROPTX | `print('-dill', ...)`; `-dill` in other calls not probed | `print` only |
| FGREM | `figure('RendererMode', ...)`; `set(gcf, ...)` and `get` are silent | `figure` only |
| DFEATUREPARAM1, DFEATUREPARAM2 | `feature('UseHG2')`; `setpref` is silent | `feature` only |
| TTSMP | `timetable(x, 'SamplingRate', ...)`; `set(gcf, ...)` is silent | `timetable` only |
| COEFFS | `dsp.DigitalDownConverter('FilterSpecification', ...)`; other calls silent | that constructor only |
| HESST, HESSM | `optimoptions` | `optimoptions` only |
| SMTHF | `axes(...)` and `text(...)` constructors; `figure`, `set` and `get` silent | `axes` and `text` only |
| RESOU | a file whose own folder is `resources` (reported on line 0); `cd resources` is silent, a deeper file is silent | path-based now (`recorded/resources/`) |
| IMPKEY | `import pkg.if` (a reserved word as the last name); `import if`, `import if.x`, `import pkg.if.*` silent; message names `'pkg.if'` and `'if'` | rule and message |
| IMPIVD | two or more `*` (`import a.*.*`); `import *` and `import a.*.b` silent | rule and message |
| JAPIMATHWORKS | also on `import com.mathworks...` | reported for the import command |
| MCPDC | `properties (Constant, Dependent)`, not the classdef attributes (those are ATUNK) | moved to the properties block |
| MCATP | `x @double` in a class `properties` block; in a function `arguments` block it is FVSYN | class properties only |
| ATVIZE | not on a properties block (ATVIZW, with ATTF for a value it cannot evaluate) | no ATVIZE, no ATUNK for `Static` on properties |
| PFUIXW | the index was assigned before the loop (otherwise PFUIXE) | PFUIXW added to the PFUIXE check |
| PFNAR | `s = e - s` is not a reduction: no PFRIN or PFRUS | skipped there |
| SPRIX | indexed assignment into a sparse matrix inside a loop only | loop only |
| NCHKI, NCHKM, NCHKN, NCHKO | `error(nargchk(.., nargin))` is NCHKN, `error(nargchk(.., nargout))` NCHKM, a `nargout` third argument NCHKO; NCHKE is never reported | remapped; NCHKE off |
| VAR_NAME messages | MATLAB quotes the name (`'Foo'`) and upper-cases the keyword or function in SPNF, SPRET, UDIM and ENDCT2 | 21 messages: ENDPAR, FCONV, MCANI, MCASC, MCCBS, MCNPN, MCNPR, MCRED, MCSCC, MHERIT, MHERM, PFNAR, PFRFH, PFSLO, PFSLRD, PFSLW, VALST, SPNF, SPRET, UDIM, ENDCT2 (names the block keyword, not the line) |

Made silent and moved to `experimental` with the evidence in the reason (`maturity.toml`):

- 17 generic compatibility IDs flagged `low_reliability` in `compatibility.toml`: NOV6 (`save`, `load`, `rand`, `griddata`, `scatteredInterpolant`, `ode23` all silent), REPUDD (every `schema.*` shape and `schema.m` files silent), LSRET, READSZK, READSZR, SMPLMODE, GETERR, SETERR, COEFF1 to COEFFC3 (the owning class never found).
- 12 engine IDs in the `mathworks` off-lists: CMDAND and CMDOR (`a &b` as a command and in `if`, `x &y` is REDEF), DEFSIZE (function and class shapes, a file named `size.m`), FUNFUN (`arrayfun('f', x)`), MEXCEP (`warning(E.message)`, in a `catch` too), MCHDT (MATLAB reports MCHDP for a handle default), SUBSINDEX, PFCEL, PFFRNG, PFMLTI (MATLAB reports PFFORA for an assigned inner loop variable), STCCS (`strcmpi(a, '123')` in several shapes), NCHKE. The bugs engine now honours `disabled_checks` for every finding, not only the ones that looked at it.

### Not triggerable

Experimental, never emitted (41 IDs): FEGLO and TMMSG (no code path), RMWRN (empty denylist), ATNAS, ATNPI and ATNPP (an unreachable arm: MATLAB reports ATAS, ATUNK and ATPPP in those shapes), ROSDFOBJECT, ROSSCFUN and ROSSCPKG (ROS Toolbox is not installed; `DataFormat()`, `rossvcclient()`, `ros.ServiceClient()` are silent), MASSO, IMSSO and the seven RISK checks (data entries without function name or pattern: class methods of the Risk Management Toolbox), and 23 suggested-improvement entries with `pattern = "unverified"` or `"object"` (EMTAG ... VIDREAD). This also resolves the 37-versus-13 mismatch of 'Rule maturity': 37 = 13 compatibility IDs plus 24 suggested-improvement entries with the same `unverified` pattern, and only the compatibility ones were selectable. The suggested-improvement ones are listed by ID now.

Stay stable without a recording (22 IDs), with the reason:

| IDs | Why |
| --- | --- |
| MDOTM, MDMCR | the file extension is not `.m` or is deployed: a fixture is a `.m` file |
| CLSUNK | needs the search path (`--index`): not a fixture shape |
| TMSMS | 1,001 parse errors: MATLAB reports it, but a fixture would hold 1,000 SYNER rows whose message text differs (`Parse error at '\'` against `at end`) |
| PFVARS | 1,100 variables in a parfor: MATLAB reports it, the fixture would hold 1,100 NASGU rows |
| CLSAT, BADOT, CLIS, SEMFU, ENDCT4 | MATLAB reports them; malt differs in a neighbour (CLSAT: malt gives SYNER and ENDCT2; BADOT: `3 .. 4` and `a..'` are missed; CLIS: an extra MCFIL; SEMFU: FNDEF where MATLAB says DEFNU; ENDCT4: line 1 where MATLAB says 2) |
| CLANS, MCSCC, ENDPAR | the files differ in a neighbour (an extra NOANS; MATLAB adds NODEF; MATLAB adds NOPTS) |
| MCMIO, FVONV, FNSWA, NOLHS, SEPEXC, UNSET, SYNEND, NOPAR2, ENDCT | no shape found where malt reports it at the command line and MATLAB the same ID: the engine fires on them in unit tests only (`= 5;` is a SYNER at the command line, `_f` a BADCH) |

### Comparing `--preset all` with MATLAB

Not built. A cheap way exists: `matlab_probe.m` runs `checkcode(path, '-id')` with MATLAB's default configuration only, so the experimental IDs would need `checkcode(path, '-config=...')` with a settings file that turns every check on (`codeAnalyzerConfiguration.json`, which malt already reads) and a second expectation file for `--preset all`. The packed trigger files (`recorded/compat_NN.m`, `si_NN.m`) already hold one shape per entry, so they would be the fixtures of that run.

### Gates

`cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings`, `MALT_MATLAB_REQUIRE=1 cargo test --workspace --exclude malt-py --locked --no-fail-fast`, `cargo test --locked -p malt_wasm --features wasm`, `bun docs/scripts/gen_rules_docs.ts --check`, `bun run docs:build` and `rumdl` pass (see the final report of the session).
