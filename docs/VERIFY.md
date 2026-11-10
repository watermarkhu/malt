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
missed finding or an extra one: malt matches MATLAB on all 209 fixtures under the
default configuration. A new fixture needs its MATLAB expectations: run
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
  - bugs: BDSCA2, STCUL, SHOCIRF, MOCUP
  - good_practices: ERTAG, WNTAG, CTCH, BDLOG2, COMPNOP, COMPNOT, CTOINW, DUALC, GTARG,
    LTARG, BDLGI, SIMPT, STRSZ, ELARLOG, ITERS, ADAPPREF, MIPC1, VTFIN
  - language_spec: FVNIV, USESWNS, PFUTVR, VTPEAL, VTPIN, VTPCON, GPNES
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
| SHVAI, SHVAU | MATLAB reported neither in 12 nested-function probes, including the shape malt flagged (gated off: `SHVAI_REPORTED`, `NOT_REPRODUCED`). |
| CLSUNK | MATLAB is silent on a missing superclass; the name heuristic was removed. |
| LNGNM | Never reported in about 35 probes with names of 63 to 120 characters (gated off: `NOT_REPRODUCED`). |
| BADOT, SYNEND, EOFMI | No evidence in MATLAB's output; the syntax engine no longer emits them. |
| TNMLP, STRSET | Not reproducible (TNMLP: loops over many functions; STRSET: the message needs a name that was not found). |
| DEBUGMSG, LLMCC | Need a custom analyzer configuration whose format is not established. |
| BDCFG, CFERR, BDOPT, CFIG | Configuration-file errors; malt does not read MATLAB configuration files. |
| CABE, SCABE, ACABE, MCABE, SMCABE, AMCABE | Cyclomatic-complexity metrics, reported only with `-cyc` or `-modcyc`. |
| MSNU, MSNE, PSET | Planned with suppression bookkeeping in the linter (plan step 3.4). |
| EMNODEF, EMBRK, EMCNT, EMRTN, EMWHL, EMS2N | MATLAB Coder checks, see below. |

## Not verifiable through `checkcode`

The Coder (code generation), Compiler (deployment) and Fixed-Point checks are in MATLAB's
list, but a `%#codegen` probe with `eval` and `global` returned only base-product messages
with MATLAB Coder installed. Until a probe shows `checkcode` emits them, the `codegen`,
`deployment` and `fixed-point` engines are verified against the documentation index only.
Eight message templates differ for that reason (EMVDF, EMGRO, EMFCN, EMRIFAV, EMLOAD,
PRMNOIN, LOOPPRAGMAWITHOUTFOR, FPASE).

## Known gaps

No fixture fails on these.

| Gap | Note |
| --- | --- |
| Overfitting | The rules were fitted to the 209 fixtures; a fresh MATLAB-checked corpus is the next measurement (plan step 4.1). |
| Message text | The test compares file, line and ID, not the text of dynamic messages (for example MATLAB quotes names in MCNPN: `'foo' is referenced...`). |
| Statement right after a stray `end` | tree-sitter parses it as ERROR tokens plus a number, so malt misses MATLAB's NASGU for `z = 1;` there. |
| `x = f(a.b=1);` | MATLAB reports SYNER at the `=`; malt reports nothing (only the statement-level call form is handled). |
| Self-call after an unclosed `arguments` block | The statement is not visited by the symbol table, so DEFNU can report the primary function where MATLAB is silent. |
| NASGU | MATLAB's constant propagation is only partly implemented (`1 == 1` is not folded, as in MATLAB; other cases are untested). |
| UNRPWR | Condition inferred from the tree shape; the column is one off from MATLAB's. |
| CMDAND, CMDOR | No MATLAB trigger found in six command-syntax probes; the message matches the official text. |
| PFTUSE, PFUIXE | Message agrees; MATLAB reports at the assignment or `parfor` line, malt at the use after the loop. |
| DEFNU | The dynamic-call exemption covers `eval`, `feval`, `str2func`; `evalin`, `evalc`, `run` are unprobed. |
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
that are not installed). The suggested-improvements data has its own condition fields
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
- NODEF is reported for a read with no reaching definition when the name is assigned somewhere
  later in the function, and silent otherwise. INUSD uses the entry value's liveness: an input
  overwritten before any read, or read only after `error(...)`, is unused.
