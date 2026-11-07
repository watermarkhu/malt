---
icon: lucide/git-branch
---

# CI/CD Integration

::: info Coming Soon

CI/CD integration guides are planned for a future release. This page outlines the intended workflow.

:::

## GitHub Actions (Planned)

Once mlt is published to crates.io, a GitHub Action will be available:

```yaml title=".github/workflows/lint.yml"
name: Lint MATLAB
on: [push, pull_request]

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: watermarkhu/mlt-action@main  # planned
```

## Manual CI Setup

Until a dedicated action is available, you can install and run mlt manually:

```yaml title=".github/workflows/lint.yml"
name: Lint MATLAB
on: [push, pull_request]

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install mlt
        run: pipx install mlt

      - name: Lint
        run: |
          find . -name '*.m' -not -path './vendor/*' | xargs mlt
```

## Exit Codes for CI

mlt's exit codes are designed for CI use:

| Code | Meaning | CI Behavior |
| ---- | ------- | ----------- |
| `0`  | No issues | Build passes |
| `1`  | Diagnostics found | Build fails |
| `2`  | Runtime error | Build fails |

## Pre-commit Hook (Planned)

A pre-commit hook configuration is planned:

```yaml title=".pre-commit-config.yaml"
repos:
  - repo: https://github.com/watermarkhu/mlt
    rev: v0.1.0  # use latest version
    hooks:
      - id: mlt
        types: [file]
        files: '\.m$'
```

## Contributing

If you'd like to help build CI/CD integrations, see the [repository](https://github.com/watermarkhu/mlt) for contribution guidelines.
