# Contributing to malt

Thank you for considering contributing! Please follow these steps to get started.

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
hk check --all                     # fmt, clippy, rumdl, actionlint, docs freshness, ...
cargo test --workspace             # Rust tests
mise run test-wasm                 # WebAssembly binding tests
uv run pytest                      # Python wrapper tests (builds the wheel)
bun run docs:build                 # documentation site
```

The **Qualify** workflow runs all of these on every pull request, builds every
distribution (wheels, sdist, npm package, crates), and reports a single
`Qualified` status.

## 3. Pull request guidelines

- Pull requests should target the `dev` branch. A rolling draft pull request
  merges `dev` into `main`.
- Use [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) for
  commit messages and pull request titles.

## 4. Releases

Releases are automatic. The **Release** workflow runs
[semVersie](https://github.com/RonaldPhilipsen/semVersie) on every pull request
into `main` and labels it with the version bump derived from its Conventional
Commit title (`fix:` → patch, `feat:` → minor, `!`/`BREAKING CHANGE` → major).
When such a pull request is merged, the workflow:

1. stamps the version into the Cargo workspace (the source of truth for the
   crates, the Python wheels and the npm package; `main` keeps `0.0.0`),
2. commits and pushes a `vX.Y.Z` tag,
3. publishes `malt-core`, `malt-rules` and `malt-cli` to crates.io,
   `malt` to PyPI, and `malt-wasm` to npm,
4. creates a GitHub release with the wheels, sdist and npm tarball attached,
5. (separately) the **Pages** workflow deploys the documentation to GitHub Pages
   from `main` on every push, through the `github-pages` environment.
