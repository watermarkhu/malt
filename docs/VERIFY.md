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
  COMPNOT, CTOINW, BDSCA2, LOGSUM, the forward-compatibility gates (FCLEN, FCCPV,
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
| NASGU | Liveness analysis (`analysis/liveness.rs`). Not done: MATLAB's constant propagation (`k = 1; if k ...`), so misses remain; bails out on dynamic code (`eval`, `evalin`, `assignin`, `load`, `clearvars`, ...), `parfor`, `spmd`, nested functions, lambdas, `global`, `persistent`. A few false positives on `elseif` chains. |
| ASGLU, STRNU | Rebuilt on the liveness analysis; recall lost under `eval` (ASGLU) and on some struct cases (STRNU). |
| Suggested improvements (EV2IN, FSTR, GVMIS, TLEV, ...) | MATLAB reports them by default; malt's engine is off by default until each entry's condition is narrowed. Same root cause for qualified names (`sigwin.hamming`) as the one fixed in the compatibility engine. |
| SYNER, NOPAR, EOLPAR | Message shows malt's error-node text, MATLAB names the offending token. `end` in value position, `[x](1)` and missing lexical checks (BADFP, BADCH) cause false positives; unclosed `{`, `max(1 2)` and an unclosed bracket at a line end (EOLPAR) are missed. |
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
