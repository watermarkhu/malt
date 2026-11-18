---
icon: lucide/badge-check
---

# How malt is verified

malt's goal is to report **exactly what MATLAB R2026b's Code Analyzer reports**
— no more, no less — under the default configuration. MATLAB itself is the
ground truth: every behaviour claim was probed against a running `checkcode`
(R2026b), and the probes, the policy and the open gaps are recorded in
[docs/VERIFY.md](VERIFY.md), the working log of the conformance work (also on
[GitHub](https://github.com/watermarkhu/malt/blob/main/docs/VERIFY.md)). This
page summarises how the claims are enforced.

## The conformance suite

`crates/malt_cli/tests/matlab_conformance.rs` lints every fixture and compares
malt's findings with `tests/fixtures/matlab_expected.tsv`, the recorded output
of MATLAB's own analyzer:

- **nearly 800 fixtures**, written to exercise each check: hand-written shapes,
  corpora of fresh files (`corpus1/` to `corpus4/`), the Phase 4 probe rounds
  (`corpus14/`, `corpus15/`) and real code (see below).
- The TSV holds facts only — file, line, check ID — recorded against the
  release named in `tests/fixtures/matlab_release.txt` (`R2026b`).
- The test fails on **any** difference: a finding MATLAB reports that malt
  misses, or a finding malt reports that MATLAB does not. It runs on every
  `cargo test`.

The four corpora were written **before** their differences were fixed, so they
measure generalisation, not memorisation: corpus1 (31 files) matched 44 of
MATLAB's 45 findings with 28 extra; corpus2 (31 files) 24 of 27 with 7 extra;
corpus3 (33 messy files, 160 findings) 134 matched; corpus4 (32 class-heavy
files, 174 findings) 170 matched. All differences were then fixed and the files
joined the fixtures.

`tests/external_toolkit.rs` lints MathWorks' own
[matlab-agentic-toolkit](https://github.com/mathworks-ref-arch/matlab-agentic-toolkit)
(145 `.m` files): 34 MATLAB findings, all matched, 83 extra before fixing,
**0 differences** after.

## The live checkcode check

A recording can go stale. `crates/malt_cli/tests/matlab_live.rs` therefore asks
a **running MATLAB** the same question over the same fixtures and compares the
answers: the recorded findings must equal what `checkcode` reports today, the
release must still be `R2026b`, and — beyond the facts of the TSV — the
**message text** of every finding must match MATLAB's, word for word (only
SYNER's token text is accepted to differ; `tests/matlab/message_allowlist.tsv`
tracks each accepted difference). At the last full run: 2795 recorded findings
agree with live MATLAB, 0 message differences, 20 accepted.

In CI (the `matlab` job), a `matlab-actions/run-command` step runs the
generator script (`tests/matlab/generate_checkcode.m`) in one MATLAB session,
then the test compares against the recorded file without starting MATLAB
(licensing: only the `run-*` actions license MATLAB on a public repository).
Locally, the test starts MATLAB itself when one is on `PATH`.

## Probing rounds

Beyond fixtures, each engine was audited with systematic probe files: small
MATLAB files written for one hypothesis, run through `checkcode`, and compared
with malt's output. The Phase 4 rounds pushed this furthest: ten or more
fresh, self-written files per round (scripts, classdefs, packages, GUI
callbacks, a deliberate syntax error), one MATLAB run per round, compared per
file, line, ID and message text under the default preset. A round is *clean*
only when the comparison showed no difference before any fix. Rounds 1–8 each
found real misses or extras (EXIST command form, MINV on a variable, NASGU
across `catch`, SBTMP on cell indices, PFBNS at the loop header, DSPSP, NODEF,
PREALL, GFLD/SFLD — all fixed with MATLAB evidence recorded in
[VERIFY.md](VERIFY.md#phase-4-rounds-2026-11-17)); **rounds 9, 10 and 11 were
clean**, meeting the exit criterion of three consecutive clean rounds.

## The path-index cross-check

The optional path index (`--index`, off by default) is checked against MATLAB
itself: `crates/malt_cli/tests/matlab_namespace.rs` generates facts with a
running MATLAB (`which`, `what`, `nargin`/`nargout`, `meta.class.fromName`)
over a committed fixture project and compares them with the index built from
the same folders, in both directions. Accepted differences are allowlisted
with a reason each; the CI `matlab` job runs both phases with
`MALT_MATLAB_REQUIRE=1`.

The same pattern covers MATLAB's `codeAnalyzerConfiguration.json`:
`crates/malt_cli/tests/matlab_config.rs` compares malt with the recorded
`codeIssues` output of 20 configuration cases, and a live variant re-runs
`codeIssues` in CI.

## Verified versus best-effort

| Area | Status |
| ---- | ------ |
| Everything the default analysis reports: syntax errors, language specification, bugs, good practices, readability, formatting, performance, unused/unset variables, incomplete analysis, suggested improvements and the non-object compatibility entries | **Verified**: nearly 800 fixtures with recorded expectations, live `checkcode` parity in CI including message text, and the probing rounds above |
| The `mathworks` off-list (checks MATLAB never reports by default) and the low-reliability compatibility entries | Off under the default preset (see [the policy](VERIFY.md#policy)); their on-side under `--preset all` is **not** compared with MATLAB |
| Naming (`naming.*`) | MATLAB reports naming checks **only** through a code analyzer configuration, so they are off under `mathworks`; the engine was probed with configured probes, and a few semantics (parent-scope variables, script-level local functions) remain unprobed |
| Compatibility entries with `pattern = "object"` (~166) and `pattern = "unverified"` | **Best effort**: MATLAB reports the object entries only on a variable of the class, which needs class tracking; `unverified` entries were silent in every probe form |
| Code generation, deployment, fixed point | **Best effort**: `checkcode` never emitted them in the probes (they need toolboxes and `%#codegen`); the engines are verified against MathWorks' documentation index only, and EMSCR/EMTC/EMIMP are gated on `%#codegen` |
| The custom-check mappings of `LLMNC`, `FCNIL`, `FCNOL`, `SCYCCOM`, `ACYCCOM`, `MCYCCOM`, `MSCYCCOM`, `MACYCCOM`, `SYSBANG` and the other `DAF*` limits | Mapped from MATLAB's message text; **not verified** against a live configuration |
| Check counts on this site | The total in the [Rules Reference](rules.md#overview) is generated from the rule sources; the preset numbers below were counted by hand from the same sources on 2026-11-17 |

Each check also carries a **maturity** (`stable` or `experimental`), listed per check on the
rule pages and per engine in the [Rules Reference](rules.md). The experimental ones are the
best-effort rows above: they are off by default and run with
[`[lint] experimental = true`](configuration.md#experimental-checks) or `--preset all`. The
classification and its evidence are in [VERIFY.md](VERIFY.md#rule-maturity-plan-63).

Known, tracked gaps (each one with its probe evidence) are listed in
[VERIFY.md](VERIFY.md): parser recovery shapes, SYNER token text, the OR2/AND2
bare-call list, and similar. No fixture fails on any of them, and none is
accepted silently — a gap is either fixed or recorded.

## How many checks, and how many are on

The [Rules Reference](rules.md#overview) counts **2692 unique check IDs**
generated from the rule sources. Which of them run depends on the preset:

| Preset | On | Off |
| ------ | -- | --- |
| `mathworks` *(default)* | 1377 | the `custom-checks` (25) and `naming` (81) categories; the Coder (14), Compiler (10), System Object (9) and Unsupported (13) engines; 24 individual checks MATLAB never reports by default (the [off-list](VERIFY.md#policy), without `VANUS`/`NOEFF` below); 905 compatibility entries (the 897 MATLAB marks low-reliability, the seven `FC*` gates and `IDISVARHIGH`); and 234 declared IDs that are never emitted at all (`VANUS` and `NOEFF` plus the code-gated ones like `SHVAU` or `LNGNM`, the 183 `object`-pattern compatibility entries that need a variable of the class, and the 37 `unverified`-pattern entries MATLAB was silent on) |
| `all` | 2458 | the 234 never-emitted IDs above |
| `recommended` | — | `mathworks` plus the `performance`, `readability`, `formatting` and `suggested-improvements` categories |

The preset counts are derived, not generated: the off-list, the
low-reliability flags and the never-emitted IDs live in the rule sources and in
[VERIFY.md](VERIFY.md), and the counts above were taken from them on
2026-11-17. `MSNU`, `MSNE`, `QUIT` (linter core), `NOFIL`, `RDERR` (CLI) and
`SZCLN` are contextual and counted as on.

## Reproducing a run

```bash
cargo test --workspace --locked                      # includes the conformance suite
MALT_MATLAB_REQUIRE=1 cargo test -p malt-cli --locked --test matlab_live
```

The second command needs a local MATLAB R2026b (`matlab` on `PATH`); without
`MALT_MATLAB_REQUIRE=1` the live tests skip when no MATLAB is installed. How to
add a fixture is documented in
`crates/malt_cli/tests/fixtures/README.md`.
