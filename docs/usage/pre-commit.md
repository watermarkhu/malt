---
icon: lucide/git-commit-horizontal
---

# pre-commit and prek

malt ships hooks for [pre-commit](https://pre-commit.com) and [prek](https://prek.j178.dev), a faster drop-in replacement that reads the same configuration. A hook lints the MATLAB files you are about to commit, and fails the commit when it finds something.

Both tools read a `.pre-commit-config.yaml`. prek also reads a `prek.toml`, with the same structure in TOML. Each example below is given in both formats, and you need only one.

## The hooks

| Hook | What it does |
| ---- | ------------ |
| `malt` | Lints the staged `.m` files and fails on findings |
| `malt-fix` | Lints them, applies the safe fixes, and fails when files changed or findings remain |

The hooks are defined in the `.pre-commit-hooks.yaml` of the [malt repository](https://github.com/watermarkhu/malt). Pin `rev` to a release tag; the latest is on the [releases page](https://github.com/watermarkhu/malt/releases).

::: code-group

```yaml [.pre-commit-config.yaml]
repos:
  - repo: https://github.com/watermarkhu/malt
    rev: v0.4.0  # use the latest release
    hooks:
      - id: malt
```

```toml [prek.toml]
[[repos]]
repo = "https://github.com/watermarkhu/malt"
rev = "v0.4.0"  # use the latest release
hooks = [{ id = "malt" }]
```

:::

Then install the hook and try it on the whole repository:

```bash
pre-commit install            # or: prek install
pre-commit run --all-files    # or: prek run --all-files
```

::: warning The first run builds malt

The hook installs the malt repository as a Python package, which compiles the `malt` binary with maturin. That needs a Rust toolchain and takes a few minutes the first time; the result is cached. If you have no Rust toolchain, or want a faster setup, use the [local hook](#without-a-rust-toolchain) below.

:::

## Fixing

`malt-fix` applies the safe fixes. A hook that changes files makes the run fail, so stage the changes and commit again.

::: code-group

```yaml [.pre-commit-config.yaml]
repos:
  - repo: https://github.com/watermarkhu/malt
    rev: v0.4.0
    hooks:
      - id: malt-fix
        args: [--unsafe-fixes]   # also apply fixes that may change behaviour (optional)
```

```toml [prek.toml]
[[repos]]
repo = "https://github.com/watermarkhu/malt"
rev = "v0.4.0"

[[repos.hooks]]
id = "malt-fix"
args = ["--unsafe-fixes"]   # also apply fixes that may change behaviour (optional)
```

:::

Note that `--unsafe-fixes` also needs `--fix`, which `malt-fix` already passes. Do not use both hooks in one configuration.

## Without a Rust toolchain

A local hook installs the prebuilt wheel from PyPI, so nothing is compiled. Pin the version of malt in `additional_dependencies`:

::: code-group

```yaml [.pre-commit-config.yaml]
repos:
  - repo: local
    hooks:
      - id: malt
        name: malt
        entry: malt
        language: python
        additional_dependencies: ["malt==0.4.0"]
        types: [file]
        files: '\.m$'
```

```toml [prek.toml]
[[repos]]
repo = "local"

[[repos.hooks]]
id = "malt"
name = "malt"
entry = "malt"
language = "python"
additional_dependencies = ["malt==0.4.0"]
types = ["file"]
files = '\.m$'
```

:::

If malt is already installed (with `pipx`, `uv tool` or `cargo`), `language: system` uses it as it is, and the version is the one on your `PATH`.

## Configuring malt

The hook runs `malt` from the root of the repository, so it reads the `.malt.toml` there, as on the command line. Options go in `args`:

::: code-group

```yaml [.pre-commit-config.yaml]
repos:
  - repo: https://github.com/watermarkhu/malt
    rev: v0.4.0
    hooks:
      - id: malt
        args: [--preset, monkeyproof, --level, warning]
```

```toml [prek.toml]
[[repos]]
repo = "https://github.com/watermarkhu/malt"
rev = "v0.4.0"

[[repos.hooks]]
id = "malt"
args = ["--preset", "monkeyproof", "--level", "warning"]
```

:::

See the [command line reference](cli.md) for the options, and the [configuration](../configuration.md) page for `.malt.toml`.

### Excluded files

`force_exclude` needs malt 0.4 or later. pre-commit and prek name every staged file on the command line, and malt lints a file that is named on the command line even when `exclude` in `.malt.toml` would skip it. To make `exclude` hold for the hook too, set `force_exclude`:

```toml title=".malt.toml"
[lint]
exclude = ["vendor/", "third_party/"]
force_exclude = true
```

Alternatively, use the `exclude` of the hook itself, a regular expression that pre-commit and prek apply before malt sees the files:

```yaml
      - id: malt
        exclude: '^(vendor|third_party)/'
```

### Linting the whole project on every commit

This needs malt 0.4 or later, which lints the current folder when no file is named. The default hooks lint only the staged files. To lint the whole project each time, for example because a check needs to know the other files, ask for no file names and always run:

::: code-group

```yaml [.pre-commit-config.yaml]
repos:
  - repo: https://github.com/watermarkhu/malt
    rev: v0.4.0
    hooks:
      - id: malt
        pass_filenames: false
        always_run: true
```

```toml [prek.toml]
[[repos]]
repo = "https://github.com/watermarkhu/malt"
rev = "v0.4.0"

[[repos.hooks]]
id = "malt"
pass_filenames = false
always_run = true
```

:::

malt then lints the current folder and everything below it, as `malt` without a path does.

## Keeping the version up to date

```bash
pre-commit autoupdate    # or: prek update
```

moves `rev` to the latest release. For a local hook, change the pinned version in `additional_dependencies` yourself. The `prek.toml` of a project can be checked with `prek validate-config prek.toml`.

## Continuous integration

The hooks run the same malt as the command line, so a CI job can run `pre-commit run --all-files` instead of calling malt directly. To get annotations and a job summary on GitHub, use the [GitHub Action](github-action.md) instead.
