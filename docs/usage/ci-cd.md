---
icon: lucide/git-branch
---

# CI/CD integration

## GitHub Actions

The [malt action](github-action.md) lints the workspace and reports annotations and a job summary, with no install step:

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

## Manual CI setup

To use the command line instead of the action, install malt and run it in the folder. Without a path it lints the current folder and everything below it, and `exclude` in `.malt.toml` decides what to skip. `--output github` turns the findings into annotations and a step summary:

```yaml title=".github/workflows/lint.yml"
name: Lint MATLAB
on: [push, pull_request]

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install malt
        run: pipx install malt

      - name: Lint
        run: malt --output github --log-level off --output-json malt-results.json

      - uses: actions/upload-artifact@v4
        if: always()
        with:
          name: malt-results
          path: malt-results.json
```

## GitLab CI

`--output gitlab` writes the Code Quality report, which GitLab shows in merge requests:

```yaml title=".gitlab-ci.yml"
malt:
  image: python:3
  script:
    - pip install malt
    - malt --output gitlab > gl-code-quality-report.json
  artifacts:
    when: always
    reports:
      codequality: gl-code-quality-report.json
```

The text lines go to the job log on stderr. See [Output formats](cli.md#output-formats) for the details.

## Large files and shared runners

malt skips a file over `max_file_size` (5 MiB by default) with a `MALT_TOO_BIG` error, which fails the build like any other error-level diagnostic (exit code `1`): exclude the file with `[lint] exclude`, or raise the limit for the project. On a small shared runner you can also lower `max_threads` and `max_file_size`. Raising a limit lets one file take gigabytes of memory; read [Resource limits](../configuration.md#resource-limits) first.

## Exit codes in CI

malt's exit codes map directly onto a pass or fail build:

| Code | Meaning | CI behavior |
| ---- | ------- | ----------- |
| `0`  | No issues | Build passes |
| `1`  | Diagnostics found | Build fails |
| `2`  | Runtime error | Build fails |

## Pre-commit hook

To lint the staged files before each commit, use the hooks for pre-commit and prek. They are set up on [their own page](pre-commit.md).

## Contributing

To help build CI/CD integrations, see the contribution guidelines in the [repository](https://github.com/watermarkhu/malt).
