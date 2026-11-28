---
icon: lucide/git-compare
---

# Migrating to the MATLAB-conformant malt

This page is for users of the earlier malt. The release was rebuilt around one
rule: under its default configuration malt reports what MATLAB R2026b's Code
Analyzer reports, and nothing else. The statements below come from the rule
sources and from probing a real `checkcode`. The evidence for each is in
[docs/VERIFY.md](VERIFY.md), and [How malt is verified](verification.md)
explains how conformance is enforced.

## Summary of changes

Correctness: every engine was rebuilt against a running MATLAB R2026b
(`checkcode`): findings that MATLAB does not report were removed, findings it
does report were added, message texts, severities, lines and columns were
aligned, and `%#ok` suppression directives now follow MATLAB's rules. On the
conformance fixtures, including four corpora of fresh files and MathWorks'
own toolkit, malt now matches MATLAB with zero differences. If you lint the
same project with the same config, the output will differ: there are fewer false
positives, and different findings wherever the old heuristics guessed.

Defaults: the default is now MATLAB's factory configuration (`mathworks`
preset): the checks MATLAB never reports by default are off, the 897
compatibility entries MATLAB marks *low reliability* are off, and the
suggested-improvement hints (which MATLAB does report) are on. `--preset
all` turns everything on again.

CLI: `--fix` is safe by default and `--fix --unsafe-fixes` opts into
behaviour-changing rewrites; `--fix` now exits `1` when findings remain. New:
`--level`, `--color`, `-v`/`-vv`, `--strict-config`, `--index`/`--no-index`.
malt also reads MATLAB's own
`resources/codeAnalyzerConfiguration.json`.

Config: unknown rule IDs and parameters in `.malt.toml` are now reported
(use `--strict-config` to make them an error). The `CUSTOM_CHECKS` engine lost
its numeric `max_*` misuses of the `DAF*` IDs (they are boolean `disallow_*`
options now), `NOSEMI` changed meaning, and a handful of made-up checks were
deleted.

## NOSEMI now means the extra semicolon

| | before | now |
| --- | --- | --- |
| `x = 1;;` | not reported | `NOSEMI`: extra semicolon is unnecessary |
| `x = 1` (script) | `NOSEMI`: trailing semicolon missing | `NOPTS`: add a semicolon to hide the output (in a script) |
| `y = f()` (function) | `NOSEMI` | `NOPRT`: add a semicolon (in a function) |

The old `NOSEMI` ("trailing semicolon missing") duplicated what MATLAB reports
as `NOPTS`/`NOPRT`. `NOSEMI` now carries MATLAB's meaning: the extra
semicolon after `x = 1;;` is unnecessary. The rule takes no parameters:
the `ignore_functions` option is gone, and a config that still sets it gets a
warning (below).

## `--fix`, `--unsafe-fixes` and the exit codes

- Before: `--fix` applied every available fix in one pass, including
  rewrites that change behaviour, and always exited `0`.
- Now: `--fix` applies safe fixes only, repeating until nothing changes
  (a fix can enable another). Fixes that may change behaviour (`str2num` →
  `str2double`, `a & b` → `a && b`, …) are reported as
  `N more with --fix --unsafe-fixes` and applied only with that flag. Since the
  second curation pass `--fix` also applies FVAL, MSNU (deleting a `%#ok`
  directive that suppresses nothing), TWOCM, STRQUOT and FORMATNOI, with the
  text MATLAB's own fix writes.

Exit codes:

| Code | Meaning |
| ---- | ------- |
| `0` | No findings (with `--fix`: none remain after fixing) |
| `1` | Findings reported (also with `--fix` when some remain) |
| `2` | Runtime error, invalid config, or (with `--strict-config`) any config warning |

CI scripts that treated `malt --fix` as always-green must now handle exit
code `1`, or pre-filter with `--level`. `--level error` (or `warning`) hides
findings below the level. Hidden findings are not counted and do not set the
exit code, and `--fix` still fixes them.

## The Safe fixes were curated against MATLAB's own fix

`--fix` still applies only Safe fixes, but the set was re-curated with
MATLAB's own `codeIssues(...).fix(ID)` as the oracle (see
[`VERIFY.md`](./VERIFY.md), '5.5 Curated safe autofixes'). What changed
for a plain `--fix`:

| Check | Before | Now |
| ----- | ------ | --- |
| NOPTS, NOPRT | inserted a `;` in front of a comma, producing `x = 1;, y = 2` (which the Code Analyzer itself reports as NOCOMMA) | replaces the comma, as MATLAB's fix does (`x = 1; y = 2;`) |
| NBRAK2 | replaced the whole bracket expression with the element text, dropping comments inside the brackets | strips only the two bracket tokens, keeping inner text, comments and spacing (MATLAB's fix is the same) |
| FLUDLR | `rot90(x, 2)` | `rot90(x,2)`, byte-identical to MATLAB's fix |
| FNDEF | no fix | renames the declaration to the file name, as MATLAB's auto fix does (Safe: the runtime resolves by file name) |
| DLABINDEX, DNUMLABS, DGOP, CAXIS, ISDIR and the other direct-replacement renames | no fix | the callee token is renamed (`labindex` → `spmdIndex`, `caxis` → `clim`, ...) |

New Unsafe fixes (only with `--fix --unsafe-fixes`): STREMP's negated form
(`~isempty(strfind(s,p))` → `contains(s,p)`), FNAN with a logical operator
behind it (`if n == NaN | n > 3` → `if isnan(n) | n > 3`), and MCFIL (renames
the `classdef` and the constructor to the file name, so a method that was never
the constructor becomes one). IJCL now rewrites `j` to `1i` (as MATLAB's fix
does; `1j` before), and AND2/OR2 replace exactly the reported operator, as
MATLAB's fix does.

## Custom checks: `disallow_*` instead of `max_*`

The old `CUSTOM_CHECKS` engine mapped MATLAB's *disallow* IDs (`DAFBR`, …) to
invented numeric thresholds (`max_branches`, `max_return_points`, …). The
`DAF*` checks are now what MATLAB means: a boolean flag that reports every
use of the construct. As in MATLAB, all `DAF*` checks are off by default;
`SYSBANG` is on (when the category is enabled at all).

| `[lint.rules.CUSTOM_CHECKS]` | before | now |
| --- | --- | --- |
| `max_function_inputs`, `max_function_outputs`, `max_function_lines`, `max_line_length`, `max_nesting_depth`, `max_cyclomatic_complexity` | threshold checks | unchanged |
| `max_strict_cyclomatic_complexity` | script complexity | renamed `max_script_cyclomatic_complexity` (the old name still works, but is reported as a typo) |
| `max_avg_cyclomatic_complexity` | anonymous complexity | renamed `max_anonymous_cyclomatic_complexity` (same) |
| `max_branches`, `max_return_points`, `max_nested_functions`, `max_anonymous_functions`, `max_local_variables`, `max_local_constants`, `max_called_functions`, `max_semicolons_per_line`, `max_tree_children`, `max_persistent_variables`, `max_conditions`, `max_input_args_used`, `max_output_args_used` | numeric limits misusing `DAF*` IDs | removed, replaced by `disallow_bang`, `disallow_try_catch`, `disallow_persistent`, `disallow_continue`, `disallow_break`, `disallow_return`, `disallow_script`, `disallow_nested_functions`, `disallow_command_syntax`, `disallow_anonymous_functions`, `disallow_char_vectors`, `disallow_cellstr`, `disallow_varargin`, `disallow_varargout` |

A config that still sets a removed `max_*` key gets a warning listing the
valid parameters (or fails with `--strict-config`).

## NO4LP is now the parenthesised `for` check

The old `NO4LP` checked the indentation of loop and conditional bodies and
took an `indent_size` parameter (while quoting MATLAB's unrelated message).
`NO4LP` now means what MATLAB means: parentheses are not needed in a `for`
statement (`for (i = 1:n)` → `for i = 1:n`, auto-fixed). The formatting engine
takes no parameters; `indent_size` is gone. Indentation is no longer linted.

## Checks that changed meaning

These IDs are still emitted, but they now report what MATLAB reports. The old
uses were invented:

| ID | before | now |
| --- | --- | --- |
| `PSET` | partial assignment in an `if` (invented) | a persistent variable is used but never assigned |
| `VUNUS` | assigned in all branches but unused after (invented) | an operator expression or bare variable whose value is discarded (`x{1};`) |
| `ASGSL` | assignment inside a condition (invented) | self-assignment `x = x;` (assignment to variable might be unnecessary) |
| `MSNU` | struct field set but never read (a heuristic that misused the ID) | a `%#ok<ID>` directive suppressed a message that was not generated (performance and unused engines) |
| `MSNE` | struct field does not exist (same misuse) | a `%#ok<...>` element that is not a known check ID |
| `DAF*` | numeric thresholds | disallow flags (above) |
| `NOSEMI`, `NO4LP` | see above | see above |

`PSIZE` keeps its meaning (`prod(size(x))` → `numel(x)`); severities and
message texts across all checks were aligned with MathWorks' documentation
index.

## Removed check IDs

These were emitted before and no longer exist:

| ID | Why removed |
| --- | --- |
| `BDCFG`, `CFERR`, `BDOPT`, `CFIG` | MATLAB emits them only for `checkcode -config` on its own configuration files, which malt does not read. The whole `CONFIG_ISSUES_ENGINE` was dropped; the `configuration-issues` category is now empty but the `[lint.categories]` key is still accepted |
| `PRTCAL` | official formatting check, but MATLAB never fired it in any probe (built-in calls, local functions, methods, scripts) or fixture |
| `VTPOD` (removed, then back) | removed because MATLAB reported `FVSYN` where malt reported `VTPOD`; emitted again for a class written before the size (`a double (1,1)`), which MATLAB reports as `VTPOD` |
| `DEEPS`, `MBIG`, `TEXTL` | official incomplete-analysis limits that MATLAB in practice never reports (files of 1 to 16 MB, 5,000,000-character lines, deep nesting); deleted |
| `EOFMI` | no evidence in MATLAB's output |
| `TNMLP` | not reproducible (it concerns loops over many functions) |
| `EMS2N` | a Coder check `checkcode` never emits (probed with MATLAB Coder installed) |

Declared but never emitted anymore (MATLAB reports none of them; the
complete table with probe evidence is [`Official IDs malt does not
emit`](VERIFY.md#official-ids-malt-does-not-emit)): the code-gated `FVUBD`,
`FVONV`, `CLSUNK`, `SHVAI`, `SHVAU`, `LNGNM`, `BADOT`, `SYNEND`, `READSZK`,
`FVATF`, `CTOINE`, `MWKCL`, `VANUS` and `NOEFF`, plus the compatibility
entries that need a variable of the class (`pattern = "object"`, 183 of them)
and the entries MATLAB was silent on in every probe form (`pattern =
"unverified"`: 13 compatibility and 24 suggested-improvement entries).

New IDs appeared on the other side: `NOPAR` and `SEPEXC` (syntax),
`BDSCA1`, `ISEQ`, `SUBSASGN`, `XPORD` (good practices), `DUPNAMEARG`, `SPBRK`
(language specification), `SZCLN` (compatibility), 29 new compatibility
data entries (comm/dsp interleavers, `PDSCHCONFIG`, `WSDLCLIENT`, …; eight
`DLAB*`/`DNUMLABS` entries moved to the suggested-improvements table), and
`MALT_TOO_DEEP` and `MALT_TOO_BIG`. These two IDs are malt's own: safety
limits on pathologically nested or very large input rather than MATLAB checks
(see [Resource limits](configuration.md#resource-limits)). A full diff of the
check inventory is easiest to see in the [Rules Reference](rules.md#overview),
which is generated from the rule sources.

## Files over the resource limits are skipped

malt now protects the machine from pathological input, and the limits are on by default: a file over 5 MiB (`max_file_size`) is not read and gets a `MALT_TOO_BIG` error at line 1 (exit code `1`); code nested more than 10,000 brackets (`max_nesting`) or 1,000 syntax-tree levels (`max_tree_depth`) gets `MALT_TOO_DEEP`; and `max_threads` (default: the number of cores) is now a total for the whole process, where before the file workers and the checks of one large file each used all the cores. Both IDs are malt's own, not MATLAB messages. If a project has files that large, either exclude them or raise the limit under `[lint.limits]`; read the warning in [Resource limits](configuration.md#resource-limits) first. The browser build (`malt-wasm`) defaults to 2 MiB and 200 levels.

## The default now mirrors MATLAB's factory configuration

Under the `mathworks` preset (still the default when no `.malt.toml` exists)
three things changed:

1. Checks MATLAB never reports by default are off: `ERTAG`, `WNTAG`,
   `CTCH`, `BDLOG2`, `LOGSUM`, and every other entry of the
   [off-list](VERIFY.md#policy), each verified by probing. Only `--preset all`
   (or `[lint] preset = "all"`) turns this part of the list back on, but a
   per-rule key does not. The compatibility part is finer-grained: an
   `FC*` gate comes back with `[lint.rules] FCDQS = "error"` (or the
   `forward-compatibility` category), and the low-reliability entries come
   back with the `behavior-changes` category.
2. The 897 low-reliability compatibility entries and the seven forward
   compatibility gates (`FCLEN`, `FCCPV`, `FCDQS`, `FCFAV`, `FCHBL`, `FCLFS`,
   `FCNVA`) are off, as is `IDISVARHIGH`. They are also on with the
   `behavior-changes` category.
3. Suggested improvements are on (MATLAB reports them by default; before
   they were an opt-in engine). The compatibility engine also matches more
   MATLAB shapes now: handle references (`@psd`), bare dotted names and the
   bare identifier of a multi-output assignment (`[h, a] = legend;`).

The presets themselves are unchanged by name (`mathworks` (default),
`recommended`, `all`), and `recommended` still disables the same categories.
To get the old broad behaviour back, run with `--preset all` (a superset of
the old default: it also enables the naming, custom-check, Coder, Compiler,
System Object and Unsupported engines). Expect different findings even then:
where the old engines guessed, the new ones reproduce MATLAB.

## New CLI features

| Feature | Effect |
| --- | --- |
| `--level <error\|warning\|info>` | show only that severity and above; hidden findings do not count and do not set the exit code |
| `--color <auto\|always\|never>` | terminal colours, honouring `NO_COLOR`, `CLICOLOR`, `FORCE_COLOR`; `auto` by default |
| `-v`, `-vv` | stderr progress: linted files (`-v`), per-file time and finding count (`-vv`); stdout unchanged |
| `--strict-config` | exit `2` on any config problem instead of warning |
| `--index` / `--no-index` | the optional path index (below) |
| MATLAB config discovery | without a `.malt.toml`, malt now picks up `resources/codeAnalyzerConfiguration.json` in the working directory, and `--config <file>.json` reads one directly; `[lint] matlab_config` layers a `.malt.toml` over it |
| `[index]` in `.malt.toml` | `enabled` and `paths` for the path index: with it on, `MCEB` also reports events in a class whose superclasses are all value classes on the path (MATLAB refuses to load such a class). Off by default, so the output stays file-local |

Files are now linted in parallel when not fixing (output order is the
command-line order), and the Rust toolchain is pinned to 1.100.0
(`rust-toolchain.toml`), so contributors need `rustup`.

## Config validation

`.malt.toml` rule keys and parameter names are now checked; each problem is one
warning on stderr and the run continues:

```text
warning: .malt.toml: unknown rule ID 'NOSMI' in [lint.rules] (did you mean 'NOSEMI'?)
warning: .malt.toml: unknown parameter 'max_variable_nam_length' for rule 'GOOD_PRACTICES_ENGINE' (did you mean 'max_variable_name_length'?)
```

Dotted IDs must be quoted: `[lint.rules."naming.class.casing"]` (an unquoted
`[lint.rules.naming.class.casing]` is reported). `--strict-config` turns every
warning into exit code `2`. Use it in CI, so a typo cannot silently
disable a rule. The removed `max_*` keys and `ignore_functions` are reported by
the same mechanism.

## Checklist

If your `.malt.toml` had… do this:

| You had | Do |
| --- | --- |
| `[lint.rules.NOSEMI] ignore_functions = [...]` | delete it; `NOSEMI` is now the extra-semicolon check. To tolerate missing semicolons, turn the real checks off: `[lint.rules] NOPTS = "off"` (scripts) and `NOPRT = "off"` (functions) |
| `NOSEMI = "off"` (to silence missing-semicolon noise) | turn off `NOPTS`/`NOPRT` instead; keep `NOSEMI` on (it now finds a different thing) |
| `[lint.rules.CUSTOM_CHECKS]` with `max_branches`, `max_return_points`, `max_local_variables`, … | replace with the `disallow_*` flags (they report every use, not a count); drop the rest |
| `max_strict_cyclomatic_complexity` / `max_avg_cyclomatic_complexity` | rename to `max_script_cyclomatic_complexity` / `max_anonymous_cyclomatic_complexity` (the old names still work but warn) |
| `[lint.rules.FORMATTING_ENGINE] indent_size = N` or `[lint.rules.NO4LP] indent_size = N` | delete it; malt no longer lints indentation |
| `[lint.categories] configuration-issues = "..."` | nothing: the key is still accepted; the category has no checks |
| `[lint.categories] suggested-improvements = "off"` and you want the old silence | keep it; otherwise delete it, since the hints are on by default now |
| a `[lint.rules]` entry that turned an off-list check back on (e.g. `ERTAG = "warn"`) | re-check: engine off-list checks (`ERTAG`, `WNTAG`, `CTCH`, …) now need `preset = "all"`. A rule key alone does not turn them on. `FC*` gates keep their per-rule switch, and the low-reliability entries come back with the `behavior-changes` category |
| a `[lint.categories]` entry re-enabling a preset-off category (e.g. `custom-checks = "warn"`) | nothing: explicit category entries still win over the preset |
| CI treating `malt --fix` as exit-0 | re-check: exit is `1` when findings remain, and unsafe rewrites need `--fix --unsafe-fixes` |
| CI that must not regress on typos | add `--strict-config` |

And two new options worth adopting anywhere: `--level warning` in editors or
gates that only care about real problems, and `-v` in scripts that want
progress without changing the machine-readable stdout.
