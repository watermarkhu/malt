# Conformance fixtures

The `.m` files here are what malt is checked against MATLAB's Code Analyzer on. Every
`.m` file under this folder, at any depth, is a fixture. `matlab_expected.tsv` holds what
MATLAB R2026b reports for each of them, `matlab_files.txt` lists them, and
`matlab_release.txt` names the release. The method and the policy are in
[`docs/VERIFY.md`](../../../../docs/VERIFY.md).

## Layout

| Where | What it holds | Naming |
| --- | --- | --- |
| `fix/` | Regression files: one per fixed case (197 files). | `<area>__<name>.m`, with a double underscore, e.g. `bugs__f1.m` |
| `phase2/`, `phase3/` | Probe shapes from the audit phases. `phase3/` is the current one. | `<check>_<nn>.m` (`bdlog2_02.m`), `<topic>_<what>.m` (`nasgu_error_literal.m`) or `yc_<letter><nn>.m` |
| `corpusN/<letter>/` | Fresh corpora, written apart from the rules to measure how well malt generalises. | `<letter>_<topic>.m` in a folder per letter (`a/`, `b/`, ...) |
| `realistic/` | Realistic programs, with its own `README.md`. | free |
| top level | The expectations and the lists: `matlab_expected.tsv`, `matlab_files.txt`, `matlab_release.txt`. | |

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
6. **Check the recording against live MATLAB** (below).

## Files MATLAB reports nothing on

A fixture with zero findings is listed in `matlab_files.txt` and has no rows in
`matlab_expected.tsv`. The conformance test then requires malt to report nothing on it. The
live check (`checkcode_files.txt`) proves MATLAB checked it: a file that is listed but
was not checked fails the live test. 97 of the 507 files have no rows at the moment.

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

## The release

`matlab_release.txt` holds the release the expectations were recorded with (`R2026b`). The
live test fails with its own message when the live MATLAB is another release, because reports
drift between releases. To move to a new release: run the generator on it, bring the rows of
`matlab_expected.tsv` in line with the live findings, and update `matlab_release.txt` and the
header of `matlab_expected.tsv` together. The CI pin (`release:` in the Qualify workflow) must
name the same release.
