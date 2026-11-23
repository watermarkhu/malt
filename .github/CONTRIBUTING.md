# Contributing to malt

Contributions are welcome. This page covers the setup, the checks and the pull request flow.

## 1. Environment setup

Development tools are managed by [mise](https://mise.jdx.dev/). Installing them
also installs the [hk](https://hk.jdx.dev/) git hooks (pre-commit, commit-msg,
pre-push):

```sh
mise install
bun install          # docs toolchain (VitePress)
uv sync              # Python wrapper (maturin, pytest, ruff, ty)
```

## 2. Checks and tests

```sh
hk check --all                     # fmt, clippy, rumdl, jactionlint, docs freshness, ...
cargo test --workspace             # Rust tests
mise run test-wasm                 # WebAssembly binding tests
uv run pytest                      # Python wrapper tests (builds the wheel)
bun run docs:build                 # documentation site
```

The MATLAB cross-check of the path index is in two phases, because on a public repository only the
matlab-actions `run-*` actions license MATLAB in CI. First MATLAB writes the facts file, then the
test compares it (MATLAB is not started by the test then). Locally, with MATLAB (tested with R2026b)
on your `PATH`:

```sh
matlab -batch "run('crates/malt_cli/tests/matlab/generate_namespace_facts.m')"
MALT_MATLAB_FACTS=target/matlab/namespace_facts.json MALT_MATLAB_REQUIRE=1 \
  cargo test -p malt-cli --locked --test matlab_namespace -- --nocapture
```

Without `MALT_MATLAB_FACTS`, the test starts MATLAB itself (`MALT_MATLAB` or `matlab` on `PATH`) and
generates the same file; without MATLAB it prints a `SKIP` line. `MALT_MATLAB_REQUIRE=1` makes a missing
MATLAB or facts file a failure (CI sets it). `MALT_MATLAB_EXTRA_PATHS` adds local folders to the
generation (never copied into the repository). See `docs/VERIFY.md`, 'Cross-check against MATLAB'.

The recorded `checkcode` expectations (`matlab_expected.tsv`) are checked against live MATLAB the same
way: the generator runs `checkcode` over every fixture, then the test compares (and checks that the
release is the recorded one):

```sh
matlab -batch "run('crates/malt_cli/tests/matlab/generate_checkcode.m')"
MALT_MATLAB_CHECKCODE=target/matlab/checkcode.tsv MALT_MATLAB_REQUIRE=1 \
  cargo test -p malt-cli --locked --test matlab_live -- --nocapture
```

A difference fails with the list of stale and new findings; a different MATLAB release fails with its own
message. See `docs/VERIFY.md`, 'Live checkcode check'.

New fixtures (the `.m` files under `crates/malt_cli/tests/fixtures`) are added as
[`crates/malt_cli/tests/fixtures/README.md`](../crates/malt_cli/tests/fixtures/README.md) describes: the file list,
the MATLAB expectations and the checks.

The Qualify workflow runs all of these on every pull request into `dev` or
`main`, builds every distribution (wheels, sdist, npm package, crates) and
reports a single `Qualified` status.

## 3. Pull request guidelines

Every pull request targets `dev`, never `main`. This applies to people and
to Renovate (`baseBranches` in `.github/renovate.json5` is `["dev"]`, so its
dependency updates open against `dev` too). Branch from `dev`, and pick `dev`
as the base when you open the pull request.

`main` only receives one pull request: the rolling `dev` → `main` pull
request. The [Dev branch](workflows/dev-branch.yml) workflow keeps it going:

1. Once a `dev` → `main` pull request is merged, the workflow deletes `dev`
   and recreates it from `main` with one empty bootstrap commit.
2. It then tries to open a draft `dev` → `main` pull request. If the
   repository setting "Allow GitHub Actions to create and approve pull
   requests" is off, that step fails and a maintainer opens the pull request by
   hand.
3. A maintainer gives that pull request a Conventional Commit title and marks
   it ready for review. Merging it releases (see [Releases](#4-releases)).

If `dev` is missing or has fallen behind `main`, run the workflow manually
from the Actions tab (`workflow_dispatch`) to recreate it. That discards
anything on `dev` that is not on `main`.

Use [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) for
commit messages and pull request titles. A pull request into `dev` is
squash-merged, so its title becomes the commit that the next release is
derived from.

## 4. Releases

Releases are automatic. The **Release** workflow runs
[semVersie](https://github.com/RonaldPhilipsen/semVersie) on every pull request
into `main` and labels it with the version bump derived from its Conventional
Commit title (`fix:` → patch, `feat:` → minor, `!`/`BREAKING CHANGE` → major).
When such a pull request is merged, the workflow:

1. stamps the version into the Cargo workspace (the source of truth for the
   crates, the Python wheels and the npm package; `main` keeps `0.0.0`),
2. commits and pushes a `vX.Y.Z` tag,
3. publishes `malt-core`, `malt-rules`, `malt-index` and `malt-cli` to crates.io,
   `malt` to PyPI, and `malt-wasm` to npm,
4. creates a GitHub release with the wheels, sdist and npm tarball attached,
5. (separately) the **Pages** workflow deploys the documentation to GitHub Pages
   from `main` on every push, through the `github-pages` environment.
