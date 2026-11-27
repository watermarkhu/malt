---
icon: lucide/settings
---

# Configuration

malt is configured via a `.malt.toml` file in your project directory. This page is the complete reference for the configuration schema.

## Config File Discovery

malt looks for `.malt.toml` in the current working directory by default. You can override this with the `--config` flag:

```bash
malt --config path/to/config.toml src/**/*.m
```

If no config file is found and `--config` is not specified, malt uses the built-in `mathworks` preset (see [Presets](#presets)). malt also reads MATLAB's own `resources/codeAnalyzerConfiguration.json`; see [MATLAB Code Analyzer configuration files](#matlab-code-analyzer-configuration-files).

## MATLAB Code Analyzer configuration files

MATLAB reads a project's `resources/codeAnalyzerConfiguration.json` (for `codeIssues` and the editor), and `checkcode(file, '-config=<path.json>')` takes such a file as well. malt reads the same format for the parts it implements. The schema version malt reads is `1.x` (`schemaVersion` is optional).

```json title="resources/codeAnalyzerConfiguration.json"
{
  "schemaVersion": "1.1.0",
  "checks": {
    "NOPRT": { "enabled": false },
    "NASGU": { "severity": "error" },
    "CYCCOM": { "enabled": true, "limit": 10 },
    "DAFSC": { "enabled": true }
  },
  "naming": {
    "variable": { "maxLength": 20, "casing": "lowerCamelCase" }
  },
  "ignoredFiles": ["generated/*.m"]
}
```

| JSON | malt |
| ---- | ---- |
| `checks.<ID>.enabled` | `false` drops that check's findings, whichever engine emits them. `true` turns on a custom check or a naming check (below). It does not turn on any other check that malt keeps off under `mathworks` (for example `ERTAG`, a gap listed in `docs/VERIFY.md`). |
| `checks.<ID>.severity` | `"error"`, `"warning"` or `"info"` (case-sensitive) becomes the severity of that check's findings. |
| `checks.<threshold>.limit` | The limit of a threshold check: `FCNIL`, `FCNOL`, `FCNLL`, `LLMNC`, `MNCSN`, `CYCCOM`, `MCYCCOM`, `SCYCCOM`, `MSCYCCOM`, `ACYCCOM`, `MACYCCOM`. The check also needs `"enabled": true`. Enabled without a `limit`, it reports nothing (as in MATLAB). |
| `checks.<DAF>.enabled`, `checks.SYSBANG.enabled` | Switch on the disallow check (`DAFSC`, `DAFAF`, `DAFNF`, `DAFTC`, `DAFPV`, `DAFCO`, `DAFBR`, `DAFRT`, `DAFCF`, `DAFCV`, `DAFCVC`, `DAFVI`, `DAFVO`, `SYSBANG`). |
| `naming.<group>.maxLength`, `minLength` | Integers from 1 to 2048. Groups: `class`, `enumeration`, `event`, `function`, `localFunction`, `method`, `nestedFunction`, `property`, `variable`. |
| `naming.<group>.regularExpression` | The whole name must match (a non-empty string). |
| `naming.<group>.casing` | `lowerCamelCase`, `UpperCamelCase`, `lowercase_with_underscores`, `UPPERCASE_WITH_UNDERSCORES`, `lowercase` or `UPPERCASE`. |
| `naming.<group>.requiredPrefix`, `disallowedPrefix`, `disallowedPhrase`, `requiredSuffix`, `disallowedSuffix` | A non-empty string, or a non-empty array of strings. |
| `ignoredFiles` | Patterns skipped by the run (below). |
| `schemaVersion`, `baseConfiguration` | Checked; see the notes below. |

Not supported, and reported as warnings that name the setting (an error under `--strict-config`): custom checks (a `rule` with a `template`), `messageText`, `formatting`. A custom check is ignored, so MATLAB's findings for it are missing from malt's output.

Where malt differs from MATLAB:

- Invalid files. A file MATLAB refuses (bad JSON, a value of the wrong type, an unsupported `schemaVersion`) is an error with exit code `2`. A value MATLAB only warns about (an unknown check or field, a bad severity, a casing style it does not know) is ignored, with a warning, and the rest of the file applies.
- Parent folders. MATLAB looks for the file in the `resources` folder of the file's folder and its parents. malt reads one file per run: the one of the working directory (or the `--config` path). The `baseConfiguration` value is therefore not used to inherit from a parent: malt's base is always the `mathworks` preset.
- Naming and the file's folder. The naming checks follow the file's own configuration, as MATLAB's do.
- Thresholds. `MNCSN` is reported at the function's line rather than at the nested statement (MATLAB reports the statement). This is a known difference in the rule.

Discovery and precedence, first match wins:

1. `--config <path>`: a `.json` file is a MATLAB configuration and is used alone. Any other file is a `.malt.toml`.
2. `.malt.toml` in the working directory. If it sets `[lint] matlab_config`, that file is read first and the TOML is layered over it (below). Otherwise the TOML is used alone, and `resources/codeAnalyzerConfiguration.json` is not read (`-v` says so).
3. `resources/codeAnalyzerConfiguration.json` in the working directory.
4. The defaults (`mathworks` preset).

The layering is per key: a `[lint.rules]` or `[lint.categories]` entry in the TOML replaces the JSON's entry for the same rule or category. The check settings of the JSON (`enabled` and `severity` of a check ID) are not changed by the TOML, so a check the JSON disables stays disabled. `--preset` still overrides the preset.

```toml title=".malt.toml"
[lint]
matlab_config = "resources/codeAnalyzerConfiguration.json"

[lint.rules."naming.variable.maxLength"]
max = 30          # overrides the JSON's maxLength for this check
```

`ignoredFiles` patterns are relative to the folder that holds the `resources` folder (for a file that is not in a `resources` folder, to its own folder). Each segment matches one folder level, with `*` and `?` inside a name: `generated/*.m` skips the `.m` files directly in `generated`. A skipped file is not linted, even when it is named on the command line (MATLAB's `codeIssues` skips it too; `checkcode` does not, and malt follows `codeIssues`). `-v` reports how many files were skipped.

The evidence for each row is in `docs/VERIFY.md`, section 'MATLAB configuration file (5.8)'.

## Presets

Presets are named, curated sets of default category overrides. Pick one with
`[lint] preset`, or the `--preset` CLI flag, which takes priority over the
config file.

| Preset | Disabled categories | Description |
| ------ | ------------------- | ----------- |
| `mathworks` *(default)* | `custom-checks`, `naming`, `monkeyproof` | Matches MATLAB Code Analyzer's factory configuration: complexity metrics and naming conventions are opt-in, and the individual checks MATLAB's default analysis never reports are off too (see [Default Behavior](#default-behavior)). |
| `all` | `monkeyproof` | Every rule of MATLAB's Code Analyzer and malt enabled, including the [default-off engines](#default-behavior). The MonkeyProof standard is a separate set and stays off. |
| `recommended` | `performance`, `readability`, `formatting`, `suggested-improvements`, `naming`, `custom-checks`, `monkeyproof` | Errors and warnings on; Info-level style/suggestion checks off. |
| `monkeyproof` | `custom-checks`, `naming` | The `mathworks` defaults plus the [MonkeyProof Coding Standard](monkeyproof.md). |

```toml
[lint]
preset = "recommended"
```

```bash
malt --preset all src/**/*.m
```

A preset only removes categories; the specialized engines that are
[off by default](#default-behavior) stay off under `mathworks` and
`recommended`. `all` is the exception and turns them on as well.

The `monkeyproof` preset is the `mathworks` defaults plus the [MonkeyProof Coding Standard](monkeyproof.md), an extra set of style checks that is not part of MATLAB's Code Analyzer.

Precedence: `--preset` > `[lint] preset` > `mathworks`. Explicit
`[lint.categories]` / `[lint.rules]` entries always win over the preset, so a
preset-disabled category can be re-enabled:

```toml
[lint]
preset = "recommended"

[lint.categories]
readability = "info"   # re-enable the readability suggestions
```

Presets are defined in `crates/malt_core/src/preset.rs`; adding one is a single
row in the `PRESETS` table.

## Experimental checks

Every check has a maturity: `stable` or `experimental`. It is metadata, shown in the
Maturity columns of the [Rules Reference](rules.md) and of each rule page. A check is
stable when MATLAB's Code Analyzer reported it on a recorded fixture, or when it belongs
to the default analysis that the conformance suite covers. A check is experimental when
MATLAB reported it nowhere that was recorded and it is off by default: it was checked
against MathWorks' documentation only, or MATLAB stayed silent on it in the probes, or its
data is marked low-reliability. The classification, with the evidence for each group, is in
[VERIFY.md](VERIFY.md#rule-maturity-plan-63) and [How malt is verified](verification.md).

Experimental checks are off by default (they are already off under the `mathworks` and
`recommended` presets). They run when any of these applies:

- `[lint] experimental = true` turns all of them on for the preset in use. The preset's
  disabled categories stay disabled (`naming` and `custom-checks` remain opt-in by category).
- `--preset all` (or `[lint] preset = "all"`) runs everything, as before.
- An explicit setting for the engine, the category or the check (for example
  `[lint.categories] code-generation = "error"` or `[lint.rules] FCDQS = "error"`) turns that
  one on and always wins; an explicit `"off"` also wins over `experimental = true`. A check
  of a default-on engine has a per-check switch only where the engine offers one (the `FC*`
  gates, `IDISVARHIGH`); the other off-list checks need `experimental = true` or `--preset all`.

```toml title=".malt.toml"
[lint]
experimental = true    # default: false
```

The default output of malt does not change: with `experimental = false` (the default) the
results are the same as before the flag existed.

## Schema Overview

```toml title=".malt.toml"
[lint]
# Global lint settings
include = ["*.m"]                # Files a folder run picks up
exclude = ["vendor/", "third_party/"]
extend_exclude = ["generated/"]  # Added to exclude
respect_gitignore = true         # Skip what .gitignore ignores
force_exclude = false            # Apply exclude to files named on the command line
preset = "recommended"          # Preset: all | mathworks | recommended | monkeyproof
experimental = false            # Also run the experimental checks (see "Experimental checks")

[lint.categories]
# Per-category severity overrides
performance = "off"            # Disable every performance rule
compatibility = "warn"         # All compatibility checks warn

[lint.rules]
# Per-rule configuration (shorthand or full table)
NOSEMI = "warn"                # Shorthand: just set severity
AGROW = "off"                  # Disable a rule
COMPAT = "error"               # Escalate to error

[lint.rules.GOOD_PRACTICES_ENGINE]   # Full table: severity + rule parameters
severity = "warn"
max_variable_name_length = 32

[lint.limits]
# Resource limits (see "Resource limits" below; raising them is risky)
max_file_size = "5 MiB"
```

## `[lint]` Section

### Files and folders

`malt` without a path lints the current folder, and a folder on the command line is searched for MATLAB files. These
settings say which files that search finds. They only matter for folders: a file named on the command line is linted as
it is, unless `force_exclude` is set.

The patterns are gitignore patterns (`vendor/`, `**/generated`, `test/*.m`, `!keep.m`). A pattern without a slash matches
at any depth, and a leading slash anchors it. They are relative to the folder of the configuration file, which is the
current folder when the file is `.malt.toml` there. The search follows no symbolic links.

#### `include`

The files a folder run picks up, as patterns. **Default:** `["*.m"]`.

```toml
[lint]
include = ["*.m", "*.matlab"]
```

#### `exclude`

Files and folders to skip. A folder that matches is not entered. Setting it replaces the default list.

```toml
[lint]
exclude = [
    "vendor/",
    "third_party/",
    "test/fixtures/",
]
```

**Type:** array of strings. **Default:** `[".git/", ".hg/", ".svn/", "node_modules/", ".venv/", "slprj/"]`

#### `extend_exclude`

Patterns added to `exclude`, so the default list stays.

```toml
[lint]
extend_exclude = ["generated/"]
```

#### `respect_gitignore`

Whether the `.gitignore` files of the tree, and of the folders above it, exclude files from a folder run.
**Default:** `true`. The `.gitignore` files count whether or not the folder is a git repository.

#### `force_exclude`

Whether `exclude` also applies to files named on the command line. **Default:** `false`, because naming a file is asking for
it. Turn it on when an editor or a pre-commit hook passes every changed file and you still want `exclude` to hold.

```toml
[lint]
exclude = ["vendor/"]
force_exclude = true
```

### `matlab_config`

The MATLAB Code Analyzer configuration file (`codeAnalyzerConfiguration.json`) that this file layers over. See [MATLAB Code Analyzer configuration files](#matlab-code-analyzer-configuration-files).

```toml
[lint]
matlab_config = "resources/codeAnalyzerConfiguration.json"
```

**Type:** string (a path, relative to the working directory)
**Default:** none

### `preset`

The named preset to base the configuration on. See [Presets](#presets).

```toml
[lint]
preset = "recommended"
```

**Type:** string (`"all"`, `"mathworks"`, `"recommended"` or `"monkeyproof"`)
**Default:** `"mathworks"`

### `experimental`

Whether the experimental checks run. See [Experimental checks](#experimental-checks).

```toml
[lint]
experimental = true
```

**Type:** boolean (anything else is an error, exit code `2`)
**Default:** `false`

## `[lint.categories]` Section

Set the severity for every rule in a category at once. Keys are category slugs; values are severity strings. This is resolved *below* per-rule config: a rule-level override always wins over its category-level setting.

```toml
[lint.categories]
performance = "off"            # Disable all performance rules
compatibility = "warn"         # All compatibility rules warn
formatting = "info"            # All formatting rules are informational
```

**Available categories:**

| Category | Config Key | Includes |
| -------- | ---------- | -------- |
| Incomplete Analysis | `incomplete-analysis` | QUIT, NOFIL, RDERR, MDEEP … |
| Syntax Errors | `syntax-errors` | BADCT, RESWD, DOUQT, NOPAR2, SYNEND … |
| Language Specification | `language-specification` | PF\*, FV\*, MC\*, AT\* checks |
| Bugs | `bugs` | IFBDUP, CTRUE, LOGEMP, INCR, DECR … |
| Custom Checks | `custom-checks` | Cyclomatic complexity, nesting, line metrics |
| Naming | `naming` | `naming.*` checks (81) |
| Compatibility | `compatibility` | Deprecated/removed function checks |
| Forward Compatibility | `forward-compatibility` | FCLEN, FCCPV, FCDQS, FCFAV … |
| Good Practices | `good-practices` | eval, error handling, OOP practices |
| Unset Variables | `unset-variables` | NODEF, PSET, USENS … |
| Unused Constructions | `unused-constructions` | NOEFF, NUSED, EQEFF, UNRCH … |
| Suggested Improvements | `suggested-improvements` | Function replacement suggestions |
| Readability | `readability` | ISCHR, IJCL, NBRAK2, STREMP … |
| Formatting | `formatting` | NOSEMI, NOCOMMA, ALIGN, NOPRT … |
| Performance | `performance` | AGROW, PFBNS, AND2, MINV … |
| Code Generation | `code-generation` | MATLAB Coder constraints |
| Fixed-Point | `fixed-point` | FPASE |
| Deployment | `deployment` | MATLAB Compiler constraints |
| System Objects | `system-objects` | SONUMIN, SOTUNPROP\* … |
| Unsupported | `unsupported` | MCADE, AWTIUD, FEATUD … |
| Behavior Changes | `behavior-changes` | Version behavior-change checks |

## `[lint.rules]` Section

Configure individual rules. Each key is a rule ID. Values can be either a severity shorthand (string) or a full configuration table.

Most rule *engines* are registered under their engine ID (e.g. `GOOD_PRACTICES_ENGINE`, `LANGUAGE_SPEC_ENGINE`, `COMPAT`), and each engine exposes its individual check IDs through its parameters. See Per-Engine Parameters below.

### Severity Shorthand

Set a rule's severity with a single string value:

```toml
[lint.rules]
NOSEMI = "warn"       # Override severity to warning
AGROW = "error"       # Override severity to error
COMPAT = "info"       # Override severity to info
CUSTOM_CHECKS = "off" # Disable the rule entirely
```

### Valid Severity Values

| Value | Aliases | Effect |
| ----- | ------- | ------ |
| `"error"` | `"err"` | Rule violations are errors |
| `"warn"` | `"warning"` | Rule violations are warnings |
| `"info"` | `"note"` | Rule violations are informational |
| `"off"` | `"false"`, `"disabled"` | Rule is completely disabled |

### Full Table Configuration

For rules that accept parameters, use a full TOML table:

```toml
[lint.rules.GOOD_PRACTICES_ENGINE]
severity = "warn"
max_variable_name_length = 32
```

Or equivalently as an inline table:

```toml
[lint.rules]
GOOD_PRACTICES_ENGINE = { severity = "warn", max_variable_name_length = 32 }
```

The `severity` key is always optional in a full table. If omitted, the rule uses its default severity.

All other keys in the table are rule-specific parameters (see Per-Engine Parameters).

### Validation

malt checks the rule keys and the parameter names when it loads the file. Each problem is one warning on stderr, and the run goes on with the valid part of the configuration:

```text
warning: .malt.toml: unknown rule ID 'NOSMI' in [lint.rules] (did you mean 'NOSEMI'?)
warning: .malt.toml: unknown parameter 'max_variable_nam_length' for rule 'GOOD_PRACTICES_ENGINE' (did you mean 'max_variable_name_length'?)
```

- Rule keys must be a rule ID malt knows, spelled exactly: an engine ID (`GOOD_PRACTICES_ENGINE`), a check ID (`NOSEMI`, `FVINR`), or a `naming.<entity>.<check>` ID. Rule keys are case-sensitive, so `nosemi` is a problem. A rule that the preset leaves off is still a valid key.
- Parameter names are checked for the engines that take parameters (see Per-Engine Parameters), including the parameters of an engine that is turned off. `severity` is always allowed in a table. An engine without parameters, such as `NOSEMI`, reports every key other than `severity`. Keys of check IDs that are not engines are not checked.
- Dotted IDs must be quoted. `[lint.rules.naming.class.casing]` is read as the rule `naming` with nested tables, and malt reports it. Write `[lint.rules."naming.class.casing"]`.
- Presets and categories are not warnings. An unknown `[lint] preset` or `[lint.categories]` key is an error (exit code `2`), as before.

Add `--strict-config` to turn the warnings into an error: malt then prints the warnings and exits with code `2` without linting. Use it in CI to keep typos out of the configuration. Without it, the exit code does not depend on the warnings.

## Resource limits

`[lint.limits]` sets the limits that stop malt from using too much memory or time on a pathological input: a generated file of tens of megabytes, a single line of a million numbers, code nested thousands of levels deep. malt measures the input before it reads or parses it, and refuses what is over a limit.

::: warning Raising a limit can crash the machine
Memory and time grow with the size of a file, not with what the code does: malt used about 180 bytes of memory per byte of ordinary statements (a 16 MB file took 2.5 GB and 51 s) and about 500 bytes per byte of one huge matrix (a 2 MB line took 1 GB and 13 s, an 8 MB line did not finish in 60 s at 3.6 GB). A raised limit applies to every file malt reads, including hostile or machine-generated ones, and the process may then take all the memory of a CI runner, make a shared machine swap until it hangs, or crash an editor or a browser tab.

The defaults are deliberately conservative. They protect shared CI runners and editors, where one file must not take the machine down. Raise a limit only for a project you trust, only as far as you need, and prefer `exclude` for generated files. The limits can be raised but not switched off: `0` is an error, there is no "unlimited".
:::

| Key | Default (native) | Default (wasm) | Beyond it | Protects against | Measured basis |
| --- | --- | --- | --- | --- | --- |
| `max_file_size` | `5 MiB` | `2 MiB` | The file is not read and gets a `MALT_TOO_BIG` error at line 1. | Memory and time that grow with file size. | 4 MiB of statements: 0.8 GB, 10 s; 2 MB single-line matrix: 1 GB, 13 s. |
| `max_tree_depth` | `1000` | `200` | The file is parsed but not analyzed and gets a `MALT_TOO_DEEP` error. | Stack overflow and slow analyses on deeply nested code. | Real code stays below 100 levels; 200 levels in wasm are safe, a chain of about 750 binary operators traps V8. |
| `max_nesting` | `10000` | `1000` | The file is not parsed and gets a `MALT_TOO_DEEP` error. | A nesting bomb such as `((((…` being parsed at all. | Parsing costs about 0.4 KB per bracket level (256,000 levels: 100 MB). |
| `max_threads` | number of cores | `1` | Not a diagnostic: fewer threads run. | Memory that multiplies with every thread that analyzes a file at once. | Each worker holds the memory of its own file, so the total is the sum over the files in flight (one 8 MB file: 1.4 GB). |

The values are whole numbers; sizes may also be strings with a unit: `"512 KiB"`, `"10 MiB"`, `"1 GiB"` (powers of 1024), or `"5 MB"` (powers of 1000), or a plain number of bytes. An invalid value (zero, negative, an unreadable size) is a configuration error and malt exits with code `2`. An unknown key is a warning, as for the other tables (an error with `--strict-config`).

A file that stops the analysis gets one diagnostic and is skipped, and the run goes on with the other files; the exit code is `1`:

```text
big.m:1:1 [E] MALT_TOO_BIG: malt did not analyze this file: it is 6 MiB (6291456 bytes) and malt's limit is 5 MiB (5242880 bytes) (max_file_size). To analyze it, raise max_file_size under [lint.limits] in .malt.toml; larger files can use very large amounts of memory. This is not a MATLAB message.
```

`MALT_TOO_BIG` and `MALT_TOO_DEEP` are malt's own IDs; MATLAB has no such messages (it analyzes files of 16 MB and more). With `--fix` a refused file is left unchanged.

- `max_file_size` is checked from the file's metadata before the file is read, so a huge file never enters memory. In the browser (`malt-wasm`), `check` compares the length of the content.
- `max_nesting` is a scan of the text, in one pass and constant memory, that counts `(`, `[` and `{` and skips comments (`%`, `...`, `%{ … %}`) and strings. It does not count block keywords (`if`, `for`, …): `end` also closes an index, so they cannot be told apart reliably before the parse. Deeply nested blocks are bounded by `max_tree_depth` after the parse, which is cheap (30,000 nested `while` loops parse in 0.16 s and 47 MB), and MATLAB's own limits (NOSPC, MDEEP, MXASET) report most nesting bombs first.
- `max_threads` is the total for the process: the workers that lint files side by side and the threads that check one large file share it, so the process never starts more. With `max_threads = 1` malt runs on one thread; the output is the same.
- In `malt-wasm`, `Linter.from_toml` reads the same table. A key you leave out keeps the lower wasm default; raising `max_tree_depth` there can make the module trap.

Raise a limit for one project, for example a repository with a generated data file of 8 MiB that you trust:

```toml title=".malt.toml"
[lint.limits]
max_file_size = "16 MiB"
```

Lower them on a small shared runner:

```toml title=".malt.toml"
[lint.limits]
max_file_size = "1 MiB"
max_threads = 2
```

The measurements behind the defaults are in [VERIFY.md, Resource limits](VERIFY.md#resource-limits).

## `[index]` Section

`[index]` configures the path index: the folders the MATLAB search path is made of, and whether the linter uses them. The index is off by default, so malt's output does not depend on the folders around a file. Other tables are not affected by it, and `--strict-config` does not warn about it. An invalid `[index]` table is an error even while the index is off.

```toml title=".malt.toml"
[index]
enabled = true
paths = [".", "lib/**", "src/*", "tools/helper"]
```

- `enabled` (default `false`) turns the index on for the linter. `malt --index` turns it on and `malt --no-index` turns it off, whatever the file says. See [Path index](#path-index) below for what changes.
- Without `paths`, the search path is the working directory. With `paths`, the listed folders replace it; add `"."` to keep it.
- Relative patterns resolve against the working directory. The folder of the linted file and any extra folders always come first.
- `lib/**` adds `lib` and its plain subfolders, recursively, as MATLAB's `genpath` does. `+pkg`, `@cls`, `private` and hidden folders are never added, nor anything below them.
- Any other pattern adds only the folders it matches, not recursively. `*`, `?` and `[...]` match within one folder name; `` in the middle matches any number of levels (`lib//deeper`).
- The result is deduplicated and sorted by path. A folder that does not exist, or a pattern that matches nothing, is a warning, and the rest of the path is used.
- An invalid `[index]` table (an unknown key, or `paths` not a list of strings) or a `.malt.toml` that is not TOML is an error.

## Path index

With the index on, malt reads the classes that a class file's superclasses name on the search path (its own folder first, then `paths`), and one check reports more than MATLAB's Code Analyzer does:

- MCEB (events in a class that is not a handle class) is also reported for a class whose superclasses are all value classes on the path. MATLAB refuses to load such a class.

A superclass that is not on the path, or whose own bases are not, is unknown, and nothing is reported for it. Nothing else changes: with the index off, the diagnostics are the Code Analyzer's, whatever folders surround the file. These extra findings are malt's, not MATLAB's; the evidence is in `docs/VERIFY.md`, section 'Path index, stage 3.4'.

## Default Behavior

When no `.malt.toml` is present:

- The `mathworks` preset is active, so `custom-checks` and `naming` are off, as are the individual checks that
  MATLAB's default Code Analyzer never reports (for example `ERTAG`, `WNTAG`, `CTCH`, `BDLOG2`, `LOGSUM`, the
  forward-compatibility gates `FC*`, and the behavior-change entries MATLAB lists as low reliability). The `all`
  preset turns them back on. Per-check and per-category entries re-enable some of them. `[lint.rules]
  FCDQS = "error"` (or the `forward-compatibility` category) turns an `FC*` gate back on, and the
  `behavior-changes` category turns the low-reliability entries back on. The engine off-list
  (`ERTAG`, `WNTAG`, `CTCH`, `BDLOG2`, `LOGSUM`, …) has no per-check switch: only `preset = "all"` turns
  those back on. The full list with the probe evidence is in
  [docs/VERIFY.md](https://github.com/watermarkhu/malt/blob/main/docs/VERIFY.md#policy).
- These specialized engines are off by default under every preset except
  `all` (opt in per rule or per category):
  - `CODEGEN_ENGINE` (`code-generation`): MATLAB Coder constraints
  - `DEPLOYMENT_ENGINE` (`deployment`): MATLAB Compiler constraints
  - `SYSTEM_OBJECTS_ENGINE` (`system-objects`): `matlab.System` validation
  - `UNSUPPORTED_ENGINE` (`unsupported`): platform-specific features
- The checks marked experimental (a superset of the above: also the off-list checks and the
  low-reliability entries) are off; `[lint] experimental = true` turns them on, see
  [Experimental checks](#experimental-checks).
- All other rules are enabled at their default severity, including `SUGGESTED_IMPROVEMENTS` (replacement hints,
  which MATLAB reports by default)
- No files are excluded
- No rule-specific parameters are set

Each rule page shows whether the rule is enabled by default. To opt in:

```toml title=".malt.toml"
[lint.categories]
code-generation = "error"   # enable the whole category

[lint.rules]
NAMING_ENGINE = "info"      # or enable a single rule
```

## Severity Override Precedence

The effective severity for each rule is resolved as:

1. Per-rule override (if `[lint.rules.XXXX]` specifies `severity` or uses the shorthand)
2. Per-category override (if `[lint.categories.XXXX]` is set for the rule's category)
3. Rule's built-in default (if neither override is present)

The config override applies to all diagnostics produced by that rule. Rules always emit their default severity, and the engine stamps the override afterward.

## Per-Engine Parameters

Multi-check engines take their parameters through their engine ID. Every engine also accepts `disabled_checks` (or `skip_checks`) to turn off specific check IDs.

```toml
[lint.rules.LANGUAGE_SPEC_ENGINE]
disabled_checks = ["PFEVC", "PFANSLP"]   # Turn off specific checks
```

| Engine ID | Category | Parameters |
| --------- | -------- | ---------- |
| `INCOMPLETE_ANALYSIS` | incomplete-analysis | `max_parse_errors` (TMSMS, default 1000), `max_paren_depth` (MDEEP, default 32), `max_function_depth` (DEEPN, default 17); MATLAB's own limits, the rule cannot be disabled |
| `SYNTAX_ERRORS_ENGINE` | syntax-errors | `disabled_checks` |
| `LANGUAGE_SPEC_ENGINE` | language-specification | `disabled_checks` |
| `BUGS_ENGINE` | bugs | `debug_functions`, `higher_order_functions` (extra function names to treat as debug/higher-order) |
| `CUSTOM_CHECKS` | custom-checks | `max_function_inputs`, `max_function_outputs`, `max_function_lines`, `max_line_length`, `max_nesting_depth`, `max_cyclomatic_complexity`, `max_script_cyclomatic_complexity`, `max_anonymous_cyclomatic_complexity`, `disallow_bang`, `disallow_try_catch`, `disallow_persistent`, `disallow_continue`, `disallow_break`, `disallow_return`, `disallow_script`, `disallow_nested_functions`, `disallow_command_syntax`, `disallow_anonymous_functions`, `disallow_char_vectors`, `disallow_cellstr`, `disallow_varargin`, `disallow_varargout` |
| `NAMING_ENGINE` | naming | `max`, `min`, `pattern`, `prefix`, `phrase`, `suffix`, `style`. Per-entity check config goes through `[lint.rules."naming.<entity>.<check>"]` (quoted, see Validation) |
| `COMPAT` | compatibility | *(data-driven lookup; no params)* |
| `GOOD_PRACTICES_ENGINE` | good-practices | `disabled_checks`, `max_variable_name_length` |
| `UNSET_VARIABLES_ENGINE` | unset-variables | `ignore` (variable names to ignore) |
| `UNUSED_ENGINE` | unused-constructions | `disabled_checks`, `ignore_patterns` |
| `READABILITY_ENGINE` | readability | `disabled_checks` |
| `FORMATTING_ENGINE` | formatting | *(no params)* |
| `PERFORMANCE_ENGINE` | performance | `skip_checks` |
| `CODEGEN_ENGINE` | code-generation | `skip_checks` |
| `DEPLOYMENT_ENGINE` | deployment | `skip_checks` |
| `UNSUPPORTED_ENGINE` | unsupported | `skip_checks` |
| `SYSTEM_OBJECTS_ENGINE` | system-objects | `skip_checks` |
| `SUGGESTED_IMPROVEMENTS` | suggested-improvements | *(data-driven lookup; no params)* |

## Complete Example

```toml title=".malt.toml"
[lint]
# Skip generated and vendored code
exclude = [
    "codegen/**",
    "vendor/**",
    "*.generated.m",
]

[lint.categories]
# Keep compatibility warnings visible but don't gate the build on them
compatibility = "warn"

# Performance and formatting suggestions are noise during development
performance = "info"
formatting = "info"

[lint.rules]
# Escalate statements that print their result (a missing semicolon) to errors
[lint.rules.NOPTS]
severity = "error"

# Turn off a noisy sub-check of the language-spec engine
[lint.rules.LANGUAGE_SPEC_ENGINE]
disabled_checks = ["PFANSLP"]

# Raise the cyclomatic-complexity threshold for this project
[lint.rules.CUSTOM_CHECKS]
max_cyclomatic_complexity = 12
```

## Next steps

- [Rules Reference](rules.md)
- [CLI Reference](usage/cli.md)
- [Quick Start](getting-started/quickstart.md)
