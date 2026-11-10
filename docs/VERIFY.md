# Conformance with MATLAB's Code Analyzer

What is verified against MATLAB R2026b (`checkcode`, run through the MATLAB MCP
server), what is not, and what still differs. This file describes the current
state only: settle an item, then delete its row.

## How conformance is measured

| What | Where |
| --- | --- |
| Run the Code Analyzer over a file list | `docs/scripts/matlab_probe.m` |
| Compare (file, line, ID) with malt, per check | `docs/scripts/diff_against_matlab.ts` |
| Compare IDs, severities and messages with MATLAB's own list (`checkcode -allmsg`) | `docs/scripts/make-allmsg-csv.ts` + `docs/scripts/check_against_mathworks.ts` |
| Fixture regression test | `crates/malt_cli/tests/matlab_conformance.rs` |

`tests/fixtures/matlab_expected.tsv` holds what MATLAB reports for every fixture
(path, line, ID: facts only). The test fails on any difference that is not listed in
`tests/fixtures/matlab_known_differences.tsv`, the backlog. The goal is an empty
backlog; `cargo test -p malt-cli --test matlab_conformance -- --ignored` fails until
it is. After fixing differences, shrink the backlog with
`UPDATE_KNOWN_DIFFERENCES=1 cargo test -p malt-cli --test matlab_conformance`.

MATLAB's own message text and its full check list are MathWorks content: keep them
outside the repository (`~/.local/share/malt-reference/`).

## Policy

- MATLAB's real behaviour is the ground truth. When it cannot be reproduced, the
  check is not emitted rather than guessed.
- The `mathworks` preset mirrors MATLAB's factory configuration. Checks MATLAB never
  reports by default are off under it and on under `--preset all`: ERTAG, WNTAG,
  BDLOG2, CTCH, VANUS, FVNIV, SUSENS, IDISVARHIGH, USESWNS, NOEFF, PUSE, COMPNOP,
  COMPNOT, CTOINW, BDSCA2, LOGSUM, DUALC, GTARG, LTARG, BDLGI, SIMPT, STRSZ, ELARLOG, MFAMB, STCUL, USENS, ITERS, SHOCIRF, MOCUP, ADAPPREF, MIPC1, VTFIN, PFUTVR, VTPCON, VTPIN, VTPEAL, GPNES, the forward-compatibility gates (FCLEN, FCCPV,
  FCDQS, FCFAV, FCHBL, FCLFS, FCNVA) and the 897 compatibility entries MATLAB lists as
  "low reliability" (`low_reliability = true` in `data/compatibility.toml`; also on
  with the `behavior-changes` category).
- Severities follow MathWorks' documentation index. For RHSFN (error) and PFRNI
  (warning) the numeric levels of `checkcode -allmsg` say otherwise and `checkcode`
  exposes no severity field, so the editor's behaviour cannot be read from here.

## Official IDs malt does not emit

| ID | Why |
| --- | --- |
| PRTCAL | Never fired in any probe (built-in calls, local functions, functions in files, methods; scripts and functions) or in the corpus. |
| XCREDS | Fires for `password`/`Password` assigned a non-empty char literal but not for `PASSWORD`, `pwd`, `token` or `apiKey`; the case asymmetry is unexplained. |
| INVALIDCHECKDISABLE, INVALIDCHECKENABLE, INVALIDCHECKPLACEMENT, INVALIDCHECKPRAGMA | `checkcode` ignores `%#check` pragmas; probably emitted by the editor's analyzer only. |
| FVUBD, FVONV | MATLAB reports a forward reference in a validator as FVVCON only, and never reports FVONV in the corpus; neither is emitted. |
| VIDAF, VIDFH, FVOOOV, VINONA | Did not fire in about 40 probes (validators with anonymous functions or handles give FVVCON, FVOCON, VTPCON, FVSYN, FVOON or MCPLD). |
| TNMLP, STRSET | Not reproducible (TNMLP: loops over many functions; STRSET: the message needs a name that was not found). |
| DEBUGMSG, LLMCC | Need a custom analyzer configuration whose format is not established. |
| BDCFG, CFERR, BDOPT, CFIG | Configuration-file errors; malt does not read MATLAB configuration files. |
| CABE, SCABE, ACABE, MCABE, SMCABE, AMCABE | Cyclomatic-complexity metrics, reported only with `-cyc` or `-modcyc`. |
| MSNU, MSNE, PSET | Planned with suppression bookkeeping in the linter (plan step 3.4). |
| EMNODEF, EMBRK, EMCNT, EMRTN, EMWHL, EMS2N | MATLAB Coder checks, see below. |

## Not verifiable through `checkcode`

The Coder (code generation), Compiler (deployment) and Fixed-Point checks are in
MATLAB's list, but a `%#codegen` probe with `eval` and `global` returned only
base-product messages with MATLAB Coder installed. Until a probe shows `checkcode`
emits them, the `codegen`, `deployment` and `fixed-point` engines are verified
against the documentation index only. Eight message templates differ for that reason
(EMVDF, EMGRO, EMFCN, EMRIFAV, EMLOAD, PRMNOIN, LOOPPRAGMAWITHOUTFOR, FPASE).

## Known differences

Per check; the full list is the backlog file.

| Check | Difference |
| --- | --- |
| NASGU | Liveness analysis with constant folding (`analysis/liveness.rs`): `if 1`, `if true`, `k = 1; if k` kill the dead branch (`1 == 1` is not folded); a `for` loop joins the values before and after its body; `parfor` bodies run once, `spmd` is a sequence, a lambda reads its captures at creation, `x += v` silences `x`, `load('a.mat','x')` stores `x`, `varargout{1} = ...` is reported when `varargout` is not an output, dynamic calls do not stop the analysis (`clearvars`, `run` still do), nested functions are analysed. A file whose root is an ERROR node that starts with `function` is analysed as a function (MATLAB reports the stores before the broken statements). |
| ASGLU, STRNU | Rebuilt on the liveness analysis; recall lost under `eval` (ASGLU) and on some struct cases (STRNU). |
| Suggested improvements (EV2IN, FSTR, GVMIS, ...) | On by default (as in MATLAB). Entries carry condition fields (`literal_args`, `arg_kinds`, `arg_at`, `max_args`, `within_parfor`, `not_in_call`, ...); `object` and `unverified` patterns are never matched. OLDSIM was not probed; `v.NumberOfFrames` needs class tracking. |
| SYNER, NOPAR, EOLPAR, ENDCT* | After a token MATLAB cannot parse (SYNER, BADCH, BADFP, BADHBH/BADHBB), it suppresses the other checks on that statement (separator to the next `;`, `,` or newline); `malt_core::linter` does the same (`drop_statements_with_errors`). SYNER shows malt's error-node text where MATLAB names the token. Still different: the stray `elseif`/`end` after a closed block (SYNER), SBTMP, FVSYN, VTPOD, FVAMI, REDEF, NOSPC/DEEPS thresholds, EOLPAR for an unclosed bracket at a line end, unclosed `{`. |
| UNRCH | `error(m)` with a bare variable is treated as a terminator (MATLAB does not); a `catch` after a `try` whose body returns is not reported; constant propagation is not implemented. |
| DEFNU | The dynamic-call exemption covers `eval`, `feval`, `str2func`; `evalin`, `evalc`, `run` are unprobed. |
| UNRPWR | Condition inferred from the tree shape; the column is one off from MATLAB's. |
| CMDAND, CMDOR | No MATLAB trigger found in six command-syntax probes; the message matches the official text. |
| PFTUSE, PFUIXE | Message agrees; MATLAB reports at the assignment or `parfor` line, malt at the use after the loop. |
| PFBFN, PFOUS, NODEF | Residual false positives in `fix/` files (PFBFN, PFOUS); NODEF misses straight-line use before any definition. |
| NBRAK2, STISA, PFRNG | Misses for unprobed shapes (parenthesised, handle, cell and multi-row elements; variables that hold the evaluated value). |
| NOPTS, NOPRT, TLEV, COMNL | Remaining misses: NOPTS/NOPRT for function input parameters, TLEV for `global` inside an `if` of a script, COMNL for `{ 'a',` continuations. |
| FNDEF | The column is always 1 (MATLAB points at the name). |
| FVSOR | One extra finding in `fix/t__fv2.m` (name-value `.?` form), undiagnosed. |
| `analysis/control_flow.rs` | Matches `kind() == "condition"`, but `condition` is a field name in this tree-sitter-matlab version; the effect on unset-variable and unreachable analysis is unverified. |
| Message tables | Explanations removed from the Check IDs tables to match MATLAB's templates (SHOCIRT, DEBUGFUN, CTRUE, CHAIN, ...) are not yet moved into the `## Rule` sections. |

## Compatibility data: conditions and object members

Entries in `data/compatibility.toml` can carry `min_args`, `max_args`, `args_any`, `args_all`, `unless_args`,
`second_arg`, `script_only` and a `pattern` (`generic`, `object`, `unverified`; only entries without a pattern are
looked up by name). The `object` entries (`tcpclient`, `visadev`, `Session` members, INTLPOUTA) are reported by MATLAB only
on a variable of that class and are not matched until class tracking exists. A variable or local function with the same
name shadows a removed function. `unverified` entries (INTRPC, ADTPATH, MATPOOL, SESSION\*) were never reported by
MATLAB in the probes (some need toolboxes that are not installed).

## Never emitted (not reproducible in MATLAB)

| ID | Note |
| --- | --- |
| SHVAI, SHVAU | MATLAB reported neither in 12 nested-function probes, including the shape malt flagged; emission is gated off (`SHVAI_REPORTED`, `NOT_REPRODUCED`). |
| CLSUNK | MATLAB is silent on a missing superclass; the name heuristic was removed. |
| FVUBD, FVONV | MATLAB reports the forward-reference case as FVVCON. |
| BADOT, SYNEND, EOFMI | No evidence in MATLAB's output; the syntax engine no longer emits them. |

## Open rows (28 differences, see the backlog)

| Group | Rows |
| --- | --- |
| Reaching definitions | INUSD 4 (entry value overwritten before a read; reads only after `error`), DEFNU 3 (`s.f(end+1) = i` records a use of `f`; a function after a classdef `end`; `t__fv4`), NODEF 3 (`ans`, `keyboard`, a never-defined name), ASGLU 2 (`bugs__f1/f2:44`), VUNUS, SETNU |
| Recovery after a parse error | SYNER 3 (`f(a.b=1)`; a SYNER after a number-literal error on the same statement), NOPAR, REDEF (`k += 1` on a loop variable), FVSOR (`t__fv2:62`), FNDEF (`readability__t3f` missed, `t__fv4` extra), NOPTS (`syntax__t5`), BDLOG1 (`syntax__v11`) |
| Unexplained | LNGNM (`gp-a__t1:24`), MCNPN (`ls-c__V1:33`), NOPTS (`ls-c__scr1:8`: `f(1)` before `f` is assigned), RHSFN (`bugs__t1:45`) |
