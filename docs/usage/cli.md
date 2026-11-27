---
icon: lucide/terminal
---

# CLI Reference

## Synopsis

```text
malt [OPTIONS] [PATH]...
```

## Arguments

| Argument   | Description              |
| ---------- | ------------------------ |
| `[PATH]...` | MATLAB files and folders to lint. A folder is searched for `.m` files, below it too. Without a path, malt lints the current folder |

## Options

| Option                | Description                                      |
| --------------------- | ------------------------------------------------ |
| `--fix`               | Apply safe automatic fixes, repeating until nothing changes |
| `--unsafe-fixes`      | With `--fix`: also apply fixes that may change behaviour |
| `--config <PATH>`     | Path to config file: a `.malt.toml`, or a MATLAB `codeAnalyzerConfiguration.json` (default: `.malt.toml` in CWD, else `resources/codeAnalyzerConfiguration.json` in CWD) |
| `--preset <PRESET>`   | Preset to use (`all`, `mathworks`, `recommended`, `monkeyproof`); overrides `[lint] preset` |
| `--strict-config`     | Exit with code `2` when the config file has an unknown rule ID or parameter (each is a warning on stderr otherwise) |
| `--index`             | Use the path index: checks that read the classes on the search path (MCEB) also report the value-class case MATLAB refuses to load. Overrides `[index] enabled` |
| `--no-index`          | Do not use the path index, even when `[index] enabled = true` |
| `-v`, `--verbose`     | Report on stderr which files are linted and the total time; `-vv` adds the time and the number of findings of each file |
| `--color <WHEN>`      | When to colour the text output: `auto` (default), `always` or `never`. Alias `--colour`; see [Colours](#colours) |
| `--output <FORMAT>`   | How findings are written to stdout: `text` (default), `github` (workflow annotations and a step summary) or `gitlab` (the Code Quality report). See [Output formats](#output-formats) |
| `--log-level <LEVEL>` | The least severe findings written as text lines: `off`, `error`, `warning` or `info` (default), whatever `--output` is. See [Output formats](#output-formats) |
| `--output-json <PATH>` | Write the results to a JSON file: the malt version, the configuration used, the files and the findings. See [JSON results](#json-results) |
| `--level <LEVEL>`    | The least severe findings to show: `error` (errors only), `warning` (errors and warnings; alias `warn`) or `info` (everything, the default). See [Levels](#levels) |
| `-h`, `--help`        | Print help information                           |
| `-V`, `--version`     | Print version information                        |

## Commands

### Lint files and folders

Lint MATLAB files and print diagnostics to stdout:

```bash
malt src/compute.m src/utils.m
```

A folder is searched, recursively, for `.m` files. Without a path, malt lints the current folder and everything below
it:

```bash
malt            # the current folder
malt src test   # two folders
```

A file named on the command line is linted whatever its name. A folder is searched for the files that `[lint] include`
picks up (`*.m`), without what `exclude` skips (version control folders, `node_modules` and a few more by default) and
without what the `.gitignore` files of the tree ignore. All of it is set in [`.malt.toml`](../configuration.md#files-and-folders).
Paths are printed relative to where you ran malt. If nothing is found, malt warns on stderr and exits with `0`.

Output format:

```text
<file>:<line>:<column> [<severity>] <rule_id>: <message>
```

Example:

```text
src/compute.m:2:5 [I] NOPTS: Add a semicolon after the statement to hide the output (in a script).
```

### Apply fixes

Run with `--fix` to automatically apply fixes:

```bash
malt --fix src/compute.m
```

Output:

```text
Fixed 2 issues in src/compute.m
src/compute.m:7:1 [W] NASGU: Value assigned to variable might be unused.

Found 1 issue in 1 file.
```

`--fix` lints, applies the fixes, and lints the result again until nothing more
changes, because a fix can enable another one. A file is only rewritten when
something was fixed. What cannot be fixed automatically is reported afterwards.

Fixes come in two kinds:

- Safe fixes keep the meaning of the code, and `--fix` applies them.
- Unsafe fixes are right in the common case but can change behaviour in
  another (`str2num` → `str2double` returns `NaN` where `str2num` evaluated an
  expression; `a & b` → `a && b` errors on array operands). They are reported
  as fixable only with `--fix --unsafe-fixes`. Review the result before you commit it.

A Safe fix is curated against MATLAB itself: for every check that MATLAB's own
Code Analyzer fix (`codeIssues(...).fix(ID)`) rewrites, malt's fix produces the
same text, byte for byte, on the conformance fixtures. A committed regression
set (`tests/fixtures/fix_matlab/`) pins that. The Safe fixes by check ID:

| Check | Rewrite | MATLAB's fix agrees |
| ----- | ------- | ------------------ |
| NOPTS, NOPRT | add the missing `;`, replacing the `,` separator when there is one (`x = 1, y = 2` → `x = 1; y = 2;`) | yes |
| NOSEMI | delete the extra `;` (malt also takes the blanks in front and converges deeper than MATLAB's single-token deletion) | deeper |
| NOCOMMA, NCOMMA | remove the extra comma, insert the missing one between output variables | yes (NOCOMMA), yes (NCOMMA) |
| NBRAK1, NBRAK2 | brackets become parentheses (operand position) or are stripped token-by-token, keeping the inner text, comments included | yes (both) |
| ASGLU | an unused assignment's target becomes `~` | yes |
| FNDEF | the function declaration takes the file name (`function r = other(x)` in `mine.m` → `function r = mine(x)`); the runtime resolves by file name, so nothing else changes | yes |
| FVINR | `(Input)` is added to an attribute-less `arguments` block | yes |
| DSPSP | `disp(sprintf(...))` → `fprintf(...\n)` (statement form only: the assigned form would give the variable `fprintf`'s return count) | statement form |
| FLUDLR | `flipud(fliplr(x))` → `rot90(x,2)` | yes |
| ISCEL, ISCHR | `strcmp(class(x), 'cell')` → `iscell(x)`, ... | yes |
| TRIM2, RPMT1, MRPBW, BADNE | the remaining single-shape rewrites (bracket to `cell` conversions, `repmat`-to-`ones`, `==` with no operands becomes `~=`) | yes |
| FVAL | `feval('f', x)` / `feval(@f, x)` → `f(x)` (also `@pkg.fn` and `"a.b"`); not offered when `f` is a variable or a function of the file, where MATLAB's own fix would index the variable | yes (no fix where MATLAB's would index a variable) |
| MSNU | delete the `%#ok<ID>` directive that suppresses nothing: the whole comment, or just `#ok<ID>` when a remark follows, or the ID and one comma from a list; a list whose every ID is unused is left | yes |
| TWOCM | delete the second comma of `,,` inside brackets (`[1,,2]` is a syntax error in MATLAB) | yes |
| STRQUOT | `string('abc')` → `"abc"` (literal single-quoted argument only) | yes |
| FORMATNOI | `format` → `format("default")` | yes |
| DLABINDEX, DNUMLABS, DGOP, CAXIS, ISDIR, MLNT, DEGTORAD, RADTODEG, WRITEFIS, GCAT, GPLUS, LAB* and the other direct-replacement renames | the callee token is renamed (`labindex` → `spmdIndex`, `caxis` → `clim`, ...), nothing else | yes |

MATLAB offers no fix for two Safe fixes: SEPEX
inserts the separator a one-line header is missing (`if x y = 1 end` →
`if x, y = 1, end`), and NO4LP and ALIGN drop the parentheses around a `for`
header and re-indent an `end` to its keyword. Both are mechanical and
preserve meaning, which is why they are Safe.

The Unsafe ones (reported, applied only with `--fix --unsafe-fixes`):
ISMT, ISCL (`length(x) == 0` → `isempty(x)`, `numel(x) == 1` → `isscalar(x)`: a class can overload
them), EFIND (`isempty(find(x))` → `isempty(find(x, 1))`), FNDSB (`x(find(L))` → `x(L)`:
`L` must be logical), STCI, STNCI (`strcmp(lower(a), b)` → `strcmpi(a, b)`), LOGL on
`logical(zeros(...))`, TNOW2, HOLDALL (`hold all` → `hold on`), PRTOG, PRTPT
(`-opengl` → `-image`, `-painters` → `-vector`), AND2, OR2 (`&`/`|` → `&&`/`||`), INUSD (an unused input becomes `~`),
COMNOT, COMPNOP (`call(...) == true` shapes),
FNAN (`x == NaN` → `isnan(x)` changes the result, which is the point),
IJCL (`i`/`j` → `1i`), STLOW, STREMP, STRCL1, STRCLFH, STRIFCND, ISROW, ISCOL
(`contains`/`strcmpi` rewrites differ on cell and non-text input), MCFIL (the
renamed method becomes the constructor), the performance rewrites
(`str2num` → `str2double`, `fliplr(sort(x))` → `sort(x, 'descend')`,
`repmat`-to-`ones` and friends where sizes can differ), and the arithmetic
`x * 1` family (removed: no equivalent). Where MATLAB marks an issue fixable
but the rewrite is not behaviour-preserving, or the opposite (malt has a fix
MATLAB does not offer), the divergence is recorded in
[`docs/VERIFY.md`](../VERIFY.md), '5.5 Curated safe autofixes'. MATLAB's fixes that malt
deliberately does not have (NUSED, ASSRT, EQEFF, LOGEMP, NOPRC: each changes behaviour or
leaves ragged text) are listed there too.

### Explicit configuration

By default, malt looks for `.malt.toml` in the current working directory, then for MATLAB's `resources/codeAnalyzerConfiguration.json` there. A `.json` file given to `--config` is read as a MATLAB configuration, alone. Override with `--config`:

```bash
malt --config configs/strict.toml src/**/*.m
```

If `--config` is specified and the file does not exist, malt exits with an error. If no `--config` is given and `.malt.toml` is not found, malt uses the `mathworks` preset (see [Configuration](../configuration.md#presets)).

### Path index

`--index` lets some checks use the classes on the MATLAB search path (the working directory, or the `[index] paths` folders, plus the linted file's own folder). Without it, a file is linted on its own, as MATLAB's Code Analyzer does. The folders are read from `[index]` in `--config`, or in `.malt.toml`; see [Configuration](../configuration.md#path-index).

```bash
malt --index src/**/*.m
```

### Verbose output

`-v` prints one line per file on stderr, in command-line order, and a closing line with the number of files and the total time. `-vv` adds the time spent on each file (reading, linting and, with `--fix`, fixing) and its number of findings. Stdout, with the diagnostics and the summary, does not change, so `-v` is safe in scripts.

```bash
malt -vv src/*.m
# malt: linted src/compute.m in 4.33 ms (3 findings)
# malt: skipped src/missing.m (NOFIL)
# malt: 2 files in 4.56 ms
```

## Levels

`--level` limits what is shown to a severity and the more severe ones:

```bash
malt --level error src/*.m      # errors only
malt --level warning src/*.m    # errors and warnings
malt src/*.m                    # everything (--level info)
```

A hidden finding is not printed, not counted in the closing summary and does not set the exit
code: with `--level error`, a file that has only warnings exits 0. An unreadable file (NOFIL,
RDERR) is an error and is always shown. `--fix` is not filtered: it still applies the fixes of
hidden findings. The severity of a rule can be changed in the config file (see
[Configuration](../configuration.md)).

## Output formats

`--output` chooses how findings reach stdout. The text lines, the exit code and every other option work the same in each.

### GitHub Actions

`--output github` prints one workflow command per finding, which GitHub shows as an annotation on the file and line:

```text
::error file=src/a.m,line=2,col=11,title=EOLPAR::A '(' might be missing a closing ')', causing invalid syntax at end of line.
```

Errors become `error` annotations, warnings `warning` and info findings `notice`. Paths are made relative to `GITHUB_WORKSPACE`. When `GITHUB_STEP_SUMMARY` is set, which it is in a workflow step, malt also appends a markdown summary to that file: the counts by severity, the rules that fired most, and the findings of each file in a folded list. GitHub shows only the first 10 errors and 10 warnings of a step as annotations, so the summary is the complete list.

### GitLab

`--output gitlab` prints the [Code Quality report](https://docs.gitlab.com/ci/testing/code_quality/) as JSON on stdout. The text lines and the closing summary go to stderr, so a redirect gets a clean report:

```yaml title=".gitlab-ci.yml"
malt:
  script:
    - malt --output gitlab > gl-code-quality-report.json
  artifacts:
    reports:
      codequality: gl-code-quality-report.json
```

Errors are `critical`, warnings `major` and info findings `info`. The fingerprint of a finding comes from its file, rule and message, so it stays the same when the line moves. GitLab has no step summary: the report shows as findings in the merge request.

### Quieting the text lines

`--log-level` says which findings are written as text lines, whatever `--output` is. `--log-level off` leaves the log clean while the annotations, the report, the JSON file and the exit code still carry every finding. It differs from `--level`, which hides findings altogether.

```bash
malt --output github --log-level off      # annotations only
malt --log-level error                    # text lines for errors, the count for all
```

### JSON results

`--output-json results.json` writes the run as JSON, next to any `--output`. The file is written whether or not there are findings. Its keys:

| Key | Content |
| --- | ------- |
| `schema_version` | The layout of the file, `1` today |
| `malt_version` | The version of malt that ran |
| `config` | The configuration used: the config files read, `preset`, `experimental`, `inline_suppression`, `include`, `exclude`, `respect_gitignore`, `force_exclude`, `categories`, `rules` with their parameters, and the resolved `limits` |
| `files` | The files linted |
| `summary` | `files_linted`, `files_with_findings`, `findings`, `by_severity` and `by_rule` |
| `findings` | Each finding: `file`, `line`, `column`, `rule`, `severity`, `message` and `fix` (`null`, `"safe"` or `"unsafe"`) |

## Colours

Diagnostics, the summary, warnings and errors are coloured when the output goes to a terminal: severity tags (`[E]` red, `[W]` yellow, `[I]` cyan), the file and rule ID in bold, the line and column dimmed, and the fix hint in green. Machine-readable output is never coloured, and nothing is coloured in a pipe or a file unless you force it.

`--color` decides first:

| `--color` | Result |
| --------- | ------ |
| `always`  | Coloured, whatever the stream, the environment, or `NO_COLOR` |
| `never`   | Plain, whatever the environment |
| `auto`    | Decided by the environment and the stream, as below |

In `auto` mode, malt checks these in order and stops at the first that applies, separately for stdout (diagnostics) and stderr (warnings, errors, `-v` lines):

| Order | Environment | Result |
| ----- | ----------- | ------ |
| 1 | `NO_COLOR` set and not empty | Plain |
| 2 | `CLICOLOR_FORCE` or `FORCE_COLOR` set and not `0` | Coloured |
| 3 | `CLICOLOR=0` | Plain |
| 4 | `TERM=dumb` | Plain |
| 5 | none of the above | Coloured if the stream is a terminal, plain otherwise |

On Windows, malt turns on the console's virtual terminal processing when it colours the output.

```bash
malt --color always src/*.m | less -R   # keep the colours through a pager
NO_COLOR=1 malt src/*.m                 # plain text, even in a terminal
```

## Exit codes

| Code | Meaning                              |
| ---- | ------------------------------------ |
| `0`  | No diagnostics found (with `--fix`: none remain after fixing) |
| `1`  | One or more diagnostics found (with `--fix`: some could not be fixed) |
| `2`  | Runtime error (file not found, parse failure, bad config; with `--strict-config`, also any unknown rule ID or parameter in the config) |

## Shell glob expansion

malt does not perform its own glob expansion. Use your shell's globbing:

```bash
# Bash/Zsh: lint all .m files recursively
malt **/*.m

# Or use find
find src -name '*.m' -exec malt {} +
```

## Examples

```bash
# Lint a single file
malt main.m

# Lint all MATLAB files in a directory (using shell glob)
malt src/**/*.m

# Fix issues with a custom config
malt --fix --config .malt-strict.toml lib/**/*.m

# Check version
malt --version
```
