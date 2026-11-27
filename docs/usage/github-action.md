---
icon: lucide/github
---

# GitHub Action

The malt action lints MATLAB code in a workflow and reports the findings as annotations on the changed lines, a job summary and a results file. It runs the WebAssembly build of malt on Node, so it needs no install step and no MATLAB.

```yaml title=".github/workflows/lint.yml"
name: Lint MATLAB
on: [push, pull_request]

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: watermarkhu/malt@v0
```

Without inputs it lints the whole workspace with the configuration of `.malt.toml` (or the `mathworks` defaults), annotates every finding, writes the job summary, and fails the step when there is an error.

## Inputs

| Input | Default | Description |
| ----- | ------- | ----------- |
| `paths` | `.` | Files and folders to lint, separated by spaces or newlines, relative to the workspace. A folder is searched for MATLAB files, below it too |
| `preset` | empty | `all`, `mathworks`, `recommended` or `monkeyproof`. Overrules the preset of `.malt.toml`. Empty keeps the file's |
| `level` | `info` | The least severe findings to report: `error`, `warning` or `info` |
| `fail-on` | `error` | The least severe finding that fails the step: `error`, `warning`, `info` or `never` |
| `annotations` | `true` | One annotation per finding |
| `summary` | `true` | Write the findings to the job summary |
| `output-json` | empty | Path of a JSON results file (the malt version, the configuration used and the findings). Nothing is written when empty |

The inputs overrule `.malt.toml`; everything else comes from the file. Which files a folder run picks up is set there, with [`include`, `exclude`, `extend_exclude`, `respect_gitignore` and `force_exclude`](../configuration.md#files-and-folders), exactly as for the command line.

## Outputs

| Output | Description |
| ------ | ----------- |
| `findings` | The number of findings reported |
| `errors` | The number of errors |
| `warnings` | The number of warnings |
| `results-file` | The absolute path of the results file, empty without `output-json` |

## Examples

Lint two folders, fail on warnings too, and keep the results as an artifact:

```yaml
- uses: actions/checkout@v4

- uses: watermarkhu/malt@v0
  id: malt
  with:
    paths: src test
    fail-on: warning
    output-json: malt-results.json

- uses: actions/upload-artifact@v4
  if: always()
  with:
    name: malt-results
    path: malt-results.json
```

Report everything but never fail the job, for a first look at a codebase:

```yaml
- uses: watermarkhu/malt@v0
  with:
    fail-on: never
```

Add the [MonkeyProof coding standard](../monkeyproof.md) on top of the configuration of the repository:

```yaml
- uses: watermarkhu/malt@v0
  with:
    preset: monkeyproof
```

## Limits

- **Annotations.** GitHub shows only the first 10 errors and 10 warnings of a step as annotations. The job summary lists every finding.
- **The workspace.** The action runs in the workspace and reads `.malt.toml` from its root. To lint a subfolder, name it in `paths`. A configuration file inside a subfolder is not read.
- **The path index** (`[index]` in `.malt.toml`) needs the file system and is not available in the WebAssembly build, so checks that read other files report as with the index off.
- **Versions.** `@v0` is the moving tag of the latest 0.x release, and `@v0.3.0` is one release. Each release tag carries the bundled action; the `main` branch does not, so do not use `@main`.

## What it is

The action is a thin shell around the `malt-wasm` package: it passes the inputs and the workspace's files to the same code the command line runs, and writes the result. Finding the files, reading `.malt.toml`, the annotations, the job summary and the results file are the same as for `malt --output github` and `--output-json`. See [Output formats](cli.md#output-formats) for the formats.
