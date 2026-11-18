# Conformance fixtures

The `.m` files here are what malt is checked against MATLAB's Code Analyzer on. Every
`.m` file under this folder, at any depth, is a fixture. `matlab_expected.tsv` holds what
MATLAB R2026b reports for each of them, `matlab_files.txt` lists them, and
`matlab_release.txt` names the release; `matlab_expected_all.tsv` is the second recording, what MATLAB
reports with every check turned on (the base of the `--preset all` comparison). The method and the policy are in
[`docs/VERIFY.md`](../../../../docs/VERIFY.md).

## Layout

| Where | What it holds | Naming |
| --- | --- | --- |
| `fix/` | Regression files: one per fixed case (207 files). | `<area>__<name>.m`, with a double underscore, e.g. `bugs__f1.m` |
| `fix_matlab/` | The committed MATLAB-fix differential: `<fixture>.<ID>.m`, MATLAB's own fixed text for the pairs whose Safe fix must equal it (10 files). | `<fixture>.<ID>.m`, e.g. `formatting__sepcomma.NOPTS.m` |
| `phase2/`, `phase3/` | Probe shapes from the audit phases. `phase3/` is the current one. | `<check>_<nn>.m` (`bdlog2_02.m`), `<topic>_<what>.m` (`nasgu_error_literal.m`) or `yc_<letter><nn>.m` |
| `corpusN/<letter>/` | Fresh corpora, written apart from the rules to measure how well malt generalises (`corpus16/` is the odd one: 20 minimal reductions of the dogfooding differences, plan 6.2). | `<letter>_<topic>.m` in a folder per letter (`a/`, `b/`, ...) |
| `recorded/` | Trigger files for checks that were stable without a MATLAB recording (plan 6.3, 'Recording the stable checks' in `docs/VERIFY.md`): the data-driven compatibility and suggested-improvement entries packed one trigger per line (`compat_NN.m`, `si_NN.m`), the generic compatibility shapes (`generic_*.m`, `resources/`), one file per engine check (`engines/`, `engines2/`, `engines3/`: harvested from the unit tests, nested limits, parfor shapes) and the shapes MATLAB is silent on (`silent/`). 234 files. | `compat_NN.m`, `<id>.m`, own folder where the path matters (`private/`, `resources/`) |
| `realistic/` | Realistic programs, with its own `README.md`. | free |
| top level | The expectations and the lists: `matlab_expected.tsv`, `matlab_expected_all.tsv`, `matlab_files.txt`, `matlab_release.txt`. | |

Where a new file goes: a probe of a new shape of a check goes in `phase3/`; the regression
for one fixed bug goes in `fix/`; a new corpus is a new `corpusN/` folder, never an addition to
an old corpus (a measured corpus is no longer fresh).

## Adding a fixture

1. **Write the file yourself.** Its content must be your own: no code copied from a
   third-party project, from MathWorks documentation or examples, or from the external
   toolkit. Do not quote MathWorks message text in a file or its comments. Toolkit facts go
   only in `tests/toolkit_expected.tsv`.
2. **List it.** Regenerate the list (sorted, relative to this folder; it is the command that
   made the committed list):

   ```sh
   cd crates/malt_cli/tests/fixtures && find . -name '*.m' | sed 's|^\./||' | LC_ALL=C sort > matlab_files.txt
   ```

   `matlab_conformance.rs` (`fixture_list_names_every_fixture_file`) fails when a `.m` file is
   missing from the list or the list names a file that does not exist.
3. **Ask MATLAB.** Run the generator over every fixture, in one `matlab -batch` session (about
   5 seconds, from the repository root):

   ```sh
   matlab -batch "run('crates/malt_cli/tests/matlab/generate_checkcode.m')"
   ```

   It writes `target/matlab/checkcode.tsv` (the findings), `checkcode_messages.tsv` (MATLAB's
   message text, MathWorks content: under `target/` only, never committed) and
   `checkcode_files.txt` (every file it checked). To look at one file's message text with the
   probe script instead, run `matlab -batch "addpath('docs/scripts'); matlab_probe(list, out)"`
   where `list` is a file of absolute paths; keep its output outside the repository.
4. **Record its rows.** Append the new file's findings to `matlab_expected.tsv`: path, line and
   check ID, one row per distinct finding. A finding on line 0 is written on line 1 (the
   generator already does this). The order of the rows does not matter; keep a file's rows
   together.

   ```sh
   awk -F'\t' -v f="phase3/new_file.m" '$1==f' target/matlab/checkcode.tsv | sort -u \
     >> crates/malt_cli/tests/fixtures/matlab_expected.tsv
   ```

   A file MATLAB reports nothing on gets no rows (see below).
5. **Check malt.** Run the conformance test (below). A difference is a malt bug, or a fixture
   that does not show what you meant. Fix malt or the fixture; never edit the expectations to
   match malt.
6. **Record it for the all-checks run too** (a second expectation file, see 'matlab_expected_all.tsv'
   below). Every new fixture needs rows in both files.
7. **Check the recordings against live MATLAB** (below).

## Files MATLAB reports nothing on

A fixture with zero findings is listed in `matlab_files.txt` and has no rows in
`matlab_expected.tsv`. The conformance test then requires malt to report nothing on it. The
live check (`checkcode_files.txt`) proves MATLAB checked it: a file that is listed but
was not checked fails the live test. 171 of the 1,133 files have no rows at the moment.

## Running the checks

From the repository root.

- **Conformance** (malt against the recording; no MATLAB needed):

  ```sh
  cargo test -p malt-cli --locked --test matlab_conformance -- --nocapture
  ```

- **Live check** (the recording against live MATLAB, and the message text). It starts MATLAB
  itself, or reads files the generator wrote:

  ```sh
  MALT_MATLAB_REQUIRE=1 cargo test -p malt-cli --locked --test matlab_live -- --nocapture
  ```

  With the generator's files (what CI does; MATLAB is not started):

  ```sh
  matlab -batch "run('crates/malt_cli/tests/matlab/generate_checkcode.m')"
  MALT_MATLAB_CHECKCODE=target/matlab/checkcode.tsv \
  MALT_MATLAB_CHECKCODE_MESSAGES=target/matlab/checkcode_messages.tsv \
  MALT_MATLAB_CHECKCODE_FILES=target/matlab/checkcode_files.txt \
  MALT_MATLAB_REQUIRE=1 cargo test -p malt-cli --locked --test matlab_live -- --nocapture
  ```

  The live test has four checks: the release, the findings, the files checked, and the message
  text. Message differences that are accepted (each with a reason) are in
  `tests/matlab/message_allowlist.tsv`.

- **Path index against MATLAB** (`matlab_namespace.rs`) does not read these fixtures; it reads
  `tests/matlab/project`. Run it when the index changes:

  ```sh
  matlab -batch "run('crates/malt_cli/tests/matlab/generate_namespace_facts.m')"
  MALT_MATLAB_FACTS=target/matlab/namespace_facts.json MALT_MATLAB_REQUIRE=1 \
    cargo test -p malt-cli --locked --test matlab_namespace -- --nocapture
  ```

## `matlab_expected_all.tsv`: every check on

`matlab_expected.tsv` is MATLAB's default analysis, which `malt` reproduces under the default `mathworks`
preset. The checks that are off by default (the experimental ones) are compared through a second
recording: MATLAB's `checkcode(file, '-id', '-config=<json>')` with a generated configuration that sets
`enabled: true` for every check ID of the catalogue (`checkcode('-allmsg', file)`), the threshold checks
(`FCNIL`, `FCNOL`, `FCNLL`, `LLMNC`, `MNCSN`, `CYCCOM`, `MCYCCOM`, `SCYCCOM`, `MSCYCCOM`, `ACYCCOM`,
`MACYCCOM`) with a `limit` equal to malt's `CUSTOM_CHECKS` defaults. Left out: the naming checks, the
`CABE` metrics, `LLMCC`, the configuration messages and the `DAF*` checks (`docs/VERIFY.md`, 'Preset all
against MATLAB'). `matlab_expected_all.tsv` has the format of `matlab_expected.tsv` (path, line, check
ID, no message text); a file with no finding has no rows.

Record it after the default one, from the repository root:

```sh
matlab -batch "run('crates/malt_cli/tests/matlab/generate_checkcode_all.m')"
```

It writes `target/matlab/checkcode_all.tsv`, `checkcode_all.release`, `checkcode_all_messages.tsv`
(MathWorks content: never committed) and `checkcode_all_files.txt`. For a new fixture append its rows:

```sh
awk -F'\t' -v f="phase3/new_file.m" '$1==f' target/matlab/checkcode_all.tsv | sort -u \
  >> crates/malt_cli/tests/fixtures/matlab_expected_all.tsv
```

Then compare malt with it (no MATLAB needed) and check the recording against live MATLAB:

```sh
cargo test -p malt-cli --locked --test matlab_conformance_all -- --nocapture
MALT_MATLAB_REQUIRE=1 cargo test -p malt-cli --locked --test matlab_live_all -- --nocapture
```

With the generator's files (what CI does) set `MALT_MATLAB_CHECKCODE_ALL=target/matlab/checkcode_all.tsv`
and `MALT_MATLAB_CHECKCODE_ALL_FILES=target/matlab/checkcode_all_files.txt`. The conformance test accepts
the differences listed in `tests/matlab/all_preset_allowlist.tsv` (path, line, ID, direction, reason; `*`
accepts a whole check) and nothing else; an entry that matches no difference fails. `MALT_ALL_DIFF_OUT=<file>`
writes every difference of a run as `path`, `line`, `ID`, `direction`, which is the starting point for a
new allowlist. A new fixture whose all-checks run differs from malt needs a fix or an allowlist entry with
a reason, never an edit of the expectations.

## The release

`matlab_release.txt` holds the release the expectations were recorded with (`R2026b`). The
live test fails with its own message when the live MATLAB is another release, because reports
drift between releases. To move to a new release: run the generator on it, bring the rows of
`matlab_expected.tsv` in line with the live findings, and update `matlab_release.txt` and the
header of `matlab_expected.tsv` together. The CI pin (`release:` in the Qualify workflow) must
name the same release.
