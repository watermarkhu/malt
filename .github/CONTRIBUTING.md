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

## 5. One-time release setup

Releases use trusted publishing (OIDC), so no long-lived publish tokens are stored
in the repository. GitHub side:

- an environment named `release` with **no deployment branch restriction**: the
  release job runs on `pull_request` events, whose ref is `refs/pull/N/merge`;
- the `APP_ID` variable and `APP_PRIVATE_KEY` secret of the GitHub App that pushes
  the release tag and creates the release;
- Settings → Actions → General → "Allow GitHub Actions to create and approve pull
  requests" (the dev-branch workflow opens the rolling draft PR).

On each registry, register a trusted publisher for repository `watermarkhu/malt`,
workflow **`release.yml`** and environment **`release`**:

| Registry | Package(s) | Where | Available |
| -------- | ---------- | ----- | --------- |
| [crates.io](https://crates.io/docs/trusted-publishing) | `malt-core`, `malt-rules`, `malt-cli` | each crate's Settings → Trusted Publishing | after the crate exists |
| [PyPI](https://docs.pypi.org/trusted-publishers/) | `malt` | project Settings → Publishing (or a pending publisher if the project does not exist yet) | any time |
| [npm](https://docs.npmjs.com/trusted-publishers) | `malt-wasm` | package Settings → Trusted publisher (allow `npm publish`) | after the package exists |

crates.io and npm cannot trust a package that does not exist yet, so the first
release needs a bootstrap:

1. **crates.io:** add a `CARGO_REGISTRY_TOKEN` secret from an API token with the
   `publish-new` and `publish-update` scopes and the crate pattern `malt-*`. The first
   release publishes with it. Then add the trusted publisher to all three crates,
   delete the secret, and remove the fallback marked in `release.yml`.
2. **npm:** publish an empty `0.0.0` placeholder of `malt-wasm` once from your
   machine, then add the trusted publisher (allow `npm publish`). Without an
   existing package CI cannot authenticate, and npm answers the publish with a
   misleading `404`. Never publish `crates/malt_wasm/pkg` by hand: wasm-pack names
   that raw output `malt_wasm` (underscore), a different package that npm treats as
   too similar to `malt-wasm`:

   ```sh
   mkdir /tmp/malt-wasm-placeholder && cd /tmp/malt-wasm-placeholder
   cat > package.json <<'JSON'
   {
     "name": "malt-wasm",
     "version": "0.0.0",
     "description": "Placeholder; releases are published from GitHub Actions",
     "license": "MIT",
     "repository": { "type": "git", "url": "git+https://github.com/watermarkhu/malt.git" }
   }
   JSON
   npm publish --access public
   ```

   A new npm trusted publisher expires if it has no successful publish within 2
   days, so add it right before the release.
3. **PyPI:** nothing to bootstrap.

Afterwards, set the npm package's Publishing access to "Require two-factor
authentication and disallow tokens".
