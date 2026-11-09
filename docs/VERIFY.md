# Unverified and disputed checks

Checks whose ID, message, severity or firing condition could not be confirmed
against MATLAB R2026b (`checkcode`, run through the MATLAB MCP server). Each entry
says what is known and what would settle it. Remove an entry when it is settled.

How to check: `docs/scripts/matlab_probe.m` runs the Code Analyzer over a file list,
`docs/scripts/diff_against_matlab.ts` compares the result with malt, and
`docs/scripts/make-allmsg-csv.ts` + `docs/scripts/check_against_mathworks.ts`
compare IDs, severities and messages with MATLAB's own list (`checkcode -allmsg`).

## Disputed severities

| ID | malt | MATLAB docs index | `checkcode` level | Note |
| --- | --- | --- | --- | --- |
| RHSFN | error | error | 1 (warning) | The two MathWorks sources disagree; malt follows the documentation. |
| PFRNI | warning | warning | 2 (error) | Same. |

## Not emitted: no known trigger

| ID | Why |
| --- | --- |
| PRTCAL | MATLAB lists "Add a semicolon after the function call to hide the output", but it did not fire for built-in calls, local functions, functions in separate files, `cellfun`, `containers.Map()` or method-like calls, in scripts or functions, and never across the 209-file fixture corpus. The check was removed rather than guessed. |

## Not verifiable through `checkcode`

The Coder, Compiler (deployment) and Fixed-Point checks are in MATLAB's list
(`checkcode -allmsg`) but a `%#codegen` probe with `eval` and `global` returned only
base-product messages with MATLAB Coder installed. Until a probe shows
`checkcode` emits them, the `codegen`, `deployment` and `fixed-point` engines are
verified against the documentation index only.

## Behaviour known to differ

| Check | Difference |
| --- | --- |
| NASGU | MATLAB reports only stores that are overwritten or cleared before use, in scripts and in functions (plus final stores in functions). malt reports a variable that is never read, and in scripts reports nothing. Needs reaching-definitions analysis. |
| Forward-compatibility gates (FCLEN, FCCPV, FCDQS, FCFAV, FCHBL, FCLFS, FCNVA) | MATLAB leaves them off by default; malt now does too (on with `--preset all` or the category). |
| Suggested improvements (EV2IN, FSTR, GVMIS, TLEV, ...) | MATLAB reports them by default; malt's engine is off by default until its conditions are narrowed. |
| INUSA | Implemented from probes (positional inputs declared in the arguments block and never read); 3 misses remain in the fixture corpus. |
| UNRPWR | malt's meaning ("use parentheses") differs from MATLAB's (exponentiation combined with unary operators). |
| FXSETA | malt's message is a placeholder; MATLAB's is the loop-variable assignment message. |
| UNRPWR | Re-implemented from probes (`2^-a^2`, `a.^-b.^c`); the condition is inferred from the tree shape and the column is one off from MATLAB's. |
| CMDAND, CMDOR | No MATLAB trigger found in six command-syntax probes, so the firing condition is unconfirmed; the message matches the official text. |
| PFTUSE, PFUIXE | The message agrees; MATLAB reports at the assignment or `parfor` line, malt at the use after the loop. |
| UNRCH | `error(m)` with a bare variable is still treated as a terminator (MATLAB does not); a `catch` after a `try` whose body returns is not reported. |
| DEFNU | The dynamic-call exemption covers `eval`, `feval` and `str2func`; `evalin`, `evalc` and `run` are unprobed. |
| SYNER | Message shows malt's error-node text; MATLAB names the offending token. About 12 false positives and 6 misses remain in the `fix/` corpus (`end` in value position, `[x](1)`, missing lexical checks such as BADFP, BADCH). |
| NBRAK2 | Parenthesised, function-handle, cell and multi-row elements are unprobed and not reported. |
| FNDEF | Column is always 1 (MATLAB points at the name). |
| ERTAG, WNTAG, BDLOG2, CTCH, VANUS, FVNIV, SUSENS | Never reported by MATLAB's default `checkcode` (probed, also with `-all`): off under the `mathworks` preset, on under `all`. |
| PFBFN, PFOUS, NODEF, ASGLU | Narrowed from probes; PFBFN keeps ~6 and PFOUS ~4 false positives in `fix/` files (unprobed names such as `save`, `clearvars`, `inputname`); NODEF misses straight-line use before any definition; ASGLU misses the first of `[a,b]=...; a=1;`. |
| NOPAR, EOLPAR, SYNER | Cell-brace unclosed cases (`{`) are not detected; an unclosed bracket at the end of a line should be EOLPAR at the opener (malt gives NOPAR2); `max(1 2)`-style arguments are a SYNER in MATLAB only. |
| Message tables | Explanations removed from the Check IDs tables to match MATLAB's templates (SHOCIRT, DEBUGFUN, CTRUE, CHAIN, ...) have not been moved into the `## Rule` sections yet. |

## Official IDs not emitted

| ID | Why |
| --- | --- |
| XCREDS | MATLAB fires for `password`/`Password` assigned a non-empty char literal, but not for `PASSWORD`, `pwd`, `token` or `apiKey`; the case asymmetry is unexplained, so no precise condition. |
| INVALIDCHECKDISABLE, INVALIDCHECKENABLE, INVALIDCHECKPLACEMENT, INVALIDCHECKPRAGMA | `checkcode` ignores `%#check` pragmas; probably emitted by the editor's analyzer only. |
| VIDAF, VIDFH, FVOOOV, VINONA | None fired in about 40 probes (validators with anonymous functions or handles give FVVCON, FVOCON, VTPCON, FVSYN, FVOON or MCPLD instead). |
